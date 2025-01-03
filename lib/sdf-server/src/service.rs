use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Serialize, Serializer};
use std::fmt::Display;
use telemetry::prelude::*;
use tracing_tunnel::TracingLevel;

pub mod action;
pub mod async_route;
pub mod attribute;
pub mod change_set;
pub mod component;
pub mod diagram;
pub mod force_change_set_response;
pub mod graphviz;
pub mod module;
pub mod node_debug;
pub mod qualification;
pub mod secret;
pub mod session;
pub mod v2;
pub mod variant;
pub mod whoami;
pub mod ws;

/// A module containing dev routes for local development only.
#[cfg(debug_assertions)]
pub mod dev;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ApiError {
    error: ApiErrorError,
    level: Option<TracingLevel>,
}

impl ApiError {
    const DEFAULT_ERROR_STATUS_CODE: StatusCode = StatusCode::INTERNAL_SERVER_ERROR;

    fn new<E: Display>(status_code: StatusCode, err: E) -> Self {
        Self {
            error: ApiErrorError {
                message: err.to_string(),
                status_code,
            },
            level: None,
        }
    }

    // keeping this here to allow for future use
    #[allow(dead_code)]
    fn with_level(mut self, level: TracingLevel) -> Self {
        self.level = Some(level);
        self
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self.level {
            Some(TracingLevel::Info) => {
                info!(err=?self, ?self.error.status_code, ?self.error.status_code, self.error.message )
            }
            Some(TracingLevel::Debug) => {
                debug!(err=?self, ?self.error.status_code, ?self.error.status_code, self.error.message )
            }
            Some(TracingLevel::Error) => {
                error!(err=?self, ?self.error.status_code, ?self.error.status_code, self.error.message )
            }
            Some(TracingLevel::Trace) => {
                trace!(err=?self, ?self.error.status_code, ?self.error.status_code, self.error.message )
            }
            Some(TracingLevel::Warn) => {
                warn!(err=?self, ?self.error.status_code, ?self.error.status_code, self.error.message )
            }
            None => {
                if self.error.status_code.is_client_error() {
                    warn!(err=?self, ?self.error.status_code, ?self.error.status_code, self.error.message );
                } else if self.error.status_code.is_server_error() {
                    error!(err=?self, ?self.error.status_code, ?self.error.status_code, self.error.message );
                }
            }
        }

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

macro_rules! impl_default_error_into_response {
    (
        $(#[$($attrss:tt)*])*
        $error_type:ident
    ) => {
        impl ::axum::response::IntoResponse for $error_type {
            fn into_response(self) -> ::axum::response::Response {
                let (status, error_message) = (
                    ::axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    self.to_string(),
                );

                let body = ::axum::Json(
                    ::serde_json::json!({
                        "error": {
                            "message": error_message,
                            "code": 42,
                            "statusCode": status.as_u16()
                        }
                    }),
                );

                if status.is_client_error() {
                        ::telemetry::prelude::tracing::warn!(si.error.message = error_message);
                } else if status.is_server_error() {
                        ::telemetry::prelude::tracing::warn!(si.error.message = error_message);
                }

                (status, body).into_response()
            }
        }
    };
}

pub(crate) use impl_default_error_into_response;
