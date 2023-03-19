use std::time::Duration;

use askama_axum::IntoResponse;
use axum::{http::StatusCode, routing::get, Router};
use sqlx::SqlitePool;

use crate::{config::Config, subcommands};

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
    without_explorer: bool,
    without_api: bool,
    without_indexer: bool,
) -> anyhow::Result<()> {
    if !without_indexer {
        let _indexer = tokio::spawn(indexer(config.clone(), conn.clone()));
    }
    let mut app = Router::new();

    if !without_explorer {
        app = app
            .route("/", get(site::index))
            .route("/faqs", get(site::faqs))
            .route("/explorer", get(site::explorer))
            .route("/explorer/:nsid", get(site::explore_nsid));
    }

    if !without_api {
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
        .await?;
    Ok(())
}

async fn indexer(config: Config, pool: SqlitePool) -> anyhow::Result<()> {
    let confirmations = config
        .server_confirmations()
        .expect("Cannot determine confirmations");
    let height = config.server_height();
    loop {
        subcommands::index_blockchain(&config, &pool, confirmations, height)
            .await
            .ok();
        subcommands::index_create_events(&config, &pool).await.ok();
        subcommands::index_records_events(&config, &pool).await.ok();

        tokio::time::sleep(Duration::from_secs(30)).await;
    }
}

mod site {
    use std::collections::HashMap;

    use anyhow::anyhow;
    use axum::extract::{Path, State};
    use serde::Deserialize;
    use sqlx::SqlitePool;

    use crate::db::{self, namespace::NamespaceDetails};

    use super::WebError;

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
    }

    pub async fn explorer(State(conn): State<SqlitePool>) -> Result<ExplorerTemplate, WebError> {
        Ok(ExplorerTemplate {
            names: db::top_level_names(&conn).await?,
        })
    }

    #[derive(askama::Template)]
    #[template(path = "nsid.html")]
    pub struct NsidTemplate {
        name: String,
        record_keys: Vec<String>,
        records: HashMap<String, String>,
        children: Vec<(String, String)>,
        blockhash: String,
        txid: String,
        vout: usize,
        height: usize,
    }

    impl From<NamespaceDetails> for NsidTemplate {
        fn from(value: NamespaceDetails) -> Self {
            let (blockhash, txid, vout, height) = value.blockdata.unwrap_or_default();
            let mut record_keys = value.records.keys().cloned().collect::<Vec<_>>();
            record_keys.sort();
            NsidTemplate {
                name: value.name.unwrap_or_default(),
                record_keys,
                records: value.records,
                children: value.children,
                blockhash,
                txid,
                vout,
                height,
            }
        }
    }

    pub async fn explore_nsid(
        State(conn): State<SqlitePool>,
        Path(nsid): Path<String>,
    ) -> Result<NsidTemplate, WebError> {
        let details = db::namespace::details(&conn, nsid).await?;
        if details.name.is_none() || details.blockdata.is_none() {
            log::error!("{details:?}");
            return Err(WebError::not_found(anyhow!("NSID not found")));
        }
        Ok(details.into())
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

        name
            .map(Json)
            .ok_or_else(|| WebError::not_found(anyhow!("Not found")))
    }
}
