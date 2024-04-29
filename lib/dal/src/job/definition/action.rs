use std::{collections::HashMap, collections::VecDeque, convert::TryFrom};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use telemetry::prelude::*;
use veritech_client::ResourceStatus;

use crate::{
    deprecated_action::runner::DeprecatedActionRunnerError,
    func::backend::js_action::DeprecatedActionRunResult,
    job::{
        consumer::{
            JobCompletionState, JobConsumer, JobConsumerError, JobConsumerMetadata,
            JobConsumerResult, JobInfo,
        },
        producer::{JobProducer, JobProducerResult},
    },
    AccessBuilder, ActionCompletionStatus, ActionPrototypeId, Component, ComponentId, DalContext,
    DeprecatedActionBatch, DeprecatedActionBatchId, DeprecatedActionKind,
    DeprecatedActionPrototype, DeprecatedActionRunner, DeprecatedActionRunnerId, Visibility,
    WsEvent,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRunnerItem {
    pub id: DeprecatedActionRunnerId,
    pub action_prototype_id: ActionPrototypeId,
    pub component_id: ComponentId,
    pub parents: Vec<DeprecatedActionRunnerId>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ActionsJobArgs {
    actions: HashMap<DeprecatedActionRunnerId, ActionRunnerItem>,
    batch_id: DeprecatedActionBatchId,
    started: bool,
}

impl From<ActionsJob> for ActionsJobArgs {
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
    actions: HashMap<DeprecatedActionRunnerId, ActionRunnerItem>,
    started: bool,
    batch_id: DeprecatedActionBatchId,
    access_builder: AccessBuilder,
    visibility: Visibility,
    job: Option<JobInfo>,
}

impl ActionsJob {
    pub fn new(
        ctx: &DalContext,
        actions: HashMap<DeprecatedActionRunnerId, ActionRunnerItem>,
        batch_id: DeprecatedActionBatchId,
    ) -> Box<Self> {
        Self::new_raw(ctx, actions, batch_id, false)
    }

    fn new_raw(
        ctx: &DalContext,
        actions: HashMap<DeprecatedActionRunnerId, ActionRunnerItem>,
        batch_id: DeprecatedActionBatchId,
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
    async fn run(&self, ctx: &mut DalContext) -> JobConsumerResult<JobCompletionState> {
        let mut actions = self.actions.clone();

        // Mark the batch as started if it has not been yet.
        if !self.started {
            let mut batch = DeprecatedActionBatch::get_by_id(ctx, self.batch_id).await?;
            batch.stamp_started(ctx).await?;
        }

        if actions.is_empty() {
            finish_batch(ctx, self.batch_id).await?;
            return Ok(JobCompletionState::Done);
        }

        // Please, let this maybe go away. If you do more than 1000 in a single apply, that's bad.
        let total_action_limit = 100;
        let mut total_action_batch_loops = 0;

        loop {
            total_action_batch_loops += 1;

            let mut action_items = Vec::new();
            for item in actions.values() {
                if item.parents.is_empty() {
                    action_items.push(item.clone());
                }
            }

            debug!(
                ?actions,
                ?total_action_batch_loops,
                "Scheduled actions for this loop"
            );

            if total_action_batch_loops >= total_action_limit {
                error!(
                    "ActionRunner batch exceeded total action limit loops ({total_action_limit})! {:?}",
                    self
                );
                for action in action_items.iter() {
                    process_failed_action(
                        ctx,
                        &mut actions,
                        self.batch_id,
                        action.id,
                        "Failed this action - too many actions in the batch! We tried, honest."
                            .to_string(),
                        Vec::new(),
                    )
                    .await;
                }
                finish_batch(ctx, self.batch_id).await?;
                break;
            }

            for action_item in action_items {
                let id = action_item.id;

                match action_task(ctx, self.batch_id, action_item, Span::current()).await {
                    Ok((action, logs)) => {
                        debug!(?action, ?logs, "action job completed");
                        let completion_status: ActionCompletionStatus = action
                            .completion_status
                            .ok_or(DeprecatedActionRunnerError::EmptyCompletionStatus)?;
                        if !matches!(completion_status, ActionCompletionStatus::Success) {
                            process_failed_action(
                                ctx,
                                &mut actions,
                                self.batch_id,
                                id,
                                action
                                    .completion_message
                                    .as_ref()
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

                        for action in actions.values_mut() {
                            action.parents.retain(|parent_id| *parent_id != id);
                        }
                    }
                    Err(err) => {
                        error!("Unable to finish action {id}: {err}");
                        process_failed_action(
                            ctx,
                            &mut actions,
                            self.batch_id,
                            id,
                            format!("Action failed: {err}"),
                            Vec::new(),
                        )
                        .await;
                    }
                }
            }

            if actions.is_empty() {
                finish_batch(ctx, self.batch_id).await?;
                break;
            }
        }

        // This is the only moment we should rebase, once all the actions have executed!
        ctx.blocking_commit().await?;
        ctx.update_snapshot_to_visibility().await?;

        Ok(JobCompletionState::Done)
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

async fn finish_batch(ctx: &mut DalContext, id: DeprecatedActionBatchId) -> JobConsumerResult<()> {
    // Mark the batch as completed.
    let mut batch = DeprecatedActionBatch::get_by_id(ctx, id).await?;
    let batch_completion_status = batch.stamp_finished(ctx).await?;
    WsEvent::action_batch_return(ctx, batch.id, batch_completion_status)
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
        ?action_item,
    )
)]
async fn action_task(
    ctx: &mut DalContext,
    batch_id: DeprecatedActionBatchId,
    action_item: ActionRunnerItem,
    parent_span: Span,
) -> JobConsumerResult<(DeprecatedActionRunner, Vec<String>)> {
    // Run the action (via the action prototype).
    let mut action = DeprecatedActionRunner::get_by_id(ctx, action_item.id).await?;
    let resource = action.run(ctx).await?;
    let completion_status = action
        .completion_status
        .ok_or(DeprecatedActionRunnerError::EmptyCompletionStatus)?;

    let logs: Vec<_> = match &resource {
        Some(r) => r
            .logs
            .iter()
            .flat_map(|l| l.split('\n'))
            .map(|l| l.to_owned())
            .collect(),
        None => vec![],
    };

    WsEvent::action_return(
        ctx,
        action.id,
        batch_id,
        action.action_kind,
        action.component_id,
        resource,
    )
    .await?
    .publish_on_commit(ctx)
    .await?;

    if matches!(completion_status, ActionCompletionStatus::Success)
        && action.action_kind == DeprecatedActionKind::Create
    {
        ctx.blocking_commit().await?;
        ctx.update_snapshot_to_visibility().await?;

        let variant =
            Component::schema_variant_for_component_id(ctx, action_item.component_id).await?;
        for prototype in DeprecatedActionPrototype::for_variant(ctx, variant.id()).await? {
            if prototype.kind == DeprecatedActionKind::Refresh {
                prototype.run(ctx, action_item.component_id).await?;

                WsEvent::resource_refreshed(ctx, action_item.component_id)
                    .await?
                    .publish_on_commit(ctx)
                    .await?;
            }
        }
    }

    Ok((action, logs))
}

#[instrument(name = "actions_job.process_failed_action", skip_all, level = "info")]
async fn process_failed_action(
    ctx: &DalContext,
    actions: &mut HashMap<DeprecatedActionRunnerId, ActionRunnerItem>,
    batch_id: DeprecatedActionBatchId,
    failed_action_id: DeprecatedActionRunnerId,
    error_message: String,
    logs: Vec<String>,
) {
    if let Err(e) = process_failed_action_inner(
        ctx,
        actions,
        batch_id,
        failed_action_id,
        error_message,
        logs,
    )
    .await
    {
        error!("{e}");
    }
}

#[instrument(
    name = "actions_job.process_failed_action_inner",
    skip_all,
    level = "info"
)]
async fn process_failed_action_inner(
    ctx: &DalContext,
    actions: &mut HashMap<DeprecatedActionRunnerId, ActionRunnerItem>,
    batch_id: DeprecatedActionBatchId,
    failed_action_id: DeprecatedActionRunnerId,
    error_message: String,
    logs: Vec<String>,
) -> JobConsumerResult<()> {
    let mut failed_actions = VecDeque::new();
    failed_actions.push_back((failed_action_id, error_message, logs));

    while let Some((id, err, logs)) = failed_actions.pop_front() {
        info!(%id, "processing failed action");
        actions.remove(&id);

        let mut action = DeprecatedActionRunner::get_by_id(ctx, id).await?;

        if action.started_at.is_none() {
            action.stamp_started(ctx).await?;
        }
        let resource = DeprecatedActionRunResult {
            status: Some(ResourceStatus::Error),
            payload: action.resource.clone().and_then(|r| r.payload),
            message: Some(err.clone()),
            logs: logs.clone(),
            last_synced: None,
        };

        if action.finished_at.is_none() {
            action
                .stamp_finished(
                    ctx,
                    ActionCompletionStatus::Error,
                    Some(err.clone()),
                    Some(resource.clone()),
                )
                .await?;
        }

        let prototype =
            DeprecatedActionPrototype::get_by_id_or_error(ctx, action.action_prototype_id).await?;

        WsEvent::action_return(
            ctx,
            action.id,
            batch_id,
            prototype.kind,
            action.component_id,
            Some(resource),
        )
        .await?
        .publish_on_commit(ctx)
        .await?;

        for action in actions.values() {
            if action.parents.contains(&id) {
                info!(%id, "pushing back action that depends on another action");
                failed_actions.push_back((
                    action.id,
                    format!("Action depends on another action that failed: {err}"),
                    Vec::new(),
                ));
            }
        }
    }

    Ok(())
}
