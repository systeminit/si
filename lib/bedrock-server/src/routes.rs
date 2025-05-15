use axum::{
    Router,
    response::{
        IntoResponse,
        Json,
        Response,
    },
    routing::{
        get,
        post,
    },
};
use hyper::StatusCode;
use serde_json::Value;
use serde_json::json;
use si_data_ssm::ParameterStoreClientError;
use thiserror::Error;

use super::{
    app_state::AppState,
    server::ServerError,
};
use crate::api_error::ApiError;

pub mod profiles;
pub mod tests;
use crate::routes::profiles::{
    profiles_route,
};
use crate::routes::tests::{
    tests_route,
};

async fn system_status_route() -> Json<Value> {
    dbg!("Health Requested");
    Json(json!({ "ok": true }))
}

pub fn public_routes(state: AppState) -> Router {
    Router::new()
        .route("/", get(system_status_route))
        .route("/profiles", get(profiles_route))
        .route("/tests", post(tests_route))
        .with_state(state)
}

#[allow(clippy::large_enum_variant)]
#[remain::sorted]
#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    ParameterStoreClient(#[from] ParameterStoreClientError),
    #[error(transparent)]
    Server(#[from] ServerError),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status_code, error_message) = (StatusCode::INTERNAL_SERVER_ERROR, self.to_string());

        ApiError::new(status_code, error_message).into_response()
    }
}
