use chrono::{DateTime, Utc};
use postgres_types::ToSql;
use serde::{Deserialize, Serialize};
use si_data_nats::NatsError;
use si_data_pg::{PgError, PgPoolError};
use strum::{Display, EnumString};
use telemetry::prelude::*;
use thiserror::Error;
use ulid::Ulid;

use crate::change_set_pointer::{ChangeSetPointer, ChangeSetPointerError};
use crate::standard_model::{object_option_from_row_option, objects_from_rows};
use crate::{
    change_set_pointer::ChangeSetPointerId, pk, ActionError, HistoryActor, HistoryEvent,
    HistoryEventError, LabelListError, StandardModelError, Tenancy, Timestamp, TransactionsError,
    User, UserError, UserPk, Visibility, WsEvent, WsEventError, WsPayload,
};
use crate::{DalContext, WsEventResult};

const CHANGE_SET_OPEN_LIST: &str = include_str!("queries/change_set/open_list.sql");
const CHANGE_SET_GET_BY_PK: &str = include_str!("queries/change_set/get_by_pk.sql");
// const GET_ACTORS: &str = include_str!("queries/change_set/get_actors.sql");

const BEGIN_MERGE_FLOW: &str = include_str!("queries/change_set/begin_merge_flow.sql");
const CANCEL_MERGE_FLOW: &str = include_str!("queries/change_set/cancel_merge_flow.sql");
const ABANDON_CHANGE_SET: &str = include_str!("queries/change_set/abandon_change_set.sql");

const BEGIN_ABANDON_FLOW: &str = include_str!("queries/change_set/begin_abandon_flow.sql");
const CANCEL_ABANDON_FLOW: &str = include_str!("queries/change_set/cancel_abandon_flow.sql");

