use std::{
    collections::{HashMap, HashSet},
    convert::TryFrom,
    sync::Arc,
};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::{
    sync::RwLock,
    task::{JoinError, JoinSet},
};
use ulid::Ulid;

//use crate::tasks::StatusReceiverClient;
//use crate::tasks::StatusReceiverRequest;
use crate::{
    attribute::value::{
        dependent_value_graph::DependentValueGraph, AttributeValueError, PrototypeExecutionResult,
    },
    job::{
        consumer::{
            JobConsumer, JobConsumerError, JobConsumerMetadata, JobConsumerResult, JobInfo,
        },
        producer::{JobProducer, JobProducerResult},
    },
    status::{StatusMessageState, StatusUpdate, StatusUpdateError},
    AccessBuilder, AttributeValue, AttributeValueId, DalContext, TransactionsError, Visibility,
    WsEvent, WsEventError,
};

#[remain::sorted]
#[derive(Debug, Error)]
pub enum DependentValueUpdateError {
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    #[error("status update error: {0}")]
    StatusUpdate(#[from] StatusUpdateError),
    #[error(transparent)]
    TokioTask(#[from] JoinError),
    #[error(transparent)]
    Transactions(#[from] TransactionsError),
    #[error("ws event error: {0}")]
    WsEvent(#[from] WsEventError),
}

pub type DependentValueUpdateResult<T> = Result<T, DependentValueUpdateError>;

#[derive(Debug, Deserialize, Serialize)]
struct DependentValuesUpdateArgs {
    attribute_values: Vec<AttributeValueId>,
}

impl From<DependentValuesUpdate> for DependentValuesUpdateArgs {
    fn from(value: DependentValuesUpdate) -> Self {
        Self {
            attribute_values: value.attribute_values,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct DependentValuesUpdate {
    attribute_values: Vec<AttributeValueId>,
    access_builder: AccessBuilder,
    visibility: Visibility,
    job: Option<JobInfo>,
    #[serde(skip)]
    set_value_lock: Arc<RwLock<()>>,
}

impl DependentValuesUpdate {
    pub fn new(
        access_builder: AccessBuilder,
        visibility: Visibility,
        attribute_values: Vec<AttributeValueId>,
    ) -> Box<Self> {
        Box::new(Self {
            attribute_values,
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
        name = "dependent_values_update.run",
        skip_all,
        level = "info",
        fields(
            attribute_values = ?self.attribute_values,
        )
    )]
    async fn run(&self, ctx: &mut DalContext) -> JobConsumerResult<()> {
        Ok(self.inner_run(ctx).await?)
    }
}

impl DependentValuesUpdate {
    async fn inner_run(&self, ctx: &mut DalContext) -> DependentValueUpdateResult<()> {
        let start = tokio::time::Instant::now();

        let mut dependency_graph =
            DependentValueGraph::for_values(ctx, self.attribute_values.clone()).await?;

        debug!(
            "DependentValueGraph calculation took: {:?}",
            start.elapsed()
        );

        // Remove the first set of independent_values since they should already have had their functions executed
        for value in dependency_graph.independent_values() {
            dependency_graph.remove_value(value);
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

                if !seen_ids.contains(&attribute_value_id) {
                    let id = Ulid::new();
                    update_join_set.spawn(values_from_prototype_function_execution(
                        id,
                        ctx.clone(),
                        attribute_value_id,
                        self.set_value_lock.clone(),
                    ));
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
                            #[allow(unused_variables)]
                            let write_guard = self.set_value_lock.write().await;

                            match AttributeValue::set_values_from_execution_result(
                                ctx,
                                finished_value_id,
                                execution_values,
                            )
                            .await
                            {
                                // Remove the value, so that any values that dependent on it will
                                // become independent values (once all other dependencies are removed)
                                Ok(_) => dependency_graph.remove_value(finished_value_id),
                                Err(err) => {
                                    error!("error setting values from executed prototype function for AttributeValue {finished_value_id}: {err}");
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

                            error!("error executing prototype function for AttributeValue {finished_value_id}: {err}");
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

        ctx.commit().await?;

        Ok(())
    }
}

/// Wrapper around `AttributeValue.values_from_prototype_function_execution(&ctx)` to get it to
/// play more nicely with being spawned into a `JoinSet`.
#[instrument(
    name = "dependent_values_update.values_from_prototype_function_execution",
    skip_all,
    level = "info",
    fields(
        attribute_value.id = %attribute_value_id,
    )
)]
async fn values_from_prototype_function_execution(
    task_id: Ulid,
    ctx: DalContext,
    attribute_value_id: AttributeValueId,
    set_value_lock: Arc<RwLock<()>>,
) -> (Ulid, DependentValueUpdateResult<PrototypeExecutionResult>) {
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

    let result =
        AttributeValue::execute_prototype_function(&ctx, attribute_value_id, set_value_lock)
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
        let args = DependentValuesUpdateArgs::deserialize(&job.arg)?;
        Ok(Self {
            attribute_values: args.attribute_values,
            access_builder: job.access_builder,
            visibility: job.visibility,
            job: Some(job),
            set_value_lock: Arc::new(RwLock::new(())),
        })
    }
}
