use std::convert::TryFrom;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use telemetry::prelude::*;
use veritech_client::{ActionRunResultSuccess, ResourceStatus};

use crate::{
    action::{prototype::ActionPrototype, Action, ActionError, ActionId, ActionState},
    component::resource::ResourceData,
    job::{
        consumer::{
            JobCompletionState, JobConsumer, JobConsumerError, JobConsumerMetadata,
            JobConsumerResult, JobInfo,
        },
        producer::{JobProducer, JobProducerResult},
    },
    AccessBuilder, Component, DalContext, Visibility, WsEvent,
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
        )
    )]
    async fn run(&self, ctx: &mut DalContext) -> JobConsumerResult<JobCompletionState> {
        match action_task(ctx, self.id, Span::current()).await {
            Ok(_) => {
                debug!(?self.id, "action job completed");
            }
            Err(err) => {
                error!("Unable to finish action {}: {err}", self.id);
                if let Err(err) =
                    process_failed_action(ctx, self.id, format!("Action failed: {err}")).await
                {
                    error!("Failed to process action failure: {err}");
                }
            }
        }

        ctx.commit().await?;

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
    parent = &parent_span,
    skip_all,
    level = "info",
    fields(
        ?id,
        si.action.kind = Empty,
        si.component.id = Empty,
    )
)]
async fn action_task(
    ctx: &mut DalContext,
    id: ActionId,
    parent_span: Span,
) -> JobConsumerResult<Option<ActionRunResultSuccess>> {
    let span = Span::current();
    let component_id = Action::component_id(ctx, id)
        .await?
        .ok_or(ActionError::ComponentNotFoundForAction(id))?;

    let prototype_id = Action::prototype_id(ctx, id).await?;
    let prototype = ActionPrototype::get_by_id(ctx, prototype_id).await?;
    span.record("si.action.kind", &tracing::field::debug(&prototype.kind));
    span.record("si.component.id", &tracing::field::debug(&component_id));
    Action::set_state(ctx, id, ActionState::Running).await?;

    // Updates the action's state
    ctx.commit().await?;
    ctx.update_snapshot_to_visibility().await?;

    let status = Action::run(ctx, id).await?;

    WsEvent::action_return(
        ctx,
        id,
        prototype.kind,
        component_id,
        status.clone().map(|s| s.into()),
    )
    .await?
    .publish_on_commit(ctx)
    .await?;

    if matches!(status.as_ref().map(|r| r.status), Some(ResourceStatus::Ok)) {
        let triggered_prototypes =
            ActionPrototype::get_prototypes_to_trigger(ctx, prototype_id).await?;
        for dependency_prototype_id in triggered_prototypes {
            Action::new(ctx, dependency_prototype_id, Some(component_id)).await?;
        }
    }

    Ok(status)
}

#[instrument(name = "action_job.process_failed_action", skip_all, level = "info")]
async fn process_failed_action(
    ctx: &DalContext,
    id: ActionId,
    _error_message: String,
) -> JobConsumerResult<()> {
    info!(%id, "processing action failed");

    let component_id = Action::component_id(ctx, id)
        .await?
        .ok_or(ActionError::ComponentNotFoundForAction(id))?;
    let component = Component::get_by_id(ctx, component_id).await?;

    let prototype_id = Action::prototype_id(ctx, id).await?;
    let maybe_resource_data = component.resource(ctx).await?;
    let resource_data = match maybe_resource_data {
        Some(mut resource_data) => {
            resource_data.set_status(ResourceStatus::Error);
            resource_data
        }
        None => ResourceData::new(ResourceStatus::Error, None),
    };

    component.set_resource(ctx, resource_data.clone()).await?;
    Action::set_state(ctx, id, ActionState::Failed).await?;

    let prototype = ActionPrototype::get_by_id(ctx, prototype_id).await?;
    WsEvent::action_return(ctx, id, prototype.kind, component_id, Some(resource_data))
        .await?
        .publish_on_commit(ctx)
        .await?;

    Ok(())
}
