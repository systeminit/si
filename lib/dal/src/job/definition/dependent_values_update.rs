use std::{
    collections::{
        HashMap,
        HashSet,
        hash_map::Entry,
    },
    sync::Arc,
};

use async_trait::async_trait;
use audit_log::DependentValueUpdateAuditLogError;
use pinga_core::api_types::job_execution_request::JobArgsVCurrent;
use serde::{
    Deserialize,
    Serialize,
};
use si_events::FuncRunValue;
use si_frontend_types::DiagramSocket;
use si_id::{
    ChangeSetId,
    SchemaVariantId,
};
use telemetry::prelude::*;
use telemetry_utils::metric;
use thiserror::Error;
use tokio::{
    sync::RwLock,
    task::{
        JoinError,
        JoinSet,
    },
};
use ulid::Ulid;

use crate::{
    AttributeValue,
    AttributeValueId,
    ChangeSet,
    ChangeSetError,
    ChangeSetStatus,
    Component,
    ComponentError,
    ComponentId,
    DalContext,
    Func,
    SchemaVariantError,
    TransactionsError,
    WorkspacePk,
    WorkspaceSnapshotError,
    WsEvent,
    WsEventError,
    action::ActionError,
    attribute::value::{
        AttributeValueError,
        dependent_value_graph::DependentValueGraph,
    },
    job::consumer::{
        DalJob,
        JobCompletionState,
        JobConsumer,
        JobConsumerResult,
    },
    prop::PropError,
    status::{
        StatusMessageState,
        StatusUpdate,
        StatusUpdateError,
    },
    workspace_snapshot::{
        DependentValueRoot,
        dependent_value_root::DependentValueRootError,
    },
};

