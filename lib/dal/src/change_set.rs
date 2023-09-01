use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use si_data_nats::NatsError;
use si_data_pg::PgError;
use strum::{Display, EnumString};
use telemetry::prelude::*;
use thiserror::Error;

use crate::standard_model::{object_option_from_row_option, objects_from_rows};
use crate::ws_event::{WsEvent, WsEventError, WsPayload};
use crate::{
    pk, Action, ActionError, HistoryEvent, HistoryEventError, LabelListError, StandardModelError,
    Tenancy, Timestamp, TransactionsError, UserError, UserPk, Visibility,
};
use crate::{Component, ComponentError, DalContext, WsEventResult};

const CHANGE_SET_OPEN_LIST: &str = include_str!("queries/change_set/open_list.sql");
const CHANGE_SET_GET_BY_PK: &str = include_str!("queries/change_set/get_by_pk.sql");

#[remain::sorted]
#[derive(Error, Debug)]
pub enum ChangeSetError {
    #[error(transparent)]
    Action(#[from] ActionError),
    #[error(transparent)]
    Component(#[from] ComponentError),
    #[error(transparent)]
    HistoryEvent(#[from] HistoryEventError),
    #[error("invalid user actor pk")]
    InvalidActor(UserPk),
    #[error(transparent)]
    LabelList(#[from] LabelListError),
    #[error(transparent)]
    Nats(#[from] NatsError),
    #[error(transparent)]
    Pg(#[from] PgError),
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
#[derive(Deserialize, Serialize, Debug, Display, EnumString, PartialEq, Eq, Clone)]
pub enum ChangeSetStatus {
    Abandoned,
    Applied,
    Closed,
    Failed,
    Open,
}

pk!(ChangeSetPk);

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
}

impl ChangeSet {
    #[instrument(skip(ctx, name, note))]
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

    #[instrument(skip(ctx))]
    pub async fn apply_raw(
        &mut self,
        ctx: &mut DalContext,
        run_confirmations: bool,
    ) -> ChangeSetResult<()> {
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

        WsEvent::change_set_applied(ctx, self.pk)
            .await?
            .publish_on_commit(ctx)
            .await?;

        // Update the visibility.
        ctx.update_visibility(Visibility::new_head(false));

        if run_confirmations {
            // Before retuning, run all confirmations now that we are on head.
            Component::run_all_confirmations(ctx).await?;
        }

        Ok(())
    }

    #[instrument(skip(ctx))]
    pub async fn apply(&mut self, ctx: &mut DalContext) -> ChangeSetResult<()> {
        self.apply_raw(ctx, true).await?;
        Ok(())
    }

    #[instrument(skip_all)]
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

    #[instrument(skip_all)]
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

    pub async fn sort_actions(&self, ctx: &DalContext) -> ChangeSetResult<()> {
        let ctx =
            ctx.clone_with_new_visibility(Visibility::new(self.pk, ctx.visibility().deleted_at));
        Ok(Action::sort_of_change_set(&ctx).await?)
    }

    pub async fn actions(&self, ctx: &DalContext) -> ChangeSetResult<Vec<Action>> {
        let ctx =
            ctx.clone_with_new_visibility(Visibility::new(self.pk, ctx.visibility().deleted_at));
        Ok(Action::find_for_change_set(&ctx).await?)
    }
}

impl WsEvent {
    pub async fn change_set_created(
        ctx: &DalContext,
        change_set_pk: ChangeSetPk,
    ) -> WsEventResult<Self> {
        WsEvent::new(ctx, WsPayload::ChangeSetCreated(change_set_pk)).await
    }

    pub async fn change_set_applied(
        ctx: &DalContext,
        change_set_pk: ChangeSetPk,
    ) -> WsEventResult<Self> {
        WsEvent::new(ctx, WsPayload::ChangeSetApplied(change_set_pk)).await
    }

    pub async fn change_set_canceled(
        ctx: &DalContext,
        change_set_pk: ChangeSetPk,
    ) -> WsEventResult<Self> {
        WsEvent::new(ctx, WsPayload::ChangeSetCanceled(change_set_pk)).await
    }

    pub async fn change_set_written(ctx: &DalContext) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ChangeSetWritten(ctx.visibility().change_set_pk),
        )
        .await
    }
}
