use billing_events::BillingEvent;
use data_warehouse_stream_client::DataWarehouseStreamClientError;
use naxum::{
    extract::State,
    response::{IntoResponse, Response},
    Json,
};
use si_data_nats::Subject;
use telemetry::prelude::*;
use thiserror::Error;

use crate::{app_state::AppState, app_state::NoopAppState};

#[remain::sorted]
#[derive(Debug, Error)]
pub(crate) enum HandlerError {
    #[error("data warehouse stream client error: {0}")]
    DataWarehouseStreamClient(#[from] DataWarehouseStreamClientError),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
}

type HandlerResult<T> = Result<T, HandlerError>;

impl IntoResponse for HandlerError {
    fn into_response(self) -> Response {
        error!(si.error.message = ?self, "failed to process message");
        Response::default_internal_server_error()
    }
}

pub(crate) async fn process_request(
    State(state): State<AppState>,
    _subject: Subject,
    Json(request): Json<BillingEvent>,
) -> HandlerResult<()> {
    let span = Span::current();

    span.record("si.workspace.id", request.workspace_id.to_string());
    span.record("si.change_set.id", request.change_set_id.to_string());

    let serialized_request = serde_json::to_vec(&request)?;
    state
        .data_warehouse_stream_client
        .publish(serialized_request)
        .await?;

    info!(kind = ?request.kind, ?request, "processed billing event");
    Ok(())
}

pub(crate) async fn process_request_noop(
    State(_state): State<NoopAppState>,
    _subject: Subject,
    Json(request): Json<BillingEvent>,
) -> HandlerResult<()> {
    let span = Span::current();

    span.record("si.workspace.id", request.workspace_id.to_string());
    span.record("si.change_set.id", request.change_set_id.to_string());

    info!(
        kind = ?request.kind,
        ?request,
        "received and processed billing event (no-op mode)"
    );
    Ok(())
}
