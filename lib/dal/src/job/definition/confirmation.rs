use std::{collections::HashMap, convert::TryFrom};

use async_trait::async_trait;
use jwt_simple::prelude::Deserialize;
use serde::Serialize;
use si_data_faktory::Job;

use crate::confirmation_status::ConfirmationStatus;

use crate::{
    job::{
        consumer::{FaktoryJobInfo, JobConsumer, JobConsumerError, JobConsumerResult},
        producer::{JobMeta, JobProducer, JobProducerResult},
    },
    AccessBuilder, ComponentId, ConfirmationPrototype, ConfirmationPrototypeError,
    ConfirmationPrototypeId, DalContext, FixResolver, FixResolverContext, StandardModel, SystemId,
    Visibility, WorkflowPrototypeId, WsEvent,
};

#[derive(Debug, Deserialize, Serialize)]
struct ConfirmationArgs {
    component_id: ComponentId,
    system_id: SystemId,
    confirmation_prototype_id: ConfirmationPrototypeId,
}

impl From<Confirmation> for ConfirmationArgs {
    fn from(value: Confirmation) -> Self {
        Self {
            component_id: value.component_id,
            system_id: value.system_id,
            confirmation_prototype_id: value.confirmation_prototype_id,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Confirmation {
    access_builder: AccessBuilder,
    visibility: Visibility,
    faktory_job: Option<FaktoryJobInfo>,
    component_id: ComponentId,
    system_id: SystemId,
    confirmation_prototype_id: ConfirmationPrototypeId,
}

impl Confirmation {
    pub fn new(
        ctx: &DalContext,
        component_id: ComponentId,
        system_id: SystemId,
        confirmation_prototype_id: ConfirmationPrototypeId,
    ) -> Box<Self> {
        let access_builder = AccessBuilder::from(ctx.clone());
        let visibility = *ctx.visibility();

        Box::new(Self {
            access_builder,
            visibility,
            faktory_job: None,
            component_id,
            system_id,
            confirmation_prototype_id,
        })
    }
}

impl JobProducer for Confirmation {
    fn args(&self) -> JobProducerResult<serde_json::Value> {
        Ok(serde_json::to_value(ConfirmationArgs::from(self.clone()))?)
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
impl JobConsumer for Confirmation {
    fn type_name(&self) -> String {
        "Confirmation".to_string()
    }

    fn access_builder(&self) -> AccessBuilder {
        self.access_builder.clone()
    }

    fn visibility(&self) -> Visibility {
        self.visibility
    }

    async fn run(&self, ctx: &DalContext) -> JobConsumerResult<()> {
        let prototype = ConfirmationPrototype::get_by_id(ctx, &self.confirmation_prototype_id)
            .await?
            .ok_or(ConfirmationPrototypeError::NotFound(
                self.confirmation_prototype_id,
            ))?;

        let (status, error_message) =
            match prototype.run(ctx, self.component_id, self.system_id).await {
                Ok(resolver) => {
                    // Creates empty fix result slot
                    let context = FixResolverContext {
                        component_id: resolver.context().component_id,
                        schema_id: resolver.context().schema_id,
                        schema_variant_id: resolver.context().schema_variant_id,
                        system_id: SystemId::NONE,
                    };
                    let _fix_resolver = FixResolver::upsert(
                        ctx,
                        WorkflowPrototypeId::NONE,
                        *resolver.id(),
                        None,
                        context,
                    )
                    .await?;
                    match resolver.success() {
                        Some(true) => (ConfirmationStatus::Success, None),
                        Some(false) => (ConfirmationStatus::Failure, None),
                        None => unreachable!(),
                    }
                }
                Err(e) => (ConfirmationStatus::Error, Some(format!("{e}"))),
            };

        WsEvent::confirmation_status_update(
            ctx,
            self.component_id,
            self.system_id,
            *prototype.id(),
            status,
            error_message,
        )
        .publish(ctx)
        .await?;
        Ok(())
    }
}

impl TryFrom<Job> for Confirmation {
    type Error = JobConsumerError;

    fn try_from(job: Job) -> Result<Self, Self::Error> {
        if job.args().len() != 3 {
            return Err(JobConsumerError::InvalidArguments(
                r#"[{ component_id: ComponentId, system_id: SystemId, confirmation_prototype_id: ConfirmationPrototypeId }, <AccessBuilder>, <Visibility>]"#.to_string(),
                job.args().to_vec(),
            ));
        }
        let args: ConfirmationArgs = serde_json::from_value(job.args()[0].clone())?;
        let access_builder: AccessBuilder = serde_json::from_value(job.args()[1].clone())?;
        let visibility: Visibility = serde_json::from_value(job.args()[2].clone())?;

        let faktory_job_info = FaktoryJobInfo::try_from(job)?;

        Ok(Self {
            access_builder,
            visibility,
            faktory_job: Some(faktory_job_info),
            component_id: args.component_id,
            system_id: args.system_id,
            confirmation_prototype_id: args.confirmation_prototype_id,
        })
    }
}
