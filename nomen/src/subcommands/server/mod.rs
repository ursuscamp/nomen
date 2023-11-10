mod api;
mod explorer;

use std::time::Duration;

use askama_axum::IntoResponse;
use axum::{
    http::StatusCode,
    routing::{get, post},
    Router,
};
use sqlx::SqlitePool;
use tokio::time::{interval, MissedTickBehavior};
use tower_http::cors::{Any, CorsLayer};

use crate::{config::Config, subcommands};

use self::explorer::ErrorTemplate;

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
            .route("/", get(explorer::index))
            .route("/explorer", get(explorer::explorer))
            .route("/explorer/:nsid", get(explorer::show_name))
            .route("/newname", get(explorer::new_name_form))
            .route("/newname", post(explorer::new_name_submit))
            .route("/updaterecords", get(explorer::new_records_form))
            .route("/updaterecords", post(explorer::new_records_submit))
            .route("/transfer", get(explorer::transfer::initiate))
            .route("/transfer", post(explorer::transfer::submit_initiate))
            .route("/transfer/sign", post(explorer::transfer::complete))
            .route("/stats", get(explorer::index_stats));
    }

    if config.well_known() {
        app = app.route("/.well-known/nomen.json", get(explorer::well_known::nomen));
    }

    if config.api() {
        let api_router = Router::new()
            .route("/names", get(api::names))
            .route("/name", get(api::name))
            .route("/create/data", get(api::op_return_v1))
            .route("/v0/create/data", get(api::op_return_v0))
            .route("/transfer/event", get(api::get_transfer_event))
            .route("/transfer/data", get(api::get_transfer))
            .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any));
        app = app.nest("/api", api_router);
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
