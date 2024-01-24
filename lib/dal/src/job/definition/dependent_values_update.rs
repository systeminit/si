use std::{
    collections::{HashMap, HashSet},
    convert::TryFrom,
};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::task::{JoinError, JoinSet};

//use crate::tasks::StatusReceiverClient;
//use crate::tasks::StatusReceiverRequest;
use crate::{
    attribute::value::{AttributeValueError, PrototypeExecutionResult},
    job::consumer::{
        JobConsumer, JobConsumerError, JobConsumerMetadata, JobConsumerResult, JobInfo,
    },
    job::producer::{JobProducer, JobProducerResult},
    AccessBuilder, AttributeValue, AttributeValueId, DalContext, TransactionsError,
    /*StatusUpdater,*/
    Visibility, /*WsEvent*/
};

#[remain::sorted]
#[derive(Debug, Error)]
pub enum DependentValueUpdateError {
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    #[error(transparent)]
    TokioTask(#[from] JoinError),
    #[error(transparent)]
    Transactions(#[from] TransactionsError),
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
}

impl DependentValuesUpdate {
    pub fn new(
        access_builder: AccessBuilder,
        visibility: Visibility,
        attribute_values: Vec<AttributeValueId>,
    ) -> Box<Self> {
        // TODO(nick,paulo,zack,jacob): ensure we do not _have_ to force non deleted visibility in the future.
        let visibility = visibility.to_non_deleted();

        Box::new(Self {
            attribute_values,
            access_builder,
            visibility,
            job: None,
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

        // Since this job happens in an async runner there is the possiblity for a commit and
        // rebase to occur between dispatch and execution. Ensure we have the latest workspace
        // snapshot for our change set
        ctx.update_snapshot_to_visibility().await?;

        let mut dependency_graph =
            AttributeValue::dependent_value_graph(ctx, self.attribute_values.clone()).await?;

        debug!(
            "DependentValueGraph calculation took: {:?}",
            start.elapsed()
        );

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
                    let join_handle = update_join_set.spawn(
                        values_from_prototype_function_execution(ctx.clone(), attribute_value_id),
                    );
                    task_id_to_av_id.insert(join_handle.id(), attribute_value_id);
                    seen_ids.insert(attribute_value_id);
                }
            }

            // Wait for a task to finish
            if let Some(join_result) = update_join_set.join_next_with_id().await {
                let (task_id, execution_result) = join_result?;
                if let Some(finished_value_id) = task_id_to_av_id.remove(&task_id) {
                    match execution_result {
                        Ok(execution_values) => {
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
    ctx: DalContext,
    attribute_value_id: AttributeValueId,
) -> DependentValueUpdateResult<PrototypeExecutionResult> {
    Ok(AttributeValue::values_from_prototype_function_execution(&ctx, attribute_value_id).await?)
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
        })
    }
}
