use postgres_types::ToSql;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use ulid::Ulid;

use crate::{change_set_pointer::ChangeSetPointerId, pk, UserPk, WsEvent, WsPayload};
use crate::{DalContext, WsEventResult};

#[remain::sorted]
#[derive(Deserialize, Serialize, Debug, Display, EnumString, PartialEq, Eq, Clone, ToSql)]
pub enum ChangeSetStatus {
    Abandoned,
    Applied,
    Closed,
    Failed,
    NeedsAbandonApproval,
    NeedsApproval,
    Open,
}

pk!(ChangeSetPk);

impl From<ChangeSetPointerId> for ChangeSetPk {
    fn from(pointer: ChangeSetPointerId) -> Self {
        Self::from(Ulid::from(pointer))
    }
}
impl From<ChangeSetPk> for ChangeSetPointerId {
    fn from(pointer: ChangeSetPk) -> Self {
        Self::from(Ulid::from(pointer))
    }
}

impl WsEvent {
    pub async fn change_set_created(
        ctx: &DalContext,
        change_set_pk: ChangeSetPk,
    ) -> WsEventResult<Self> {
        WsEvent::new(ctx, WsPayload::ChangeSetCreated(change_set_pk)).await
    }

    pub async fn change_set_written(ctx: &DalContext) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ChangeSetWritten(ctx.visibility().change_set_pk),
        )
        .await
    }

    pub async fn change_set_abandoned(
        ctx: &DalContext,
        change_set_pk: ChangeSetPk,
        user_pk: Option<UserPk>,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ChangeSetAbandoned(ChangeSetActorPayload {
                change_set_pk,
                user_pk,
            }),
        )
        .await
    }

    pub async fn change_set_applied(
        ctx: &DalContext,
        change_set_pk: ChangeSetPk,
        user_pk: Option<UserPk>,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ChangeSetApplied(ChangeSetActorPayload {
                change_set_pk,
                user_pk,
            }),
        )
        .await
    }

    pub async fn change_set_canceled(
        ctx: &DalContext,
        change_set_pk: ChangeSetPk,
    ) -> WsEventResult<Self> {
        WsEvent::new(ctx, WsPayload::ChangeSetCanceled(change_set_pk)).await
    }

    pub async fn change_set_merge_vote(
        ctx: &DalContext,
        change_set_pk: ChangeSetPk,
        user_pk: UserPk,
        vote: String,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ChangeSetMergeVote(ChangeSetMergeVotePayload {
                change_set_pk,
                user_pk,
                vote,
            }),
        )
        .await
    }

    pub async fn change_set_begin_approval_process(
        ctx: &DalContext,
        change_set_pk: ChangeSetPk,
        user_pk: Option<UserPk>,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ChangeSetBeginApprovalProcess(ChangeSetActorPayload {
                change_set_pk,
                user_pk,
            }),
        )
        .await
    }

    pub async fn change_set_cancel_approval_process(
        ctx: &DalContext,
        change_set_pk: ChangeSetPk,
        user_pk: Option<UserPk>,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ChangeSetCancelApprovalProcess(ChangeSetActorPayload {
                change_set_pk,
                user_pk,
            }),
        )
        .await
    }

    pub async fn change_set_abandon_vote(
        ctx: &DalContext,
        change_set_pk: ChangeSetPk,
        user_pk: UserPk,
        vote: String,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ChangeSetAbandonVote(ChangeSetMergeVotePayload {
                change_set_pk,
                user_pk,
                vote,
            }),
        )
        .await
    }

    pub async fn change_set_begin_abandon_approval_process(
        ctx: &DalContext,
        change_set_pk: ChangeSetPk,
        user_pk: Option<UserPk>,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ChangeSetBeginAbandonProcess(ChangeSetActorPayload {
                change_set_pk,
                user_pk,
            }),
        )
        .await
    }

    pub async fn change_set_cancel_abandon_approval_process(
        ctx: &DalContext,
        change_set_pk: ChangeSetPk,
        user_pk: Option<UserPk>,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ChangeSetCancelAbandonProcess(ChangeSetActorPayload {
                change_set_pk,
                user_pk,
            }),
        )
        .await
    }
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ChangeSetActorPayload {
    change_set_pk: ChangeSetPk,
    user_pk: Option<UserPk>,
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ChangeSetMergeVotePayload {
    change_set_pk: ChangeSetPk,
    user_pk: UserPk,
    vote: String,
}
