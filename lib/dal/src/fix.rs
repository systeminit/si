//! This module contains the concept of "fixes".

use chrono::Utc;
use postgres_types::{FromSql, ToSql};
use serde::{Deserialize, Serialize};
use si_data_pg::PgError;
use strum::{AsRefStr, Display, EnumIter, EnumString};
use telemetry::prelude::*;
use thiserror::Error;
use veritech_client::ResourceStatus;

use crate::fix::batch::FixBatchId;
use crate::func::binding_return_value::FuncBindingReturnValueError;
use crate::schema::SchemaUiMenu;
use crate::{
    func::backend::js_command::CommandRunResult, impl_standard_model, pk, standard_model,
    standard_model_accessor, standard_model_accessor_ro, standard_model_belongs_to,
    ActionPrototypeError, AttributeValueId, Component, ComponentError, ComponentId, DalContext,
    FixResolverError, FuncError, HistoryEventError, ResourceView, SchemaError, StandardModel,
    StandardModelError, Tenancy, Timestamp, Visibility, WorkflowPrototypeId, WorkflowRunnerError,
    WorkflowRunnerId, WorkflowRunnerStatus, WsEvent, WsPayload,
};
use crate::{FixBatch, TransactionsError, WorkflowRunner, WsEventResult};

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

impl TryFrom<WorkflowRunnerStatus> for FixCompletionStatus {
    type Error = FixError;

