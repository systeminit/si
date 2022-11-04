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
    AccessBuilder, Component, ComponentError, ComponentId, DalContext, QualificationPrototypeId,
    StandardModel, SystemId, Visibility,
};

#[derive(Debug, Deserialize, Serialize)]
struct QualificationArgs {
    component_id: ComponentId,
    system_id: SystemId,
    prototype_id: QualificationPrototypeId,
}

impl From<Qualification> for QualificationArgs {
    fn from(value: Qualification) -> Self {
        Self {
            component_id: value.component_id,
            system_id: value.system_id,
            prototype_id: value.prototype_id,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Qualification {
    component_id: ComponentId,
    prototype_id: QualificationPrototypeId,
    system_id: SystemId,
    access_builder: AccessBuilder,
    visibility: Visibility,
    faktory_job: Option<FaktoryJobInfo>,
}

impl Qualification {
    pub async fn new(
        ctx: &DalContext,
        component_id: ComponentId,
        prototype_id: QualificationPrototypeId,
        system_id: SystemId,
    ) -> ComponentResult<Box<Self>> {
        let component = Component::get_by_id(ctx, &component_id)
            .await?
            .ok_or(ComponentError::NotFound(component_id))?;

        component
            .prepare_qualification_check(ctx, system_id, prototype_id)
            .await?;

        let access_builder = AccessBuilder::from(ctx.clone());
        let visibility = *ctx.visibility();

        Ok(Box::new(Self {
            component_id,
            prototype_id,
            system_id,
            access_builder,
            visibility,
            faktory_job: None,
        }))
    }
}

impl JobProducer for Qualification {
    fn args(&self) -> JobProducerResult<serde_json::Value> {
        Ok(serde_json::to_value(QualificationArgs::from(self.clone()))?)
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
        serde_json::to_string(self).expect("Cannot serialize Qualification")
    }
}

#[async_trait]
impl JobConsumer for Qualification {
    fn type_name(&self) -> String {
        "Qualification".to_string()
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

        component
            .check_qualification(ctx, self.system_id, self.prototype_id)
            .await?;

        Ok(())
    }
}

impl TryFrom<Job> for Qualification {
    type Error = JobConsumerError;

    fn try_from(job: Job) -> Result<Self, Self::Error> {
        if job.args().len() != 3 {
            return Err(JobConsumerError::InvalidArguments(
                r#"[{ "component_id": <ComponentId>, "system_id": [SystemId], "prototype_id": <QualificationPrototypeId> }, <AccessBuilder>, <Visibility>]"#.to_string(),
                job.args().to_vec(),
            ));
        }
        let args: QualificationArgs = serde_json::from_value(job.args()[0].clone())?;
        let access_builder: AccessBuilder = serde_json::from_value(job.args()[1].clone())?;
        let visibility: Visibility = serde_json::from_value(job.args()[2].clone())?;

        let faktory_job_info = FaktoryJobInfo::try_from(job)?;

        Ok(Self {
            component_id: args.component_id,
            system_id: args.system_id,
            prototype_id: args.prototype_id,
            access_builder,
            visibility,
            faktory_job: Some(faktory_job_info),
        })
    }
}
