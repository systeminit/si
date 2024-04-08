use std::convert::TryFrom;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use telemetry::prelude::*;

use crate::validation::{ValidationOutput, ValidationOutputNode};
use crate::{
    job::consumer::{
        JobConsumer, JobConsumerError, JobConsumerMetadata, JobConsumerResult, JobInfo,
    },
    job::producer::{JobProducer, JobProducerResult},
    AccessBuilder, AttributeValue, AttributeValueId, DalContext, Visibility,
};

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
    async fn run(&self, ctx: &mut DalContext) -> JobConsumerResult<()> {
        for &av_id in &self.attribute_values {
            let value = AttributeValue::get_by_id(ctx, av_id)
                .await?
                .value(ctx)
                .await?;

            let maybe_validation =
                ValidationOutput::compute_for_attribute_value_and_value(ctx, av_id, value.clone())
                    .await?;

            ValidationOutputNode::upsert_or_wipe_for_attribute_value(
                ctx,
                av_id,
                maybe_validation.clone(),
            )
            .await?;
        }

        ctx.commit().await?;

        Ok(())
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
