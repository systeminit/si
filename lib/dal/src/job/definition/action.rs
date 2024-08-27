use std::{
    collections::HashMap,
    {collections::VecDeque, convert::TryFrom},
};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use si_events::ActionResultState;
use telemetry::prelude::*;
use telemetry_utils::metric;
use veritech_client::{ActionRunResultSuccess, ResourceStatus};

use crate::{
    action::{
        prototype::{ActionKind, ActionPrototype},
        Action, ActionError, ActionId, ActionState,
    },
    change_status::ChangeStatus,
    job::{
        consumer::{
            JobCompletionState, JobConsumer, JobConsumerError, JobConsumerMetadata,
            JobConsumerResult, JobInfo,
        },
        producer::{JobProducer, JobProducerResult},
    },
    AccessBuilder, ActionPrototypeId, Component, ComponentId, DalContext, Visibility, WsEvent,
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
        metric!(counter.action_concurrency_count = 1);
        if let Err(err) = inner_run(ctx, self.id).await {
            error!(si.error.message = ?err, si.action.id = %self.id, "unable to finish action");
            if let Err(err) = process_failed_action(ctx, self.id).await {
                error!(si.error.message = ?err, "failed to process action failure");
            }
        }
        metric!(counter.action_concurrency_count = -1);
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
    let (prototype_id, component_id) = prepare_for_execution(ctx, action_id).await?;

    // Execute the action function
    let maybe_resource = ActionPrototype::run(ctx, prototype_id, component_id).await?;

    // process the result
    process_execution(ctx, maybe_resource.as_ref(), action_id).await?;

    // if the action kind was a delete, let's see if any components are ready to be removed that weren't already
    let prototype = ActionPrototype::get_by_id(ctx, prototype_id).await?;
    if prototype.kind == ActionKind::Destroy {
        // after we commit check for removable components if we just successfully deleted a component
        let to_delete_components = Component::list_to_be_deleted(ctx).await?;
        let mut work_queue = VecDeque::from(to_delete_components);
        while let Some(component_to_delete) = work_queue.pop_front() {
            let component = Component::try_get_by_id(ctx, component_to_delete).await?;
            if let Some(component) = component {
                if component.allowed_to_be_removed(ctx).await? {
                    Component::remove(ctx, component.id()).await?;
                    WsEvent::component_deleted(ctx, component.id())
                        .await?
                        .publish_on_commit(ctx)
                        .await?;
                    // refetch to see if new potential candidates can be removed
                    work_queue.extend(Component::list_to_be_deleted(ctx).await?);
                }
            }
        }
    }
    ctx.commit().await?;
    Ok(maybe_resource)
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
    ctx.update_snapshot_to_visibility().await?;

    let component_id = Action::component_id(ctx, action_id)
        .await?
        .ok_or(ActionError::ComponentNotFoundForAction(action_id))?;

    Ok((prototype_id, component_id))
}

#[instrument(name = "action_job.process_execution",
skip_all, level = "info", fields(
    si.action.id = ?action_id))]
async fn process_execution(
    ctx: &mut DalContext,
    maybe_resource: Option<&ActionRunResultSuccess>,
    action_id: ActionId,
) -> JobConsumerResult<()> {
    let prototype_id = Action::prototype_id(ctx, action_id).await?;
    let prototype = ActionPrototype::get_by_id(ctx, prototype_id).await?;

    let component_id = Action::component_id(ctx, action_id)
        .await?
        .ok_or(ActionError::ComponentNotFoundForAction(action_id))?;
    let component = Component::get_by_id(ctx, component_id).await?;
    if let Some(resource) = maybe_resource {
        // Set the resource if we have a payload, regardless of status *and* assemble a
        // summary
        if resource.payload.is_some() {
            component.set_resource(ctx, resource.into()).await?;
        }

        if resource.status == ResourceStatus::Ok {
            // Remove `ActionId` from graph as the execution succeeded
            Action::remove_by_id(ctx, action_id).await?;
            if resource.payload.is_none() {
                // Clear the resource if the status is ok and we don't have a payload. This could
                // be from invoking a delete action directly, rather than deleting the component.
                component.clear_resource(ctx).await?;

                if component.to_delete() {
                    Component::remove(ctx, component.id()).await?;
                    WsEvent::component_deleted(ctx, component.id())
                        .await?
                        .publish_on_commit(ctx)
                        .await?;
                } else {
                    let mut diagram_sockets = HashMap::new();
                    let summary = component
                        .into_frontend_type(ctx, ChangeStatus::Unmodified, &mut diagram_sockets)
                        .await?;
                    WsEvent::resource_refreshed(ctx, summary)
                        .await?
                        .publish_on_commit(ctx)
                        .await?;
                }
            } else {
                let mut diagram_sockets = HashMap::new();
                let summary = component
                    .into_frontend_type(ctx, ChangeStatus::Unmodified, &mut diagram_sockets)
                    .await?;
                WsEvent::resource_refreshed(ctx, summary)
                    .await?
                    .publish_on_commit(ctx)
                    .await?;
            }

            let triggered_prototypes =
                ActionPrototype::get_prototypes_to_trigger(ctx, prototype.id()).await?;
            for dependency_prototype_id in triggered_prototypes {
                Action::new(ctx, dependency_prototype_id, Some(component_id)).await?;
            }
        } else {
            // If status is not ok, set action state to failed
            Action::set_state(ctx, action_id, ActionState::Failed).await?;
        }
    } else {
        // If the maybe_resource is none, set action state to failed
        Action::set_state(ctx, action_id, ActionState::Failed).await?;
    }

    WsEvent::action_list_updated(ctx)
        .await?
        .publish_on_commit(ctx)
        .await?;

    // Send the rebase request with the resource updated (if applicable)
    ctx.commit().await?;
    ctx.update_snapshot_to_visibility().await?;
    Ok(())
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
