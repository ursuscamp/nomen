use std::{collections::HashMap, str::FromStr};

use anyhow::anyhow;
use axum::{
    extract::{Path, Query, State},
    Form,
};
use axum_extra::extract::WithRejection;
use bitcoin::psbt::Psbt;
use itertools::Itertools;
use nomen_core::{CreateBuilder, Name};
use secp256k1::XOnlyPublicKey;
use serde::Deserialize;

use crate::{
    db::{self, NameDetails},
    subcommands::util::{extend_psbt, name_event},
    util::{format_time, KeyVal},
};

use super::{AppState, WebError};

#[derive(askama::Template)]
#[template(path = "error.html")]
pub struct ErrorTemplate {
    pub message: String,
}

#[derive(askama::Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {}

#[allow(clippy::unused_async)]
pub async fn index() -> IndexTemplate {
    IndexTemplate {}
}

#[allow(clippy::module_name_repetitions)]
#[derive(Deserialize)]
pub struct ExplorerQuery {
    pub q: Option<String>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(askama::Template)]
#[template(path = "explorer.html")]
pub struct ExplorerTemplate {
    q: String,
    names: Vec<(String, String)>,
    last_index_time: String,
}

pub async fn explorer(
    State(state): State<AppState>,
    Query(query): Query<ExplorerQuery>,
) -> Result<ExplorerTemplate, WebError> {
    let conn = state.pool;
    let last_index_time = db::last_index_time(&conn).await?;
    let last_index_time = format_time(last_index_time)?;
    let q = query.q.map(|s| s.trim().to_string());

    Ok(ExplorerTemplate {
        q: q.clone().unwrap_or_default(),
        names: db::top_level_names(&conn, q).await?,
        last_index_time,
    })
}

#[derive(askama::Template)]
#[template(path = "name.html")]
pub struct NameTemplate {
    name: String,
    record_keys: Vec<String>,
    records: HashMap<String, String>,
    blockhash: String,
    blocktime: String,
    txid: String,
    vout: i64,
    height: i64,
    pubkey: String,
    protocol: i64,
}

impl TryFrom<NameDetails> for NameTemplate {
    type Error = anyhow::Error;

    fn try_from(value: NameDetails) -> Result<Self, Self::Error> {
        let records: HashMap<String, String> = serde_json::from_str(&value.records)?;
        let mut record_keys = records.keys().cloned().collect_vec();
        record_keys.sort();
        let blocktime = format_time(value.blocktime)?;

        Ok(NameTemplate {
            name: value.name,
            record_keys,
            records,
            blockhash: value.blockhash,
            blocktime,
            txid: value.txid,
            vout: value.vout,
            height: value.blockheight,
            pubkey: value.pubkey,
            protocol: value.protocol,
        })
    }
}

pub async fn show_name(
    State(state): State<AppState>,
    Path(nsid): Path<String>,
) -> Result<NameTemplate, WebError> {
    let conn = state.pool;
    let details = db::name_details(&conn, &nsid).await?;

    Ok(details.try_into()?)
}

#[derive(askama::Template, Default)]
#[template(path = "newname.html")]
pub struct NewNameTemplate {
    upgrade: bool,
    data: String,
    name: String,
    pubkey: String,
    confirmations: usize,
    is_psbt: bool,
}

#[derive(Deserialize)]
pub struct NewNameForm {
    upgrade: bool,
    name: String,
    pubkey: XOnlyPublicKey,
    psbt: String,
}

#[derive(Deserialize)]
pub struct NewNameQuery {
    upgrade: Option<bool>,
}

#[allow(clippy::unused_async)]
pub async fn new_name_form(
    State(state): State<AppState>,
    WithRejection(Query(query), _): WithRejection<Query<NewNameQuery>, WebError>,
) -> Result<NewNameTemplate, WebError> {
    Ok(NewNameTemplate {
        confirmations: state.config.confirmations(),
        upgrade: query.upgrade.unwrap_or_default(),
        ..Default::default()
    })
}

