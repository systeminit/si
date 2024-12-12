//! Application level error wrapper that handles converting to/from [`anyhow::Error`].

use naxum::response::{IntoResponse, Response};
use telemetry::prelude::*;
use telemetry_utils::metric;

use crate::handlers::HandlerError;

/// Wrapper around [`anyhow::Error`] that implements [`IntoResponse`] to allow
/// using [`anyhow::Error`] in handler methods.
#[derive(Debug)]
pub struct AppError(anyhow::Error);

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        if let Some(handler_err) = self.0.downcast_ref::<HandlerError>() {
            metric!(counter.change_set_processor_task.failed_rebase = 1);
            match handler_err {
                HandlerError::SubjectParse(_, _) => {
                    warn!(si.error.message = ?self.0, "subject parse error");
                    Response::default_bad_request()
                }
                HandlerError::TaskInterrupted(subject) => {
                    debug!(subject, "task interrupted");
                    Response::default_service_unavailable()
                }
                _ => {
                    // TODO(fnichol): there are different responses, esp. for expected interrupted
                    error!(si.error.message = ?self.0, "failed to process message");
                    Response::default_internal_server_error()
                }
            }
        } else {
            // TODO(fnichol): there are different responses, esp. for expected interrupted
            error!(si.error.message = ?self.0, "failed to process message");
            Response::default_internal_server_error()
        }
    }
}
