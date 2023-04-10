use serde::{Deserialize, Serialize};
use si_data_pg::PgError;
use telemetry::prelude::*;
use thiserror::Error;

use crate::workflow_runner::workflow_runner_state::WorkflowRunnerState;
use crate::workflow_runner::workflow_runner_state::WorkflowRunnerStatus;
use crate::TransactionsError;
use crate::{
    func::backend::js_command::CommandRunResult,
    func::binding_return_value::{FuncBindingReturnValue, FuncBindingReturnValueError},
    func::execution::{FuncExecution, FuncExecutionError},
    func::{binding::FuncBindingId, FuncId},
    impl_standard_model, pk, standard_model, standard_model_accessor, Component, ComponentError,
    ComponentId, Func, FuncBindingError, HistoryEventError, InternalProviderError, SchemaId,
    SchemaVariantId, StandardModel, StandardModelError, Tenancy, Timestamp, Visibility,
    WorkflowError, WorkflowPrototype, WorkflowPrototypeError, WorkflowPrototypeId,
    WorkflowResolverError, WorkflowResolverId, WsEvent, WsEventError,
};
use crate::{DalContext, FuncError};

pub mod workflow_runner_state;

#[derive(Error, Debug)]
pub enum WorkflowRunnerError {
    #[error(transparent)]
    WsEvent(#[from] WsEventError),
    #[error(transparent)]
    HistoryEvent(#[from] HistoryEventError),
    #[error(transparent)]
    Pg(#[from] PgError),
    #[error(transparent)]
    Workflow(#[from] WorkflowError),
    #[error(transparent)]
    FuncBindingReturnValue(#[from] FuncBindingReturnValueError),
    #[error(transparent)]
    FuncExecution(#[from] FuncExecutionError),
    #[error(transparent)]
    Component(#[from] Box<ComponentError>),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error(transparent)]
    Transactions(#[from] TransactionsError),
    #[error(transparent)]
    WorkflowResolver(#[from] WorkflowResolverError),
    #[error(transparent)]
    WorkflowPrototype(#[from] WorkflowPrototypeError),
    #[error(transparent)]
    FuncBinding(#[from] FuncBindingError),
    #[error(transparent)]
    InternalProvider(#[from] InternalProviderError),
    #[error(transparent)]
    Func(#[from] FuncError),

    #[error("prototype not found {0}")]
    PrototypeNotFound(WorkflowPrototypeId),
    #[error("missing workflow {0}")]
    MissingWorkflow(String),
    #[error("component {0} not found")]
    ComponentNotFound(ComponentId),
}

pub type WorkflowRunnerResult<T> = Result<T, WorkflowRunnerError>;

const FIND_FOR_PROTOTYPE: &str = include_str!("queries/workflow_runner_find_for_prototype.sql");

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct WorkflowRunnerContext {
    component_id: ComponentId,
    schema_id: SchemaId,
    schema_variant_id: SchemaVariantId,
}

// Hrm - is this a universal runner context? -- Adam
impl Default for WorkflowRunnerContext {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkflowRunnerContext {
    pub fn new() -> Self {
        WorkflowRunnerContext {
            component_id: ComponentId::NONE,
            schema_id: SchemaId::NONE,
            schema_variant_id: SchemaVariantId::NONE,
        }
    }

    pub fn component_id(&self) -> ComponentId {
        self.component_id
    }

    pub fn set_component_id(&mut self, component_id: ComponentId) {
        self.component_id = component_id;
    }

    pub fn schema_id(&self) -> SchemaId {
        self.schema_id
    }

    pub fn set_schema_id(&mut self, schema_id: SchemaId) {
        self.schema_id = schema_id;
    }

    pub fn schema_variant_id(&self) -> SchemaVariantId {
        self.schema_variant_id
    }

    pub fn set_schema_variant_id(&mut self, schema_variant_id: SchemaVariantId) {
        self.schema_variant_id = schema_variant_id;
    }
}

pk!(WorkflowRunnerPk);
pk!(WorkflowRunnerId);

// An WorkflowRunner joins a `FuncBinding` to the context in which
// its corresponding `FuncBindingResultValue` is consumed.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct WorkflowRunner {
    pk: WorkflowRunnerPk,
    id: WorkflowRunnerId,
    workflow_prototype_id: WorkflowPrototypeId,
    workflow_resolver_id: WorkflowResolverId,
    func_id: FuncId,
    func_binding_id: FuncBindingId,
    resources: serde_json::Value,
    #[serde(flatten)]
    context: WorkflowRunnerContext,
    #[serde(flatten)]
    tenancy: Tenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
}

impl_standard_model! {
    model: WorkflowRunner,
    pk: WorkflowRunnerPk,
    id: WorkflowRunnerId,
    table_name: "workflow_runners",
    history_event_label_base: "workflow_runner",
    history_event_message_name: "Workflow Runner"
}

impl WorkflowRunner {
    #[allow(clippy::too_many_arguments)]
    #[instrument(skip_all)]
    pub async fn new(
        ctx: &DalContext,
        workflow_prototype_id: WorkflowPrototypeId,
        workflow_resolver_id: WorkflowResolverId,
        func_id: FuncId,
        func_binding_id: FuncBindingId,
        context: WorkflowRunnerContext,
        resources: &[CommandRunResult],
    ) -> WorkflowRunnerResult<Self> {
        let row = ctx.txns().await?.pg().query_one(
            "SELECT object FROM workflow_runner_create_v1($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
            &[
                ctx.tenancy(),
                ctx.visibility(),
                &workflow_prototype_id,
                &workflow_resolver_id,
                &func_id,
                &func_binding_id,
                &context.component_id(),
                &context.schema_id(),
                &context.schema_variant_id(),
                &serde_json::to_value(resources)?,
            ],
        )
            .await?;

        let object: Self = standard_model::finish_create_from_row(ctx, row).await?;
        Ok(object)
    }

    /// Create a [`WorkflowRunner`](Self) and "run" it immediately. This not only creates the
    /// runner, but also a corresponding, _terminating_
    /// [`WorkflowRunnerState`](crate::workflow_runner::workflow_runner_state::WorkflowRunnerState).
    pub async fn run(
        ctx: &DalContext,
        run_id: usize,
        prototype_id: WorkflowPrototypeId,
        component_id: ComponentId,
    ) -> WorkflowRunnerResult<(
        Self,
        WorkflowRunnerState,
        Vec<FuncBindingReturnValue>,
        Vec<CommandRunResult>,
    )> {
        Self::run_raw(ctx, run_id, prototype_id, component_id, true).await
    }

    /// Version of `Self::run` that doesn't trigger dependent values update for /root/resource updates
    ///
    /// Create a [`WorkflowRunner`](Self) and "run" it immediately. This not only creates the
    /// runner, but also a corresponding, _terminating_
    /// [`WorkflowRunnerState`](crate::workflow_runner::workflow_runner_state::WorkflowRunnerState).
    pub async fn run_without_triggering_dependent_values_update(
        ctx: &DalContext,
        run_id: usize,
        prototype_id: WorkflowPrototypeId,
        component_id: ComponentId,
    ) -> WorkflowRunnerResult<(
        Self,
        WorkflowRunnerState,
        Vec<FuncBindingReturnValue>,
        Vec<CommandRunResult>,
    )> {
        Self::run_raw(ctx, run_id, prototype_id, component_id, false).await
    }

    pub async fn run_raw(
        ctx: &DalContext,
        run_id: usize,
        prototype_id: WorkflowPrototypeId,
        component_id: ComponentId,
        trigger_dependent_values_update: bool,
    ) -> WorkflowRunnerResult<(
        Self,
        WorkflowRunnerState,
        Vec<FuncBindingReturnValue>,
        Vec<CommandRunResult>,
    )> {
        let prototype = WorkflowPrototype::get_by_id(ctx, &prototype_id)
            .await?
            .ok_or(WorkflowRunnerError::PrototypeNotFound(prototype_id))?;

        // NOTE(nick,wendy): it seems like we already generate the tree inside
        // "prototype.resolve()". What is the value of doing this in two steps? It is possible
        // this is better for multiple runs because "prototype.resolve()" will only return the
        // resolver rather than generate a tree and build the resolver. Maybe, we could return
        // a boolean called "created" and only run "resolver.tree()" if "created" is false?
        let resolver = prototype.resolve(ctx, component_id).await?;
        let tree = resolver.tree(ctx).await?;

        // Perform the workflow runner "run" by running the workflow tree.
        let func_binding_return_values = tree.run(ctx, run_id).await?;

        // FIXME(nick): right now, there's nothing stopping a WorkflowTree from operating on
        // multiple Components. Therefore, we take in a "vec" of resources here even though we know
        // that there can only be one (or none) Resource for a given Component.
        let (func_id, func_binding_id, resources) = Self::process_successful_workflow_run(
            ctx,
            &func_binding_return_values,
            component_id,
            trigger_dependent_values_update,
        )
        .await?;
        let (workflow_runner_status, error_message) =
            Self::detect_failure_from_tree_execution(&func_binding_return_values);

        let mut context = WorkflowRunnerContext::new();
        context.set_component_id(prototype.context().component_id);
        context.set_schema_id(prototype.context().schema_id);
        context.set_schema_variant_id(prototype.context().schema_variant_id);

        // TODO(nick,wendy,paulo): create the runner independent of it being ran (either at the
        // beginning of this function or outside of it).
        let runner = Self::new(
            ctx,
            *prototype.id(),
            *resolver.id(),
            func_id,
            func_binding_id,
            context,
            &resources,
        )
        .await?;

        let runner_state = WorkflowRunnerState::new(
            ctx,
            *runner.id(),
            workflow_runner_status,
            None,
            None,
            error_message,
        )
        .await?;

        Ok((runner, runner_state, func_binding_return_values, resources))
    }

    /// Greedy algorithm to find the first instance of failure in a list of given [`FuncBindingReturnValues`](Vec<FuncBindingReturnValue>)
    /// for a given tree execution.
    fn detect_failure_from_tree_execution(
        func_binding_return_values: &Vec<FuncBindingReturnValue>,
    ) -> (WorkflowRunnerStatus, Option<String>) {
        for func_binding_return_value in func_binding_return_values {
            if let Some(value) = func_binding_return_value.value() {
                if let Some((status, message)) =
                    value.get("status").and_then(|s| s.as_str()).and_then(|s| {
                        value
                            .get("message")
                            .and_then(|m| m.as_str().map(|m| (s, m)))
                    })
                {
                    if status != "ok" {
                        return (WorkflowRunnerStatus::Failure, Some(message.to_string()));
                    }
                }
                if let Some(maybe_error) = value.get("error") {
                    if let Some(error) = maybe_error.as_str() {
                        return (WorkflowRunnerStatus::Failure, Some(error.to_string()));
                    }
                }
            }
        }
        (WorkflowRunnerStatus::Success, None)
    }

    /// Upon a successful workflow runner "run" (within [`Self::run()`]), process the result
    /// as desired (e.g. processing logs).
    async fn process_successful_workflow_run(
        ctx: &DalContext,
        func_binding_return_values: &Vec<FuncBindingReturnValue>,
        component_id: ComponentId,
        trigger_dependent_values_update: bool,
    ) -> WorkflowRunnerResult<(FuncId, FuncBindingId, Vec<CommandRunResult>)> {
        let (identity, func_binding, mut func_binding_return_value) =
            Func::identity_with_binding_and_return_value(ctx).await?;

        let mut resources = Vec::new();
        let mut logs = Vec::new();
        for return_value in func_binding_return_values {
            for stream in return_value
                .get_output_stream(ctx)
                .await?
                .unwrap_or_default()
            {
                logs.push(stream);
            }

            if let Some(value) = return_value.value() {
                let mut result = CommandRunResult::deserialize(value)?;
                result.logs = logs.iter().map(|l| l.message.clone()).collect();
                let deleted_ctx = &ctx.clone_with_delete_visibility();
                let mut component = Component::get_by_id(deleted_ctx, &component_id)
                    .await?
                    .ok_or(WorkflowRunnerError::ComponentNotFound(component_id))?;

                if component.needs_destroy() && result.value.is_none() {
                    component
                        .set_needs_destroy(deleted_ctx, false)
                        .await
                        .map_err(Box::new)?;
                }

                if component
                    .set_resource(ctx, result.clone(), trigger_dependent_values_update)
                    .await
                    .map_err(Box::new)?
                {
                    WsEvent::resource_refreshed(ctx, *component.id())
                        .await?
                        .publish_on_commit(ctx)
                        .await?;
                }

                resources.push(result);
            }
        }
        logs.sort_by_key(|log| log.timestamp);
        if func_binding_return_value.func_execution_pk().is_none() {
            let pk = FuncExecution::new(ctx, &identity, &func_binding)
                .await?
                .pk();
            func_binding_return_value
                .set_func_execution_pk(ctx, pk)
                .await?;
        }

        let mut func_execution =
            FuncExecution::get_by_pk(ctx, &func_binding_return_value.func_execution_pk()).await?;
        func_execution.set_output_stream(ctx, logs).await?;

        Ok((*identity.id(), *func_binding.id(), resources))
    }

    standard_model_accessor!(
        workflow_prototype_id,
        Pk(WorkflowPrototypeId),
        WorkflowRunnerResult
    );
    standard_model_accessor!(func_id, Pk(FuncId), WorkflowRunnerResult);
    standard_model_accessor!(func_binding_id, Pk(FuncBindingId), WorkflowRunnerResult);

    pub async fn find_for_prototype(
        ctx: &DalContext,
        workflow_prototype_id: &WorkflowPrototypeId,
        context: WorkflowRunnerContext,
    ) -> WorkflowRunnerResult<Vec<Self>> {
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(
                FIND_FOR_PROTOTYPE,
                &[
                    ctx.tenancy(),
                    ctx.visibility(),
                    workflow_prototype_id,
                    &context.component_id(),
                    &context.schema_id(),
                    &context.schema_variant_id(),
                ],
            )
            .await?;
        let object = standard_model::objects_from_rows(rows)?;
        Ok(object)
    }

    pub fn resources(&self) -> WorkflowRunnerResult<Vec<CommandRunResult>> {
        Ok(serde_json::from_value(self.resources.clone())?)
    }
}
