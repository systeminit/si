use std::fmt::Display;

use axum::{
    Json,
    http::StatusCode,
    response::{
        IntoResponse,
        Response,
    },
};
use serde::{
    Serialize,
    Serializer,
};
use telemetry::prelude::*;
use tracing_tunnel::TracingLevel;

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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiError {
    error: ApiErrorError,
    level: Option<TracingLevel>,
}

impl ApiError {
    pub fn new<E: Display>(status_code: StatusCode, err: E) -> Self {
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
