use std::time::Duration;

use askama_axum::IntoResponse;
use axum::{
    http::StatusCode,
    routing::{get, post},
    Router,
};
use sqlx::SqlitePool;

use crate::{
    config::{Config, ServerSubcommand},
    subcommands,
};

pub struct WebError(anyhow::Error, Option<StatusCode>);

impl WebError {
    pub fn not_found(err: anyhow::Error) -> WebError {
        WebError(err, Some(StatusCode::NOT_FOUND))
    }
}

impl IntoResponse for WebError {
    fn into_response(self) -> askama_axum::Response {
        (
            self.1.unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            format!("Something went wrong: {}", self.0),
        )
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

pub async fn start(
    config: &Config,
    conn: &SqlitePool,
    server: &ServerSubcommand,
) -> anyhow::Result<()> {
    if !server.without_indexer {
        let _indexer = tokio::spawn(indexer(config.clone(), conn.clone()));
    }
    let mut app = Router::new();

    if !server.without_explorer {
        app = app
            .route("/", get(site::index))
            .route("/faqs", get(site::faqs))
            .route("/explorer", get(site::explorer))
            .route("/explorer/:nsid", get(site::explore_nsid))
            .route("/newname", get(site::new_name_form))
            .route("/newname", post(site::new_name_submit));
    }

    if !server.without_api {
        app = app.route("/api/name", get(api::name));
    }

    let app = app.with_state(conn.clone());

    let addr = config
        .server_bind()
        .expect("Server bind unconfigured")
        .parse()?;

    log::info!("Starting server on {addr}");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(elegant_departure::tokio::depart().on_termination())
        .await?;
    Ok(())
}

async fn indexer(config: Config, pool: SqlitePool) -> anyhow::Result<()> {
    let guard = elegant_departure::get_shutdown_guard();
    loop {
        subcommands::index(&config, &pool).await?;
        tokio::select! {
            _ = tokio::time::sleep(Duration::from_secs(30)) => {},
            _ = guard.wait() => return Ok(())
        }
    }
}

mod site {
    use std::collections::HashMap;

    use anyhow::{anyhow, bail};
    use axum::{
        extract::{Path, Query, State},
        Form,
    };
    use bitcoin::{Address, Txid};
    use itertools::Itertools;
    use serde::Deserialize;
    use sqlx::SqlitePool;

    use crate::db::{self, NameDetails};

    use super::{util, WebError};

    #[derive(askama::Template)]
    #[template(path = "index.html")]
    pub struct IndexTemplate {}

    pub async fn index() -> IndexTemplate {
        IndexTemplate {}
    }

    #[derive(askama::Template)]
    #[template(path = "faqs.html")]
    pub struct FaqsTemplate {}

    pub async fn faqs() -> FaqsTemplate {
        FaqsTemplate {}
    }

    #[derive(Deserialize)]
    pub struct ExplorerQuery {
        pub nsid: Option<String>,
    }

    #[derive(askama::Template)]
    #[template(path = "explorer.html")]
    pub struct ExplorerTemplate {
        names: Vec<(String, String)>,
        last_index_time: String,
    }

    pub async fn explorer(State(conn): State<SqlitePool>) -> Result<ExplorerTemplate, WebError> {
        let last_index_time = db::last_index_time(&conn).await?;
        let last_index_time = util::format_time(last_index_time)?;

        Ok(ExplorerTemplate {
            names: db::top_level_names(&conn).await?,
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
        State(conn): State<SqlitePool>,
        Path(nsid): Path<String>,
    ) -> Result<NsidTemplate, WebError> {
        let details = db::name_details(&conn, nsid.parse()?).await?;

        Ok(details.try_into()?)
    }

    #[derive(askama::Template, Default)]
    #[template(path = "newname.html")]
    pub struct NewNameTemplate {
        txid: String,
        vout: String,
        name: String,
        address: String,
        error: String,
    }

    // impl TryFrom<NewNameForm> for NewNameTemplate {
    //     type Error = anyhow::Error;

    //     fn try_from(value: NewNameForm) -> Result<Self, Self::Error> {
    //         Ok(NewNameTemplate {
    //             txid: value.txid.unwrap_or_default(),
    //             vout: value.vout.unwrap_or_default(),
    //             name: value.name.unwrap_or_default(),
    //             address: value.address.unwrap_or_default(),
    //             error: Default::default(),
    //         })
    //     }
    // }

    #[derive(Deserialize)]
    pub struct NewNameForm {
        txid: Txid,
        vout: u64,
        name: String,
        address: Address,
    }

    pub async fn new_name_form() -> Result<NewNameTemplate, WebError> {
        Ok(Default::default())
    }

    pub async fn new_name_submit(
        Form(form): Form<NewNameForm>,
    ) -> Result<NewNameTemplate, WebError> {
        Ok(NewNameTemplate {
            txid: form.txid.to_string(),
            vout: form.vout.to_string(),
            name: form.name,
            address: form.address.to_string(),
            error: String::new(),
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

    use super::WebError;

    #[derive(Deserialize)]
    pub struct NameQuery {
        name: String,
    }

    pub async fn name(
        Query(name): Query<NameQuery>,
        State(conn): State<SqlitePool>,
    ) -> Result<Json<HashMap<String, String>>, WebError> {
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
