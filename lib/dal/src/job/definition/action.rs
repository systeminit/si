use std::{convert::TryFrom, time::Duration};

use async_trait::async_trait;
use futures::Future;
use serde::{Deserialize, Serialize};
use si_events::ActionResultState;
use telemetry::prelude::*;
use tryhard::RetryPolicy;
use veritech_client::{ActionRunResultSuccess, ResourceStatus};

use crate::{
    action::{prototype::ActionPrototype, Action, ActionError, ActionId, ActionState},
    change_status::ChangeStatus,
    diagram::SummaryDiagramComponent,
    job::{
        consumer::{
            JobCompletionState, JobConsumer, JobConsumerError, JobConsumerMetadata,
            JobConsumerResult, JobInfo,
        },
        producer::{JobProducer, JobProducerResult},
    },
    AccessBuilder, ActionPrototypeId, Component, ComponentId, DalContext, TransactionsError,
    Visibility, WsEvent,
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
            error!(error = ?err, si.action.id = %self.id, "unable to finish action");
            if let Err(err) = process_failed_action(ctx, self.id).await {
                error!(error = ?err, "failed to process action failure");
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
    let (prototype_id, component_id) = prepare_for_execution(ctx, action_id).await?;

    // Execute the action function
    let maybe_resource = ActionPrototype::run(ctx, prototype_id, component_id).await?;

    // Retry process_and_record_execution on a conflict error up to a max
    retry_on_conflicts(
        || process_and_record_execution(ctx.clone(), maybe_resource.as_ref(), action_id),
        10,
        Duration::from_millis(10),
        "si.action_job.process.retries",
    )
    .await?;

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

async fn process_and_record_execution(
    mut ctx: DalContext,
    maybe_resource: Option<&ActionRunResultSuccess>,
    action_id: ActionId,
) -> JobConsumerResult<()> {
    ctx.update_snapshot_to_visibility().await?;

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

            if resource.payload.is_none() {
                // Clear the resource if the status is ok and we don't have a payload. This could
                // be from invoking a delete action directly, rather than deleting the component.
                component.clear_resource(&ctx).await?;

                if component.to_delete() {
                    // Before we remove a component, we delete any other components that exist
                    // solely because this component exists.
                    for component_to_be_removed_id in
                        component.find_components_to_be_removed(&ctx).await?
                    {
                        Component::remove(&ctx, component_to_be_removed_id).await?;
                    }
                    Component::remove(&ctx, component.id()).await?;
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

    ctx.commit().await?;

    Ok(())
}

#[instrument(name = "action_job.process_failed_action", skip_all, level = "info")]
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
