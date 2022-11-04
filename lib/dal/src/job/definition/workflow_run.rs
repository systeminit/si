use std::{collections::HashMap, convert::TryFrom};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use si_data_faktory::Job;

use crate::{
    job::{
        consumer::{FaktoryJobInfo, JobConsumer, JobConsumerError, JobConsumerResult},
        producer::{JobMeta, JobProducer, JobProducerResult},
    },
    resource::ResourceView,
    AccessBuilder, ComponentId, DalContext, Visibility, WorkflowPrototypeId, WorkflowRunner,
    WsEvent,
};

#[derive(Debug, Deserialize, Serialize)]
struct WorkflowRunArgs {
    run_id: usize,
    prototype_id: WorkflowPrototypeId,
    component_id: ComponentId,
}

impl From<WorkflowRun> for WorkflowRunArgs {
    fn from(value: WorkflowRun) -> Self {
        Self {
            run_id: value.run_id,
            prototype_id: value.prototype_id,
            component_id: value.component_id,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct WorkflowRun {
    run_id: usize,
    prototype_id: WorkflowPrototypeId,
    component_id: ComponentId,
    access_builder: AccessBuilder,
    visibility: Visibility,
    faktory_job: Option<FaktoryJobInfo>,
}

impl WorkflowRun {
    pub fn new(
        ctx: &DalContext,
        run_id: usize,
        prototype_id: WorkflowPrototypeId,
        component_id: ComponentId,
    ) -> Box<Self> {
        let access_builder = AccessBuilder::from(ctx.clone());
        let visibility = *ctx.visibility();

        Box::new(Self {
            run_id,
            prototype_id,
            component_id,
            access_builder,
            visibility,
            faktory_job: None,
        })
    }
}

impl JobProducer for WorkflowRun {
    fn args(&self) -> JobProducerResult<serde_json::Value> {
        Ok(serde_json::to_value(WorkflowRunArgs::from(self.clone()))?)
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
        serde_json::to_string(self).expect("Cannot serialize WorkflowRun")
    }
}

#[async_trait]
impl JobConsumer for WorkflowRun {
    fn type_name(&self) -> String {
        "WorkflowRun".to_string()
    }

    fn access_builder(&self) -> AccessBuilder {
        self.access_builder.clone()
    }

    fn visibility(&self) -> Visibility {
        self.visibility
    }

    async fn run(&self, ctx: &DalContext) -> JobConsumerResult<()> {
        let (
            _runner,
            runner_state,
            func_binding_return_values,
            created_resources,
            updated_resources,
        ) = WorkflowRunner::run(ctx, self.run_id, self.prototype_id, self.component_id).await?;

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

        WsEvent::command_return(
            ctx,
            self.run_id,
            created_resources
                .into_iter()
                .map(ResourceView::new)
                .collect(),
            updated_resources
                .into_iter()
                .map(ResourceView::new)
                .collect(),
            runner_state,
            logs,
        )
        .publish(ctx)
        .await?;
        Ok(())
    }
}

impl TryFrom<Job> for WorkflowRun {
    type Error = JobConsumerError;

    fn try_from(job: Job) -> Result<Self, Self::Error> {
        if job.args().len() != 3 {
            return Err(JobConsumerError::InvalidArguments(
                r#"[{ "component_id": <ComponentId>, "prototype_id": [WorkflowPrototypeId], "run_id": usize }, <AccessBuilder>, <Visibility>]"#.to_string(),
                job.args().to_vec(),
            ));
        }
        let args: WorkflowRunArgs = serde_json::from_value(job.args()[0].clone())?;
        let access_builder: AccessBuilder = serde_json::from_value(job.args()[1].clone())?;
        let visibility: Visibility = serde_json::from_value(job.args()[2].clone())?;

        let faktory_job_info = FaktoryJobInfo::try_from(job)?;

        Ok(Self {
            component_id: args.component_id,
            prototype_id: args.prototype_id,
            run_id: args.run_id,
            access_builder,
            visibility,
            faktory_job: Some(faktory_job_info),
        })
    }
}