#[remain::sorted]
#[derive(Error, Debug)]
pub enum ChangeSetError {
    #[error("action error: {0}")]
    Action(#[from] ActionError),
    #[error("change set pointer error: {0}")]
    ChangeSetPointer(#[from] ChangeSetPointerError),
    #[error(transparent)]
    HistoryEvent(#[from] HistoryEventError),
    #[error("invalid user actor pk")]
    InvalidActor(UserPk),
    #[error("invalid user system init")]
    InvalidUserSystemInit,
    #[error(transparent)]
    LabelList(#[from] LabelListError),
    #[error(transparent)]
    Nats(#[from] NatsError),
    #[error(transparent)]
    Pg(#[from] PgError),
    #[error(transparent)]
    PgPool(#[from] PgPoolError),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error(transparent)]
    Transactions(#[from] TransactionsError),
    #[error(transparent)]
    User(#[from] UserError),
    #[error(transparent)]
    WsEvent(#[from] WsEventError),
}

pub type ChangeSetResult<T> = Result<T, ChangeSetError>;

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

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
pub struct ChangeSet {
    pub pk: ChangeSetPk,
    pub name: String,
    pub note: Option<String>,
    pub status: ChangeSetStatus,
    #[serde(flatten)]
    pub tenancy: Tenancy,
    #[serde(flatten)]
    pub timestamp: Timestamp,
    pub merge_requested_at: Option<DateTime<Utc>>,
    pub merge_requested_by_user_id: Option<UserPk>,
    pub abandon_requested_at: Option<DateTime<Utc>>,
    pub abandon_requested_by_user_id: Option<UserPk>,
}

impl ChangeSet {
    pub async fn new(
        ctx: &DalContext,
        name: impl AsRef<str>,
        note: Option<&String>,
    ) -> ChangeSetResult<Self> {
        let name = name.as_ref();
        let note = note.as_ref();
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "SELECT object FROM change_set_create_v1($1, $2, $3, $4)",
                &[
                    &name,
                    &note,
                    &ChangeSetStatus::Open.to_string(),
                    ctx.tenancy(),
                ],
            )
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        let _history_event =
            HistoryEvent::new(ctx, "change_set.create", "Change Set created", &json).await?;
        let object: Self = serde_json::from_value(json)?;
        WsEvent::change_set_created(ctx, object.pk)
            .await?
            .publish_on_commit(ctx)
            .await?;
        Ok(object)
    }

    pub fn generate_name() -> String {
        Utc::now().format("%Y-%m-%d-%H:%M").to_string()
    }

    pub async fn begin_approval_flow(&mut self, ctx: &mut DalContext) -> ChangeSetResult<()> {
        let user_pk = match ctx.history_actor() {
            HistoryActor::User(user_pk) => Some(*user_pk),

            HistoryActor::SystemInit => None,
        };

        let row = ctx
            .pg_pool()
            .get()
            .await?
            .query_one(BEGIN_MERGE_FLOW, &[&self.pk, &user_pk])
            .await?;
        let updated_at: DateTime<Utc> = row.try_get("updated_at")?;
        self.timestamp.updated_at = updated_at;
        self.status = ChangeSetStatus::NeedsApproval;
        self.merge_requested_at = Some(updated_at);
        self.merge_requested_by_user_id = user_pk;

        Ok(())
    }

    pub async fn cancel_approval_flow(&mut self, ctx: &mut DalContext) -> ChangeSetResult<()> {
        let row = ctx
            .pg_pool()
            .get()
            .await?
            .query_one(CANCEL_MERGE_FLOW, &[&self.pk])
            .await?;
        let updated_at: DateTime<Utc> = row.try_get("updated_at")?;
        self.timestamp.updated_at = updated_at;
        self.status = ChangeSetStatus::Open;

        Ok(())
    }

    pub async fn begin_abandon_approval_flow(
        &mut self,
        ctx: &mut DalContext,
    ) -> ChangeSetResult<()> {
        let user_pk = match ctx.history_actor() {
            HistoryActor::User(user_pk) => Some(*user_pk),

            HistoryActor::SystemInit => None,
        };

        let row = ctx
            .pg_pool()
            .get()
            .await?
            .query_one(BEGIN_ABANDON_FLOW, &[&self.pk, &user_pk])
            .await?;
        let updated_at: DateTime<Utc> = row.try_get("updated_at")?;
        self.timestamp.updated_at = updated_at;
        self.status = ChangeSetStatus::NeedsAbandonApproval;
        self.abandon_requested_at = Some(updated_at);
        self.abandon_requested_by_user_id = user_pk;

        Ok(())
    }

    pub async fn cancel_abandon_approval_flow(
        &mut self,
        ctx: &mut DalContext,
    ) -> ChangeSetResult<()> {
        let row = ctx
            .pg_pool()
            .get()
            .await?
            .query_one(CANCEL_ABANDON_FLOW, &[&self.pk])
            .await?;
        let updated_at: DateTime<Utc> = row.try_get("updated_at")?;
        self.timestamp.updated_at = updated_at;
        self.status = ChangeSetStatus::Open;

        Ok(())
    }

    #[instrument(level = "debug", skip_all)]
    pub async fn apply(&mut self, ctx: &mut DalContext) -> ChangeSetResult<()> {
        let actor = serde_json::to_value(ctx.history_actor())?;
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "SELECT timestamp_updated_at FROM change_set_apply_v1($1, $2, $3)",
                &[&self.pk, &actor, &self.tenancy],
            )
            .await?;
        let updated_at: DateTime<Utc> = row.try_get("timestamp_updated_at")?;
        self.timestamp.updated_at = updated_at;
        self.status = ChangeSetStatus::Applied;
        let _history_event = HistoryEvent::new(
            ctx,
            "change_set.apply",
            "Change Set applied",
            &serde_json::json![{ "pk": &self.pk }],
        )
        .await?;

        let user_pk = match ctx.history_actor() {
            HistoryActor::User(user_pk) => {
                let user = User::get_by_pk(ctx, *user_pk)
                    .await?
                    .ok_or(ChangeSetError::InvalidActor(*user_pk))?;

                Some(user.pk())
            }

            HistoryActor::SystemInit => None,
        };

        WsEvent::change_set_applied(ctx, self.pk, user_pk)
            .await?
            .publish_on_commit(ctx)
            .await?;

        // Update the visibility.
        ctx.update_visibility_deprecated(Visibility::new_head(false));

        Ok(())
    }

    pub async fn abandon(&mut self, ctx: &mut DalContext) -> ChangeSetResult<()> {
        let row = ctx
            .pg_pool()
            .get()
            .await?
            .query_one(ABANDON_CHANGE_SET, &[&self.pk])
            .await?;
        let updated_at: DateTime<Utc> = row.try_get("updated_at")?;
        self.timestamp.updated_at = updated_at;
        self.status = ChangeSetStatus::Abandoned;

        Ok(())
    }

    #[instrument(level = "debug", skip_all)]
    pub async fn list_open(ctx: &DalContext) -> ChangeSetResult<Vec<Self>> {
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(CHANGE_SET_OPEN_LIST, &[ctx.tenancy()])
            .await?;
        let results = objects_from_rows(rows)?;
        Ok(results)
    }

    pub async fn get_by_pk(
        ctx: &DalContext,
        pk: &ChangeSetPk,
    ) -> ChangeSetResult<Option<ChangeSet>> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_opt(CHANGE_SET_GET_BY_PK, &[ctx.tenancy(), &pk])
            .await?;
        let change_set: Option<ChangeSet> = object_option_from_row_option(row)?;
        Ok(change_set)
    }

    // pub async fn actors(&self, ctx: &DalContext) -> ChangeSetResult<Vec<String>> {
    //     let rows = ctx
    //         .txns()
    //         .await?
    //         .pg()
    //         .query(GET_ACTORS, &[&ctx.tenancy().workspace_pk(), &self.pk])
    //         .await?;

    //     let mut result: Vec<String> = vec![];
    //     for row in rows.into_iter() {
    //         let email: String = row.try_get("email")?;
    //         result.push(email);
    //     }

    //     Ok(result)
    // }

    pub async fn force_new(ctx: &mut DalContext) -> ChangeSetResult<Option<ChangeSetPk>> {
        let maybe_fake_pk =
            if ctx.change_set_id() == ctx.get_workspace_default_change_set_id().await? {
                let change_set = ChangeSetPointer::fork_head(ctx, Self::generate_name()).await?;
                ctx.update_visibility_and_snapshot_to_visibility(change_set.id)
                    .await?;

                let fake_pk = ChangeSetPk::from(Ulid::from(change_set.id));

                WsEvent::change_set_created(ctx, fake_pk)
                    .await?
                    .publish_on_commit(ctx)
                    .await?;

                Some(fake_pk)
            } else {
                None
            };
        Ok(maybe_fake_pk)
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
