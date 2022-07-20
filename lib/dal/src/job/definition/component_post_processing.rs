use std::{collections::HashMap, convert::TryFrom};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

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
struct ComponentPostProcessingArgs {
    component_id: ComponentId,
    system_id: SystemId,
    qualification_prototype_id: Option<QualificationPrototypeId>,
}

impl From<ComponentPostProcessing> for ComponentPostProcessingArgs {
    fn from(value: ComponentPostProcessing) -> Self {
        Self {
            component_id: value.component_id,
            system_id: value.system_id,
            qualification_prototype_id: value.qualification_prototype_id,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ComponentPostProcessing {
    component_id: ComponentId,
    system_id: SystemId,
    qualification_prototype_id: Option<QualificationPrototypeId>,
    access_builder: AccessBuilder,
    visibility: Visibility,
    faktory_job: Option<FaktoryJobInfo>,
}

impl ComponentPostProcessing {
    pub async fn new(
        ctx: &DalContext<'_, '_>,
        component_id: ComponentId,
        system_id: SystemId,
        qualification_prototype_id: Option<QualificationPrototypeId>,
    ) -> ComponentResult<Box<Self>> {
        let component = Component::get_by_id(ctx, &component_id)
            .await?
            .ok_or(ComponentError::NotFound(component_id))?;

        if let Some(qualification_prototype_id) = qualification_prototype_id {
            component
                .prepare_qualification_check(ctx, system_id, qualification_prototype_id)
                .await?;
        } else {
            component.prepare_code_generation(ctx, system_id).await?;
            component
                .prepare_qualifications_check(ctx, system_id)
                .await?;
        }

        let access_builder = AccessBuilder::from(ctx.clone());
        let visibility = *ctx.visibility();

        Ok(Box::new(Self {
            component_id,
            system_id,
            qualification_prototype_id,
            access_builder,
            visibility,
            faktory_job: None,
        }))
    }
}

impl JobProducer for ComponentPostProcessing {
    fn args(&self) -> JobProducerResult<serde_json::Value> {
        Ok(serde_json::to_value(ComponentPostProcessingArgs::from(
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
        serde_json::to_string(self).expect("Cannot serialize ComponentPostProcessing")
    }
}

#[async_trait]
impl JobConsumer for ComponentPostProcessing {
    fn type_name(&self) -> String {
        "ComponentPostProcessing".to_string()
    }

    fn access_builder(&self) -> AccessBuilder {
        self.access_builder.clone()
    }

    fn visibility(&self) -> Visibility {
        self.visibility
    }

    async fn run(&self, ctx: &DalContext<'_, '_>) -> JobConsumerResult<()> {
        let component = Component::get_by_id(ctx, &self.component_id)
            .await?
            .ok_or(ComponentError::NotFound(self.component_id))?;

        if let Some(qualification_prototype_id) = self.qualification_prototype_id {
            component
                .check_qualification(ctx, self.system_id, qualification_prototype_id)
                .await?;
        } else {
            component.generate_code(ctx, self.system_id).await?;
            // Some qualifications depend on code generation, so remember to generate the code first
            component.check_qualifications(ctx, self.system_id).await?;
        }

        Ok(())
    }
}

impl TryFrom<faktory_async::Job> for ComponentPostProcessing {
    type Error = JobConsumerError;

    fn try_from(job: faktory_async::Job) -> Result<Self, Self::Error> {
        if job.args().len() != 3 {
            return Err(JobConsumerError::InvalidArguments(
                r#"[{ "component_id": <ComponentId>, "system_id": [SystemId] }, <AccessBuilder>, <Visibility>]"#.to_string(),
                job.args().to_vec(),
            ));
        }
        let args: ComponentPostProcessingArgs = serde_json::from_value(job.args()[0].clone())?;
        let access_builder: AccessBuilder = serde_json::from_value(job.args()[1].clone())?;
        let visibility: Visibility = serde_json::from_value(job.args()[2].clone())?;

        let faktory_job_info = FaktoryJobInfo::try_from(job)?;

        Ok(Self {
            component_id: args.component_id,
            system_id: args.system_id,
            qualification_prototype_id: args.qualification_prototype_id,
            access_builder,
            visibility,
            faktory_job: Some(faktory_job_info),
        })
    }
}
