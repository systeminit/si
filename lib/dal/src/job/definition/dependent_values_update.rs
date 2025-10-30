use std::{
    collections::{
        BTreeMap,
        BTreeSet,
        btree_map,
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
use si_id::ChangeSetId;
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
    AttributePrototype,
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
    attribute::{
        prototype::AttributePrototypeError,
        value::{
            AttributeValueError,
            PrototypeExecution,
            dependent_value_graph::{
                DependentValue,
                DependentValueGraph,
            },
        },
    },
    job::consumer::{
        DalJob,
        JobCompletionState,
        JobConsumer,
        JobConsumerResult,
    },
    prop::PropError,
    schema::leaf::{
        LeafPrototype,
        LeafPrototypeError,
    },
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
    #[error("attribute prototype error: {0}")]
    AttributePrototype(#[from] Box<AttributePrototypeError>),
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
    #[error("leaf prototype error: {0}")]
    LeafPrototype(#[from] Box<LeafPrototypeError>),
    #[error("prop error: {0}")]
    Prop(#[from] Box<PropError>),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] Box<SchemaVariantError>),
    #[error("status update error: {0}")]
    StatusUpdate(#[from] Box<StatusUpdateError>),
    #[error("tokio task error: {0}")]
    TokioTask(#[from] JoinError),
    #[error("transactions error: {0}")]
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

impl From<LeafPrototypeError> for DependentValueUpdateError {
    fn from(value: LeafPrototypeError) -> Self {
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
    values_by_component: BTreeMap<ComponentId, BTreeSet<AttributeValueId>>,
    components_by_value: BTreeMap<AttributeValueId, ComponentId>,
    active_components: BTreeSet<ComponentId>,
}

impl StatusUpdateTracker {
    async fn new_for_values(
        ctx: &DalContext,
        value_ids: Vec<AttributeValueId>,
    ) -> DependentValueUpdateResult<Self> {
        let mut tracker = Self {
            values_by_component: BTreeMap::new(),
            components_by_value: BTreeMap::new(),
            active_components: BTreeSet::new(),
        };

        for value_id in value_ids {
            let component_id = AttributeValue::component_id(ctx, value_id).await?;
            tracker
                .values_by_component
                .entry(component_id)
                .and_modify(|values: &mut BTreeSet<AttributeValueId>| {
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

    fn would_start_component(&self, value: impl Into<AttributeValueId>) -> bool {
        self.components_by_value
            .get(&(value.into()))
            .is_some_and(|component_id| !self.active_components.contains(component_id))
    }

    fn start_value(&mut self, value: impl Into<AttributeValueId>) -> Option<ComponentId> {
        self.components_by_value
            .get(&(value.into()))
            .and_then(|component_id| {
                self.active_components
                    .insert(*component_id)
                    .then_some(*component_id)
            })
    }

    fn finish_value(&mut self, value: impl Into<AttributeValueId>) -> Option<ComponentId> {
        let value = value.into();
        self.components_by_value
            .get(&value)
            .and_then(
                |component_id| match self.values_by_component.entry(*component_id) {
                    btree_map::Entry::Occupied(mut values_entry) => {
                        let values = values_entry.get_mut();
                        values.remove(&value);
                        values.is_empty().then_some(*component_id)
                    }
                    btree_map::Entry::Vacant(_) => None,
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
        value: impl Into<AttributeValueId>,
    ) -> Option<StatusUpdate> {
        let value = value.into();
        match state {
            StatusMessageState::StatusFinished => self.finish_value(value),
            StatusMessageState::StatusStarted => self.start_value(value),
        }
        .map(|component_id| StatusUpdate::new_dvu(state, component_id))
    }
}

#[remain::sorted]
enum RemoveOrCycle {
    CycleOnSelf,
    RemoveAndLog(AttributeValueId),
    RemoveButIgnore,
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
        let mut unfinished_values: BTreeSet<Ulid> = BTreeSet::new();
        let mut finished_values: BTreeSet<Ulid> = BTreeSet::new();

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
        for independent_value in dependency_graph.independent_values() {
            let av_id = independent_value.attribute_value_id();
            if !dependency_graph.values_needs_to_execute_from_prototype_function(independent_value)
                || (finished_values.contains(&av_id.into())
                    && !unfinished_values.contains(&av_id.into()))
            {
                dependency_graph.remove_value(independent_value);
            }
        }
        let all_value_ids: Vec<_> = dependency_graph.all_value_ids().collect();

        let mut tracker = StatusUpdateTracker::new_for_values(
            ctx,
            all_value_ids
                .iter()
                .map(|v| v.attribute_value_id())
                .collect(),
        )
        .await?;

        let mut spawned_ids = BTreeSet::new();
        let mut task_id_to_av_id = BTreeMap::new();
        let mut update_join_set = JoinSet::new();
        let mut independent_values: BTreeSet<DependentValue> =
            dependency_graph.independent_values().into_iter().collect();
        let mut would_start_ids = BTreeSet::new();

        loop {
            if independent_values.is_empty() && task_id_to_av_id.is_empty() {
                break;
            }

            if independent_values
                .difference(&would_start_ids)
                .next()
                .is_none()
            {
                if task_id_to_av_id.is_empty() {
                    break;
                }
            } else {
                for &independent_value in &independent_values {
                    let parent_span = span.clone();
                    if !spawned_ids.contains(&independent_value)
                        && !would_start_ids.contains(&independent_value)
                    {
                        let id = Ulid::new();

                        if tracker.would_start_component(independent_value)
                            && tracker.active_components_count() >= concurrency_limit
                        {
                            would_start_ids.insert(independent_value);
                            continue;
                        }

                        let status_update = tracker.get_status_update(
                            StatusMessageState::StatusStarted,
                            independent_value,
                        );

                        let before_value = match get_before_value(
                            ctx,
                            independent_value,
                            self.set_value_lock.clone(),
                        )
                        .await
                        {
                            Ok(value) => value,
                            Err(err) => {
                                execution_error(
                                    ctx,
                                    err.to_string(),
                                    independent_value.attribute_value_id(),
                                )
                                .await;

                                dependency_graph.cycle_on_self(independent_value);
                                spawned_ids.insert(independent_value);

                                // Couldn't get the before value? skip!
                                continue;
                            }
                        };

                        metric!(counter.dvu.function_execution = 1);
                        update_join_set.spawn(values_from_prototype_function_execution(
                            id,
                            parent_span,
                            ctx.clone(),
                            independent_value,
                            before_value,
                            self.set_value_lock.clone(),
                            status_update,
                        ));
                        task_id_to_av_id.insert(id, independent_value);
                        spawned_ids.insert(independent_value);
                    }
                }
            }

            // Wait for a task to finish
            if let Some(join_result) = update_join_set.join_next().await {
                let DependentProtoExecution {
                    task_id,
                    result: execution_result,
                    before_value,
                } = join_result?;

                metric!(counter.dvu.function_execution = -1);

                if let Some(finished_value) = task_id_to_av_id.remove(&task_id) {
                    match execution_result {
                        Ok(proto_execution) => {
                            let PrototypeExecution {
                                func_run_value: execution_values,
                                func,
                                input_attribute_value_ids,
                                value_id: executed_value_id,
                            } = proto_execution;

                            // Lock the graph for writing inside this job. The
                            // lock will be released when this guard is dropped
                            // at the end of the scope.
                            let value_is_changed = is_value_changed(
                                before_value.as_ref(),
                                execution_values.unprocessed_value(),
                            );

                            let after_value = if value_is_changed {
                                execution_values.unprocessed_value().cloned()
                            } else {
                                None
                            };

                            let write_guard = self.set_value_lock.write().await;

                            let remove_or_cycle = set_attribute_value_after_func_execution(
                                ctx,
                                finished_value,
                                is_on_head,
                                before_value,
                                execution_values,
                                func.clone(),
                                input_attribute_value_ids,
                                executed_value_id,
                                value_is_changed,
                                after_value,
                            )
                            .await;

                            drop(write_guard);

                            match remove_or_cycle {
                                RemoveOrCycle::RemoveButIgnore | RemoveOrCycle::RemoveAndLog(_) => {
                                    dependency_graph.remove_value(finished_value)
                                }
                                RemoveOrCycle::CycleOnSelf => {
                                    dependency_graph.cycle_on_self(finished_value)
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
                            execution_error(
                                ctx,
                                err.to_string(),
                                finished_value.attribute_value_id(),
                            )
                            .await;
                            drop(read_guard);
                            dependency_graph.cycle_on_self(finished_value);
                        }
                    }

                    if let Some(status_update) = tracker
                        .get_status_update(StatusMessageState::StatusFinished, finished_value)
                    {
                        if let Err(err) = send_status_update(ctx, status_update).await {
                            error!(si.error.message = ?err, "status update finished event send failed for AttributeValue {finished_value:?}");
                        }
                    }
                }
            }

            independent_values = dependency_graph.independent_values().into_iter().collect();
        }

        let mut added_unfinished = false;
        for value in &independent_values {
            if spawned_ids.contains(value) {
                DependentValueRoot::add_dependent_value_root(
                    ctx,
                    DependentValueRoot::Finished(value.attribute_value_id().into()),
                )
                .await?;
            } else {
                added_unfinished = true;
                DependentValueRoot::add_dependent_value_root(
                    ctx,
                    DependentValueRoot::Unfinished(value.attribute_value_id().into()),
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
        if independent_values.is_empty() || !added_unfinished {
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

#[allow(clippy::too_many_arguments)]
async fn set_attribute_value_after_func_execution(
    ctx: &mut DalContext,
    finished_value: DependentValue,
    is_on_head: bool,
    before_value: Option<serde_json::Value>,
    execution_values: FuncRunValue,
    func: Func,
    input_attribute_value_ids: Vec<AttributeValueId>,
    executed_value_id: AttributeValueId,
    value_is_changed: bool,
    after_value: Option<serde_json::Value>,
) -> RemoveOrCycle {
    let remove_or_cycle = match finished_value {
        DependentValue::AttributeValue(_) => {
            set_normal_attribute_value_after_func_execution(
                ctx,
                execution_values,
                func.clone(),
                executed_value_id,
            )
            .await
        }
        DependentValue::OverlayDestination { .. } => {
            set_leaf_prototype_value_after_func_execution(
                ctx,
                execution_values,
                func.clone(),
                executed_value_id,
            )
            .await
        }
    };

    if let RemoveOrCycle::RemoveAndLog(av_id) = &remove_or_cycle {
        if value_is_changed {
            // if we're not on head, and the value is different after
            // processing, let's see if we should enqueue an update
            // function. If either of these calls fail, keep going,
            // failures here should not prevent finishing the DVU.
            if !is_on_head {
                if let Err(err) =
                    Component::enqueue_update_action_if_applicable(ctx, executed_value_id).await
                {
                    execution_error(ctx, err.to_string(), executed_value_id).await;
                }
            }

            // Publish the audit log for the updated dependent value.
            if let Err(err) = audit_log::write(
                ctx,
                *av_id,
                input_attribute_value_ids,
                func,
                before_value,
                after_value,
                matches!(finished_value, DependentValue::OverlayDestination { .. }),
            )
            .await
            {
                execution_error(ctx, err.to_string(), executed_value_id).await;
            }
        }
    }

    remove_or_cycle
}

async fn set_normal_attribute_value_after_func_execution(
    ctx: &mut DalContext,
    execution_values: FuncRunValue,
    func: Func,
    executed_value_id: AttributeValueId,
) -> RemoveOrCycle {
    match AttributeValue::is_set_by_dependent_function(ctx, executed_value_id).await {
        Ok(true) => match AttributeValue::set_values_from_func_run_value(
            ctx,
            executed_value_id,
            execution_values,
            func,
        )
        .await
        {
            Ok(_) => RemoveOrCycle::RemoveAndLog(executed_value_id),
            Err(err) => {
                execution_error(ctx, err.to_string(), executed_value_id).await;
                RemoveOrCycle::CycleOnSelf
            }
        },
        Ok(false) => RemoveOrCycle::RemoveButIgnore,
        Err(err) => {
            execution_error(ctx, err.to_string(), executed_value_id).await;
            RemoveOrCycle::CycleOnSelf
        }
    }
}

async fn set_leaf_prototype_value_after_func_execution(
    ctx: &mut DalContext,
    execution_values: FuncRunValue,
    func: Func,
    executed_value_id: AttributeValueId,
) -> RemoveOrCycle {
    let leaf_result_av_id =
        match AttributeValue::map_child_opt(ctx, executed_value_id, &func.name).await {
            Ok(None) => match insert_map_child(ctx, executed_value_id, func.name.clone()).await {
                Ok(map_child_av_id) => map_child_av_id,
                Err(err) => {
                    execution_error(ctx, err.to_string(), executed_value_id).await;
                    return RemoveOrCycle::CycleOnSelf;
                }
            },
            Ok(Some(map_child_av_id)) => map_child_av_id,
            Err(err) => {
                execution_error(ctx, err.to_string(), executed_value_id).await;
                return RemoveOrCycle::CycleOnSelf;
            }
        };

    if let Err(err) =
        set_leaf_values_and_ensure_prototype(ctx, execution_values, leaf_result_av_id, func).await
    {
        execution_error(ctx, err.to_string(), executed_value_id).await;
        return RemoveOrCycle::CycleOnSelf;
    }

    RemoveOrCycle::RemoveAndLog(leaf_result_av_id)
}

async fn insert_map_child(
    ctx: &DalContext,
    map_parent_id: AttributeValueId,
    key: String,
) -> DependentValueUpdateResult<AttributeValueId> {
    let element_prop_id = AttributeValue::element_prop_id_for_id(ctx, map_parent_id).await?;

    let new_attribute_value =
        AttributeValue::new(ctx, element_prop_id, None, Some(map_parent_id), Some(key)).await?;

    Ok(new_attribute_value.id())
}

async fn set_leaf_values_and_ensure_prototype(
    ctx: &DalContext,
    execution_values: FuncRunValue,
    map_child_value_id: AttributeValueId,
    func: Func,
) -> DependentValueUpdateResult<()> {
    let new_proto_required = match AttributeValue::component_prototype_id(ctx, map_child_value_id)
        .await
        .map_err(Box::new)?
    {
        Some(existing_prototype_id) => {
            let existing_func_id = AttributePrototype::func_id(ctx, existing_prototype_id)
                .await
                .map_err(Box::new)?;
            func.id != existing_func_id
        }
        None => true,
    };

    if new_proto_required {
        let new_prototype = AttributePrototype::new(ctx, func.id)
            .await
            .map_err(Box::new)?;
        AttributeValue::set_component_prototype_id(
            ctx,
            map_child_value_id,
            new_prototype.id,
            func.name.clone().into(),
        )
        .await
        .map_err(Box::new)?;
    }

    AttributeValue::set_values_from_func_run_value(ctx, map_child_value_id, execution_values, func)
        .await
        .map_err(Box::new)?;

    Ok(())
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

#[derive(Debug)]
struct DependentProtoExecution {
    task_id: Ulid,
    result: DependentValueUpdateResult<PrototypeExecution>,
    before_value: Option<serde_json::Value>,
}

/// Wrapper around `AttributeValue.values_from_prototype_function_execution(&ctx)` to get it to
/// play more nicely with being spawned into a `JoinSet`.
#[instrument(
    name = "dependent_values_update.values_from_prototype_function_execution",
    level = "debug",
    parent = &parent_span,
    skip_all,
    fields(
        si.attribute_value.id = %value.attribute_value_id(),
    ),
)]
async fn values_from_prototype_function_execution(
    task_id: Ulid,
    parent_span: Span,
    ctx: DalContext,
    value: DependentValue,
    before_value: Option<serde_json::Value>,
    set_value_lock: Arc<RwLock<()>>,
    status_update: Option<StatusUpdate>,
) -> DependentProtoExecution {
    if let Some(status_update) = status_update {
        if let Err(err) = send_status_update(&ctx, status_update).await {
            return DependentProtoExecution {
                task_id,
                result: Err(err),
                before_value,
            };
        }
    }

    let result = match value {
        DependentValue::AttributeValue(attribute_value_id) => {
            AttributeValue::execute_prototype_function(&ctx, attribute_value_id, set_value_lock)
                .await
                .map_err(Into::into)
        }
        DependentValue::OverlayDestination {
            leaf_prototype_id,
            destination_map_id,
            root_attribute_value_id,
            ..
        } => LeafPrototype::execute(
            &ctx,
            leaf_prototype_id,
            destination_map_id,
            root_attribute_value_id,
            set_value_lock,
        )
        .await
        .map_err(Into::into),
    };

    DependentProtoExecution {
        task_id,
        result,
        before_value,
    }
}

async fn send_status_update(
    ctx: &DalContext,
    status_update: StatusUpdate,
) -> DependentValueUpdateResult<()> {
    WsEvent::status_update(ctx, status_update.clone())
        .await?
        .publish_immediately(ctx)
        .await?;
    Ok(())
}

/// If the before_value is None (the value was never set), and the "after_value"
/// is something empty: either the blank string, the empty object, or the empty
/// array, then don't consider the value to have "changed". Otherwise compare
/// the two values.
fn is_value_changed(
    before_value: Option<&serde_json::Value>,
    unprocessed_execution_value: Option<&serde_json::Value>,
) -> bool {
    let empty_array: serde_json::Value = serde_json::json!([]);
    let empty_object: serde_json::Value = serde_json::json!({});
    let empty_string: serde_json::Value = serde_json::json!("");

    if before_value.is_none() {
        unprocessed_execution_value.is_some_and(|after_value| {
            after_value != &empty_array
                && after_value != &empty_object
                && after_value != &empty_string
        })
    } else {
        before_value != unprocessed_execution_value
    }
}

async fn get_before_value(
    ctx: &DalContext,
    value: DependentValue,
    set_value_lock: Arc<RwLock<()>>,
) -> DependentValueUpdateResult<Option<serde_json::Value>> {
    let guard = set_value_lock.read().await;

    let before_value = match value {
        DependentValue::AttributeValue(attribute_value_id) => {
            AttributeValue::get_by_id(ctx, attribute_value_id)
                .await
                .map_err(Box::new)?
                .unprocessed_value(ctx)
                .await
                .map_err(Box::new)?
        }
        DependentValue::OverlayDestination {
            leaf_prototype_id,
            destination_map_id,
            ..
        } => {
            let func_id = LeafPrototype::func_id(ctx, leaf_prototype_id)
                .await
                .map_err(Box::new)?;

            let func = Func::get_by_id(ctx, func_id).await.map_err(Box::new)?;

            match AttributeValue::map_child_opt(ctx, destination_map_id, &func.name)
                .await
                .map_err(Box::new)?
            {
                Some(value_id) => AttributeValue::get_by_id(ctx, value_id)
                    .await
                    .map_err(Box::new)?
                    .unprocessed_value(ctx)
                    .await
                    .map_err(Box::new)?,
                None => None,
            }
        }
    };

    drop(guard);

    Ok(before_value)
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
        after_value: Option<serde_json::Value>,
        is_leaf_overlay: bool,
    ) -> Result<(), DependentValueUpdateAuditLogError> {
        // Metadata for who "owns" the attribute value.
        let component_id = AttributeValue::component_id(ctx, finished_value_id).await?;
        let component = Component::get_by_id(ctx, component_id).await?;
        let component_name = component.name(ctx).await?;
        let component_schema_variant = component.schema_variant(ctx).await?;

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
                        is_leaf_overlay,
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
