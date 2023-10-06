use axum::{
    extract::{Query, State},
    Json,
};
use nomen_core::{CreateBuilder, TransferBuilder};

use crate::db;

use self::models::{OpReturnResponse, TransferEventResponse};

use super::{AppState, WebError};

mod models {
    use std::collections::HashMap;

    use askama_axum::IntoResponse;
    use axum::{http::StatusCode, Json};
    use nostr_sdk::UnsignedEvent;
    use secp256k1::{schnorr::Signature, XOnlyPublicKey};
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

    #[derive(Deserialize)]
    pub struct TransferEventQuery {
        pub name: String,
        pub new_owner: XOnlyPublicKey,
        pub old_owner: XOnlyPublicKey,
    }

    #[derive(Serialize)]
    pub struct TransferEventResponse {
        pub event: UnsignedEvent,
    }

    #[derive(Deserialize)]
    pub struct TransferQuery {
        pub name: String,
        pub new_owner: XOnlyPublicKey,
        pub signature: Signature,
    }

    #[derive(Serialize)]
    pub struct NameResponse {
        pub name: String,
        pub pubkey: String,
    }

    #[derive(Serialize)]
    pub struct NamesResponse {
        pub names: Vec<NameResponse>,
    }
}

pub async fn names(
    State(state): State<AppState>,
) -> Result<Json<models::NamesResponse>, models::JsonError> {
    let names = db::all_names(&state.pool)
        .await?
        .into_iter()
        .map(|(n, pk)| models::NameResponse {
            name: n,
            pubkey: pk,
        })
        .collect();
    Ok(Json(models::NamesResponse { names }))
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

#[allow(clippy::unused_async)]
pub async fn get_transfer_event(
    Query(query): Query<models::TransferEventQuery>,
) -> Result<Json<models::TransferEventResponse>, models::JsonError> {
    let tb = TransferBuilder {
        new_pubkey: &query.new_owner,
        name: &query.name,
    };
    Ok(Json(TransferEventResponse {
        event: tb.unsigned_event(&query.old_owner),
    }))
}

#[allow(clippy::unused_async)]
pub async fn get_transfer(
    Query(query): Query<models::TransferQuery>,
) -> Result<Json<models::OpReturnResponse>, models::JsonError> {
    let tb = TransferBuilder {
        new_pubkey: &query.new_owner,
        name: &query.name,
    };
    let or1 = hex::encode(tb.transfer_op_return());
    let or2 = hex::encode(tb.signature_provided_op_return(query.signature));
    Ok(Json(OpReturnResponse {
        op_return: vec![or1, or2],
    }))
}
