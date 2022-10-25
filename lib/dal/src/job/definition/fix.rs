use std::{collections::HashMap, convert::TryFrom};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{
    job::{
        consumer::{FaktoryJobInfo, JobConsumer, JobConsumerError, JobConsumerResult},
        producer::{JobMeta, JobProducer, JobProducerResult},
    },
    AccessBuilder, ChangeSetPk, Component, ComponentId, ConfirmationResolverId, DalContext,
    FixResolver, FixResolverContext, StandardModel, SystemId, Visibility, WorkflowPrototypeId,
    WorkflowRunner, WsEvent,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fix {
    pub workflow_prototype_id: WorkflowPrototypeId,
    pub component_id: ComponentId,
    pub confirmation_resolver_id: ConfirmationResolverId,
}

#[derive(Debug, Deserialize, Serialize)]
struct FixesArgs {
    fixes: Vec<Fix>,
}

impl From<Fixes> for FixesArgs {
    fn from(value: Fixes) -> Self {
        Self { fixes: value.fixes }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Fixes {
    fixes: Vec<Fix>,
    access_builder: AccessBuilder,
    visibility: Visibility,
    faktory_job: Option<FaktoryJobInfo>,
}

impl Fixes {
    pub fn new(ctx: &DalContext, fixes: Vec<Fix>) -> Box<Self> {
        let access_builder = AccessBuilder::from(ctx.clone());
        let visibility = *ctx.visibility();

        Box::new(Self {
            fixes,
            access_builder,
            visibility,
            faktory_job: None,
        })
    }
}

impl JobProducer for Fixes {
    fn args(&self) -> JobProducerResult<serde_json::Value> {
        Ok(serde_json::to_value(FixesArgs::from(self.clone()))?)
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
        serde_json::to_string(self).expect("Cannot serialize Fixes")
    }
}

#[async_trait]
impl JobConsumer for Fixes {
    fn type_name(&self) -> String {
        "Fixes".to_string()
    }

    fn access_builder(&self) -> AccessBuilder {
        self.access_builder.clone()
    }

    fn visibility(&self) -> Visibility {
        self.visibility
    }

    async fn run(&self, ctx: &DalContext) -> JobConsumerResult<()> {
        let run_id = rand::random();

        if self.fixes.is_empty() {
            return Ok(());
        }

        let fix = &self.fixes[0];

        let component = Component::get_by_id(ctx, &fix.component_id)
            .await?
            .ok_or(JobConsumerError::ComponentNotFound(fix.component_id))?;
        let schema_variant = component
            .schema_variant(ctx)
            .await?
            .ok_or(JobConsumerError::NoSchemaVariantFound(fix.component_id))?;
        let schema = component
            .schema(ctx)
            .await?
            .ok_or(JobConsumerError::NoSchemaFound(fix.component_id))?;

        let context = FixResolverContext {
            component_id: fix.component_id,
            schema_id: *schema.id(),
            schema_variant_id: *schema_variant.id(),
            system_id: SystemId::NONE,
        };
        let _fix_resolver = FixResolver::upsert(
            ctx,
            fix.workflow_prototype_id,
            fix.confirmation_resolver_id,
            context,
        )
        .await?;

        let (
            _runner,
            runner_state,
            func_binding_return_values,
            _created_resources,
            _updated_resources,
        ) = WorkflowRunner::run(ctx, run_id, fix.workflow_prototype_id, fix.component_id).await?;

        // NOTE(nick,wendy): this looks similar to code insider WorkflowRunner::run(). Do we need to run
        // it twice?
        // reference: https://github.com/systeminit/si/blob/87c5cce99d6b972f441358295bbabe27f1d787da/lib/dal/src/workflow_runner.rs#L209-L227
        let mut logs = Vec::new();
        for func_binding_return_value in func_binding_return_values {
            for stream in func_binding_return_value
                .get_output_stream(ctx)
                .await?
                .unwrap_or_default()
            {
                match stream.data {
                    Some(data) => logs.push((
                        stream.timestamp,
                        format!(
                            "{} {}",
                            stream.message,
                            serde_json::to_string_pretty(&data)?
                        ),
                    )),
                    None => logs.push((stream.timestamp, stream.message)),
                }
            }
        }
        logs.sort_by_key(|(timestamp, _)| *timestamp);
        let logs = logs.into_iter().map(|(_, log)| log).collect();

        WsEvent::fix_return(ctx, fix.confirmation_resolver_id, runner_state, logs)
            .publish(ctx)
            .await?;

        if self.fixes.len() > 1 {
            ctx.enqueue_job(Fixes::new(
                ctx,
                self.fixes.iter().skip(1).cloned().collect(),
            ))
            .await;
        } else {
            // Re-trigger confirmations and informs the frontend to re-fetch everything on head
            WsEvent::change_set_applied(ctx, ChangeSetPk::NONE)
                .await?
                .publish(ctx)
                .await?;
        }

        Ok(())
    }
}

impl TryFrom<faktory_async::Job> for Fixes {
    type Error = JobConsumerError;

    fn try_from(job: faktory_async::Job) -> Result<Self, Self::Error> {
        if job.args().len() != 3 {
            return Err(JobConsumerError::InvalidArguments(
                r#"[{ "fixes": [Fixes] }, <AccessBuilder>, <Visibility>]"#.to_string(),
                job.args().to_vec(),
            ));
        }
        let args: FixesArgs = serde_json::from_value(job.args()[0].clone())?;
        let access_builder: AccessBuilder = serde_json::from_value(job.args()[1].clone())?;
        let visibility: Visibility = serde_json::from_value(job.args()[2].clone())?;

        let faktory_job_info = FaktoryJobInfo::try_from(job)?;

        Ok(Self {
            fixes: args.fixes,
            access_builder,
            visibility,
            faktory_job: Some(faktory_job_info),
        })
    }
}
