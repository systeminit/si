use billing_events::BillingWorkspaceChangeEvent;
use naxum::{
    extract::State,
    response::{IntoResponse, Response},
    Json,
};
use telemetry::prelude::*;
use thiserror::Error;

use crate::AppState;

// NOTE(nick,jkeiser): we will likely have fallible contents soon, so let's keep this for now.
#[remain::sorted]
#[derive(Debug, Error)]
pub enum HandlerError {}

type HandlerResult<T> = Result<T, HandlerError>;

impl IntoResponse for HandlerError {
    fn into_response(self) -> Response {
        error!(si.error.message = ?self, "failed to process message");
        Response::server_error()
    }
}

pub async fn process_request(
    State(_state): State<AppState>,
    Json(request): Json<BillingWorkspaceChangeEvent>,
) -> HandlerResult<()> {
    debug!(?request, "receieved billing workspace change event");
    Ok(())
}
