use std::{convert::TryFrom, time::Duration};

use async_trait::async_trait;
use futures::Future;
use serde::{Deserialize, Serialize};
use si_events::ActionResultState;
use telemetry::prelude::*;
use tryhard::RetryPolicy;
use ulid::Ulid;
use veritech_client::{ActionRunResultSuccess, ResourceStatus};

use crate::{
    action::{
        prototype::{ActionKind, ActionPrototype},
        Action, ActionError, ActionId, ActionState,
    },
    change_status::ChangeStatus,
    diagram::SummaryDiagramComponent,
    job::{
        consumer::{
            JobCompletionState, JobConsumer, JobConsumerError, JobConsumerMetadata,
            JobConsumerResult, JobInfo,
        },
        producer::{JobProducer, JobProducerResult},
    },
    workspace_snapshot::graph::WorkspaceSnapshotGraphError,
    AccessBuilder, ActionPrototypeId, Component, ComponentId, DalContext, TransactionsError,
    Visibility, WorkspaceSnapshotError, WsEvent,
};

#[derive(Debug, Deserialize, Serialize)]
struct ActionJobArgs {
    id: ActionId,
}

impl From<ActionJob> for ActionJobArgs {
    fn from(value: ActionJob) -> Self {
        Self { id: value.id }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ActionJob {
    id: ActionId,
    access_builder: AccessBuilder,
    visibility: Visibility,
    job: Option<JobInfo>,
}

impl ActionJob {
    pub fn new(ctx: &DalContext, id: ActionId) -> Box<Self> {
        let access_builder = AccessBuilder::from(ctx.clone());
        let visibility = *ctx.visibility();

        Box::new(Self {
            id,
            access_builder,
            visibility,
            job: None,
        })
    }
}

impl JobProducer for ActionJob {
    fn arg(&self) -> JobProducerResult<serde_json::Value> {
        Ok(serde_json::to_value(ActionJobArgs::from(self.clone()))?)
    }
}

impl JobConsumerMetadata for ActionJob {
    fn type_name(&self) -> String {
        "ActionJob".to_string()
    }

    fn access_builder(&self) -> AccessBuilder {
        self.access_builder
    }

    fn visibility(&self) -> Visibility {
        self.visibility
    }
}

#[async_trait]
impl JobConsumer for ActionJob {
    #[instrument(
        name = "action_job.run",
        skip_all,
        level = "info",
        fields(
            id=?self.id,
            job=?self.job,
            // TODO: determine what this field is called for retries
            si.poopadoop.retries = Empty,
        )
    )]
    async fn run(&self, ctx: &mut DalContext) -> JobConsumerResult<JobCompletionState> {
        if let Err(err) = inner_run(ctx, self.id).await {
            error!(si.error.message = ?err, si.action.id = %self.id, "unable to finish action");
            if let Err(err) = process_failed_action(ctx, self.id).await {
                error!(si.error.message = ?err, "failed to process action failure");
            }
        }

        Ok(JobCompletionState::Done)
    }
}

impl TryFrom<JobInfo> for ActionJob {
    type Error = JobConsumerError;

    fn try_from(job: JobInfo) -> Result<Self, Self::Error> {
        let args = ActionJobArgs::deserialize(&job.arg)?;

        Ok(Self {
            id: args.id,
            access_builder: job.access_builder,
            visibility: job.visibility,
            job: Some(job),
        })
    }
}

