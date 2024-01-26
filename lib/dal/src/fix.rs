//! This module contains the concept of "fixes".

use chrono::Utc;
use postgres_types::{FromSql, ToSql};
use serde::{Deserialize, Serialize};
use si_data_pg::PgError;
use strum::{AsRefStr, Display, EnumIter, EnumString};
use telemetry::prelude::*;
use thiserror::Error;

use crate::fix::batch::FixBatchId;
use crate::func::binding_return_value::FuncBindingReturnValueError;
use crate::{
    func::backend::js_action::ActionRunResult, impl_standard_model, pk, standard_model,
    standard_model_accessor, standard_model_accessor_ro, standard_model_belongs_to, ActionId,
    ActionKind, ActionPrototype, ActionPrototypeError, ActionPrototypeId, Component,
    ComponentError, ComponentId, DalContext, FixBatch, FixResolverError, Func, FuncError,
    HistoryEventError, ResourceView, SchemaError, StandardModel, StandardModelError, Tenancy,
    Timestamp, TransactionsError, Visibility, WsEvent, WsEventError, WsEventResult, WsPayload,
};
use veritech_client::ResourceStatus;

pub mod batch;
pub mod resolver;

/// The completion status of a [`Fix`] or [`FixBatch`](crate::FixBatch).
#[remain::sorted]
#[derive(
    Deserialize,
    Serialize,
    AsRefStr,
    Display,
    EnumIter,
    EnumString,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    ToSql,
    FromSql,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum FixCompletionStatus {
    /// The [`Fix`] or at least one [`Fix`] in the [`FixBatch`](crate::FixBatch) executed with
    /// error(s).
    Error,
    /// The [`Fix`] or at least one [`Fix`] in the [`FixBatch`](crate::FixBatch) was or may have
    /// been executed without error, but it or they were unsuccessful during execution.
    Failure,
    /// The [`Fix`] or all [`Fixes`](Fix) in the [`FixBatch`](crate::FixBatch) were executed and
    /// successful.
    Success,
    /// The ['Fix'] is still running or has not yet started
    Unstarted,
}

