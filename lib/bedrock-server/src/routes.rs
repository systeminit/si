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
use serde_json::{
    Value,
    json,
};
use si_data_ssm::ParameterStoreClientError;
use thiserror::Error;

use super::{
    app_state::AppState,
    server::ServerError,
};
use crate::api_error::ApiError;

pub mod profiles;
pub mod tests;
pub mod publish;
pub mod record;
pub mod prepare;

use crate::routes::{
    profiles::profiles_route,
    tests::execute_tests_route,
    prepare::prepare_route,
    publish::publish_route,
    record::start_recording_route,
    record::stop_recording_route,
};

async fn system_status_route() -> Json<Value> {
    dbg!("Health Requested");
    Json(json!({ "ok": true }))
}

pub fn public_routes(state: AppState) -> Router {
    Router::new()
        .route("/", get(system_status_route))
        .route("/profiles", get(profiles_route))
        .route("/tests", post(execute_tests_route))
        .route("/prepare", post(prepare_route))
        .route("/start", post(start_recording_route))
        .route("/stop", post(stop_recording_route))
        .route("/publish", post(publish_route))
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
