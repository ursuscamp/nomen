use std::collections::HashMap;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tokio_rusqlite::Connection;
use toml::map;

use crate::{config::Config, db};

pub async fn start(config: &Config) -> anyhow::Result<()> {
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
