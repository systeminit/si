use std::{collections::HashMap, convert::TryFrom};

use async_trait::async_trait;
use serde::Serialize;
use si_data_faktory::Job;

use crate::confirmation_status::ConfirmationStatus;
use crate::job::definition::confirmation::Confirmation;

use crate::{
    job::{
        consumer::{FaktoryJobInfo, JobConsumer, JobConsumerError, JobConsumerResult},
        producer::{JobMeta, JobProducer, JobProducerResult},
    },
    AccessBuilder, Component, ConfirmationPrototype, DalContext, StandardModel, SystemId,
    Visibility, WsEvent,
};

#[derive(Clone, Debug, Serialize)]
pub struct Confirmations {
    access_builder: AccessBuilder,
    visibility: Visibility,
    faktory_job: Option<FaktoryJobInfo>,
}

impl Confirmations {
    pub fn new(ctx: &DalContext) -> Box<Self> {
        let access_builder = AccessBuilder::from(ctx.clone());
        let visibility = *ctx.visibility();

        Box::new(Self {
            access_builder,
            visibility,
            faktory_job: None,
        })
    }
}

impl JobProducer for Confirmations {
    fn args(&self) -> JobProducerResult<serde_json::Value> {
        Ok(serde_json::Value::Null)
    }

    fn meta(&self) -> JobProducerResult<JobMeta> {
        let mut custom = HashMap::new();
        custom.insert(
            "access_builder".to_string(),
            serde_json::to_value(self.access_builder.clone())?,
        );
        custom.insert(
            "visibility".to_string(),
            serde_json::to_value(self.visibility)?,
        );

        Ok(JobMeta {
            retry: Some(0),
            custom,
            ..JobMeta::default()
        })
    }

    fn identity(&self) -> String {
        serde_json::to_string(self).expect("Cannot serialize Confirmations")
    }
}

#[async_trait]
impl JobConsumer for Confirmations {
    fn type_name(&self) -> String {
        "Confirmations".to_string()
    }

    fn access_builder(&self) -> AccessBuilder {
        self.access_builder.clone()
    }

    fn visibility(&self) -> Visibility {
        self.visibility
    }

    async fn run(&self, ctx: &DalContext) -> JobConsumerResult<()> {
        let system_id = SystemId::NONE;

        for component in Component::list(ctx).await? {
            let prototypes =
                ConfirmationPrototype::list_for_component(ctx, *component.id(), system_id).await?;
            for prototype in prototypes {
                prototype.prepare(ctx, *component.id(), system_id).await?;

                WsEvent::confirmation_status_update(
                    ctx,
                    *component.id(),
                    system_id,
                    *prototype.id(),
                    ConfirmationStatus::Running,
                    None,
                )
                .publish(ctx)
                .await?;
                ctx.enqueue_job(Confirmation::new(
                    ctx,
                    *component.id(),
                    system_id,
                    *prototype.id(),
                ))
                .await;
            }
        }
        Ok(())
    }
}

impl TryFrom<Job> for Confirmations {
    type Error = JobConsumerError;

    fn try_from(job: Job) -> Result<Self, Self::Error> {
        if job.args().len() != 3 {
            return Err(JobConsumerError::InvalidArguments(
                r#"[null, <AccessBuilder>, <Visibility>]"#.to_string(),
                job.args().to_vec(),
            ));
        }
        let access_builder: AccessBuilder = serde_json::from_value(job.args()[1].clone())?;
        let visibility: Visibility = serde_json::from_value(job.args()[2].clone())?;

        let faktory_job_info = FaktoryJobInfo::try_from(job)?;

        Ok(Self {
            access_builder,
            visibility,
            faktory_job: Some(faktory_job_info),
        })
    }
}
