use serde::{Deserialize, Serialize};

use crate::{ChangeSetId, DalContext, UserPk, WsEvent, WsEventResult, WsPayload};

impl WsEvent {
    pub async fn change_set_created(
        ctx: &DalContext,
        change_set_id: ChangeSetId,
    ) -> WsEventResult<Self> {
        WsEvent::new(ctx, WsPayload::ChangeSetCreated(change_set_id)).await
    }

    pub async fn change_set_written(ctx: &DalContext) -> WsEventResult<Self> {
        WsEvent::new(ctx, WsPayload::ChangeSetWritten(ctx.change_set_id())).await
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
        user_pk: Option<UserPk>,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ChangeSetApplied(ChangeSetActorPayload {
                change_set_id,
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

    pub async fn change_set_merge_vote(
        ctx: &DalContext,
        change_set_id: ChangeSetId,
        user_pk: Option<UserPk>,
        vote: String,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ChangeSetMergeVote(ChangeSetMergeVotePayload {
                change_set_id,
                user_pk,
                vote,
            }),
        )
        .await
    }

    pub async fn change_set_begin_approval_process(
        ctx: &DalContext,
        change_set_id: ChangeSetId,
        user_pk: Option<UserPk>,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ChangeSetBeginApprovalProcess(ChangeSetActorPayload {
                change_set_id,
                user_pk,
            }),
        )
        .await
    }

    pub async fn change_set_cancel_approval_process(
        ctx: &DalContext,
        change_set_id: ChangeSetId,
        user_pk: Option<UserPk>,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ChangeSetCancelApprovalProcess(ChangeSetActorPayload {
                change_set_id,
                user_pk,
            }),
        )
        .await
    }

    pub async fn change_set_abandon_vote(
        ctx: &DalContext,
        change_set_id: ChangeSetId,
        user_pk: Option<UserPk>,
        vote: String,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ChangeSetAbandonVote(ChangeSetMergeVotePayload {
                change_set_id,
                user_pk,
                vote,
            }),
        )
        .await
    }

    pub async fn change_set_begin_abandon_approval_process(
        ctx: &DalContext,
        change_set_id: ChangeSetId,
        user_pk: Option<UserPk>,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ChangeSetBeginAbandonProcess(ChangeSetActorPayload {
                change_set_id,
                user_pk,
            }),
        )
        .await
    }

    pub async fn change_set_cancel_abandon_approval_process(
        ctx: &DalContext,
        change_set_id: ChangeSetId,
        user_pk: Option<UserPk>,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ChangeSetCancelAbandonProcess(ChangeSetActorPayload {
                change_set_id,
                user_pk,
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
pub struct ChangeSetMergeVotePayload {
    change_set_id: ChangeSetId,
    user_pk: Option<UserPk>,
    vote: String,
}
