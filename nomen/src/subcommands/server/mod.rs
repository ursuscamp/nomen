mod api;

use std::time::Duration;

use askama_axum::IntoResponse;
use axum::{
    http::StatusCode,
    routing::{get, post},
    Router,
};
use sqlx::SqlitePool;
use tokio::time::{interval, MissedTickBehavior};

use crate::{config::Config, subcommands};

use self::site::ErrorTemplate;

pub struct WebError(anyhow::Error, Option<StatusCode>);

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

pub async fn start(config: &Config, conn: &SqlitePool) -> anyhow::Result<()> {
    if config.indexer() {
        let _indexer = tokio::spawn(indexer(config.clone()));
    }
    let mut app = Router::new();

    if config.explorer() {
        app = app
            .route("/", get(site::index))
            .route("/explorer", get(site::explorer))
            .route("/explorer/:nsid", get(site::explore_nsid))
            .route("/newname", get(site::new_name_form))
            .route("/newname", post(site::new_name_submit))
            .route("/updaterecords", get(site::new_records_form))
            .route("/updaterecords", post(site::new_records_submit))
            .route("/stats", get(site::index_stats))
            .route("/uncorroborated_claims", get(site::uncorroborated_claims))
            .route(
                "/uncorroborated_claims/:txid",
                get(site::uncorroborated_claim),
            );
    }

    if config.api() {
        app = app
            .route("/api/name", get(api::name))
            .route("/api/op_return", get(api::op_return_v1));
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

    tracing::info!("Starting server on {addr}");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(elegant_departure::tokio::depart().on_termination())
        .await?;

    tracing::info!("Server shutdown complete.");
    elegant_departure::shutdown().await;
    Ok(())
}

async fn indexer(config: Config) -> anyhow::Result<()> {
    let mut interval = interval(Duration::from_secs(config.server_indexer_delay()));
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

    loop {
        match subcommands::index(&config).await {
            Ok(_) => {}
            Err(err) => tracing::error!("Indexing error: {}", err),
        }
        interval.tick().await;
    }
}

mod site {
    use std::collections::HashMap;

    use axum::{
        extract::{Path, Query, State},
        Form,
    };
    use axum_extra::extract::WithRejection;
    use bitcoin::psbt::Psbt;
    use itertools::Itertools;
    use secp256k1::XOnlyPublicKey;
    use serde::Deserialize;

    use crate::{
        db::{self, NameDetails},
        subcommands::util::{insert_outputs, name_event},
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
        let mut psbt: Psbt = form.psbt.parse()?;
        insert_outputs(&mut psbt, &form.pubkey, &form.name)?;
        Ok(NewNameTemplate {
            psbt: psbt.to_string(),
            name: form.name,
            pubkey: form.pubkey.to_string(),
            confirmations: state.config.confirmations(),
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

    async fn records_from_query(
        query: &NewRecordsQuery,
        state: &AppState,
    ) -> Result<String, WebError> {
        let records = match &query.name {
            Some(name) => {
                let (records,) = sqlx::query_as::<_, (String,)>(
                    "SELECT records FROM valid_names_vw WHERE name = ?;",
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

    #[derive(askama::Template)]
    #[template(path = "uncorroborated_claims.html")]
    pub struct UncorroboratedClaims {
        claims: Vec<String>,
    }

    pub async fn uncorroborated_claims(
        State(state): State<AppState>,
    ) -> Result<UncorroboratedClaims, WebError> {
        Ok(UncorroboratedClaims {
            claims: db::uncorroborated_claims(&state.pool).await?,
        })
    }

    #[derive(askama::Template)]
    #[template(path = "uncorroborated_claim.html")]
    pub struct UncorroboratedClaim {
        claim: db::UncorroboratedClaim,
        blocktime: String,
        indexed_at: String,
    }

    pub async fn uncorroborated_claim(
        State(state): State<AppState>,
        Path(nsid): Path<String>,
    ) -> Result<UncorroboratedClaim, WebError> {
        let claim = db::uncorroborated_claim(&state.pool, &nsid).await?;
        let blocktime = claim.fmt_blocktime()?;
        let indexed_at = claim.fmt_indexed_at()?;
        Ok(UncorroboratedClaim {
            claim,
            blocktime,
            indexed_at,
        })
    }
}
