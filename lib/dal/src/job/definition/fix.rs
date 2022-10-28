use std::{collections::HashMap, convert::TryFrom};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::fix::FixError;
use crate::{
    job::{
        consumer::{FaktoryJobInfo, JobConsumer, JobConsumerError, JobConsumerResult},
        producer::{JobMeta, JobProducer, JobProducerResult},
    },
    AccessBuilder, ActionPrototype, ChangeSetPk, Component, ComponentId, ConfirmationResolverId,
    DalContext, Fix, FixBatch, FixBatchId, FixCompletionStatus, FixId, FixResolver,
    FixResolverContext, StandardModel, SystemId, Visibility, WsEvent,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixItem {
    pub id: FixId,
    pub action: String,
    pub component_id: ComponentId,
    pub confirmation_resolver_id: ConfirmationResolverId,
}

#[derive(Debug, Deserialize, Serialize)]
struct FixesJobArgs {
    fixes: Vec<FixItem>,
    batch_id: FixBatchId,
    started: bool,
}

impl From<FixesJob> for FixesJobArgs {
    fn from(value: FixesJob) -> Self {
        Self {
            fixes: value.fixes,
            batch_id: value.batch_id,
            started: value.started,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct FixesJob {
    fixes: Vec<FixItem>,
    started: bool,
    batch_id: FixBatchId,
    access_builder: AccessBuilder,
    visibility: Visibility,
    faktory_job: Option<FaktoryJobInfo>,
}

impl FixesJob {
    pub fn new(ctx: &DalContext, fixes: Vec<FixItem>, batch_id: FixBatchId) -> Box<Self> {
        Self::new_raw(ctx, fixes, batch_id, false)
    }

    /// Used for creating another fix job in a "fixes" sequence.
    fn new_iteration(ctx: &DalContext, fixes: Vec<FixItem>, batch_id: FixBatchId) -> Box<Self> {
        Self::new_raw(ctx, fixes, batch_id, true)
    }

    fn new_raw(
        ctx: &DalContext,
        fixes: Vec<FixItem>,
        batch_id: FixBatchId,
        started: bool,
    ) -> Box<Self> {
        let access_builder = AccessBuilder::from(ctx.clone());
        let visibility = *ctx.visibility();

        Box::new(Self {
            fixes,
            started,
            batch_id,
            access_builder,
            visibility,
            faktory_job: None,
        })
    }
}

impl JobProducer for FixesJob {
    fn args(&self) -> JobProducerResult<serde_json::Value> {
        Ok(serde_json::to_value(FixesJobArgs::from(self.clone()))?)
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
        serde_json::to_string(self).expect("Cannot serialize FixesJob")
    }
}

#[async_trait]
impl JobConsumer for FixesJob {
    fn type_name(&self) -> String {
        "FixesJob".to_string()
    }

    fn access_builder(&self) -> AccessBuilder {
        self.access_builder.clone()
    }

    fn visibility(&self) -> Visibility {
        self.visibility
    }

    async fn run(&self, ctx: &DalContext) -> JobConsumerResult<()> {
        // Mark the batch as started if it has not been yet.
        if !self.started {
            let mut batch = FixBatch::get_by_id(ctx, &self.batch_id)
                .await?
                .ok_or(JobConsumerError::MissingFixBatch(self.batch_id))?;
            batch.stamp_started(ctx).await?;
        }

        if self.fixes.is_empty() {
            return Ok(());
        }
        let fix_item = &self.fixes[0];

        // Get the workflow for the action we need to run.
        let component = Component::get_by_id(ctx, &fix_item.component_id)
            .await?
            .ok_or(JobConsumerError::ComponentNotFound(fix_item.component_id))?;
        let schema_variant =
            component
                .schema_variant(ctx)
                .await?
                .ok_or(JobConsumerError::NoSchemaVariantFound(
                    fix_item.component_id,
                ))?;
        let schema = component
            .schema(ctx)
            .await?
            .ok_or(JobConsumerError::NoSchemaFound(fix_item.component_id))?;
        let action = ActionPrototype::find_by_name(
            ctx,
            &fix_item.action,
            *schema.id(),
            *schema_variant.id(),
            SystemId::NONE,
        )
        .await?
        .ok_or_else(|| {
            JobConsumerError::ActionNotFound(fix_item.action.clone(), fix_item.component_id)
        })?;
        let workflow_prototype_id = action.workflow_prototype_id();

        // Run the fix (via the action's workflow prototype).
        let mut fix = Fix::get_by_id(ctx, &fix_item.id)
            .await?
            .ok_or(FixError::MissingFix(fix_item.id))?;
        let run_id = rand::random();
        fix.run(
            ctx,
            run_id,
            workflow_prototype_id,
            action.name().to_string(),
        )
        .await?;
        let completion_status: FixCompletionStatus = *fix
            .completion_status()
            .ok_or(FixError::EmptyCompletionStatus)?;

        // Upsert the relevant fix resolver.
        let context = FixResolverContext {
            component_id: fix_item.component_id,
            schema_id: *schema.id(),
            schema_variant_id: *schema_variant.id(),
            system_id: SystemId::NONE,
        };
        let _fix_resolver = FixResolver::upsert(
            ctx,
            workflow_prototype_id,
            fix_item.confirmation_resolver_id,
            Some(matches!(completion_status, FixCompletionStatus::Success)),
            context,
        )
        .await?;

        // TODO(nick): once the logs' type changes, to Vec<String>, remove this.
        let logs = match fix.logs() {
            Some(logs) => logs
                .split('\n')
                .map(|log| log.to_string())
                .collect::<Vec<String>>(),
            None => vec![],
        };

        WsEvent::fix_return(
            ctx,
            *fix.id(),
            self.batch_id,
            fix_item.confirmation_resolver_id,
            fix_item.action.clone(),
            completion_status,
            logs,
        )
        .publish(ctx)
        .await?;

        if self.fixes.len() > 1 {
            ctx.enqueue_job(FixesJob::new_iteration(
                ctx,
                self.fixes.iter().skip(1).cloned().collect(),
                self.batch_id,
            ))
            .await;
        } else {
            // Mark the batch as completed.
            let mut batch = FixBatch::get_by_id(ctx, &self.batch_id)
                .await?
                .ok_or(JobConsumerError::MissingFixBatch(self.batch_id))?;
            let batch_completion_status = batch.stamp_finished(ctx).await?;
            WsEvent::fix_batch_return(ctx, *batch.id(), batch_completion_status)
                .publish(ctx)
                .await?;

            // Re-trigger confirmations and informs the frontend to re-fetch everything on head
            WsEvent::change_set_applied(ctx, ChangeSetPk::NONE)
                .await
                .publish(ctx)
                .await?;
        }

        Ok(())
    }
}

impl TryFrom<faktory_async::Job> for FixesJob {
    type Error = JobConsumerError;

    fn try_from(job: faktory_async::Job) -> Result<Self, Self::Error> {
        if job.args().len() != 3 {
            return Err(JobConsumerError::InvalidArguments(
                r#"[{ fixes: Vec<FixItem>, batch_id: FixBatchId, started: bool }, <AccessBuilder>, <Visibility>]"#
                    .to_string(),
                job.args().to_vec(),
            ));
        }
        let args: FixesJobArgs = serde_json::from_value(job.args()[0].clone())?;
        let access_builder: AccessBuilder = serde_json::from_value(job.args()[1].clone())?;
        let visibility: Visibility = serde_json::from_value(job.args()[2].clone())?;

        let faktory_job_info = FaktoryJobInfo::try_from(job)?;

        Ok(Self {
            fixes: args.fixes,
            batch_id: args.batch_id,
            started: args.started,
            access_builder,
            visibility,
            faktory_job: Some(faktory_job_info),
        })
    }
}
