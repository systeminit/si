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
        put,
    },
};
use hyper::StatusCode;
use serde_json::{
    Value,
    json,
};
use si_data_ssm::ParameterStoreClientError;
use thiserror::Error;

mod clear_cache;
mod create_parameter;
mod get_parameter;
mod list_parameters;

use super::{
    app_state::AppState,
    server::ServerError,
};
use crate::api_error::ApiError;

pub fn public_routes(state: AppState) -> Router {
    Router::new()
        .route("/", get(system_status_route))
        .with_state(state)
}

pub fn protected_routes(state: AppState) -> Router {
    Router::new()
        .route("/cache/clear", post(clear_cache::clear_cache_route))
        .route("/parameter/*path", get(get_parameter::get_parameter_route))
        .route(
            "/parameter/*path",
            put(create_parameter::create_parameter_route),
        )
        .route(
            "/parameters/*path",
            get(list_parameters::list_parameters_route),
        )
        .with_state(state)
}

async fn system_status_route() -> Json<Value> {
    Json(json!({ "ok": true }))
}

#[allow(clippy::large_enum_variant)]
#[remain::sorted]
#[derive(Debug, Error)]
pub enum AppError {
    #[error("parameter store client error: {0}")]
    ParameterStoreClient(#[from] ParameterStoreClientError),
    #[error("server error: {0}")]
    Server(#[from] ServerError),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status_code, error_message) = (StatusCode::INTERNAL_SERVER_ERROR, self.to_string());

        ApiError::new(status_code, error_message).into_response()
    }
}
