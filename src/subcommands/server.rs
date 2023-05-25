use std::time::Duration;

use askama_axum::IntoResponse;
use axum::{
    http::StatusCode,
    routing::{get, post},
    Router,
};
use sqlx::SqlitePool;
use tokio::time::{interval, MissedTickBehavior};

use crate::{
    config::{Cli, Config, ServerSubcommand},
    subcommands,
};

use self::site::ErrorTemplate;

pub struct WebError(anyhow::Error, Option<StatusCode>);

impl WebError {
    pub fn not_found(err: anyhow::Error) -> WebError {
        WebError(err, Some(StatusCode::NOT_FOUND))
    }
}

impl IntoResponse for WebError {
    fn into_response(self) -> askama_axum::Response {
        ErrorTemplate {
            message: self.0.to_string(),
        }
        .into_response()
    }
}

impl<E> From<E> for WebError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into(), None)
    }
}

#[derive(Clone)]
pub struct AppState {
    config: Config,
    pool: SqlitePool,
}

pub async fn start(
    config: &Config,
    conn: &SqlitePool,
    server: &ServerSubcommand,
) -> anyhow::Result<()> {
    if !server.without_indexer {
        let _indexer = tokio::spawn(indexer(config.clone(), server.clone(), conn.clone()));
    }
    let mut app = Router::new();

    if !server.without_explorer {
        app = app
            .route("/", get(site::index))
            .route("/explorer", get(site::explorer))
            .route("/explorer/:nsid", get(site::explore_nsid))
            .route("/newname", get(site::new_name_form))
            .route("/newname", post(site::new_name_submit))
            .route("/newrecords", get(site::new_records_form))
            .route("/newrecords", post(site::new_records_submit));
    }

    if !server.without_api {
        app = app.route("/api/name", get(api::name));
    }

    let state = AppState {
        config: config.clone(),
        pool: conn.clone(),
    };
    let app = app.with_state(state);

    let addr = config
        .server_bind()
        .expect("Server bind unconfigured")
        .parse()?;

    log::info!("Starting server on {addr}");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(elegant_departure::tokio::depart().on_termination())
        .await?;

    log::info!("Server shutdown complete.");
    elegant_departure::shutdown();
    Ok(())
}

async fn indexer(config: Config, server: ServerSubcommand, pool: SqlitePool) -> anyhow::Result<()> {
    let mut interval = interval(Duration::from_secs(config.server_indexer_delay()));
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

    loop {
        subcommands::index(&config, &pool).await?;
        interval.tick().await;
    }
    Ok(())
}

mod site {
    use std::collections::HashMap;

    use anyhow::{anyhow, bail};
    use axum::{
        extract::{rejection::FailedToDeserializeForm, Path, Query, State},
        http::StatusCode,
        Form,
    };
    use axum_extra::extract::WithRejection;
    use bitcoin::{address::NetworkUnchecked, psbt::Psbt, Address, Transaction, Txid};
    use bitcoincore_rpc::RawTx;
    use itertools::Itertools;
    use secp256k1::XOnlyPublicKey;
    use serde::Deserialize;
    use sqlx::SqlitePool;

    use crate::{
        config::{Cli, TxInfo},
        db::{self, name_available, NameDetails},
        subcommands::{insert_outputs, name_event},
        util::{check_name_availability, Hash160, KeyVal, Name, NomenKind, NsidBuilder},
    };

    use super::{util, AppState, WebError};

    #[derive(askama::Template)]
    #[template(path = "error.html")]
    pub struct ErrorTemplate {
        pub message: String,
    }

    #[derive(askama::Template)]
    #[template(path = "index.html")]
    pub struct IndexTemplate {}

    pub async fn index() -> IndexTemplate {
        IndexTemplate {}
    }

    #[derive(Deserialize)]
    pub struct ExplorerQuery {
        pub q: Option<String>,
    }

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
        let last_index_time = util::format_time(last_index_time)?;