#[instrument(
    name = "action_job.action_task",
    skip_all,
    level = "info",
    fields(
        si.action.id = ?action_id,
        si.action.kind = Empty,
        si.component.id = Empty,
        // TODO: determine what this field is called for retries
        si.action_job.process.retries = Empty,
    )
)]
async fn inner_run(
    ctx: &mut DalContext,
    action_id: ActionId,
) -> JobConsumerResult<Option<ActionRunResultSuccess>> {
    info!("workspace snapshot id when pinga job begins: {}", ctx.workspace_snapshot()?.id().await);

    let (prototype_id, component_id) = prepare_for_execution(ctx, action_id).await?;

    // Execute the action function
    let maybe_resource = ActionPrototype::run(ctx, prototype_id, component_id).await?;

    // Retry process_and_record_execution on a conflict error up to a max
    let nodes_to_remove: Vec<Ulid> = retry_on_conflicts(
        || process_and_record_execution(ctx.clone(), maybe_resource.as_ref(), action_id),
        // used to be 10
        1,
        Duration::from_millis(10),
        "si.action_job.process.retry.process_and_record_execution",
    )
    .await?;

    // check if the things we expected to be removed actually were. The snapshot has already been updated to latest by this point
    if !nodes_to_remove.is_empty() {
        retry_on_conflicts(
            || ensure_deletes_happened(ctx.clone(), nodes_to_remove.clone()),
            1,
            Duration::from_millis(10),
            "si.action_job.process.retry.ensure_deletes_happened",
        )
        .await?;
    }

    // if the action kind was a delete, let's see if any components are ready to be removed that weren't already
    let prototype = ActionPrototype::get_by_id(ctx, prototype_id).await?;
    if prototype.kind == ActionKind::Destroy {
        // after we commit check for removable components if we just successfully deleted a component
        ctx.update_snapshot_to_visibility().await?;
        let to_delete_components = Component::list_to_be_deleted(ctx).await?;
        let mut did_remove = false;
        for component_to_delete in to_delete_components {
            let component = Component::try_get_by_id(ctx, component_to_delete).await?;
            if let Some(component) = component {
                if component.allowed_to_be_removed(ctx).await? {
                    Component::remove(ctx, component.id()).await?;
                    did_remove = true;
                }
            }
        }
        if did_remove {
            if let Err(err) = ctx.commit().await {
                match err {
                    // if this fails due to conflicts, it's recoverable.
                    // the next time a destroy action runs we'll try again
                    TransactionsError::ConflictsOccurred(ref conflicts) => warn!(
                        ?err,
                        ?conflicts,
                        "Conflicts occurred while deleting components"
                    ),
                    _ => return Err(JobConsumerError::Transactions(err)),
                }
            }
        }
    }
    Ok(maybe_resource)
}

#[instrument(name = "action_job.ensure_deletes_happened", level = "info", skip(ctx))]
async fn ensure_deletes_happened(
    mut ctx: DalContext,
    nodes_to_remove: Vec<Ulid>,
) -> JobConsumerResult<()> {
    info!("workspace snapshot id when ensure deletes happened begins: {}", ctx.workspace_snapshot()?.id().await);
    ctx.update_snapshot_to_visibility().await?;
    info!("workspace snapshot id when ensure deletes happened after snapshot visibility updates: {}", ctx.workspace_snapshot()?.id().await);
    let snapshot = ctx.workspace_snapshot()?;
    let mut removed_at_least_once = false;

    for node_id in nodes_to_remove {
        match snapshot.get_node_weight_by_id(node_id).await {
            Ok(_) => {
                error!("bye bye bye");
                std::process::exit(0x0100);
                warn!(?node_id, "removing node with edge because it is lingering");
                snapshot
                    .remove_node_by_id(ctx.vector_clock_id()?, node_id)
                    .await?;
                removed_at_least_once = true;
            }
            Err(WorkspaceSnapshotError::WorkspaceSnapshotGraph(
                WorkspaceSnapshotGraphError::NodeWithIdNotFound(_),
            )) => {
                trace!(?node_id, "skipping node id: not found");
            }
            Err(err) => return Err(err.into()),
        }
    }

    if removed_at_least_once {
        ctx.commit().await?;
    }

    Ok(())
}

async fn prepare_for_execution(
    ctx: &mut DalContext,
    action_id: ActionId,
) -> JobConsumerResult<(ActionPrototypeId, ComponentId)> {
    let span = Span::current();

    let component_id = Action::component_id(ctx, action_id)
        .await?
        .ok_or(ActionError::ComponentNotFoundForAction(action_id))?;

    let prototype_id = Action::prototype_id(ctx, action_id).await?;
    let prototype = ActionPrototype::get_by_id(ctx, prototype_id).await?;
    span.record("si.action.kind", &tracing::field::debug(&prototype.kind));
    span.record("si.component.id", &tracing::field::debug(&component_id));
    Action::set_state(ctx, action_id, ActionState::Running).await?;

    // Updates the action's state
    ctx.commit().await?;
    info!("workspace snapshot id sent to the rebaser in prepare for execution: {}", ctx.workspace_snapshot()?.id().await);
    ctx.update_snapshot_to_visibility().await?;
    info!("workspace snapshot after updating prepare for execution: {}", ctx.workspace_snapshot()?.id().await);

    let component_id = Action::component_id(ctx, action_id)
        .await?
        .ok_or(ActionError::ComponentNotFoundForAction(action_id))?;

    Ok((prototype_id, component_id))
}

#[instrument(name = "action_job.process_and_record_execution",
skip_all, level = "info", fields(
    si.action.id = ?action_id))]
