use std::convert::TryFrom;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use telemetry::prelude::*;

use crate::{
    job::{
        consumer::{
            JobConsumer, JobConsumerError, JobConsumerMetadata, JobConsumerResult, JobInfo,
        },
        producer::{JobProducer, JobProducerResult},
    },
    AccessBuilder, ActionKind, Component, ComponentId, DalContext, StandardModel, Visibility,
    WsEvent,
};

#[derive(Debug, Deserialize, Serialize)]
struct RefreshJobArgs {
    component_ids: Vec<ComponentId>,
}

impl From<RefreshJob> for RefreshJobArgs {
    fn from(value: RefreshJob) -> Self {
        Self {
            component_ids: value.component_ids,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct RefreshJob {
    component_ids: Vec<ComponentId>,
    access_builder: AccessBuilder,
    visibility: Visibility,
    job: Option<JobInfo>,
}

impl RefreshJob {
    pub fn new(
        access_builder: AccessBuilder,
        visibility: Visibility,
        component_ids: Vec<ComponentId>,
    ) -> Box<Self> {
        Box::new(Self {
            component_ids,
            access_builder,
            visibility,
            job: None,
        })
    }
}

impl JobProducer for RefreshJob {
    fn arg(&self) -> JobProducerResult<serde_json::Value> {
        Ok(serde_json::to_value(RefreshJobArgs::from(self.clone()))?)
    }
}

impl JobConsumerMetadata for RefreshJob {
    fn type_name(&self) -> String {
        "RefreshJob".to_string()
    }

    fn access_builder(&self) -> AccessBuilder {
        self.access_builder
    }

    fn visibility(&self) -> Visibility {
        self.visibility
    }
}

#[async_trait]
impl JobConsumer for RefreshJob {
    #[instrument(
        name = "refresh_job.run",
        skip_all,
        level = "info",
        fields(
            component_ids = ?self.component_ids,
        )
    )]
    async fn run(&self, ctx: &mut DalContext) -> JobConsumerResult<()> {
        // TODO(nick,paulo,zack,jacob): ensure we do not _have_ to do this in the future.
        ctx.update_with_deleted_visibility();

        for component_id in &self.component_ids {
            let component = Component::get_by_id(ctx, component_id)
                .await?
                .ok_or(JobConsumerError::ComponentNotFound(*component_id))?;
            component.act(ctx, ActionKind::Refresh).await?;

            WsEvent::resource_refreshed(ctx, *component.id())
                .await?
                .publish_on_commit(ctx)
                .await?;

            // Save the refreshed resource for the component
            ctx.commit().await?;
        }

        Ok(())
    }
}

impl TryFrom<JobInfo> for RefreshJob {
    type Error = JobConsumerError;

    fn try_from(job: JobInfo) -> Result<Self, Self::Error> {
        let args = RefreshJobArgs::deserialize(&job.arg)?;

        Ok(Self {
            component_ids: args.component_ids,
            access_builder: job.access_builder,
            visibility: job.visibility,
            job: Some(job),
        })
    }
}
