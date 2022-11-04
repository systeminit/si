use std::{collections::HashMap, convert::TryFrom};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use si_data_faktory::Job;

use crate::{
    component::ComponentResult,
    job::{
        consumer::{FaktoryJobInfo, JobConsumer, JobConsumerError, JobConsumerResult},
        producer::{JobMeta, JobProducer, JobProducerResult},
    },
    AccessBuilder, Component, ComponentError, ComponentId, DalContext, StandardModel, SystemId,
    Visibility,
};

#[derive(Debug, Deserialize, Serialize)]
struct QualificationsArgs {
    component_id: ComponentId,
    system_id: SystemId,
}

impl From<Qualifications> for QualificationsArgs {
    fn from(value: Qualifications) -> Self {
        Self {
            component_id: value.component_id,
            system_id: value.system_id,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Qualifications {
    component_id: ComponentId,
    system_id: SystemId,
    access_builder: AccessBuilder,
    visibility: Visibility,
    faktory_job: Option<FaktoryJobInfo>,
}

impl Qualifications {
    pub async fn new(
        ctx: &DalContext,
        component_id: ComponentId,
        system_id: SystemId,
    ) -> ComponentResult<Box<Self>> {
        let component = Component::get_by_id(ctx, &component_id)
            .await?
            .ok_or(ComponentError::NotFound(component_id))?;

        component
            .prepare_qualifications_check(ctx, system_id)
            .await?;

        let access_builder = AccessBuilder::from(ctx.clone());
        let visibility = *ctx.visibility();

        Ok(Box::new(Self {
            component_id,
            system_id,
            access_builder,
            visibility,
            faktory_job: None,
        }))
    }
}

impl JobProducer for Qualifications {
    fn args(&self) -> JobProducerResult<serde_json::Value> {
        Ok(serde_json::to_value(QualificationsArgs::from(
            self.clone(),
        ))?)
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
        serde_json::to_string(self).expect("Cannot serialize Qualifications")
    }
}

#[async_trait]
impl JobConsumer for Qualifications {
    fn type_name(&self) -> String {
        "Qualifications".to_string()
    }

    fn access_builder(&self) -> AccessBuilder {
        self.access_builder.clone()
    }

    fn visibility(&self) -> Visibility {
        self.visibility
    }

    async fn run(&self, ctx: &DalContext) -> JobConsumerResult<()> {
        let component = Component::get_by_id(ctx, &self.component_id)
            .await?
            .ok_or(ComponentError::NotFound(self.component_id))?;

        component.check_qualifications(ctx, self.system_id).await?;

        Ok(())
    }
}

impl TryFrom<Job> for Qualifications {
    type Error = JobConsumerError;

    fn try_from(job: Job) -> Result<Self, Self::Error> {
        if job.args().len() != 3 {
            return Err(JobConsumerError::InvalidArguments(
                r#"[{ "component_id": <ComponentId>, "system_id": [SystemId] }, <AccessBuilder>, <Visibility>]"#.to_string(),
                job.args().to_vec(),
            ));
        }
        let args: QualificationsArgs = serde_json::from_value(job.args()[0].clone())?;
        let access_builder: AccessBuilder = serde_json::from_value(job.args()[1].clone())?;
        let visibility: Visibility = serde_json::from_value(job.args()[2].clone())?;

        let faktory_job_info = FaktoryJobInfo::try_from(job)?;

        Ok(Self {
            component_id: args.component_id,
            system_id: args.system_id,
            access_builder,
            visibility,
            faktory_job: Some(faktory_job_info),
        })
    }
}
