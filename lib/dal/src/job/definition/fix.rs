use std::{collections::HashMap, collections::VecDeque, convert::TryFrom};

use async_trait::async_trait;
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
    #[instrument(
        name = "fixes_job.run",
        skip_all,
        level = "info",
        fields(
            // TODO(fnichol): add some?
        )
    )]
    async fn run(&self, ctx: &mut DalContext) -> JobConsumerResult<()> {
        let mut fixes = self.fixes.clone();

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

        let should_blocking_commit = !fixes.is_empty();
        for fix_item in fix_items {
            let task_ctx = dbg!(
                ctx.to_builder()
                    .build(self.access_builder().build(self.visibility()))
                    .await
            )?;
            handles.push(async move {
                let id = fix_item.id;
                let res = tokio::task::spawn(fix_task(
                    task_ctx,
                    self.batch_id,
                    fix_item,
                    Span::current(),
                    should_blocking_commit,
                ))
                .await;
                (id, res)
            });
        }

        while let Some((id, future_result)) = handles.next().await {
            match future_result {
                Ok(job_consumer_result) => match job_consumer_result {
                    Ok((fix, logs)) => {
                        let completion_status: FixCompletionStatus = *fix
                            .completion_status()
                            .ok_or(FixError::EmptyCompletionStatus)?;
                        if !matches!(completion_status, FixCompletionStatus::Success) {
                            process_failed_fix(
                                ctx,
                                &mut fixes,
                                self.batch_id,
                                id,
                                fix.completion_message()
                                    .map(ToOwned::to_owned)
                                    .unwrap_or_else(|| {
                                        format!(
                                            "Action failed with unknown error: {completion_status}"
                                        )
                                    }),
                                logs,
                            )
                            .await;
                            continue;
                        }

                        fixes.remove(&id);

                        for fix in fixes.values_mut() {
                            fix.parents.retain(|parent_id| *parent_id != id);
                        }
                    }
                    Err(err) => {
                        error!("Unable to finish fix {id}: {err}");
                        process_failed_fix(
                            ctx,
                            &mut fixes,
                            self.batch_id,
                            id,
                            format!("Action failed: {err}"),
                            Vec::new(),
                        )
                        .await;
                    }
                },
                Err(err) => {
                    process_failed_fix(
                        ctx,
                        &mut fixes,
                        self.batch_id,
                        id,
                        format!("Action failed: {err}"),
                        Vec::new(),
                    )
                    .await;

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

#[instrument(
    name = "fixes_job.fix_task",
    parent = &parent_span,
    skip_all,
    level = "info",
    fields(
        // TODO(fnichol): add some?
    )
)]
async fn fix_task(
    ctx: DalContext,
    batch_id: FixBatchId,
    fix_item: FixItem,
    parent_span: Span,
    should_blocking_commit: bool,
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
    if should_blocking_commit {
        ctx.blocking_commit().await?;
    } else {
        ctx.commit().await?;
    }

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

#[instrument(name = "fixes_job.process_failed_fix", skip_all, level = "info")]
async fn process_failed_fix(
    ctx: &DalContext,
    fixes: &mut HashMap<FixId, FixItem>,
    batch_id: FixBatchId,
    failed_fix_id: FixId,
    error_message: String,
    logs: Vec<String>,
) {
    if let Err(e) =
        process_failed_fix_inner(ctx, fixes, batch_id, failed_fix_id, error_message, logs).await
    {
        error!("{e}");
    }
}

#[instrument(name = "fixes_job.process_failed_fix_inner", skip_all, level = "info")]
async fn process_failed_fix_inner(
    ctx: &DalContext,
    fixes: &mut HashMap<FixId, FixItem>,
    batch_id: FixBatchId,
    failed_fix_id: FixId,
    error_message: String,
    logs: Vec<String>,
) -> JobConsumerResult<()> {
    let mut failed_fixes = VecDeque::new();
    failed_fixes.push_back((failed_fix_id, error_message, logs));

    while let Some((id, err, logs)) = failed_fixes.pop_front() {
        info!(%id, "processing failed action/fix");
        fixes.remove(&id);

        if let Some(mut fix) = Fix::get_by_id(ctx, &id).await? {
            Component::restore_and_propagate(ctx, *fix.component_id()).await?;

            if fix.started_at().is_none() {
                fix.stamp_started(ctx).await?;
            }

            if fix.finished_at().is_none() {
                let resource = ActionRunResult {
                    status: Some(ResourceStatus::Error),
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
                batch_id,
                *action.kind(),
                FixCompletionStatus::Error,
                logs,
            )
            .await?
            .publish_on_commit(ctx)
            .await?;
        } else {
            warn!(%id, "fix not found by id");
        }

        for fix in fixes.values() {
            if fix.parents.contains(&id) {
                info!(%id, "pushing back action/fix that depends on another action/fix");
                failed_fixes.push_back((
                    fix.id,
                    format!("Action depends on another action that failed: {err}"),
                    Vec::new(),
                ));
            }
        }

        ctx.commit().await?;
    }

    Ok(())
}