#[allow(clippy::unused_async)]
pub async fn new_name_submit(
    State(state): State<AppState>,
    WithRejection(Form(form), _): WithRejection<Form<NewNameForm>, WebError>,
) -> Result<NewNameTemplate, WebError> {
    let _name = Name::from_str(&form.name).map_err(|_| anyhow!("Invalid name"))?;

    // If we're upgrading an existing name, we don't actually want to error if the name exists.
    let available = if form.upgrade {
        true
    } else {
        db::check_name_availability(&state.pool, form.name.as_ref()).await?
    };
    if !available {
        Err(anyhow!("Name unavailable"))?;
    }
    let (is_psbt, data) = if form.psbt.is_empty() {
        let d = CreateBuilder::new(&form.pubkey, &form.name).v1_op_return();
        (false, hex::encode(d))
    } else {
        let mut psbt: Psbt = form.psbt.parse()?;
        extend_psbt(&mut psbt, &form.name, &form.pubkey);
        (true, psbt.to_string())
    };
    Ok(NewNameTemplate {
        upgrade: form.upgrade,
        data,
        name: form.name,
        pubkey: form.pubkey.to_string(),
        confirmations: state.config.confirmations(),
        is_psbt,
    })
}

#[derive(askama::Template)]
#[template(path = "updaterecords.html")]
pub struct NewRecordsTemplate {
    name: String,
    pubkey: String,
    unsigned_event: String,
    relays: Vec<String>,
    records: String,
}

#[derive(Deserialize)]
pub struct NewRecordsQuery {
    name: Option<String>,
    pubkey: Option<XOnlyPublicKey>,
}

pub async fn new_records_form(
    State(state): State<AppState>,
    Query(query): Query<NewRecordsQuery>,
) -> Result<NewRecordsTemplate, WebError> {
    let records = records_from_query(&query, &state).await?;
    Ok(NewRecordsTemplate {
        name: query.name.unwrap_or_default(),
        pubkey: query.pubkey.map(|s| s.to_string()).unwrap_or_default(),
        unsigned_event: String::default(),
        relays: state.config.relays(),
        records,
    })
}

