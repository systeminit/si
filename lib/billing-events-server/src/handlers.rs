use billing_events::BillingWorkspaceChangeEvent;
use data_warehouse_stream_client::DataWarehouseStreamClientError;
use naxum::{
    extract::State,
    response::{IntoResponse, Response},
    Json,
};
use telemetry::prelude::*;
use thiserror::Error;

use crate::AppState;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum HandlerError {
    #[error("data warehouse stream client error: {0}")]
    DataWarehouseStreamClient(#[from] DataWarehouseStreamClientError),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
}

type HandlerResult<T> = Result<T, HandlerError>;

impl IntoResponse for HandlerError {
    fn into_response(self) -> Response {
        error!(si.error.message = ?self, "failed to process message");
        Response::server_error()
    }
}

pub async fn process_request(
    State(state): State<AppState>,
    Json(request): Json<BillingWorkspaceChangeEvent>,
) -> HandlerResult<()> {
    info!(?request, "receieved billing workspace change event");

    let serialized_request = serde_json::to_vec(&request)?;
    state
        .data_warehouse_stream_client
        .publish(serialized_request)
        .await?;

    Ok(())
}

pub async fn process_request_noop(
    Json(request): Json<BillingWorkspaceChangeEvent>,
) -> HandlerResult<()> {
    info!(
        ?request,
        "receieved billing workspace change event (no-op mode)"
    );
    Ok(())
}
