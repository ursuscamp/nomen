use axum::{
    extract::{Query, State},
    Json,
};
use nomen_core::CreateBuilder;

use crate::db;

use super::{AppState, WebError};

mod models {
    use std::collections::HashMap;

    use askama_axum::IntoResponse;
    use axum::{http::StatusCode, Json};
    use secp256k1::XOnlyPublicKey;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize)]
    pub struct JsonError {
        pub error: String,
        #[serde(skip)]
        pub status: StatusCode,
    }

    impl JsonError {
        pub fn message(err: &str) -> JsonError {
            JsonError {
                error: err.into(),
                status: StatusCode::BAD_REQUEST,
            }
        }
    }

    impl IntoResponse for JsonError {
        fn into_response(self) -> askama_axum::Response {
            (self.status, Json(self)).into_response()
        }
    }

    impl From<anyhow::Error> for JsonError {
        fn from(value: anyhow::Error) -> Self {
            JsonError {
                error: value.to_string(),
                status: StatusCode::INTERNAL_SERVER_ERROR,
            }
        }
    }

    #[derive(Deserialize)]
    pub struct NameQuery {
        pub name: String,
    }

    #[derive(Serialize)]
    pub struct NameResult {
        pub records: HashMap<String, String>,
    }

    #[derive(Deserialize)]
    pub struct OpReturnQuery {
        pub name: String,
        pub pubkey: XOnlyPublicKey,
    }

    #[derive(Serialize, Default)]
    pub struct OpReturnResponse {
        pub op_return: Vec<String>,
    }
}

pub async fn name(
    Query(name): Query<models::NameQuery>,
    State(state): State<AppState>,
) -> Result<Json<models::NameResult>, models::JsonError> {
    // TODO: return some metdata as well
    let conn = state.pool;
    let name = db::name_records(&conn, name.name).await?;

    name.map(|records| models::NameResult { records })
        .map(Json)
        .ok_or_else(|| models::JsonError::message("Name not found"))
}

#[allow(clippy::unused_async)]
pub async fn op_return_v1(
    Query(query): Query<models::OpReturnQuery>,
) -> Result<Json<models::OpReturnResponse>, WebError> {
    // TODO: validate name length and format
    let bytes = CreateBuilder::new(&query.pubkey, &query.name).v1_op_return();
    let orr = models::OpReturnResponse {
        op_return: vec![hex::encode(bytes)],
    };

    Ok(Json(orr))
}