    fn try_from(
        status: WorkflowRunnerStatus,
    ) -> Result<Self, <FixCompletionStatus as TryFrom<WorkflowRunnerStatus>>::Error> {
        if let WorkflowRunnerStatus::Success = status {
            Ok(Self::Success)
        } else if let WorkflowRunnerStatus::Failure = status {
            Ok(Self::Failure)
        } else {
            Err(FixError::IncompatibleWorkflowRunnerStatus(status))
        }
    }
}

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
    #[error("workflow runner status {0} cannot be converted to fix completion status")]
    IncompatibleWorkflowRunnerStatus(WorkflowRunnerStatus),
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
    WorkflowRunner(#[from] WorkflowRunnerError),
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

    /// Corresponds to the [`AttributeValue`](crate::AttributeValue) of the
    /// [`"confirmation"`](crate::schema::variant::leaves).
    attribute_value_id: AttributeValueId,
    /// The [`Component`](crate::Component) being fixed.
    component_id: ComponentId,
    /// The [`WorkflowRunner`](crate::WorkflowRunner) that got executed.
    workflow_runner_id: Option<WorkflowRunnerId>,
    /// The name of the [`ActionPrototype`](crate::action_prototype::ActionPrototype) used.
    action: String,

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
        attribute_value_id: AttributeValueId,
        component_id: ComponentId,
        action: &str,
    ) -> FixResult<Self> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "SELECT object FROM fix_create_v1($1, $2, $3, $4, $5)",
                &[
                    ctx.tenancy(),
                    ctx.visibility(),
                    &attribute_value_id,
                    &component_id,
                    &action,
                ],
            )
            .await?;
        let object: Fix = standard_model::finish_create_from_row(ctx, row).await?;
        object.set_fix_batch(ctx, fix_batch_id).await?;
        Ok(object)
    }

    pub fn component_id(&self) -> ComponentId {
        self.component_id
    }

    standard_model_accessor_ro!(attribute_value_id, AttributeValueId);

    standard_model_accessor!(workflow_runner_id, Option<Pk(WorkflowRunnerId)>, FixResult);
    standard_model_accessor!(action, String, FixResult);
    standard_model_accessor!(started_at, Option<String>, FixResult);
    standard_model_accessor!(finished_at, Option<String>, FixResult);
    standard_model_accessor!(
        completion_status,
        Option<Enum(FixCompletionStatus)>,
        FixResult
    );
    standard_model_accessor!(completion_message, Option<String>, FixResult);

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
        run_id: usize,
        action_workflow_prototype_id: WorkflowPrototypeId,
    ) -> FixResult<Vec<CommandRunResult>> {
        // Stamp started and run the workflow.
        self.stamp_started(ctx).await?;
        let runner_result = WorkflowRunner::run_without_triggering_dependent_values_update(
            ctx,
            run_id,
            action_workflow_prototype_id,
            self.component_id,
        )
        .await;

        // Evaluate the workflow result.
        match runner_result {
            Ok(post_run_data) => {
                let (runner, runner_state, _func_binding_return_values, resources) = post_run_data;
                self.set_workflow_runner_id(ctx, Some(runner.id())).await?;

                // Set the run as completed. Record the error message if it exists.
                let completion_status: FixCompletionStatus = runner_state.status().try_into()?;
                self.stamp_finished(
                    ctx,
                    completion_status,
                    runner_state.error_message().map(|s| s.to_string()),
                )
                .await?;

                Ok(resources)
            }
            Err(e) => {
                error!("Unable to run fix: {e}");
                // If the workflow had an error, we can record an error completion status with
                // the error as a message.
                self.stamp_finished(ctx, FixCompletionStatus::Error, Some(format!("{e:?}")))
                    .await?;
                Ok(Vec::new())
            }
        }
    }

    /// A safe wrapper around setting completion-related columns.
    pub async fn stamp_finished(
        &mut self,
        ctx: &DalContext,
        completion_status: FixCompletionStatus,
        completion_message: Option<String>,
    ) -> FixResult<()> {
        if self.started_at.is_some() {
            self.set_finished_at(ctx, Some(Utc::now().to_rfc3339()))
                .await?;
            self.set_completion_status(ctx, Some(completion_status))
                .await?;
            if completion_message.is_some() {
                self.set_completion_message(ctx, completion_message).await?;
            }
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

    pub async fn workflow_runner(&self, ctx: &DalContext) -> FixResult<Option<WorkflowRunner>> {
        match &self.workflow_runner_id {
            Some(id) => Ok(WorkflowRunner::get_by_id(ctx, id).await?),
            None => Ok(None),
        }
    }

    /// Generates a [`FixHistoryView`] based on [`self`](Fix).
    pub async fn history_view(
        &self,
        ctx: &DalContext,
        batch_timed_out: bool,
    ) -> FixResult<Option<FixHistoryView>> {
        // Technically WorkflowRunner returns a vec of resources, but we only handle one resource at a time
        // It's a technical debt we haven't tackled yet, so let's assume it's only one resource
        let maybe_resource = self
            .workflow_runner(ctx)
            .await?
            .map(|r| Ok::<_, WorkflowRunnerError>(r.resources()?.pop()))
            .transpose()?
            .flatten();

        let resource = if let Some(resource) = maybe_resource {
            Some(resource)
        } else if batch_timed_out {
            Some(CommandRunResult {
                status: ResourceStatus::Error,
                payload: None,
                message: Some("Execution timed-out".to_owned()),
                // TODO: add proper logs here
                logs: vec![],
                last_synced: None,
            })
        } else {
            // If a fix hasn't finished we don't show it in the front-end
            None
        };

        // Gather component-related information, even if the component has been deleted.
        let (component_name, schema_name, category) =
            Self::component_details_for_history_view(ctx, self.component_id).await?;

        Ok(Some(FixHistoryView {
            id: self.id,
            status: if resource.is_none() {
                FixCompletionStatus::Unstarted
            } else {
                self.completion_status()
                    .copied()
                    .unwrap_or(FixCompletionStatus::Failure)
            },
            action: self.action().to_owned(),
            schema_name,
            attribute_value_id: *self.attribute_value_id(),
            component_name,
            component_id: self.component_id,
            provider: category,
            resource: resource.map(ResourceView::new),
            started_at: self.started_at().map(|s| s.to_string()),
            finished_at: self.finished_at().map(|s| s.to_string()),
        }))
    }

    /// Gather details related to the [`Component`](crate::Component) for assembling a
    /// [`FixHistoryView`].
    ///
    /// This private method should only be called by [`Self::history_view`].
    async fn component_details_for_history_view(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> FixResult<(String, String, Option<String>)> {
        // For the component-related information, we want to ensure that we can gather the
        // fix history view if the component has been deleted. This is helpful if deletion fixes
        // fail and the component still needs to be deleted.
        let ctx_with_deleted = &ctx.clone_with_delete_visibility();

        let component = Component::get_by_id(ctx_with_deleted, &component_id)
            .await?
            .ok_or_else(|| ComponentError::NotFound(component_id))?;
        let schema = component
            .schema(ctx_with_deleted)
            .await?
            .ok_or_else(|| ComponentError::NoSchema(component_id))?;
        let category = SchemaUiMenu::find_for_schema(ctx_with_deleted, *schema.id())
            .await?
            .map(|um| um.category().to_string());

        Ok((
            component.name(ctx_with_deleted).await?,
            schema.name().to_owned(),
            category,
        ))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FixHistoryView {
    id: FixId,
    status: FixCompletionStatus,
    action: String,
    schema_name: String,
    component_name: String,
    component_id: ComponentId,
    attribute_value_id: AttributeValueId,
    provider: Option<String>,
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
    attribute_value_id: AttributeValueId,
    action: String,
    status: FixCompletionStatus,
    output: Vec<String>,
}

impl WsEvent {
    pub async fn fix_return(
        ctx: &DalContext,
        id: FixId,
        batch_id: FixBatchId,
        attribute_value_id: AttributeValueId,
        action: String,
        status: FixCompletionStatus,
        output: Vec<String>,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::FixReturn(FixReturn {
                id,
                batch_id,
                attribute_value_id,
                action,
                status,
                output,
            }),
        )
        .await
    }
}