async fn records_from_query(query: &NewRecordsQuery, state: &AppState) -> Result<String, WebError> {
    let records = match &query.name {
        Some(name) => {
            let (records,) = sqlx::query_as::<_, (String,)>(
                "SELECT records FROM valid_names_records_vw WHERE name = ?;",
            )
            .bind(name)
            .fetch_optional(&state.pool)
            .await?
            .unwrap_or_else(|| (String::from(r#"{"KEY":"value"}"#),));
            let records: HashMap<String, String> = serde_json::from_str(&records)?;
            records
                .iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect_vec()
                .join("\n")
        }
        None => "KEY=value".into(),
    };
    Ok(records)
}

#[derive(Deserialize, Debug)]
pub struct NewRecordsForm {
    records: String,
    name: String,
    pubkey: XOnlyPublicKey,
}

#[allow(clippy::unused_async)]
pub async fn new_records_submit(
    State(state): State<AppState>,
    Form(form): Form<NewRecordsForm>,
) -> Result<NewRecordsTemplate, WebError> {
    let records = form
        .records
        .lines()
        .map(str::parse)
        .collect::<Result<Vec<KeyVal>, _>>()?
        .iter()
        .map(|kv| kv.clone().pair())
        .collect::<HashMap<_, _>>();
    let event = name_event(form.pubkey, &records, &form.name)?;
    let unsigned_event = serde_json::to_string_pretty(&event)?;
    Ok(NewRecordsTemplate {
        name: form.name.to_string(),
        pubkey: form.pubkey.to_string(),
        unsigned_event,
        relays: state.config.relays(),
        records: "KEY=value".into(),
    })
}

#[derive(askama::Template)]
#[template(path = "stats.html")]
pub struct IndexerInfo {
    version: &'static str,
    commit: &'static str,
    build_date: &'static str,
    known_names: i64,
    index_height: i64,
    nostr_events: i64,
}

pub async fn index_stats(State(state): State<AppState>) -> Result<IndexerInfo, WebError> {
    Ok(IndexerInfo {
        version: env!("CARGO_PKG_VERSION"),
        commit: env!("VERGEN_GIT_DESCRIBE"),
        build_date: env!("VERGEN_BUILD_TIMESTAMP"),
        known_names: db::stats::known_names(&state.pool).await?,
        index_height: db::stats::index_height(&state.pool).await?,
        nostr_events: db::stats::nostr_events(&state.pool).await?,
    })
}

pub mod transfer {
    use axum::{extract::State, Form};
    use bitcoin::{
        address::{NetworkChecked, NetworkUnchecked},
        Address, Amount, Txid,
    };
    use nomen_core::TransferBuilder;
    use secp256k1::{schnorr::Signature, XOnlyPublicKey};
    use serde::Deserialize;

    use crate::subcommands::{
        util::{transfer_psbt1, transfer_psbt2},
        AppState, WebError,
    };

    #[derive(askama::Template)]
    #[template(path = "transfer/initiate.html")]
    pub struct InitiateTransferTemplate;

    #[derive(Deserialize)]
    pub struct InitiateTransferForm {
        txid: Txid,
        vout: u64,
        address: Address<NetworkUnchecked>,
        fee: u64,
        name: String,
        pubkey: XOnlyPublicKey,
        old_pubkey: XOnlyPublicKey,
    }

    #[allow(clippy::unused_async)]
    pub async fn initiate() -> InitiateTransferTemplate {
        InitiateTransferTemplate
    }

    #[derive(askama::Template)]
    #[template(path = "transfer/sign.html")]
    pub struct SignEventTemplate {
        txid: Txid,
        vout: u64,
        address: Address<NetworkChecked>,
        fee: u64,
        name: String,
        pubkey: XOnlyPublicKey,
        old_pubkey: XOnlyPublicKey,
        event: String,
    }

    #[allow(clippy::unused_async)]
    pub async fn submit_initiate(
        State(state): State<AppState>,
        Form(transfer): Form<InitiateTransferForm>,
    ) -> Result<SignEventTemplate, WebError> {
        let te = TransferBuilder {
            new_pubkey: &transfer.pubkey,
            name: &transfer.name,
        };
        let event = te.unsigned_event(&transfer.old_pubkey);
        Ok(SignEventTemplate {
            txid: transfer.txid,
            vout: transfer.vout,
            address: transfer.address.require_network(state.config.network())?,
            fee: transfer.fee,
            name: transfer.name,
            pubkey: transfer.pubkey,
            old_pubkey: transfer.old_pubkey,
            event: serde_json::to_string(&event)?,
        })
    }

    #[derive(Deserialize)]
    pub struct FinalTransferForm {
        txid: Txid,
        vout: u32,
        address: Address<NetworkUnchecked>,
        fee: usize,
        name: String,
        pubkey: XOnlyPublicKey,
        sig: Signature,
    }

    #[derive(askama::Template)]
    #[template(path = "transfer/complete.html")]
    pub struct CompleteTransferTemplate {
        tx1: String,
        tx2: String,
    }

    pub async fn complete(
        State(state): State<AppState>,
        Form(transfer): Form<FinalTransferForm>,
    ) -> Result<CompleteTransferTemplate, WebError> {
        let address = transfer.address.require_network(state.config.network())?;
        let tx1 = transfer_psbt1(
            state.config.rpc_client()?,
            transfer.txid,
            transfer.vout,
            &address.to_string(),
            &transfer.name,
            &transfer.pubkey,
            transfer.fee,
        )
        .await?;

        let consensus_tx = tx1.clone().extract_tx();
        let value = Amount::from_sat(consensus_tx.output[0].value);

        let tx2 = transfer_psbt2(
            state.config.rpc_client()?,
            consensus_tx.txid(),
            0,
            address.to_string(),
            transfer.name,
            transfer.pubkey,
            transfer.fee,
            transfer.sig,
            value,
        )
        .await?;

        Ok(CompleteTransferTemplate {
            tx1: tx1.to_string(),
            tx2: tx2.to_string(),
        })
    }
}
