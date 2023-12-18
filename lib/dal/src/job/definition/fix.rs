use std::{collections::HashMap, collections::VecDeque, convert::TryFrom};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use futures::{stream::FuturesUnordered, StreamExt};
use serde::{Deserialize, Serialize};
use telemetry::prelude::*;
use veritech_client::ResourceStatus;

use crate::{
    fix::FixError,
    func::backend::js_action::ActionRunResult,
    job::{
        consumer::{
            JobConsumer, JobConsumerError, JobConsumerMetadata, JobConsumerResult, JobInfo,
        },
        producer::{JobProducer, JobProducerResult},
    },
    standard_model::{self, TypeHint},
    AccessBuilder, ActionKind, ActionPrototype, ActionPrototypeId, Component, ComponentId,
    DalContext, Fix, FixBatch, FixBatchId, FixCompletionStatus, FixId, FixResolver, StandardModel,
    Visibility, WsEvent,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixItem {
    pub id: FixId,
    pub action_prototype_id: ActionPrototypeId,
    pub component_id: ComponentId,
    pub parents: Vec<FixId>,
}

#[derive(Debug, Deserialize, Serialize)]
struct FixesJobArgs {
    fixes: HashMap<FixId, FixItem>,
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
    fixes: HashMap<FixId, FixItem>,
    started: bool,
    batch_id: FixBatchId,
    access_builder: AccessBuilder,
    visibility: Visibility,
    job: Option<JobInfo>,
}

impl FixesJob {
    pub fn new(
        ctx: &DalContext,
        fixes: HashMap<FixId, FixItem>,
        batch_id: FixBatchId,
    ) -> Box<Self> {
        Self::new_raw(ctx, fixes, batch_id, false)
    }

    /// Used for creating another fix job in a "fixes" sequence.
    fn new_iteration(
        ctx: &DalContext,
        fixes: HashMap<FixId, FixItem>,
        batch_id: FixBatchId,
    ) -> Box<Self> {
        Self::new_raw(ctx, fixes, batch_id, true)
    }

    fn new_raw(
        ctx: &DalContext,
        fixes: HashMap<FixId, FixItem>,
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
        let mut fixes = dbg!(self.fixes.clone());

        // Mark the batch as started if it has not been yet.
        if !self.started {
            let mut batch = FixBatch::get_by_id(ctx, &self.batch_id)
                .await?
                .ok_or(JobConsumerError::MissingFixBatch(self.batch_id))?;
            batch.stamp_started(ctx).await?;
        }

        if fixes.is_empty() {
            return finish_batch(ctx, self.batch_id).await;
        }

        let mut fix_items = Vec::new();
        for item in fixes.values() {
            if item.parents.is_empty() {
                fix_items.push(item.clone());
            }
        }

        let mut handles = FuturesUnordered::new();

        // So we don't keep an open transaction while the tasks run, each task has its own transaction
        // Block just in case
        ctx.blocking_commit().await?;

        for fix_item in fix_items {
            let task_ctx = ctx
                .to_builder()
                .build(self.access_builder().build(self.visibility()))
                .await?;
            handles.push(async move {
                let id = fix_item.id;
                let res = tokio::task::spawn(fix_task(task_ctx, self.batch_id, fix_item)).await;
                (id, res)
            });
        }

        let mut failed_fixes = VecDeque::new();

        while let Some((id, future_result)) = handles.next().await {
            match future_result {
                Ok(Ok((fix, logs))) => {
                    let completion_status: FixCompletionStatus = *fix
                        .completion_status()
                        .ok_or(FixError::EmptyCompletionStatus)?;
                    if !matches!(completion_status, FixCompletionStatus::Success) {
                        failed_fixes.push_back((
                            id,
                            fix.completion_message()
                                .map(ToOwned::to_owned)
                                .unwrap_or_else(|| {
                                    format!("Action failed with unknown error: {completion_status}")
                                }),
                            logs,
                        ));
                        continue;
                    }

                    fixes.remove(&id);

                    for fix in fixes.values_mut() {
                        fix.parents.retain(|parent_id| *parent_id != id);
                    }
                }
                Ok(Err(err)) => {
                    error!("Unable to finish fix {id}: {err}");

                    failed_fixes.push_back((id, format!("Action failed: {err}"), Vec::new()));
                }
                Err(err) => {
                    failed_fixes.push_back((id, format!("Action failed: {err}"), Vec::new()));
                    match err.try_into_panic() {
                        Ok(panic) => {
                            std::panic::resume_unwind(panic);
                        }
                        Err(err) => {
                            if err.is_cancelled() {
                                warn!("Fix Task {id} was cancelled: {err}");
                            } else {
                                error!("Unknown failure in fix task {id}: {err}");
                            }
                        }
                    }
                }
            }
        }

        while let Some((id, err, logs)) = failed_fixes.pop_front() {
            fixes.remove(&id);

            if let Some(mut fix) = Fix::get_by_id(ctx, &id).await? {
                let deleted_ctx = &ctx.clone_with_delete_visibility();
                let mut component = Component::get_by_id(deleted_ctx, fix.component_id())
                    .await?
                    .ok_or_else(|| JobConsumerError::ComponentNotFound(*fix.component_id()))?;
                if component.visibility().deleted_at.is_some() {
                    component.set_deleted_at(deleted_ctx, None).await?;
                    component.set_needs_destroy(deleted_ctx, false).await?;
                    standard_model::update(
                        deleted_ctx,
                        "components",
                        "visibility_deleted_at",
                        component.id(),
                        None::<DateTime<Utc>>,
                        TypeHint::TimestampWithTimeZone,
                    )
                    .await?;
                }

                if fix.started_at().is_none() {
                    fix.stamp_started(ctx).await?;
                }

                if fix.finished_at().is_none() {
                    let resource = ActionRunResult {
                        status: ResourceStatus::Error,
                        payload: fix.resource().cloned(),
                        message: Some(err.clone()),
                        logs: logs.clone(),
                        last_synced: None,
                    };

                    fix.stamp_finished(
                        ctx,
                        FixCompletionStatus::Error,
                        Some(err.clone()),
                        Some(resource),
                    )
                    .await?;
                }

                let action = ActionPrototype::get_by_id(ctx, fix.action_prototype_id())
                    .await?
                    .ok_or_else(|| {
                        JobConsumerError::ActionPrototypeNotFound(*fix.action_prototype_id())
                    })?;

                FixResolver::upsert(ctx, *action.id(), Some(false), *fix.id()).await?;

                WsEvent::fix_return(
                    ctx,
                    *fix.id(),
                    self.batch_id,
                    *action.kind(),
                    FixCompletionStatus::Error,
                    logs,
                )
                .await?
                .publish_on_commit(ctx)
                .await?;
            }

            for fix in fixes.values() {
                if fix.parents.contains(&id) {
                    failed_fixes.push_back((
                        fix.id,
                        format!("Action depends on another action that failed: {err}"),
                        Vec::new(),
                    ));
                }
            }
            ctx.blocking_commit().await?;
        }

        if fixes.is_empty() {
            finish_batch(ctx, self.batch_id).await?;
        } else {
            ctx.enqueue_job(FixesJob::new_iteration(ctx, fixes, self.batch_id))
                .await?;
        }

        ctx.commit().await?;

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

async fn fix_task(
    ctx: DalContext,
    batch_id: FixBatchId,
    fix_item: FixItem,
) -> JobConsumerResult<(Fix, Vec<String>)> {
    let deleted_ctx = &ctx.clone_with_delete_visibility();
    // Get the workflow for the action we need to run.
    let component = Component::get_by_id(deleted_ctx, &fix_item.component_id)
        .await?
        .ok_or(JobConsumerError::ComponentNotFound(fix_item.component_id))?;
    if component.is_destroyed() {
        return Err(JobConsumerError::ComponentIsDestroyed(*component.id()));
    }

    let action = ActionPrototype::get_by_id(&ctx, &fix_item.action_prototype_id)
        .await?
        .ok_or_else(|| JobConsumerError::ActionPrototypeNotFound(fix_item.action_prototype_id))?;

    // Run the fix (via the action prototype).
    let mut fix = Fix::get_by_id(&ctx, &fix_item.id)
        .await?
        .ok_or(FixError::MissingFix(fix_item.id))?;
    let resource = fix.run(&ctx, &action).await?;
    let completion_status: FixCompletionStatus = *fix
        .completion_status()
        .ok_or(FixError::EmptyCompletionStatus)?;

    FixResolver::upsert(
        &ctx,
        *action.id(),
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

    WsEvent::fix_return(
        &ctx,
        *fix.id(),
        batch_id,
        *action.kind(),
        completion_status,
        logs.clone(),
    )
    .await?
    .publish_on_commit(&ctx)
    .await?;

    // Commit progress so far, and wait for dependent values propagation so we can run
    // consecutive fixes that depend on the /root/resource from the previous fix.
    // `blocking_commit()` will wait for any jobs that have ben created through
    // `enqueue_job(...)` to finish before moving on.
    ctx.blocking_commit().await?;

    if matches!(completion_status, FixCompletionStatus::Success) {
        if let Err(err) = component.act(&ctx, ActionKind::Refresh).await {
            error!("Unable to refresh component: {err}");
        }
        if let Err(err) = ctx.blocking_commit().await {
            error!("Unable to blocking commit after component refresh: {err}");
        }
    }

    Ok((fix, logs))
}
