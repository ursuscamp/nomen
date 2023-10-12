use std::collections::HashMap;

use axum::{
    extract::{Path, Query, State},
    Form,
};
use axum_extra::extract::WithRejection;
use bitcoin::Txid;
use itertools::Itertools;
use secp256k1::XOnlyPublicKey;
use serde::Deserialize;

use crate::{
    db::{self, NameDetails},
    subcommands::util::{create_psbt, name_event},
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
#[template(path = "nsid.html")]
pub struct NsidTemplate {
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

impl TryFrom<NameDetails> for NsidTemplate {
    type Error = anyhow::Error;

    fn try_from(value: NameDetails) -> Result<Self, Self::Error> {
        let records: HashMap<String, String> = serde_json::from_str(&value.records)?;
        let mut record_keys = records.keys().cloned().collect_vec();
        record_keys.sort();
        let blocktime = format_time(value.blocktime)?;

        Ok(NsidTemplate {
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

pub async fn explore_nsid(
    State(state): State<AppState>,
    Path(nsid): Path<String>,
) -> Result<NsidTemplate, WebError> {
    let conn = state.pool;
    let details = db::name_details(&conn, &nsid).await?;

    Ok(details.try_into()?)
}

#[derive(askama::Template, Default)]
#[template(path = "newname.html")]
pub struct NewNameTemplate {
    psbt: String,
    txid: String,
    vout: u32,
    address: String,
    name: String,
    pubkey: String,
    confirmations: usize,
    fee: usize,
}

#[derive(Deserialize)]
pub struct NewNameForm {
    txid: Txid,
    vout: u32,
    address: String,
    name: String,
    pubkey: XOnlyPublicKey,
    fee: usize,
}

#[allow(clippy::unused_async)]
pub async fn new_name_form(State(state): State<AppState>) -> Result<NewNameTemplate, WebError> {
    Ok(NewNameTemplate {
        confirmations: state.config.confirmations(),
        ..Default::default()
    })
}

#[allow(clippy::unused_async)]
pub async fn new_name_submit(
    State(state): State<AppState>,
    WithRejection(Form(form), _): WithRejection<Form<NewNameForm>, WebError>,
) -> Result<NewNameTemplate, WebError> {
    // TODO: use a proper Name here.
    // let name: Name = form.name.parse()?;
    // TODO: check name availability here
    // check_name_availability(&state.config, form.name.as_ref()).await?;
    let rpc = state.config.rpc_client()?;
    let psbt = create_psbt(
        rpc,
        form.txid,
        form.vout,
        form.address.clone(),
        form.name.clone(),
        form.pubkey,
        form.fee,
    )
    .await?;
    Ok(NewNameTemplate {
        psbt: psbt.to_string(),
        txid: form.txid.to_string(),
        vout: form.vout,
        address: form.address,
        name: form.name,
        pubkey: form.pubkey.to_string(),
        confirmations: state.config.confirmations(),
        fee: form.fee,
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
