//! Application handlers for change set request tasks.

use std::result;

use dal::{Tenancy, Visibility, WorkspacePk, WsEvent};
use naxum::{
    extract::State,
    response::{IntoResponse, Response},
};
use si_data_nats::InnerMessage;
use si_layer_cache::{
    activities::{rebase::RebaseStatus, Activity, ActivityRebaseRequest},
    db::serialize,
};
use telemetry::prelude::*;
use thiserror::Error;

use crate::rebase::perform_rebase;

use super::app_state::AppState;

/// An error that can occur when processing an [`ActivityRebaseRequest`] message.
#[remain::sorted]
#[derive(Debug, Error)]
pub enum HandlerError {
    /// When failing to create a DAL context
    #[error("error creating a dal ctx: {0}")]
    DalTransactions(#[from] dal::TransactionsError),
    /// When failing to deserialize a message from bytes
    #[error("failed to deserialize message from bytes: {0}")]
    Deserialize(#[source] si_layer_cache::LayerDbError),
    /// When failing to successfully send a "rebase finished" message
    #[error("failed to send rebase finished activity: {0}")]
    SendRebaseFinished(#[source] si_layer_cache::LayerDbError),
    /// When failing to send a [`WsEvent`]
    #[error("failed to construct ws event: {0}")]
    WsEvent(#[from] dal::WsEventError),
}

type Result<T> = result::Result<T, HandlerError>;

impl IntoResponse for HandlerError {
    fn into_response(self) -> Response {
        error!(error = ?self, "failed to process message");
        Response::server_error()
    }
}

/// Process an [`ActivityRebaseRequest`].
pub async fn process_request(State(state): State<AppState>, msg: InnerMessage) -> Result<()> {
    let activity =
        serialize::from_bytes::<Activity>(&msg.payload).map_err(HandlerError::Deserialize)?;
    let message: ActivityRebaseRequest = activity.try_into().map_err(HandlerError::Deserialize)?;

    let mut ctx = state.ctx_builder.build_default().await?;
    // TODO(fnichol): I'm about 95% sure that preparing the `ctx` is not necessary, but I
    // am explicitly copying implementation across from the last iteration
    ctx.update_visibility_deprecated(Visibility::new_head());
    ctx.update_tenancy(Tenancy::new(WorkspacePk::NONE));

    let rebase_status = perform_rebase(&mut ctx, &message)
        .await
        .unwrap_or_else(|err| {
            error!(error = ?err, ?message, "performing rebase failed, attempting to reply");
            RebaseStatus::Error {
                message: err.to_string(),
            }
        });

    ctx.layer_db()
        .activity()
        .rebase()
        .finished(
            rebase_status,
            message.payload.to_rebase_change_set_id,
            message.payload.onto_workspace_snapshot_address,
            message.metadata.clone(),
            message.id,
        )
        .await
        .map_err(HandlerError::SendRebaseFinished)?;

    let mut event =
        WsEvent::change_set_written(&ctx, message.payload.to_rebase_change_set_id.into()).await?;
    event.set_workspace_pk(message.metadata.tenancy.workspace_pk.into_inner().into());
    event.publish_immediately(&ctx).await?;

    Ok(())
}
