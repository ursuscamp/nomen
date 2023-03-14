use axum::{routing::get, Router};

use crate::config::Config;

pub async fn start(config: &Config) -> anyhow::Result<()> {
    let app = Router::new().route("/api/name", get(name));

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

async fn name() -> &'static str {
    "Hello world!"
}
