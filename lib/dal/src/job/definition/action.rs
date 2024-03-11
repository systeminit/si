use std::{collections::HashMap, collections::VecDeque, convert::TryFrom};

use async_trait::async_trait;
use futures::{stream::FuturesUnordered, StreamExt};
use serde::{Deserialize, Serialize};
use telemetry::prelude::*;
use veritech_client::ResourceStatus;

use crate::{
    fix::ActionRunnerError,
    func::backend::js_action::ActionRunResult,
    job::{
        consumer::{
            JobConsumer, JobConsumerError, JobConsumerMetadata, JobConsumerResult, JobInfo,
        },
        producer::{JobProducer, JobProducerResult},
    },
    AccessBuilder, ActionKind, ActionPrototype, ActionPrototypeId, Component, ComponentId,
    DalContext, ActionRunner, ActionBatch, ActionBatchId, ActionCompletionStatus, ActionRunnerId, ActionResolver, StandardModel,
    Visibility, WsEvent,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRunnerItem {
    pub id: ActionRunnerId,
    pub action_prototype_id: ActionPrototypeId,
    pub component_id: ComponentId,
    pub parents: Vec<ActionRunnerId>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ActionsJobArgs {
    actions: HashMap<ActionRunnerId, ActionRunnerItem>,
    batch_id: ActionBatchId,
    started: bool,
}

impl From<ActionsJob> for actionsJobArgs {
    fn from(value: ActionsJob) -> Self {
        Self {
            actions: value.actions,
            batch_id: value.batch_id,
            started: value.started,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ActionsJob {
    actions: HashMap<ActionRunnerId, ActionRunnerItem>,
    started: bool,
    batch_id: ActionBatchId,
    access_builder: AccessBuilder,
    visibility: Visibility,
    job: Option<JobInfo>,
}

impl ActionsJob {
    pub fn new(
        ctx: &DalContext,
        actions: HashMap<ActionRunnerId, ActionRunnerItem>,
        batch_id: ActionBatchId,
    ) -> Box<Self> {
        Self::new_raw(ctx, actions, batch_id, false)
    }

    fn new_raw(
        ctx: &DalContext,
        actions: HashMap<ActionRunnerId, ActionRunnerItem>,
        batch_id: ActionBatchId,
        started: bool,
    ) -> Box<Self> {
        let access_builder = AccessBuilder::from(ctx.clone());
        let visibility = *ctx.visibility();

        Box::new(Self {
            actions,
            started,
            batch_id,
            access_builder,
            visibility,
            job: None,
        })
    }
}

impl JobProducer for ActionsJob {
    fn arg(&self) -> JobProducerResult<serde_json::Value> {
        Ok(serde_json::to_value(ActionsJobArgs::from(self.clone()))?)
    }
}

impl JobConsumerMetadata for ActionsJob {
    fn type_name(&self) -> String {
        "ActionsJob".to_string()
    }

    fn access_builder(&self) -> AccessBuilder {
        self.access_builder
    }

    fn visibility(&self) -> Visibility {
        self.visibility
    }
}

#[async_trait]
impl JobConsumer for ActionsJob {
    #[instrument(
        name = "actions_job.run",
        skip_all,
        level = "info",
        fields(
            batch_id=?self.batch_id,
            actions=?self.actions,
            job=?self.job,
        )
    )]
    async fn run(&self, ctx: &mut DalContext) -> JobConsumerResult<()> {
        let mut actions = self.actions.clone();

        // Mark the batch as started if it has not been yet.
        if !self.started {
            let mut batch = ActionBatch::get_by_id(ctx, &self.batch_id)
                .await?
                .ok_or(JobConsumerError::MissingActionBatch(self.batch_id))?;
            batch.stamp_started(ctx).await?;
        }

        if actions.is_empty() {
            return finish_batch(ctx, self.batch_id).await;
        }

        // Please, let this maybe go away. If you do more than 1000 in a single apply, that's bad.
        let total_fix_limit = 100;
        let mut total_fix_batch_loops = 0;

        loop {
            total_fix_batch_loops += 1;

            let mut fix_items = Vec::new();
            for item in actions.values() {
                if item.parents.is_empty() {
                    fix_items.push(item.clone());
                }
            }
            let should_blocking_commit = actions.len() != fix_items.len();

            debug!(
                ?actions,
                ?total_fix_batch_loops,
                "Scheduled actions for this loop"
            );

            if total_fix_batch_loops >= total_fix_limit {
                error!(
                    "ActionRunner batch exceeded total fix limit loops ({total_fix_limit})! {:?}",
                    self
                );
                for fix in fix_items.iter() {
                    process_failed_fix(
                        ctx,
                        &mut actions,
                        self.batch_id,
                        fix.id,
                        "Failed this action - too many actions in the batch! We tried, honest."
                            .to_string(),
                        Vec::new(),
                    )
                    .await;
                }
                finish_batch(ctx, self.batch_id).await?;
                break;
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
                            debug!(?fix, ?logs, "fix job completed");
                            let completion_status: ActionCompletionStatus =
                                *fix.completion_status()
                                    .ok_or(ActionRunnerError::EmptyCompletionStatus)?;
                            if !matches!(completion_status, ActionCompletionStatus::Success) {
                                process_failed_fix(
                                    ctx,
                                    &mut actions,
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

                            actions.remove(&id);

                            for fix in actions.values_mut() {
                                fix.parents.retain(|parent_id| *parent_id != id);
                            }
                        }
                        Err(err) => {
                            error!("Unable to finish fix {id}: {err}");
                            process_failed_fix(
                                ctx,
                                &mut actions,
                                self.batch_id,
                                id,
                                format!("Action failed: {err}"),
                                Vec::new(),
                            )
                            .await;
                        }
                    },
                    Err(err) => {
                        error!(?err, "Failed a fix due to an error");
                        process_failed_fix(
                            ctx,
                            &mut actions,
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
                                    warn!("ActionRunner Task {id} was cancelled: {err}");
                                } else {
                                    error!("Unknown failure in fix task {id}: {err}");
                                }
                            }
                        }
                    }
                }
            }

            ctx.commit().await?;

            if actions.is_empty() {
                finish_batch(ctx, self.batch_id).await?;
                break;
            }
        }

        ctx.commit().await?;

        Ok(())
    }
}

