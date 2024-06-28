use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Serialize, Serializer};

pub mod action;
pub mod async_route;
pub mod attribute;
pub mod change_set;
pub mod component;
pub mod diagram;
pub mod func;
pub mod graphviz;
pub mod module;
pub mod node_debug;
pub mod qualification;
pub mod secret;
pub mod session;
pub mod v2;
pub mod variant;
pub mod ws;

/// A module containing dev routes for local development only.
#[cfg(debug_assertions)]
pub mod dev;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ApiError {
    error: ApiErrorError,
}

impl ApiError {
    const DEFAULT_ERROR_STATUS_CODE: StatusCode = StatusCode::INTERNAL_SERVER_ERROR;

    fn new<E>(status_code: StatusCode, err: E) -> Self
    where
        E: std::error::Error,
    {
        Self {
            error: ApiErrorError {
                message: err.to_string(),
                status_code,
            },
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.error.status_code, Json(self)).into_response()
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ApiErrorError {
    message: String,
    #[serde(serialize_with = "status_code_to_u16")]
    status_code: StatusCode,
}

fn status_code_to_u16<S>(status_code: &StatusCode, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_u16(status_code.as_u16())
}