#[remain::sorted]
#[derive(Debug, Error)]
pub enum DependentValueUpdateError {
    #[error("action error: {0}")]
    Action(#[from] Box<ActionError>),
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] Box<AttributeValueError>),
    #[error("change set error: {0}")]
    ChangeSet(#[from] Box<ChangeSetError>),
    #[error("component error: {0}")]
    Component(#[from] Box<ComponentError>),
    #[error("dependent value root error: {0}")]
    DependentValueRoot(#[from] Box<DependentValueRootError>),
    #[error("dependent values update audit log error: {0}")]
    DependentValuesUpdateAuditLog(#[from] Box<DependentValueUpdateAuditLogError>),
    #[error("func error: {0}")]
    FuncError(#[from] Box<crate::FuncError>),
    #[error("prop error: {0}")]
    Prop(#[from] Box<PropError>),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] Box<SchemaVariantError>),
    #[error("status update error: {0}")]
    StatusUpdate(#[from] Box<StatusUpdateError>),
    #[error(transparent)]
    TokioTask(#[from] JoinError),
    #[error(transparent)]
    Transactions(#[from] Box<TransactionsError>),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] Box<WorkspaceSnapshotError>),
    #[error("ws event error: {0}")]
    WsEvent(#[from] Box<WsEventError>),
}

impl From<ActionError> for DependentValueUpdateError {
    fn from(value: ActionError) -> Self {
        Box::new(value).into()
    }
}

impl From<AttributeValueError> for DependentValueUpdateError {
    fn from(value: AttributeValueError) -> Self {
        Box::new(value).into()
    }
}

impl From<ChangeSetError> for DependentValueUpdateError {
    fn from(value: ChangeSetError) -> Self {
        Box::new(value).into()
    }
}

impl From<ComponentError> for DependentValueUpdateError {
    fn from(value: ComponentError) -> Self {
        Box::new(value).into()
    }
}

impl From<DependentValueRootError> for DependentValueUpdateError {
    fn from(value: DependentValueRootError) -> Self {
        Box::new(value).into()
    }
}

impl From<DependentValueUpdateAuditLogError> for DependentValueUpdateError {
    fn from(value: DependentValueUpdateAuditLogError) -> Self {
        Box::new(value).into()
    }
}

impl From<crate::FuncError> for DependentValueUpdateError {
    fn from(value: crate::FuncError) -> Self {
        Box::new(value).into()
    }
}

impl From<PropError> for DependentValueUpdateError {
    fn from(value: PropError) -> Self {
        Box::new(value).into()
    }
}

impl From<SchemaVariantError> for DependentValueUpdateError {
    fn from(value: SchemaVariantError) -> Self {
        Box::new(value).into()
    }
}

impl From<StatusUpdateError> for DependentValueUpdateError {
    fn from(value: StatusUpdateError) -> Self {
        Box::new(value).into()
    }
}

impl From<TransactionsError> for DependentValueUpdateError {
    fn from(value: TransactionsError) -> Self {
        Box::new(value).into()
    }
}

impl From<WorkspaceSnapshotError> for DependentValueUpdateError {
    fn from(value: WorkspaceSnapshotError) -> Self {
        Box::new(value).into()
    }
}

impl From<WsEventError> for DependentValueUpdateError {
    fn from(value: WsEventError) -> Self {
        Box::new(value).into()
    }
}

pub type DependentValueUpdateResult<T> = Result<T, DependentValueUpdateError>;

#[derive(Debug, Deserialize, Serialize)]
struct DependentValuesUpdateArgs;

impl From<DependentValuesUpdate> for DependentValuesUpdateArgs {
    fn from(_value: DependentValuesUpdate) -> Self {
        Self
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct DependentValuesUpdate {
    workspace_id: WorkspacePk,
    change_set_id: ChangeSetId,
    #[serde(skip)]
    set_value_lock: Arc<RwLock<()>>,
}

impl DependentValuesUpdate {
    pub fn new(workspace_id: WorkspacePk, change_set_id: ChangeSetId) -> Box<Self> {
        Box::new(Self {
            workspace_id,
            change_set_id,
            set_value_lock: Arc::new(RwLock::new(())),
        })
    }
}

impl DalJob for DependentValuesUpdate {
    fn args(&self) -> JobArgsVCurrent {
        JobArgsVCurrent::DependentValuesUpdate
    }

    fn workspace_id(&self) -> WorkspacePk {
        self.workspace_id
    }

    fn change_set_id(&self) -> ChangeSetId {
        self.change_set_id
    }
}

#[async_trait]
impl JobConsumer for DependentValuesUpdate {
    #[instrument(
        name = "dependent_values_update.run",
        level = "info",
        skip_all,
        fields(
            si.change_set.id = Empty,
            si.workspace.id = Empty,
        ),
    )]
    async fn run(&self, ctx: &mut DalContext) -> JobConsumerResult<JobCompletionState> {
        let span = current_span_for_instrument_at!("info");

        span.record("si.change_set.id", ctx.change_set_id().to_string());
        span.record(
            "si.workspace.id",
            ctx.tenancy()
                .workspace_pk_opt()
                .unwrap_or(WorkspacePk::NONE)
                .to_string(),
        );

        let change_set = ChangeSet::get_by_id(ctx, ctx.change_set_id()).await?;

        if change_set.status == ChangeSetStatus::Abandoned {
            info!("DVU enqueued for abandoned change set. Returning early");
            return Ok(JobCompletionState::Done);
        }

        Ok(self.inner_run(ctx).await?)
    }
}

struct StatusUpdateTracker {
    values_by_component: HashMap<ComponentId, HashSet<AttributeValueId>>,
    components_by_value: HashMap<AttributeValueId, ComponentId>,
    active_components: HashSet<ComponentId>,
}

impl StatusUpdateTracker {
    async fn new_for_values(
        ctx: &DalContext,
        value_ids: Vec<AttributeValueId>,
    ) -> DependentValueUpdateResult<Self> {
        let mut tracker = Self {
            values_by_component: HashMap::new(),
            components_by_value: HashMap::new(),
            active_components: HashSet::new(),
        };

        for value_id in value_ids {
            let component_id = AttributeValue::component_id(ctx, value_id).await?;
            tracker
                .values_by_component
                .entry(component_id)
                .and_modify(|values: &mut HashSet<AttributeValueId>| {
                    values.insert(value_id);
                })
                .or_default();
            tracker.components_by_value.insert(value_id, component_id);
        }

        Ok(tracker)
    }

    fn active_components_count(&self) -> usize {
        self.active_components.len()
    }

    fn would_start_component(&self, value_id: AttributeValueId) -> bool {
        self.components_by_value
            .get(&value_id)
            .is_some_and(|component_id| !self.active_components.contains(component_id))
    }

    fn start_value(&mut self, value_id: AttributeValueId) -> Option<ComponentId> {
        self.components_by_value
            .get(&value_id)
            .and_then(|component_id| {
                self.active_components
                    .insert(*component_id)
                    .then_some(*component_id)
            })
    }

    fn finish_value(&mut self, value_id: AttributeValueId) -> Option<ComponentId> {
        self.components_by_value
            .get(&value_id)
            .and_then(
                |component_id| match self.values_by_component.entry(*component_id) {
                    Entry::Occupied(mut values_entry) => {
                        let values = values_entry.get_mut();
                        values.remove(&value_id);
                        values.is_empty().then_some(*component_id)
                    }
                    Entry::Vacant(_) => None,
                },
            )
    }

    fn finish_remaining(&self) -> Vec<StatusUpdate> {
        self.values_by_component
            .iter()
            .filter(|(_, values)| !values.is_empty())
            .map(|(component_id, _)| {
                StatusUpdate::new_dvu(StatusMessageState::StatusFinished, *component_id)
            })
            .collect()
    }

    fn get_status_update(
        &mut self,
        state: StatusMessageState,
        value_id: AttributeValueId,
    ) -> Option<StatusUpdate> {
        match state {
            StatusMessageState::StatusFinished => self.finish_value(value_id),
            StatusMessageState::StatusStarted => self.start_value(value_id),
        }
        .map(|component_id| StatusUpdate::new_dvu(state, component_id))
    }
}

impl DependentValuesUpdate {
    async fn inner_run(
        &self,
        ctx: &mut DalContext,
    ) -> DependentValueUpdateResult<JobCompletionState> {
        let start = tokio::time::Instant::now();
        let span = Span::current();
        let roots = DependentValueRoot::take_dependent_values(ctx).await?;
        let is_on_head = ChangeSet::get_by_id(ctx, self.change_set_id)
            .await?
            .is_head(ctx)
            .await?;
        let mut unfinished_values: HashSet<Ulid> = HashSet::new();
        let mut finished_values: HashSet<Ulid> = HashSet::new();

        roots.iter().for_each(|root| match root {
            DependentValueRoot::Finished(ulid) => {
                finished_values.insert((*ulid).into());
            }
            DependentValueRoot::Unfinished(ulid) => {
                unfinished_values.insert((*ulid).into());
            }
        });

        // If we have no unfinished values, and only finished values, this is a
        // legacy snapshot where all dvus were accidentally marked "finished"
        if unfinished_values.is_empty() {
            unfinished_values.clone_from(&finished_values);
            finished_values.clear();
        }

        let concurrency_limit = ctx.get_workspace().await?.component_concurrency_limit() as usize;

        let mut dependency_graph = DependentValueGraph::new(ctx, roots).await?;

        debug!(
            "DependentValueGraph calculation took: {:?}",
            start.elapsed()
        );

        // Remove the first set of independent_values since they should already have had their functions executed
        for value_id in dependency_graph.independent_values() {
            if !dependency_graph.values_needs_to_execute_from_prototype_function(value_id)
                || (finished_values.contains(&value_id.into())
                    && !unfinished_values.contains(&value_id.into()))
            {
                dependency_graph.remove_value(value_id);
            }
        }
        let all_value_ids = dependency_graph.all_value_ids();

        let mut tracker = StatusUpdateTracker::new_for_values(ctx, all_value_ids).await?;

        let mut spawned_ids = HashSet::new();
        let mut task_id_to_av_id = HashMap::new();
        let mut update_join_set = JoinSet::new();
        let mut independent_value_ids: HashSet<AttributeValueId> =
            dependency_graph.independent_values().into_iter().collect();
        let mut would_start_ids = HashSet::new();

        loop {
            if independent_value_ids.is_empty() && task_id_to_av_id.is_empty() {
                break;
            }

            if independent_value_ids
                .difference(&would_start_ids)
                .next()
                .is_none()
            {
                if task_id_to_av_id.is_empty() {
                    break;
                }
            } else {
                for attribute_value_id in &independent_value_ids {
                    let attribute_value_id = *attribute_value_id;
                    let parent_span = span.clone();
                    if !spawned_ids.contains(&attribute_value_id)
                        && !would_start_ids.contains(&attribute_value_id)
                    {
                        let id = Ulid::new();

                        if tracker.would_start_component(attribute_value_id)
                            && tracker.active_components_count() >= concurrency_limit
                        {
                            would_start_ids.insert(attribute_value_id);
                            continue;
                        }

                        let status_update = tracker.get_status_update(
                            StatusMessageState::StatusStarted,
                            attribute_value_id,
                        );

                        let before_value = AttributeValue::get_by_id(ctx, attribute_value_id)
                            .await?
                            .value(ctx)
                            .await?;
                        metric!(counter.dvu.function_execution = 1);

                        update_join_set.spawn(values_from_prototype_function_execution(
                            id,
                            parent_span,
                            ctx.clone(),
                            attribute_value_id,
                            before_value,
                            self.set_value_lock.clone(),
                            status_update,
                        ));
                        task_id_to_av_id.insert(id, attribute_value_id);
                        spawned_ids.insert(attribute_value_id);
                    }
                }
            }

            // Wait for a task to finish
            if let Some(join_result) = update_join_set.join_next().await {
                let (task_id, execution_result, before_value) = join_result?;

                metric!(counter.dvu.function_execution = -1);

                if let Some(finished_value_id) = task_id_to_av_id.remove(&task_id) {
                    match execution_result {
                        Ok((execution_values, func, input_attribute_value_ids)) => {
                            // Lock the graph for writing inside this job. The
                            // lock will be released when this guard is dropped
                            // at the end of the scope.
                            let value_is_changed =
                                before_value.as_ref() != execution_values.value();
                            let write_guard = self.set_value_lock.write().await;

                            // Only set values if their functions are actually
                            // "dependent". Other values may have been
                            // introduced to the attribute value graph because
                            // of child-parent prop dependencies, but these
                            // values themselves do not need to change (they are
                            // always Objects, Maps, or Arrays set by
                            // setObject/setArray/setMap and are not updated in
                            // the dependent value execution). If we forced
                            // these container values to update here, we might
                            // touch child properties unnecessarily.
                            match AttributeValue::is_set_by_dependent_function(
                                ctx,
                                finished_value_id,
                            )
                            .await
                            {
                                Ok(true) => match AttributeValue::set_values_from_func_run_value(
                                    ctx,
                                    finished_value_id,
                                    execution_values,
                                    func.clone(),
                                )
                                .await
                                {
                                    Ok(_) => {
                                        // Remove the value, so that any values that depend on it will
                                        // become independent values (once all other dependencies are removed)
                                        dependency_graph.remove_value(finished_value_id);
                                        drop(write_guard);
                                        // if we're not on head, and the value is different after processing,
                                        // let's see if we should enqueue an update function
                                        if !is_on_head && value_is_changed {
                                            Component::enqueue_update_action_if_applicable(
                                                ctx,
                                                finished_value_id,
                                            )
                                            .await?;
                                        }

                                        // Publish the audit log for the updated dependent value.
                                        audit_log::write(
                                            ctx,
                                            finished_value_id,
                                            input_attribute_value_ids,
                                            func,
                                            before_value,
                                        )
                                        .await?;
                                    }
                                    Err(err) => {
                                        execution_error(ctx, err.to_string(), finished_value_id)
                                            .await;
                                        dependency_graph.cycle_on_self(finished_value_id);
                                    }
                                },
                                Ok(false) => {
                                    dependency_graph.remove_value(finished_value_id);
                                }
                                Err(err) => {
                                    execution_error(ctx, err.to_string(), finished_value_id).await;
                                    dependency_graph.cycle_on_self(finished_value_id);
                                }
                            }
                        }
                        Err(err) => {
                            // By adding an outgoing edge from the failed node to itself it will
                            // never appear in the `independent_values` call above since that looks for
                            // nodes *without* outgoing edges. Thus we will never try to re-execute
                            // the function for this value, nor will we execute anything in the
                            // dependency graph connected to this value
                            let read_guard = self.set_value_lock.read().await;
                            execution_error(ctx, err.to_string(), finished_value_id).await;
                            drop(read_guard);
                            dependency_graph.cycle_on_self(finished_value_id);
                        }
                    }

                    if let Some(status_update) = tracker
                        .get_status_update(StatusMessageState::StatusFinished, finished_value_id)
                    {
                        if let Err(err) = send_status_update(ctx, status_update).await {
                            error!(si.error.message = ?err, "status update finished event send failed for AttributeValue {finished_value_id}");
                        }
                    }
                }
            }

            independent_value_ids = dependency_graph.independent_values().into_iter().collect();
        }

        let mut added_unfinished = false;
        for value_id in &independent_value_ids {
            if spawned_ids.contains(value_id) {
                DependentValueRoot::add_dependent_value_root(
                    ctx,
                    DependentValueRoot::Finished(value_id.into()),
                )
                .await?;
            } else {
                added_unfinished = true;
                DependentValueRoot::add_dependent_value_root(
                    ctx,
                    DependentValueRoot::Unfinished(value_id.into()),
                )
                .await?;
            }
        }

        // If we encounter a failure when executing the values above, we may not
        // process the downstream attributes and thus will fail to send the
        // "finish" update. So we send the "finish" update here to ensure the
        // frontend can continue to work on the snapshot.
        //
        // We also want to ensure that we don't add a set of only finished
        // values.
        if independent_value_ids.is_empty() || !added_unfinished {
            for status_update in tracker.finish_remaining() {
                if let Err(err) = send_status_update(ctx, status_update).await {
                    error!(si.error.message = ?err, "status update finished event send for leftover component failed");
                }
            }
            DependentValueRoot::take_dependent_values(ctx).await?;
        }

        debug!("DependentValuesUpdate took: {:?}", start.elapsed());

        ctx.commit().await?;
        Ok(JobCompletionState::Done)
    }
}

async fn execution_error(
    ctx: &DalContext,
    err_string: String,
    attribute_value_id: AttributeValueId,
) {
    let fallback = format!(
        "error executing prototype function for AttributeValue {attribute_value_id}: {err_string}"
    );
    let error_message = match execution_error_detail(ctx, attribute_value_id).await {
        Ok(detail) => {
            format!("{detail}: {err_string}")
        }
        _ => fallback,
    };

    warn!(name = "function_execution_error", si.error.message = error_message, %attribute_value_id);
}

async fn execution_error_detail(
    ctx: &DalContext,
    id: AttributeValueId,
) -> DependentValueUpdateResult<String> {
    let is_for = AttributeValue::is_for(ctx, id)
        .await?
        .debug_info(ctx)
        .await?;
    let prototype_func =
        Func::node_weight(ctx, AttributeValue::prototype_func_id(ctx, id).await?).await?;

    Ok(format!(
        "error executing prototype function \"{}\" to set the value of {is_for} ({id})",
        prototype_func.name()
    ))
}

type PrototypeFunctionExecutionResult = (
    Ulid,
    DependentValueUpdateResult<(FuncRunValue, Func, Vec<AttributeValueId>)>,
    Option<serde_json::Value>,
);

/// Wrapper around `AttributeValue.values_from_prototype_function_execution(&ctx)` to get it to
/// play more nicely with being spawned into a `JoinSet`.
#[instrument(
    name = "dependent_values_update.values_from_prototype_function_execution",
    level = "debug",
    parent = &parent_span,
    skip_all,
    fields(
        si.attribute_value.id = %attribute_value_id,
    ),
)]
async fn values_from_prototype_function_execution(
    task_id: Ulid,
    parent_span: Span,
    ctx: DalContext,
    attribute_value_id: AttributeValueId,
    before_value: Option<serde_json::Value>,
    set_value_lock: Arc<RwLock<()>>,
    status_update: Option<StatusUpdate>,
) -> PrototypeFunctionExecutionResult {
    if let Some(status_update) = status_update {
        if let Err(err) = send_status_update(&ctx, status_update).await {
            return (task_id, Err(err), before_value.clone());
        }
    }

    let result =
        AttributeValue::execute_prototype_function(&ctx, attribute_value_id, set_value_lock)
            .await
            .map_err(Into::into);

    (task_id, result, before_value)
}

async fn send_status_update(
    ctx: &DalContext,
    status_update: StatusUpdate,
) -> DependentValueUpdateResult<()> {
    WsEvent::status_update(ctx, status_update.clone())
        .await?
        .publish_immediately(ctx)
        .await?;
    // If this is the finished event, we should also ensure we send
    // component_updated when the rebase happens in the job.

    // another wack-a-mole event needed that I'm excited to not have to do
    // ever again.
    if let StatusUpdate::DependentValueUpdate {
        status,
        component_id,
        ..
    } = status_update
    {
        if status == StatusMessageState::StatusFinished {
            let mut diagram_sockets: HashMap<SchemaVariantId, Vec<DiagramSocket>> = HashMap::new();
            let component = Component::get_by_id(ctx, component_id).await?;
            let payload = component
                .into_frontend_type_for_default_view(
                    ctx,
                    component.change_status(ctx).await?,
                    &mut diagram_sockets,
                )
                .await?;
            // don't publish immediately, we want this fired when the rebase lands
            WsEvent::component_updated(ctx, payload)
                .await?
                .publish_on_commit(ctx)
                .await?;
        }
    }
    Ok(())
}

pub mod audit_log {
    use si_events::audit_log::AuditLogKind;
    use telemetry::prelude::*;
    use thiserror::Error;

    use crate::{
        AttributeValue,
        AttributeValueId,
        Component,
        ComponentError,
        DalContext,
        Func,
        InputSocket,
        OutputSocket,
        Prop,
        TransactionsError,
        attribute::value::{
            AttributeValueError,
            ValueIsFor,
        },
        prop::PropError,
        socket::{
            input::InputSocketError,
            output::OutputSocketError,
        },
    };

    #[remain::sorted]
    #[derive(Debug, Error)]
    pub enum DependentValueUpdateAuditLogError {
        #[error("attribute value error: {0}")]
        AttributeValue(#[from] Box<AttributeValueError>),
        #[error("component error: {0}")]
        Component(#[from] Box<ComponentError>),
        #[error("input socket error: {0}")]
        InputSocket(#[from] Box<InputSocketError>),
        #[error("output socket error: {0}")]
        OutputSocket(#[from] Box<OutputSocketError>),
        #[error("prop error: {0}")]
        Prop(#[from] Box<PropError>),
        #[error("write audit log error: {0}")]
        WriteAuditLog(#[source] Box<TransactionsError>),
    }

    impl From<AttributeValueError> for DependentValueUpdateAuditLogError {
        fn from(value: AttributeValueError) -> Self {
            Box::new(value).into()
        }
    }

    impl From<ComponentError> for DependentValueUpdateAuditLogError {
        fn from(value: ComponentError) -> Self {
            Box::new(value).into()
        }
    }

    impl From<InputSocketError> for DependentValueUpdateAuditLogError {
        fn from(value: InputSocketError) -> Self {
            Box::new(value).into()
        }
    }

    impl From<OutputSocketError> for DependentValueUpdateAuditLogError {
        fn from(value: OutputSocketError) -> Self {
            Box::new(value).into()
        }
    }

    impl From<PropError> for DependentValueUpdateAuditLogError {
        fn from(value: PropError) -> Self {
            Box::new(value).into()
        }
    }

    impl From<TransactionsError> for DependentValueUpdateAuditLogError {
        fn from(value: TransactionsError) -> Self {
            Self::WriteAuditLog(Box::new(value))
        }
    }

    #[instrument(
        name = "dependent_values_update.audit_log.write",
        level = "trace",
        skip_all
    )]
    pub async fn write(
        ctx: &DalContext,
        finished_value_id: AttributeValueId,
        input_attribute_value_ids: Vec<AttributeValueId>,
        func: Func,
        before_value: Option<serde_json::Value>,
    ) -> Result<(), DependentValueUpdateAuditLogError> {
        // Metadata for who "owns" the attribute value.
        let component_id = AttributeValue::component_id(ctx, finished_value_id).await?;
        let component = Component::get_by_id(ctx, component_id).await?;
        let component_name = component.name(ctx).await?;
        let component_schema_variant = component.schema_variant(ctx).await?;

        // Metadata for the attribute value.
        let after_value = AttributeValue::get_by_id(ctx, finished_value_id)
            .await?
            .value(ctx)
            .await?;
        let is_for = AttributeValue::is_for(ctx, finished_value_id).await?;

        // Write an audit log based on what the attribute value is for.
        match is_for {
            ValueIsFor::InputSocket(input_socket_id) => {
                let input_socket = InputSocket::get_by_id(ctx, input_socket_id).await?;
                ctx.write_audit_log(
                    AuditLogKind::UpdateDependentInputSocket {
                        input_socket_id,
                        input_socket_name: input_socket.name().to_owned(),
                        attribute_value_id: finished_value_id,
                        input_attribute_value_ids: input_attribute_value_ids.into_iter().collect(),
                        func_id: func.id,
                        func_display_name: func.display_name,
                        func_name: func.name,
                        component_id,
                        component_name,
                        schema_variant_id: component_schema_variant.id(),
                        schema_variant_display_name: component_schema_variant
                            .display_name()
                            .to_string(),
                        before_value,
                        after_value,
                    },
                    input_socket.name().to_owned(),
                )
                .await
                .map_err(|e| DependentValueUpdateAuditLogError::WriteAuditLog(Box::new(e)))?;
            }
            ValueIsFor::OutputSocket(output_socket_id) => {
                let output_socket = OutputSocket::get_by_id(ctx, output_socket_id).await?;
                ctx.write_audit_log(
                    AuditLogKind::UpdateDependentOutputSocket {
                        output_socket_id,
                        output_socket_name: output_socket.name().to_owned(),
                        attribute_value_id: finished_value_id,
                        input_attribute_value_ids: input_attribute_value_ids.into_iter().collect(),
                        func_id: func.id,
                        func_display_name: func.display_name,
                        func_name: func.name,
                        component_id,
                        component_name,
                        schema_variant_id: component_schema_variant.id(),
                        schema_variant_display_name: component_schema_variant
                            .display_name()
                            .to_string(),
                        before_value,
                        after_value,
                    },
                    output_socket.name().to_owned(),
                )
                .await
                .map_err(|e| DependentValueUpdateAuditLogError::WriteAuditLog(Box::new(e)))?;
            }
            ValueIsFor::Prop(prop_id) => {
                let prop = Prop::get_by_id(ctx, prop_id).await?;
                ctx.write_audit_log(
                    AuditLogKind::UpdateDependentProperty {
                        prop_id,
                        prop_name: prop.name.to_owned(),
                        attribute_value_id: finished_value_id,
                        input_attribute_value_ids: input_attribute_value_ids.into_iter().collect(),
                        func_id: func.id,
                        func_display_name: func.display_name,
                        func_name: func.name,
                        component_id,
                        component_name,
                        schema_variant_id: component_schema_variant.id(),
                        schema_variant_display_name: component_schema_variant
                            .display_name()
                            .to_string(),
                        before_value,
                        after_value,
                    },
                    prop.name,
                )
                .await
                .map_err(|e| DependentValueUpdateAuditLogError::WriteAuditLog(Box::new(e)))?;
            }
        }

        Ok(())
    }
}