// a type alias for satisfying the standard model macros
type JsonValue = serde_json::Value;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum FixError {
    #[error(transparent)]
    ActionPrototype(#[from] ActionPrototypeError),
    #[error("cannot stamp batch or fix as started since it already finished")]
    AlreadyFinished,
    #[error("cannot stamp batch or fix as started since it already started")]
    AlreadyStarted,
    #[error("cannot set batch for {0}: fix batch ({1}) already finished")]
    BatchAlreadyFinished(FixId, FixBatchId),
    #[error("cannot set batch for {0}: fix batch ({1}) already started")]
    BatchAlreadyStarted(FixId, FixBatchId),
    #[error(transparent)]
    Component(#[from] ComponentError),
    #[error("completion status is empty")]
    EmptyCompletionStatus,
    #[error(transparent)]
    FixResolver(#[from] FixResolverError),
    #[error(transparent)]
    Func(#[from] FuncError),
    #[error(transparent)]
    FuncBindingReturnValue(#[from] FuncBindingReturnValueError),
    #[error(transparent)]
    HistoryEvent(#[from] HistoryEventError),
    #[error("action run status cannot be converted to fix completion status")]
    IncompatibleActionRunStatus,
    #[error("missing finished timestamp for fix: {0}")]
    MissingFinishedTimestampForFix(FixId),
    #[error("fix not found for id: {0}")]
    MissingFix(FixId),
    #[error("fix batch not found for id: {0}")]
    MissingFixBatch(FixBatchId),
    #[error("missing started timestamp for fix: {0}")]
    MissingStartedTimestampForFix(FixId),
    #[error("no fixes in batch: fix batch is empty")]
    NoFixesInBatch(FixBatchId),
    #[error("not found: {0}")]
    NotFound(FixId),
    #[error("not found for action: {0}")]
    NotFoundForAction(ActionId),
    #[error("cannot stamp batch or fix as finished since it has not yet been started")]
    NotYetStarted,
    #[error(transparent)]
    Pg(#[from] PgError),
    #[error(transparent)]
    Schema(#[from] SchemaError),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error(transparent)]
    Transactions(#[from] TransactionsError),
    #[error(transparent)]
    WsEvent(#[from] WsEventError),
}

pub type FixResult<T> = Result<T, FixError>;

pk!(FixPk);
pk!(FixId);

/// A record of a "fix" after it has been executed.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Fix {
    pk: FixPk,
    id: FixId,
    #[serde(flatten)]
    tenancy: Tenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,

    /// The [`Component`] being fixed.
    component_id: ComponentId,
    component_name: String,
    /// The [`ActionPrototype`] that runs the action for this fix.
    action_prototype_id: ActionPrototypeId,

    action_kind: ActionKind,

    // The resource returned by this fix (if any)
    resource: Option<JsonValue>,

    // TODO(nick): convert to Option<DateTime<Utc>> once standard model accessor can accommodate both
    // Option<T<U>> and can handle "timestamp with time zone <--> DateTime<Utc>".
    /// Indicates when the [`Fix`] started execution when populated.
    started_at: Option<String>,
    // TODO(nick): convert to Option<DateTime<Utc>> once standard model accessor can accommodate both
    // Option<T<U>> and can handle "timestamp with time zone <--> DateTime<Utc>".
    /// Indicates when the [`Fix`] finished execution when populated.
    finished_at: Option<String>,
    /// Indicates the state of the [`Fix`] when finished.
    completion_status: Option<FixCompletionStatus>,

    /// Contains a message related to the completion.
    completion_message: Option<String>,
}

impl_standard_model! {
    model: Fix,
    pk: FixPk,
    id: FixId,
    table_name: "fixes",
    history_event_label_base: "fix",
    history_event_message_name: "Fix"
}

impl Fix {
    /// Create [`Self`] and ensure it belongs to a [`FixBatch`](crate::FixBatch)
    /// since every [`fix`](Self) must belong to a [`batch`](crate::FixBatch).
    #[instrument(skip_all)]
    pub async fn new(
        ctx: &DalContext,
        fix_batch_id: FixBatchId,
        component_id: ComponentId,
        component_name: String,
        action_prototype_id: ActionPrototypeId,
    ) -> FixResult<Self> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "SELECT object FROM fix_create_v2($1, $2, $3, $4, $5)",
                &[
                    ctx.tenancy(),
                    ctx.visibility(),
                    &component_id,
                    &component_name,
                    &action_prototype_id,
                ],
            )
            .await?;
        let object: Fix = standard_model::finish_create_from_row(ctx, row).await?;
        object.set_fix_batch(ctx, fix_batch_id).await?;
        Ok(object)
    }

    standard_model_accessor_ro!(component_id, ComponentId);
    standard_model_accessor_ro!(component_name, String);
    standard_model_accessor_ro!(action_prototype_id, ActionPrototypeId);
    standard_model_accessor_ro!(action_kind, ActionKind);
    standard_model_accessor!(started_at, Option<String>, FixResult);
    standard_model_accessor!(finished_at, Option<String>, FixResult);
    standard_model_accessor!(
        completion_status,
        Option<Enum(FixCompletionStatus)>,
        FixResult
    );
    standard_model_accessor!(completion_message, Option<String>, FixResult);
    standard_model_accessor!(resource, OptionJson<JsonValue>, FixResult);

    standard_model_belongs_to!(
        lookup_fn: fix_batch,
        set_fn: set_fix_batch_unchecked,
        unset_fn: unset_fix_batch,
        table: "fix_belongs_to_fix_batch",
        model_table: "fix_batches",
        belongs_to_id: FixBatchId,
        returns: FixBatch,
        result: FixResult,
    );

    /// A wrapper around the standard model function in order to ensure that the
    /// [`batch`](crate::FixBatch) is has not yet been executed.
    pub async fn set_fix_batch(&self, ctx: &DalContext, batch_id: FixBatchId) -> FixResult<()> {
        let batch = FixBatch::get_by_id(ctx, &batch_id)
            .await?
            .ok_or(FixError::MissingFixBatch(batch_id))?;
        if batch.started_at().is_some() {
            return Err(FixError::BatchAlreadyStarted(self.id, batch_id));
        }
        if batch.finished_at().is_some() {
            return Err(FixError::BatchAlreadyFinished(self.id, batch_id));
        }
        self.set_fix_batch_unchecked(ctx, &batch_id).await?;
        Ok(())
    }

    /// Executes the [`fix`](Self). Returns true if some resource got updated, false if not
    pub async fn run(
        &mut self,
        ctx: &DalContext,
        action_prototype: &ActionPrototype,
    ) -> FixResult<Option<ActionRunResult>> {
        // Stamp started and run the workflow.
        self.stamp_started(ctx).await?;

        Ok(match action_prototype.run(ctx, self.component_id).await {
            Ok(Some(run_result)) => {
                let completion_status = match run_result.status {
                    Some(ResourceStatus::Ok) | Some(ResourceStatus::Warning) => {
                        FixCompletionStatus::Success
                    }
                    Some(ResourceStatus::Error) => FixCompletionStatus::Failure,
                    None => FixCompletionStatus::Unstarted,
                };

                self.stamp_finished(
                    ctx,
                    completion_status,
                    run_result.message.clone(),
                    Some(run_result.clone()),
                )
                .await?;

                Some(run_result)
            }
            Ok(None) => {
                error!("Fix did not return a value!");
                self.stamp_finished(
                    ctx,
                    FixCompletionStatus::Error,
                    Some("Fix did not return a value".into()),
                    None,
                )
                .await?;

                None
            }
            Err(e) => {
                error!("Unable to run fix: {e}");
                self.stamp_finished(
                    ctx,
                    FixCompletionStatus::Error,
                    Some(format!("{e:?}")),
                    None,
                )
                .await?;

                None
            }
        })
    }

    /// A safe wrapper around setting completion-related columns.
    pub async fn stamp_finished(
        &mut self,
        ctx: &DalContext,
        completion_status: FixCompletionStatus,
        completion_message: Option<String>,
        resource: Option<ActionRunResult>,
    ) -> FixResult<()> {
        if self.started_at.is_some() {
            self.set_finished_at(ctx, Some(Utc::now().to_rfc3339()))
                .await?;
            self.set_completion_status(ctx, Some(completion_status))
                .await?;
            if completion_message.is_some() {
                self.set_completion_message(ctx, completion_message).await?;
            }
            let resource_value = match resource {
                Some(resource) => Some(serde_json::to_value(resource)?),
                None => None,
            };
            self.set_resource(ctx, resource_value).await?;

            Ok(())
        } else {
            Err(FixError::NotYetStarted)
        }
    }

    /// A safe wrapper around setting the started column.
    pub async fn stamp_started(&mut self, ctx: &DalContext) -> FixResult<()> {
        if self.started_at.is_some() {
            Err(FixError::AlreadyStarted)
        } else if self.finished_at.is_some() {
            Err(FixError::AlreadyFinished)
        } else {
            self.set_started_at(ctx, Some(Utc::now().to_rfc3339()))
                .await?;
            Ok(())
        }
    }

    /// Generates a [`FixHistoryView`] based on [`self`](Fix).
    pub async fn history_view(
        &self,
        ctx: &DalContext,
        batch_timed_out: bool,
    ) -> FixResult<Option<FixHistoryView>> {
        let resource: Option<ActionRunResult> = match self.resource() {
            Some(resource) => Some(serde_json::from_value(resource.clone())?),
            None => {
                if batch_timed_out {
                    Some(ActionRunResult {
                        status: Some(ResourceStatus::Error),
                        payload: None,
                        message: Some("Execution timed-out".to_owned()),
                        // TODO: add proper logs here
                        logs: vec![],
                        last_synced: None,
                    })
                } else {
                    None
                }
            }
        };
        // Gather component-related information, even if the component has been deleted.
        let component =
            Component::get_by_id(&ctx.clone_with_delete_visibility(), self.component_id())
                .await?
                .ok_or_else(|| ComponentError::NotFound(*self.component_id()))?;
        let schema_name = component
            .schema(&ctx.clone_with_delete_visibility())
            .await?
            .ok_or_else(|| ComponentError::NoSchema(*self.component_id()))?
            .name()
            .to_owned();

        let mut display_name = self.action_kind().clone().to_string();
        let action_prototype = ActionPrototype::get_by_id(ctx, self.action_prototype_id()).await?;

        if let Some(ap) = action_prototype {
            let func_details = Func::get_by_id(ctx, &ap.func_id()).await?;
            if let Some(func) = func_details {
                if let Some(name) = func.display_name() {
                    display_name = name.to_string();
                }
            }
        }

        Ok(Some(FixHistoryView {
            id: self.id,
            status: if resource.is_none() {
                FixCompletionStatus::Unstarted
            } else {
                self.completion_status()
                    .copied()
                    .unwrap_or(FixCompletionStatus::Failure)
            },
            action_kind: *self.action_kind(),
            display_name,
            schema_name,
            component_name: self.component_name().to_owned(),
            component_id: self.component_id,
            resource: resource.map(ResourceView::new),
            started_at: self.started_at().map(|s| s.to_string()),
            finished_at: self.finished_at().map(|s| s.to_string()),
        }))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FixHistoryView {
    id: FixId,
    status: FixCompletionStatus,
    action_kind: ActionKind,
    display_name: String,
    schema_name: String,
    component_name: String,
    component_id: ComponentId,
    started_at: Option<String>,
    finished_at: Option<String>,
    resource: Option<ResourceView>,
}

impl FixHistoryView {
    pub fn status(&self) -> FixCompletionStatus {
        self.status
    }
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FixReturn {
    id: FixId,
    batch_id: FixBatchId,
    action: ActionKind,
    status: FixCompletionStatus,
    output: Vec<String>,
}

impl WsEvent {
    pub async fn fix_return(
        ctx: &DalContext,
        id: FixId,
        batch_id: FixBatchId,
        action: ActionKind,
        status: FixCompletionStatus,
        output: Vec<String>,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::FixReturn(FixReturn {
                id,
                batch_id,
                action,
                status,
                output,
            }),
        )
        .await
    }
}
