use std::{collections::HashMap, time::Duration};

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

pub async fn start(config: &Config) -> anyhow::Result<()> {
    let _indexer = tokio::spawn(indexer(config.clone()));
    let conn = config.sqlite().await?;
    let app = Router::new().route("/api/name", get(name)).with_state(conn);

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

#[derive(Deserialize)]
struct NameQuery {
    name: String,
}

async fn name(
    Query(name): Query<NameQuery>,
    State(conn): State<Connection>,
) -> Result<Json<HashMap<String, String>>, StatusCode> {
    let name = db::name_records(&conn, name.name)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    match name {
        Some(map) => Ok(Json(map)),
        None => Err(StatusCode::NOT_FOUND),
    }
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