async fn process_and_record_execution(
    mut ctx: DalContext,
    maybe_resource: Option<&ActionRunResultSuccess>,
    action_id: ActionId,
) -> JobConsumerResult<Vec<Ulid>> {
    let mut to_remove_nodes = Vec::new();
    info!("workspace snapshot id when process and record execution begins: {}", ctx.workspace_snapshot()?.id().await);
    ctx.update_snapshot_to_visibility().await?;
    info!("workspace snapshot id when process and record execution updates snapshot and visibility: {}", ctx.workspace_snapshot()?.id().await);

    let prototype_id = Action::prototype_id(&ctx, action_id).await?;
    let prototype = ActionPrototype::get_by_id(&ctx, prototype_id).await?;

    let component_id = Action::component_id(&ctx, action_id)
        .await?
        .ok_or(ActionError::ComponentNotFoundForAction(action_id))?;
    let component = Component::get_by_id(&ctx, component_id).await?;
    if let Some(resource) = maybe_resource {
        // Set the resource if we have a payload, regardless of status *and* assemble a
        // summary
        if resource.payload.is_some() {
            component.set_resource(&ctx, resource.into()).await?;
        }

        if resource.status == ResourceStatus::Ok {
            // Remove `ActionId` from graph as the execution succeeded
            Action::remove_by_id(&ctx, action_id).await?;
            to_remove_nodes.push(action_id.into());
            if resource.payload.is_none() {
                // Clear the resource if the status is ok and we don't have a payload. This could
                // be from invoking a delete action directly, rather than deleting the component.
                component.clear_resource(&ctx).await?;

                if component.to_delete() {
                    Component::remove(&ctx, component.id()).await?;
                    to_remove_nodes.push(component.id().into());
                } else {
                    let summary = SummaryDiagramComponent::assemble(
                        &ctx,
                        &component,
                        ChangeStatus::Unmodified,
                    )
                    .await?;
                    WsEvent::resource_refreshed(&ctx, summary)
                        .await?
                        .publish_on_commit(&ctx)
                        .await?;
                }
            } else {
                let summary =
                    SummaryDiagramComponent::assemble(&ctx, &component, ChangeStatus::Unmodified)
                        .await?;
                WsEvent::resource_refreshed(&ctx, summary)
                    .await?
                    .publish_on_commit(&ctx)
                    .await?;
            }

            let triggered_prototypes =
                ActionPrototype::get_prototypes_to_trigger(&ctx, prototype.id()).await?;
            for dependency_prototype_id in triggered_prototypes {
                Action::new(&ctx, dependency_prototype_id, Some(component_id)).await?;
            }
        } else {
            // If status is not ok, set action state to failed
            Action::set_state(&ctx, action_id, ActionState::Failed).await?;
        }
    } else {
        // If the maybe_resource is none, set action state to failed
        Action::set_state(&ctx, action_id, ActionState::Failed).await?;
    }

    WsEvent::action_list_updated(&ctx)
        .await?
        .publish_on_commit(&ctx)
        .await?;
    info!("workspace snapshot id when process and record execution before it commits: {}", ctx.workspace_snapshot()?.id().await);
    ctx.commit().await?;
    info!("workspace snapshot id when process and record execution after it commits (what the rebaser used for onto): {}", ctx.workspace_snapshot()?.id().await);

    Ok(to_remove_nodes)
}

#[instrument(
    name = "action_job.process_failed_action",
    skip_all,
    level = "info",
    fields(si.action.id = ?action_id))]
async fn process_failed_action(ctx: &DalContext, action_id: ActionId) -> JobConsumerResult<()> {
    info!(%action_id, "processing action failed");

    Action::set_state(ctx, action_id, ActionState::Failed).await?;

    ctx.layer_db()
        .func_run()
        .set_action_result_state_for_action_id(
            action_id.into(),
            ActionResultState::Failure,
            ctx.events_tenancy(),
            ctx.events_actor(),
        )
        .await?;

    ctx.commit().await?;
    Ok(())
}

async fn retry_on_conflicts<F, Fut, T>(
    f: F,
    max_attempts: u32,
    fixed_delay: Duration,
    span_field: &'static str,
) -> Result<T, JobConsumerError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, JobConsumerError>>,
{
    let span = Span::current();

    // Jitter implementation thanks to the `fure` crate, released under the MIT license.
    //
    // See: https://github.com/Leonqn/fure/blob/8945c35655f7e0f6966d8314ab21a297181cc080/src/backoff.rs#L44-L51
    fn jitter(duration: Duration) -> Duration {
        let jitter = rand::random::<f64>();
        let secs = ((duration.as_secs() as f64) * jitter).ceil() as u64;
        let nanos = ((f64::from(duration.subsec_nanos())) * jitter).ceil() as u32;
        Duration::new(secs, nanos)
    }

    tryhard::retry_fn(f)
        .retries(max_attempts.saturating_sub(1))
        .custom_backoff(|attempt, err: &JobConsumerError| {
            if let JobConsumerError::Transactions(TransactionsError::ConflictsOccurred(_)) = err {
                span.record(span_field, attempt);
                RetryPolicy::Delay(jitter(fixed_delay))
            } else {
                RetryPolicy::Break
            }
        })
        .await
}
