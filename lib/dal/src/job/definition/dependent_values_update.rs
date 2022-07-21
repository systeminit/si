use std::convert::TryFrom;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{
    attribute::value::dependent_update::AttributeValueDependentUpdateHarness,
    job::consumer::{FaktoryJobInfo, JobConsumer, JobConsumerError, JobConsumerResult},
    job::producer::{JobMeta, JobProducer, JobProducerResult},
    AccessBuilder, AttributeValueId, DalContext, Visibility,
};

#[derive(Debug, Deserialize, Serialize)]
struct DependentValuesUpdateArgs {
    attribute_value_id: AttributeValueId,
}

#[derive(Clone, Debug, Serialize)]
pub struct DependentValuesUpdate {
    attribute_value_id: AttributeValueId,
    access_builder: AccessBuilder,
    visibility: Visibility,
    faktory_job: Option<FaktoryJobInfo>,
}

impl DependentValuesUpdate {
    pub fn new(
        ctx: &DalContext<'_, '_>,
        attribute_value_id: AttributeValueId,
        visibility: Visibility,
    ) -> Box<Self> {
        let access_builder = AccessBuilder::from(ctx.clone());

        Box::new(Self {
            attribute_value_id,
            access_builder,
            visibility,
            faktory_job: None,
        })
    }
}

impl JobProducer for DependentValuesUpdate {
    fn args(&self) -> JobProducerResult<serde_json::Value> {
        Ok(serde_json::json!({
            "attribute_value_id": self.attribute_value_id,
        }))
    }

    fn meta(&self) -> JobProducerResult<JobMeta> {
        Ok(JobMeta {
            retry: Some(0),
            ..JobMeta::default()
        })
    }

    fn identity(&self) -> String {
        serde_json::to_string(self).expect("Cannot serialize DependentValueUpdate")
    }
}

#[async_trait]
impl JobConsumer for DependentValuesUpdate {
    fn type_name(&self) -> String {
        "DependentValuesUpdate".to_string()
    }

    fn access_builder(&self) -> AccessBuilder {
        self.access_builder.clone()
    }

    fn visibility(&self) -> Visibility {
        self.visibility
    }

    async fn run(&self, ctx: &DalContext<'_, '_>) -> JobConsumerResult<()> {
        Ok(
            AttributeValueDependentUpdateHarness::update_dependent_values(
                ctx,
                self.attribute_value_id,
            )
            .await?,
        )
    }
}

impl TryFrom<faktory_async::Job> for DependentValuesUpdate {
    type Error = JobConsumerError;

    fn try_from(job: faktory_async::Job) -> Result<Self, Self::Error> {
        if job.args().len() != 3 {
            return Err(JobConsumerError::InvalidArguments(
                r#"[{ "attribute_value_id": <id> }, <AccessBuilder>, <Visibility>]"#.to_string(),
                job.args().to_vec(),
            ));
        }
        let args: DependentValuesUpdateArgs = serde_json::from_value(job.args()[0].clone())?;
        let access_builder: AccessBuilder = serde_json::from_value(job.args()[1].clone())?;
        let visibility: Visibility = serde_json::from_value(job.args()[2].clone())?;

        let faktory_job_info = FaktoryJobInfo::try_from(job)?;

        Ok(Self {
            attribute_value_id: args.attribute_value_id,
            access_builder,
            visibility,
            faktory_job: Some(faktory_job_info),
        })
    }
}
