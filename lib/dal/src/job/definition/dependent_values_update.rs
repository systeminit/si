use std::{collections::HashMap, convert::TryFrom};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use si_data_faktory::Job;
use telemetry::prelude::*;
use tokio::task::JoinSet;

use crate::{
    job::consumer::{FaktoryJobInfo, JobConsumer, JobConsumerError, JobConsumerResult},
    job::producer::{JobMeta, JobProducer, JobProducerResult},
    AccessBuilder, AttributeValue, AttributeValueError, AttributeValueId, AttributeValueResult,
    DalContext, StandardModel, Visibility, WsEvent,
};

#[derive(Debug, Deserialize, Serialize)]
struct DependentValuesUpdateArgs {
    attribute_value_id: AttributeValueId,
}

impl From<DependentValuesUpdate> for DependentValuesUpdateArgs {
    fn from(value: DependentValuesUpdate) -> Self {
        Self {
            attribute_value_id: value.attribute_value_id,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct DependentValuesUpdate {
    attribute_value_id: AttributeValueId,
    access_builder: AccessBuilder,
    visibility: Visibility,
    faktory_job: Option<FaktoryJobInfo>,
}

impl DependentValuesUpdate {
    pub fn new(ctx: &DalContext, attribute_value_id: AttributeValueId) -> Box<Self> {
        let access_builder = AccessBuilder::from(ctx.clone());
        let visibility = *ctx.visibility();

        Box::new(Self {
            attribute_value_id,
            access_builder,
            visibility,
            faktory_job: None,
        })
    }
}

impl JobProducer for DependentValuesUpdate {
    fn args(&self) -> JobProducerResult<serde_json::Value> {
        Ok(serde_json::to_value(DependentValuesUpdateArgs::from(
            self.clone(),
        ))?)
    }

    fn meta(&self) -> JobProducerResult<JobMeta> {
        let mut custom = HashMap::new();
        custom.insert(
            "access_builder".to_string(),
            serde_json::to_value(self.access_builder.clone())?,
        );
        custom.insert(
            "visibility".to_string(),
            serde_json::to_value(self.visibility)?,
        );

        Ok(JobMeta {
            retry: Some(0),
            custom,
            ..JobMeta::default()
        })
    }

    fn identity(&self) -> String {
        serde_json::to_string(self).expect("Cannot serialize DependentValueUpdate")
    }
}

#[async_trait]
impl JobConsumer for DependentValuesUpdate {
    fn type_name(&self) -> String {
        "DependentValuesUpdate".to_string()
    }

    fn access_builder(&self) -> AccessBuilder {
        self.access_builder.clone()
    }

    fn visibility(&self) -> Visibility {
        self.visibility
    }

    async fn run(&self, ctx: &DalContext) -> JobConsumerResult<()> {
        let now = std::time::Instant::now();

        let mut source_attribute_value = AttributeValue::get_by_id(ctx, &self.attribute_value_id)
            .await?
            .ok_or_else(|| {
                AttributeValueError::NotFound(self.attribute_value_id, *ctx.visibility())
            })?;
        let mut dependency_graph = source_attribute_value.dependent_value_graph(ctx).await?;
        // Remove the `AttributeValueId` from the list of values that are in the dependencies,
        // as we consider that one to have already been updated. This lets us check for
        // `AttributeValuesId`s where the list of *unsatisfied* dependencies is empty.
        for (_, val) in dependency_graph.iter_mut() {
            val.retain(|&id| id != self.attribute_value_id);
        }
        info!(
            "DependentValuesUpdate for {:?}: dependency_graph {:?}",
            self.attribute_value_id, &dependency_graph
        );

        let mut update_tasks = JoinSet::new();

        loop {
            // // If only HashMap.drain_filter were in stable...
            //
            // let satisfied_dependencies: HashMap<AttributeValueId, Vec<AttributeValueId>> =
            //     dependency_graph.drain_filter(|_, v| v.is_empty()).collect();
            //
            let mut satisfied_dependencies: Vec<AttributeValueId> =
                dependency_graph.keys().copied().collect();
            satisfied_dependencies.retain(|&id| {
                let result = if let Some(dependencies) = dependency_graph.get(&id) {
                    dependencies.is_empty()
                } else {
                    false
                };

                // We can go ahead and remove the entry in the dependency graph now,
                // since we know that all of its dependencies have been satisfied.
                // This also saves us from having to loop through the Vec again to
                // remove these entries immediately after this loop, anyway.
                if result {
                    dependency_graph.remove(&id);
                }

                result
            });

            for id in satisfied_dependencies {
                let attribute_value = AttributeValue::get_by_id(ctx, &id)
                    .await?
                    .ok_or_else(|| AttributeValueError::NotFound(id, *ctx.visibility()))?;
                let ctx_copy = ctx.clone();
                update_tasks
                    .build_task()
                    .name("AttributeValue.update_from_prototype_function")
                    .spawn(update_value(ctx_copy, attribute_value))?;
            }

            match update_tasks.join_next().await {
                Some(future_result) => {
                    // We get back a `Some<Result<Result<..>>>`. We've already unwrapped the
                    // `Some`, the outermost `Result` is a `JoinError` to let us know if
                    // anything went wrong in joining the task. The innermost `Result` is
                    // the one we're really interested in the contents of, which is why there
                    // is the `??`.
                    let finished_id = future_result??;
                    // Remove the `AttributeValueId` that just finished from the list of
                    // unsatisfied dependencies of all entries, so we can check what work
                    // has been unblocked.
                    for (_, val) in dependency_graph.iter_mut() {
                        val.retain(|&id| id != finished_id);
                    }
                }
                // If we get `None` back from the `JoinSet` that means that there are no
                // further tasks in the `JoinSet` for us to wait on. This should only happen
                // after we've stopped adding new tasks to the `JoinSet`, which means either:
                //   * We have completely walked the initial graph, and have visited every
                //     node.
                //   * We've encountered a cycle that means we can no longer make any
                //     progress on walking the graph.
                // In both cases, there isn't anything more we can do, so we can stop looking
                // at the graph to find more work.
                None => break,
            }
        }

        WsEvent::change_set_written(ctx).publish(ctx).await?;

        let elapsed_time = now.elapsed();
        info!(
            "DependentValuesUpdate for {:?} took {:?}",
            &self.attribute_value_id, elapsed_time
        );

        Ok(())
    }
}

/// Wrapper around `AttributeValue.update_from_prototype_function(&ctx)` to get it to
/// play more nicely with being spawned into a `JoinSet`.
async fn update_value(
    ctx: DalContext,
    mut attribute_value: AttributeValue,
) -> AttributeValueResult<AttributeValueId> {
    info!("DependentValueUpdate {:?}: START", attribute_value.id());
    let start = std::time::Instant::now();
    attribute_value.update_from_prototype_function(&ctx).await?;
    info!(
        "DependentValueUpdate {:?}: DONE {:?}",
        attribute_value.id(),
        start.elapsed()
    );

    Ok(*attribute_value.id())
}

impl TryFrom<Job> for DependentValuesUpdate {
    type Error = JobConsumerError;

    fn try_from(job: Job) -> Result<Self, Self::Error> {
        if job.args().len() != 3 {
            return Err(JobConsumerError::InvalidArguments(
                r#"[{ "attribute_value_id": <AttributeValueId> }, <AccessBuilder>, <Visibility>]"#
                    .to_string(),
                job.args().to_vec(),
            ));
        }
        let args: DependentValuesUpdateArgs = serde_json::from_value(job.args()[0].clone())?;
        let access_builder: AccessBuilder = serde_json::from_value(job.args()[1].clone())?;
        let visibility: Visibility = serde_json::from_value(job.args()[2].clone())?;

        let faktory_job_info = FaktoryJobInfo::try_from(job)?;

        Ok(Self {
            attribute_value_id: args.attribute_value_id,
            access_builder,
            visibility,
            faktory_job: Some(faktory_job_info),
        })
    }
}