impl TryFrom<JobInfo> for ActionsJob {
    type Error = JobConsumerError;

    fn try_from(job: JobInfo) -> Result<Self, Self::Error> {
        let args = ActionsJobArgs::deserialize(&job.arg)?;

        Ok(Self {
            actions: args.actions,
            batch_id: args.batch_id,
            started: args.started,
            access_builder: job.access_builder,
            visibility: job.visibility,
            job: Some(job),
        })
    }
}

async fn finish_batch(ctx: &DalContext, id: ActionBatchId) -> JobConsumerResult<()> {
    // Mark the batch as completed.
    let mut batch = ActionBatch::get_by_id(ctx, &id)
        .await?
        .ok_or(JobConsumerError::MissingActionBatch(id))?;
    let batch_completion_status = batch.stamp_finished(ctx).await?;
    WsEvent::fix_batch_return(ctx, *batch.id(), batch_completion_status)
        .await?
        .publish_on_commit(ctx)
        .await?;
    Ok(())
}

#[instrument(
    name = "actions_job.action_task",
    parent = &parent_span,
    skip_all,
    level = "info",
    fields(
        ?batch_id,
        ?fix_item,
    )
)]
async fn action_task(
    ctx: DalContext,
    batch_id: ActionBatchId,
    fix_item: ActionRunnerItem,
    parent_span: Span,
    should_blocking_commit: bool,
) -> JobConsumerResult<(ActionRunner, Vec<String>)> {
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
    let mut fix = ActionRunner::get_by_id(&ctx, &fix_item.id)
        .await?
        .ok_or(ActionRunnerError::MissingActionRunner(fix_item.id))?;
    let resource = fix.run(&ctx, &action).await?;
    let completion_status: ActionCompletionStatus = *fix
        .completion_status()
        .ok_or(ActionRunnerError::EmptyCompletionStatus)?;

    /*
    ActionResolver::new(
        &ctx,
        *action.id(),
        Some(matches!(completion_status, ActionCompletionStatus::Success)),
        *fix.id(),
    )
    .await?;
    */

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
    // consecutive actions that depend on the /root/resource from the previous fix.
    // `blocking_commit()` will wait for any jobs that have ben created through
    // `enqueue_job(...)` to finish before moving on.
    if should_blocking_commit {
        ctx.blocking_commit().await?;
    } else {
        if ctx.blocking() {
            info!("Blocked on commit that should not block of fix definition");
        }
        ctx.commit().await?;
    }

    if matches!(completion_status, ActionCompletionStatus::Success) {
        if let Err(err) = component.act(&ctx, ActionKind::Refresh).await {
            error!("Unable to refresh component: {err}");
        }
        if let Err(err) = ctx.blocking_commit().await {
            error!("Unable to blocking commit after component refresh: {err}");
        }
    }

    Ok((fix, logs))
}

#[instrument(name = "actions_job.process_failed_fix", skip_all, level = "info")]
async fn process_failed_fix(
    ctx: &DalContext,
    actions: &mut HashMap<ActionRunnerId, ActionRunnerItem>,
    batch_id: ActionBatchId,
    failed_fix_id: ActionRunnerId,
    error_message: String,
    logs: Vec<String>,
) {
    if let Err(e) =
        process_failed_fix_inner(ctx, actions, batch_id, failed_fix_id, error_message, logs).await
    {
        error!("{e}");
    }
}

#[instrument(name = "actions_job.process_failed_fix_inner", skip_all, level = "info")]
async fn process_failed_fix_inner(
    ctx: &DalContext,
    actions: &mut HashMap<ActionRunnerId, ActionRunnerItem>,
    batch_id: ActionBatchId,
    failed_fix_id: ActionRunnerId,
    error_message: String,
    logs: Vec<String>,
) -> JobConsumerResult<()> {
    let mut failed_actions = VecDeque::new();
    failed_actions.push_back((failed_fix_id, error_message, logs));

    while let Some((id, err, logs)) = failed_actions.pop_front() {
        info!(%id, "processing failed action/fix");
        actions.remove(&id);

        if let Some(mut fix) = ActionRunner::get_by_id(ctx, &id).await? {
            // If this was a delete, we need to un-delete ourselves.
            if matches!(fix.action_kind(), ActionKind::Delete) {
                Component::restore_and_propagate(ctx, *fix.component_id()).await?;
            }

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
                    ActionCompletionStatus::Error,
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

            // ActionResolver::upsert(ctx, *action.id(), Some(false), *fix.id()).await?;

            WsEvent::fix_return(
                ctx,
                *fix.id(),
                batch_id,
                *action.kind(),
                ActionCompletionStatus::Error,
                logs,
            )
            .await?
            .publish_on_commit(ctx)
            .await?;
        } else {
            warn!(%id, "fix not found by id");
        }

        for fix in actions.values() {
            if fix.parents.contains(&id) {
                info!(%id, "pushing back action/fix that depends on another action/fix");
                failed_actions.push_back((
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
