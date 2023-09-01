use std::convert::TryFrom;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{
    fix::FixError,
    job::{
        consumer::{
            JobConsumer, JobConsumerError, JobConsumerMetadata, JobConsumerResult, JobInfo,
        },
        producer::{JobProducer, JobProducerResult},
    },
    AccessBuilder, ActionKind, ActionPrototype, ActionPrototypeId, AttributeValueId, Component,
    ComponentId, DalContext, DependentValuesUpdate, Fix, FixBatch, FixBatchId, FixCompletionStatus,
    FixId, FixResolver, RootPropChild, StandardModel, Visibility, WsEvent,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixItem {
    pub id: FixId,
    pub action_prototype_id: ActionPrototypeId,
    pub component_id: ComponentId,
    pub attribute_value_id: AttributeValueId,
}

#[derive(Debug, Deserialize, Serialize)]
struct FixesJobArgs {
    fixes: Vec<FixItem>,
    batch_id: FixBatchId,
    started: bool,
}

impl From<FixesJob> for FixesJobArgs {
    fn from(value: FixesJob) -> Self {
        Self {
            fixes: value.fixes,
            batch_id: value.batch_id,
            started: value.started,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct FixesJob {
    fixes: Vec<FixItem>,
    started: bool,
    batch_id: FixBatchId,
    access_builder: AccessBuilder,
    visibility: Visibility,
    job: Option<JobInfo>,
}

impl FixesJob {
    pub fn new(ctx: &DalContext, fixes: Vec<FixItem>, batch_id: FixBatchId) -> Box<Self> {
        Self::new_raw(ctx, fixes, batch_id, false)
    }

    /// Used for creating another fix job in a "fixes" sequence.
    fn new_iteration(ctx: &DalContext, fixes: Vec<FixItem>, batch_id: FixBatchId) -> Box<Self> {
        Self::new_raw(ctx, fixes, batch_id, true)
    }

    fn new_raw(
        ctx: &DalContext,
        fixes: Vec<FixItem>,
        batch_id: FixBatchId,
        started: bool,
    ) -> Box<Self> {
        let access_builder = AccessBuilder::from(ctx.clone());
        let visibility = *ctx.visibility();

        Box::new(Self {
            fixes,
            started,
            batch_id,
            access_builder,
            visibility,
            job: None,
        })
    }
}

impl JobProducer for FixesJob {
    fn arg(&self) -> JobProducerResult<serde_json::Value> {
        Ok(serde_json::to_value(FixesJobArgs::from(self.clone()))?)
    }
}

impl JobConsumerMetadata for FixesJob {
    fn type_name(&self) -> String {
        "FixesJob".to_string()
    }

    fn access_builder(&self) -> AccessBuilder {
        self.access_builder
    }

    fn visibility(&self) -> Visibility {
        self.visibility
    }
}

#[async_trait]
impl JobConsumer for FixesJob {
    async fn run(&self, ctx: &mut DalContext) -> JobConsumerResult<()> {
        // Mark the batch as started if it has not been yet.
        if !self.started {
            let mut batch = FixBatch::get_by_id(ctx, &self.batch_id)
                .await?
                .ok_or(JobConsumerError::MissingFixBatch(self.batch_id))?;
            batch.stamp_started(ctx).await?;
        }

        if self.fixes.is_empty() {
            return finish_batch(ctx, self.batch_id).await;
        }
        let fix_item = &self.fixes[0];

        let deleted_ctx = &ctx.clone_with_delete_visibility();
        // Get the workflow for the action we need to run.
        let component = Component::get_by_id(deleted_ctx, &fix_item.component_id)
            .await?
            .ok_or(JobConsumerError::ComponentNotFound(fix_item.component_id))?;
        if component.is_destroyed() {
            return Err(JobConsumerError::ComponentIsDestroyed(*component.id()));
        }

        let action = ActionPrototype::get_by_id(ctx, &fix_item.action_prototype_id)
            .await?
            .ok_or_else(|| {
                JobConsumerError::ActionPrototypeNotFound(fix_item.action_prototype_id)
            })?;

        // Run the fix (via the action prototype).
        let mut fix = Fix::get_by_id(ctx, &fix_item.id)
            .await?
            .ok_or(FixError::MissingFix(fix_item.id))?;
        let resource = fix.run(ctx, &action).await?;
        let completion_status: FixCompletionStatus = *fix
            .completion_status()
            .ok_or(FixError::EmptyCompletionStatus)?;

        // Upsert the fix resolver.
        FixResolver::upsert(
            ctx,
            *action.id(),
            fix_item.attribute_value_id,
            Some(matches!(completion_status, FixCompletionStatus::Success)),
            *fix.id(),
        )
        .await?;

        let logs: Vec<_> = match resource {
            Some(r) => r
                .logs
                .iter()
                .flat_map(|l| l.split('\n'))
                .map(|l| l.to_owned())
                .collect(),
            None => vec![],
        };

        let attribute_value = Component::root_prop_child_attribute_value_for_component(
            ctx,
            *component.id(),
            RootPropChild::Resource,
        )
        .await?;

        // Always retriggers confirmations, and propagates resource if it changed.
        ctx.enqueue_job(DependentValuesUpdate::new(
            ctx.access_builder(),
            *ctx.visibility(),
            vec![*attribute_value.id()],
        ))
        .await?;

        // Commit progress so far, and wait for dependent values propagation so we can run
        // consecutive fixes that depend on the /root/resource from the previous fix.
        // `blocking_commit()` will wait for any jobs that have ben created through
        // `enqueue_job(...)` to finish before moving on.
        ctx.blocking_commit().await?;

        component.act(ctx, ActionKind::Refresh).await?;

        ctx.blocking_commit().await?;

        WsEvent::fix_return(
            ctx,
            *fix.id(),
            self.batch_id,
            fix_item.attribute_value_id,
            *action.kind(),
            completion_status,
            logs,
        )
        .await?
        .publish_on_commit(ctx)
        .await?;

        if self.fixes.len() == 1 {
            finish_batch(ctx, self.batch_id).await?;
        } else {
            ctx.enqueue_job(FixesJob::new_iteration(
                ctx,
                self.fixes.iter().skip(1).cloned().collect(),
                self.batch_id,
            ))
            .await?;
        }

        Ok(())
    }
}

impl TryFrom<JobInfo> for FixesJob {
    type Error = JobConsumerError;

    fn try_from(job: JobInfo) -> Result<Self, Self::Error> {
        let args = FixesJobArgs::deserialize(&job.arg)?;

        Ok(Self {
            fixes: args.fixes,
            batch_id: args.batch_id,
            started: args.started,
            access_builder: job.access_builder,
            visibility: job.visibility,
            job: Some(job),
        })
    }
}

async fn finish_batch(ctx: &DalContext, id: FixBatchId) -> JobConsumerResult<()> {
    // Mark the batch as completed.
    let mut batch = FixBatch::get_by_id(ctx, &id)
        .await?
        .ok_or(JobConsumerError::MissingFixBatch(id))?;
    let batch_completion_status = batch.stamp_finished(ctx).await?;
    WsEvent::fix_batch_return(ctx, *batch.id(), batch_completion_status)
        .await?
        .publish_on_commit(ctx)
        .await?;
    Ok(())
}
