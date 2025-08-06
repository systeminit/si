use serde::{
    Deserialize,
    Serialize,
};
use si_events::WorkspaceSnapshotAddress;

use crate::{
    ChangeSetId,
    ChangeSetStatus,
    DalContext,
    UserPk,
    WsEvent,
    WsEventResult,
    WsPayload,
};

impl WsEvent {
    pub async fn change_set_written(
        ctx: &DalContext,
        change_set_id: ChangeSetId,
    ) -> WsEventResult<Self> {
        WsEvent::new(ctx, WsPayload::ChangeSetWritten(change_set_id)).await
    }

    pub async fn change_set_created(
        ctx: &DalContext,
        change_set_id: ChangeSetId,
        workspace_snapshot_address: WorkspaceSnapshotAddress,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ChangeSetCreated(ChangeSetCreatedPayload {
                change_set_id,
                workspace_snapshot_address,
            }),
        )
        .await
    }

    pub async fn change_set_approval_status_changed(
        ctx: &DalContext,
        change_set_id: ChangeSetId,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ChangeSetApprovalStatusChanged(change_set_id),
        )
        .await
    }

    pub async fn change_set_status_changed(
        ctx: &DalContext,
        from_status: ChangeSetStatus,
        change_set: si_frontend_types::ChangeSet,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ChangeSetStatusChanged(ChangeSetStateChangePayload {
                from_status,
                change_set,
            }),
        )
        .await
    }

    pub async fn change_set_abandoned(
        ctx: &DalContext,
        change_set_id: ChangeSetId,
        user_pk: Option<UserPk>,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ChangeSetAbandoned(ChangeSetActorPayload {
                change_set_id,
                user_pk,
            }),
        )
        .await
    }

    pub async fn change_set_applied(
        ctx: &DalContext,
        change_set_id: ChangeSetId,
        to_rebase_change_set_id: ChangeSetId,
        user_pk: Option<UserPk>,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ChangeSetApplied(ChangeSetAppliedPayload {
                change_set_id,
                to_rebase_change_set_id,
                user_pk,
            }),
        )
        .await
    }

    pub async fn change_set_canceled(
        ctx: &DalContext,
        change_set_id: ChangeSetId,
    ) -> WsEventResult<Self> {
        WsEvent::new(ctx, WsPayload::ChangeSetCanceled(change_set_id)).await
    }

    pub async fn rename_change_set(
        ctx: &DalContext,
        change_set_id: ChangeSetId,
        new_name: String,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ChangeSetRename(ChangeSetRenamePayload {
                change_set_id,
                new_name,
            }),
        )
        .await
    }
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ChangeSetActorPayload {
    change_set_id: ChangeSetId,
    user_pk: Option<UserPk>,
}
#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ChangeSetStateChangePayload {
    from_status: ChangeSetStatus,
    change_set: si_frontend_types::ChangeSet,
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ChangeSetAppliedPayload {
    change_set_id: ChangeSetId,
    to_rebase_change_set_id: ChangeSetId,
    user_pk: Option<UserPk>,
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ChangeSetRenamePayload {
    change_set_id: ChangeSetId,
    new_name: String,
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ChangeSetCreatedPayload {
    change_set_id: ChangeSetId,
    workspace_snapshot_address: WorkspaceSnapshotAddress,
}
