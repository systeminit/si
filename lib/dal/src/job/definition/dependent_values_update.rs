use std::{
    collections::{HashMap, HashSet},
    convert::TryFrom,
    sync::Arc,
};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use si_events::FuncRunValue;
use telemetry::prelude::*;
use thiserror::Error;
use tokio::{
    sync::RwLock,
    task::{JoinError, JoinSet},
};
use ulid::Ulid;

use crate::{
    attribute::value::{dependent_value_graph::DependentValueGraph, AttributeValueError},
    context::SystemActor,
    job::{
        consumer::{
            JobCompletionState, JobConsumer, JobConsumerError, JobConsumerMetadata,
            JobConsumerResult, JobInfo, RetryBackoff,
        },
        producer::{JobProducer, JobProducerResult},
    },
    prop::PropError,
    status::{StatusMessageState, StatusUpdate, StatusUpdateError},
    AccessBuilder, AttributeValue, AttributeValueId, DalContext, TransactionsError, Visibility,
    WorkspacePk, WorkspaceSnapshotError, WsEvent, WsEventError,
};

const MAX_RETRIES: u32 = 8;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum DependentValueUpdateError {
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("status update error: {0}")]
    StatusUpdate(#[from] StatusUpdateError),
    #[error(transparent)]
    TokioTask(#[from] JoinError),
    #[error(transparent)]
    Transactions(#[from] TransactionsError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
    #[error("ws event error: {0}")]
    WsEvent(#[from] WsEventError),
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
    access_builder: AccessBuilder,
    visibility: Visibility,
    job: Option<JobInfo>,
    #[serde(skip)]
    set_value_lock: Arc<RwLock<()>>,
}

impl DependentValuesUpdate {
    pub fn new(access_builder: AccessBuilder, visibility: Visibility) -> Box<Self> {
        Box::new(Self {
            access_builder,
            visibility,
            job: None,
            set_value_lock: Arc::new(RwLock::new(())),
        })
    }
}

impl JobProducer for DependentValuesUpdate {
    fn arg(&self) -> JobProducerResult<serde_json::Value> {
        Ok(serde_json::to_value(DependentValuesUpdateArgs::from(
            self.clone(),
        ))?)
    }
}

impl JobConsumerMetadata for DependentValuesUpdate {
    fn type_name(&self) -> String {
        "DependentValuesUpdate".to_string()
    }

    fn access_builder(&self) -> AccessBuilder {
        self.access_builder
    }

    fn visibility(&self) -> Visibility {
        self.visibility
    }
}

#[async_trait]
impl JobConsumer for DependentValuesUpdate {
    #[instrument(
        level="info",
        name = "dependent_values_update.run",
        skip_all,
        fields(
                si.change_set.id = Empty,
                si.workspace.pk = Empty,
            ),
        )]
    async fn run(&self, ctx: &mut DalContext) -> JobConsumerResult<JobCompletionState> {
        let span = Span::current();
        span.record("si.change_set.id", ctx.change_set_id().to_string());
        span.record(
            "si.workspace.pk",
            ctx.tenancy()
                .workspace_pk()
                .unwrap_or(WorkspacePk::NONE)
                .to_string(),
        );
        Ok(self.inner_run(ctx).await?)
    }

    fn system_actor_id_override(&self) -> Option<SystemActor> {
        Some(SystemActor::Dvu)
    }
}

impl DependentValuesUpdate {
    async fn inner_run(
        &self,
        ctx: &mut DalContext,
    ) -> DependentValueUpdateResult<JobCompletionState> {
        let start = tokio::time::Instant::now();
        let span = Span::current();
        let node_ids = ctx
            .workspace_snapshot()?
            .take_dependent_values(ctx.vector_clock_id()?)
            .await?;

        let mut dependency_graph = DependentValueGraph::new(ctx, node_ids).await?;

        debug!(
            "DependentValueGraph calculation took: {:?}",
            start.elapsed()
        );

        // Remove the first set of independent_values since they should already have had their functions executed
        for value in dependency_graph.independent_values() {
            if !dependency_graph.values_needs_to_execute_from_prototype_function(value) {
                dependency_graph.remove_value(value);
            }
        }

        let mut seen_ids = HashSet::new();
        let mut task_id_to_av_id = HashMap::new();
        let mut update_join_set = JoinSet::new();

        let mut independent_value_ids = dependency_graph.independent_values();

        loop {
            if independent_value_ids.is_empty() && task_id_to_av_id.is_empty() {
                break;
            }

            for attribute_value_id in &independent_value_ids {
                let attribute_value_id = attribute_value_id.to_owned(); // release our borrow
                let parent_span = span.clone();
                if !seen_ids.contains(&attribute_value_id) {
                    let id = Ulid::new();
                    update_join_set.spawn(
                        values_from_prototype_function_execution(
                            id,
                            ctx.clone(),
                            attribute_value_id,
                            self.set_value_lock.clone(),
                        )
                        .instrument(info_span!(parent: parent_span, "dependent_values_update.values_from_prototype_function_execution",
                            attribute_value.id = %attribute_value_id,
                        )),
                    );
                    task_id_to_av_id.insert(id, attribute_value_id);
                    seen_ids.insert(attribute_value_id);
                }
            }

            // Wait for a task to finish
            if let Some(join_result) = update_join_set.join_next().await {
                let (task_id, execution_result) = join_result?;
                if let Some(finished_value_id) = task_id_to_av_id.remove(&task_id) {
                    match execution_result {
                        Ok(execution_values) => {
                            // Lock the graph for writing inside this job. The
                            // lock will be released when this guard is dropped
                            // at the end of the scope.
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
                                )
                                .await
                                {
                                    Ok(_) => {
                                        // Remove the value, so that any values that depend on it will
                                        // become independent values (once all other dependencies are removed)
                                        dependency_graph.remove_value(finished_value_id);
                                        drop(write_guard);
                                    }
                                    Err(err) => {
                                        execution_error(ctx, err.to_string(), finished_value_id)
                                            .await;
                                        dependency_graph.cycle_on_self(finished_value_id);
                                    }
                                },
                                Ok(false) => dependency_graph.remove_value(finished_value_id),
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

                    if let Err(err) = send_update_message(
                        ctx,
                        finished_value_id,
                        StatusMessageState::StatusFinished,
                        self.set_value_lock.clone(),
                    )
                    .await
                    {
                        error!("status update finished event send failed for AttributeValue {finished_value_id}: {err}");
                    };
                }
            }

            independent_value_ids = dependency_graph.independent_values();
        }

        debug!("DependentValuesUpdate took: {:?}", start.elapsed());
        Ok(match ctx.commit().await {
            Ok(_) => JobCompletionState::Done,
            Err(err) => match err {
                TransactionsError::ConflictsOccurred(_) => {
                    // retry still on conflicts in DVU
                    warn!("Retrying DependentValueUpdate due to conflicts");
                    JobCompletionState::Retry {
                        limit: MAX_RETRIES,
                        backoff: RetryBackoff::Exponential,
                    }
                }
                err => return Err(DependentValueUpdateError::Transactions(err)),
            },
        })
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
    let error_message = if let Ok(detail) = execution_error_detail(ctx, attribute_value_id).await {
        format!("{detail}: {err_string}")
    } else {
        fallback
    };

    error!("{}", error_message);
}

async fn execution_error_detail(
    ctx: &DalContext,
    id: AttributeValueId,
) -> DependentValueUpdateResult<String> {
    let is_for = AttributeValue::is_for(ctx, id)
        .await?
        .debug_info(ctx)
        .await?;
    let prototype_func = AttributeValue::prototype_func(ctx, id).await?.name;

    Ok(format!(
        "error executing prototype function \"{prototype_func}\" to set the value of {is_for} ({id})"
    ))
}

/// Wrapper around `AttributeValue.values_from_prototype_function_execution(&ctx)` to get it to
/// play more nicely with being spawned into a `JoinSet`.
async fn values_from_prototype_function_execution(
    task_id: Ulid,
    ctx: DalContext,
    attribute_value_id: AttributeValueId,
    set_value_lock: Arc<RwLock<()>>,
) -> (Ulid, DependentValueUpdateResult<FuncRunValue>) {
    if let Err(err) = send_update_message(
        &ctx,
        attribute_value_id,
        StatusMessageState::StatusStarted,
        set_value_lock.clone(),
    )
    .await
    {
        return (task_id, Err(err));
    }
    let parent_span = Span::current();
    let result =
        AttributeValue::execute_prototype_function(&ctx, attribute_value_id, set_value_lock)
            .instrument(info_span!(parent:parent_span, "value.execute_prototype_function", attribute_value.id= %attribute_value_id))
            .await
            .map_err(Into::into);

    (task_id, result)
}

async fn send_update_message(
    ctx: &DalContext,
    attribute_value_id: AttributeValueId,
    status: StatusMessageState,
    set_value_lock: Arc<RwLock<()>>,
) -> DependentValueUpdateResult<()> {
    let read_lock = set_value_lock.read().await;

    let status_update =
        StatusUpdate::new_for_attribute_value_id(ctx, attribute_value_id, status).await?;

    WsEvent::status_update(ctx, status_update)
        .await?
        .publish_immediately(ctx)
        .await?;

    // We explicitly drop so that we don't have an unused variable
    drop(read_lock);

    Ok(())
}

impl TryFrom<JobInfo> for DependentValuesUpdate {
    type Error = JobConsumerError;

    fn try_from(job: JobInfo) -> Result<Self, Self::Error> {
        Ok(Self {
            access_builder: job.access_builder,
            visibility: job.visibility,
            job: Some(job),
            set_value_lock: Arc::new(RwLock::new(())),
        })
    }
}
