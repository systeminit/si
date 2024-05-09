//! Application handlers for change set request tasks.

use std::result;

use dal::{
    Tenancy, Visibility, Workspace, WorkspaceError, WorkspacePk, WorkspaceSnapshot,
    WorkspaceSnapshotError, WsEvent,
};
use naxum::{
    extract::State,
    response::{IntoResponse, Response},
};
use si_data_nats::InnerMessage;
use si_layer_cache::{
    activities::{
        rebase::{RebaseStatus, RebaseStatusDiscriminants},
        Activity, ActivityRebaseRequest,
    },
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
    #[error("Workspace error: {0}")]
    /// When failing to find the workspace
    Workspace(#[from] WorkspaceError),
    /// When failing to do an operation using the [`WorkspaceSnapshot`]
    #[error("Workspace Snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
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
    ctx.update_visibility_deprecated(Visibility::new_head_fake());
    ctx.update_tenancy(Tenancy::new(WorkspacePk::NONE));

    let rebase_status = perform_rebase(&mut ctx, &message)
        .await
        .unwrap_or_else(|err| {
            error!(error = ?err, ?message, "performing rebase failed, attempting to reply");
            RebaseStatus::Error {
                message: err.to_string(),
            }
        });

    // Dispatch eligible actions if the change set is the default for the workspace.
    // Actions are **ONLY** ever dispatched from the default change set for a workspace.
    if RebaseStatusDiscriminants::Success == rebase_status.clone().into() {
        if let Some(workspace_pk) = ctx.tenancy().workspace_pk() {
            if let Some(workspace) = Workspace::get_by_pk(&ctx, &workspace_pk).await? {
                if let Ok(change_set) = ctx.change_set() {
                    if workspace.default_change_set_id() == change_set.id {
                        WorkspaceSnapshot::dispatch_actions(&ctx).await?;
                    }
                }
            }
        }
    }

    ctx.layer_db()
        .activity()
        .rebase()
        .finished(
            rebase_status.clone(),
            message.payload.to_rebase_change_set_id,
            message.payload.onto_workspace_snapshot_address,
            message.metadata.clone(),
            message.id,
        )
        .await
        .map_err(HandlerError::SendRebaseFinished)?;

    // only enqueue values if rebase succeeded. if it failed, there's no work to do
    if let RebaseStatus::Success { .. } = rebase_status {
        if let Some(values) = message.payload.dvu_values {
            state.dvu_debouncer.enqueue_values(values);
        }
    }

    let mut event =
        WsEvent::change_set_written(&ctx, message.payload.to_rebase_change_set_id.into()).await?;
    event.set_workspace_pk(message.metadata.tenancy.workspace_pk.into_inner().into());
    event.set_change_set_id(Some(message.payload.to_rebase_change_set_id.into()));
    event.publish_immediately(&ctx).await?;

    Ok(())
}
