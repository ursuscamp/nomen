use std::{collections::HashMap, time::Duration};

use askama_axum::IntoResponse;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tokio_rusqlite::Connection;
use toml::map;

use crate::{config::Config, db, subcommands};

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

pub async fn start(config: &Config) -> anyhow::Result<()> {
    let _indexer = tokio::spawn(indexer(config.clone()));
    let conn = config.sqlite().await?;
    let app = Router::new()
        .route("/", get(site::index))
        .route("/faqs", get(site::faqs))
        .route("/explorer", get(site::explorer))
        .route("/explorer/:nsid", get(site::explore_nsid))
        .route("/api/name", get(api::name))
        .with_state(conn);

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

async fn indexer(config: Config) -> anyhow::Result<()> {
    loop {
        subcommands::index_blockchain(&config, 3, None).await;
        subcommands::index_create_events(&config).await;
        subcommands::index_records_events(&config).await;

        tokio::time::sleep(Duration::from_secs(30)).await;
    }

    Ok(())
}

mod site {
    use std::collections::HashMap;

    use anyhow::anyhow;
    use axum::{
        extract::{Path, Query, State},
        http::StatusCode,
    };
    use serde::Deserialize;
    use tokio_rusqlite::Connection;

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

    pub async fn explorer(State(conn): State<Connection>) -> Result<ExplorerTemplate, WebError> {
        Ok(ExplorerTemplate {
            names: db::top_level_names(&conn).await?,
        })
    }

    #[derive(askama::Template)]
    #[template(path = "nsid.html")]
    pub struct NsidTemplate {
        name: String,
        records: HashMap<String, String>,
        children: Vec<(String, String)>,
    }

    impl From<NamespaceDetails> for NsidTemplate {
        fn from(value: NamespaceDetails) -> Self {
            NsidTemplate {
                name: value.name.unwrap_or_default(),
                records: value.records,
                children: value.children,
            }
        }
    }

    pub async fn explore_nsid(
        State(conn): State<Connection>,
        Path(nsid): Path<String>,
    ) -> Result<NsidTemplate, WebError> {
        let details = db::namespace::details(&conn, nsid).await?;
        if details.name.is_none() {
            return Err(WebError::not_found(anyhow!("NSID not found")));
        }
        Ok(details.into())
    }
}

mod api {
    use std::collections::HashMap;

    use anyhow::anyhow;
    use askama_axum::IntoResponse;
    use axum::{
        extract::{Query, State},
        http::StatusCode,
        Json,
    };
    use serde::Deserialize;
    use tokio_rusqlite::Connection;

    use crate::db;

    use super::WebError;

    #[derive(Deserialize)]
    pub struct NameQuery {
        name: String,
    }

    pub async fn name(
        Query(name): Query<NameQuery>,
        State(conn): State<Connection>,
    ) -> Result<Json<HashMap<String, String>>, WebError> {
        let name = db::name_records(&conn, name.name).await?;

        Ok(name
            .map(|m| Json(m))
            .ok_or_else(|| WebError::not_found(anyhow!("Not found")))?)
    }
}
