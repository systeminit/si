use std::convert::TryFrom;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use telemetry::prelude::*;

use crate::job::consumer::JobCompletionState;
use crate::validation::{ValidationOutput, ValidationOutputNode};
use crate::{
    AccessBuilder, AttributeValueId, DalContext, Visibility,
    job::consumer::{
        JobConsumer, JobConsumerError, JobConsumerMetadata, JobConsumerResult, JobInfo,
    },
    job::producer::{JobProducer, JobProducerResult},
};
use crate::{ChangeSet, ChangeSetStatus};

#[derive(Debug, Deserialize, Serialize)]
struct ComputeValidationArgs {
    attribute_values: Vec<AttributeValueId>,
}

impl From<ComputeValidation> for ComputeValidationArgs {
    fn from(value: ComputeValidation) -> Self {
        Self {
            attribute_values: value.attribute_values,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ComputeValidation {
    attribute_values: Vec<AttributeValueId>,
    access_builder: AccessBuilder,
    visibility: Visibility,
    job: Option<JobInfo>,
}

impl ComputeValidation {
    pub fn new(
        access_builder: AccessBuilder,
        visibility: Visibility,
        attribute_values: Vec<AttributeValueId>,
    ) -> Box<Self> {
        Box::new(Self {
            attribute_values,
            access_builder,
            visibility,
            job: None,
        })
    }
}

impl JobProducer for ComputeValidation {
    fn arg(&self) -> JobProducerResult<serde_json::Value> {
        Ok(serde_json::to_value(ComputeValidationArgs::from(
            self.clone(),
        ))?)
    }
}

impl JobConsumerMetadata for ComputeValidation {
    fn type_name(&self) -> String {
        "ComputeValidation".to_string()
    }

    fn access_builder(&self) -> AccessBuilder {
        self.access_builder
    }

    fn visibility(&self) -> Visibility {
        self.visibility
    }
}

#[async_trait]
impl JobConsumer for ComputeValidation {
    #[instrument(
        name = "compute_validation.run",
        skip_all,
        level = "info",
        fields(
            attribute_values = ?self.attribute_values,
        )
    )]
    async fn run(&self, ctx: &mut DalContext) -> JobConsumerResult<JobCompletionState> {
        let change_set = ChangeSet::get_by_id(ctx, ctx.change_set_id()).await?;

        if change_set.status == ChangeSetStatus::Abandoned {
            info!("Validation enqueued for abandoned change set. Returning early");
            return Ok(JobCompletionState::Done);
        }

        let workspace_snapshot = ctx.workspace_snapshot()?;
        for &av_id in &self.attribute_values {
            // It's possible that one or more of the initial AttributeValueIds provided by the enqueued ComputeValidation
            // may have been removed from the snapshot between when the CV job was created and when we're processing
            // things now. This could happen if there are other modifications to the snapshot before the CV job starts
            // executing, as the job always operates on the current state of the change set's snapshot, not the state at the time
            // the job was created.
            if !workspace_snapshot.node_exists(av_id).await {
                debug!("Attribute Value {av_id} missing, skipping it in ComputeValidations");
                continue;
            }

            let maybe_validation =
                ValidationOutput::compute_for_attribute_value(ctx, av_id).await?;

            ValidationOutputNode::upsert_or_wipe_for_attribute_value(
                ctx,
                av_id,
                maybe_validation.clone(),
            )
            .await?;
        }

        ctx.commit().await?;
        Ok(JobCompletionState::Done)
    }
}

impl TryFrom<JobInfo> for ComputeValidation {
    type Error = JobConsumerError;

    fn try_from(job: JobInfo) -> Result<Self, Self::Error> {
        let args = ComputeValidationArgs::deserialize(&job.arg)?;
        Ok(Self {
            attribute_values: args.attribute_values,
            access_builder: job.access_builder,
            visibility: job.visibility,
            job: Some(job),
        })
    }
}
