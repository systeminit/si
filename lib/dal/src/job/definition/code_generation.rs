use std::{collections::HashMap, convert::TryFrom};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{
    component::ComponentResult,
    job::{
        consumer::{FaktoryJobInfo, JobConsumer, JobConsumerError, JobConsumerResult},
        definition::Qualifications,
        producer::{JobMeta, JobProducer, JobProducerResult},
    },
    AccessBuilder, Component, ComponentError, ComponentId, DalContext, StandardModel, Visibility,
};

#[derive(Debug, Deserialize, Serialize)]
struct CodeGenerationArgs {
    component_id: ComponentId,
}

impl From<CodeGeneration> for CodeGenerationArgs {
    fn from(value: CodeGeneration) -> Self {
        Self {
            component_id: value.component_id,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct CodeGeneration {
    component_id: ComponentId,
    access_builder: AccessBuilder,
    visibility: Visibility,
    faktory_job: Option<FaktoryJobInfo>,
}

impl CodeGeneration {
    pub async fn new(ctx: &DalContext, component_id: ComponentId) -> ComponentResult<Box<Self>> {
        let component = Component::get_by_id(ctx, &component_id)
            .await?
            .ok_or(ComponentError::NotFound(component_id))?;

        component.generate_code(ctx, true).await?;

        let access_builder = AccessBuilder::from(ctx.clone());
        let visibility = *ctx.visibility();

        Ok(Box::new(Self {
            component_id,
            access_builder,
            visibility,
            faktory_job: None,
        }))
    }
}

impl JobProducer for CodeGeneration {
    fn args(&self) -> JobProducerResult<serde_json::Value> {
        Ok(serde_json::to_value(CodeGenerationArgs::from(
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
        serde_json::to_string(self).expect("Cannot serialize CodeGeneration")
    }
}

#[async_trait]
impl JobConsumer for CodeGeneration {
    fn type_name(&self) -> String {
        "CodeGeneration".to_string()
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

        component.generate_code(ctx, false).await?;

        // Some qualifications depend on code generation, so remember to generate the code first
        ctx.enqueue_job(Qualifications::new(ctx, self.component_id).await?)
            .await;

        Ok(())
    }
}

impl TryFrom<faktory_async::Job> for CodeGeneration {
    type Error = JobConsumerError;

    fn try_from(job: faktory_async::Job) -> Result<Self, Self::Error> {
        if job.args().len() != 3 {
            return Err(JobConsumerError::InvalidArguments(
                r#"[{ "component_id": <ComponentId> }, <AccessBuilder>, <Visibility>]"#.to_string(),
                job.args().to_vec(),
            ));
        }
        let args: CodeGenerationArgs = serde_json::from_value(job.args()[0].clone())?;
        let access_builder: AccessBuilder = serde_json::from_value(job.args()[1].clone())?;
        let visibility: Visibility = serde_json::from_value(job.args()[2].clone())?;

        let faktory_job_info = FaktoryJobInfo::try_from(job)?;

        Ok(Self {
            component_id: args.component_id,
            access_builder,
            visibility,
            faktory_job: Some(faktory_job_info),
        })
    }
}