        Ok(ExplorerTemplate {
            q: query.q.clone().unwrap_or_default(),
            names: db::top_level_names(&conn, query.q).await?,
            last_index_time,
        })
    }

    #[derive(askama::Template)]
    #[template(path = "nsid.html")]
    pub struct NsidTemplate {
        name: String,
        record_keys: Vec<String>,
        records: HashMap<String, String>,
        records_created_at: String,
        blockhash: String,
        blocktime: String,
        txid: String,
        vout: i64,
        height: i64,
        pubkey: String,
    }

    impl TryFrom<NameDetails> for NsidTemplate {
        type Error = anyhow::Error;

        fn try_from(value: NameDetails) -> Result<Self, Self::Error> {
            let records: HashMap<String, String> = serde_json::from_str(&value.records)?;
            let mut record_keys = records.keys().cloned().collect_vec();
            record_keys.sort();
            let blocktime = util::format_time(value.blocktime)?;
            let records_created_at = util::format_time(value.records_created_at)?;

            Ok(NsidTemplate {
                name: value.name,
                record_keys,
                records,
                records_created_at,
                blockhash: value.blockhash,
                blocktime,
                txid: value.txid,
                vout: value.vout,
                height: value.blockheight,
                pubkey: value.pubkey,
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
        name: String,
        pubkey: String,
        confirmations: usize,
    }

    #[derive(Deserialize)]
    pub struct NewNameForm {
        psbt: String,
        name: String,
        pubkey: XOnlyPublicKey,
    }

    pub async fn new_name_form(State(state): State<AppState>) -> Result<NewNameTemplate, WebError> {
        Ok(NewNameTemplate {
            confirmations: state.config.confirmations()?,
            ..Default::default()
        })
    }

    pub async fn new_name_submit(
        State(state): State<AppState>,
        WithRejection(Form(mut form), _): WithRejection<Form<NewNameForm>, WebError>,
    ) -> Result<NewNameTemplate, WebError> {
        let name: Name = form.name.parse()?;
        check_name_availability(&state.config, form.name.as_ref()).await?;
        let fingerprint = Hash160::default()
            .chain_update(name.as_ref().as_bytes())
            .fingerprint();
        let nsid = NsidBuilder::new(form.name.as_ref(), &form.pubkey).finalize();
        let mut psbt: Psbt = form.psbt.parse()?;
        insert_outputs(&mut psbt, fingerprint, nsid, NomenKind::Create)?;
        Ok(NewNameTemplate {
            psbt: psbt.to_string(),
            name: form.name,
            pubkey: form.pubkey.to_string(),
            confirmations: state.config.confirmations()?,
        })
    }

    #[derive(askama::Template)]
    #[template(path = "newrecords.html")]
    pub struct NewRecordsTemplate {
        name: String,
        pubkey: String,
        unsigned_event: String,
        relays: Vec<String>,
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
        Ok(NewRecordsTemplate {
            name: query.name.unwrap_or_default(),
            pubkey: query.pubkey.map(|s| s.to_string()).unwrap_or_default(),
            unsigned_event: Default::default(),
            relays: state.config.relays(),
        })
    }

    #[derive(Deserialize, Debug)]
    pub struct NewRecordsForm {
        records: String,
        name: String,
        pubkey: XOnlyPublicKey,
    }

    pub async fn new_records_submit(
        State(state): State<AppState>,
        Form(form): Form<NewRecordsForm>,
    ) -> Result<NewRecordsTemplate, WebError> {
        let name: Name = form.name.parse()?;
        let records = form
            .records
            .lines()
            .map(|line| line.parse::<KeyVal>())
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
        })
    }
}

mod api {
    use std::collections::HashMap;

    use anyhow::anyhow;

    use axum::{
        extract::{Query, State},
        Json,
    };
    use serde::Deserialize;
    use sqlx::SqlitePool;

    use crate::db;

    use super::{AppState, WebError};

    #[derive(Deserialize)]
    pub struct NameQuery {
        name: String,
    }

    pub async fn name(
        Query(name): Query<NameQuery>,
        State(state): State<AppState>,
    ) -> Result<Json<HashMap<String, String>>, WebError> {
        let conn = state.pool;
        let name = db::name_records(&conn, name.name).await?;

        name.map(Json)
            .ok_or_else(|| WebError::not_found(anyhow!("Not found")))
    }
}

mod util {
    use time::{macros::format_description, OffsetDateTime};

    pub fn format_time(timestamp: i64) -> anyhow::Result<String> {
        let dt = OffsetDateTime::from_unix_timestamp(timestamp)?;
        let format = format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");
        Ok(dt.format(format)?)
    }
}
