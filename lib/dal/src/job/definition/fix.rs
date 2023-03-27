use std::{collections::HashMap, convert::TryFrom};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{
    fix::FixError,
    job::{
        consumer::{
            JobConsumer, JobConsumerError, JobConsumerMetadata, JobConsumerResult, JobInfo,
        },
        producer::{JobMeta, JobProducer, JobProducerResult},
    },
    AccessBuilder, ActionPrototype, AttributeValueId, Component, ComponentId, DalContext, Fix,
    FixBatch, FixBatchId, FixCompletionStatus, FixId, FixResolver, StandardModel, Visibility,
    WsEvent,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixItem {
    pub id: FixId,
    pub action: String,
    pub component_id: ComponentId,
    pub attribute_value_id: AttributeValueId,
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
    job: Option<JobInfo>,
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
            job: None,
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

impl JobConsumerMetadata for FixesJob {
    fn type_name(&self) -> String {
        "FixesJob".to_string()
    }

    fn access_builder(&self) -> AccessBuilder {
        self.access_builder.clone()
    }

    fn visibility(&self) -> Visibility {
        self.visibility
    }
}

#[async_trait]
impl JobConsumer for FixesJob {
    async fn run(&self, ctx: &DalContext) -> JobConsumerResult<()> {
        // Mark the batch as started if it has not been yet.
        if !self.started {
            let mut batch = FixBatch::get_by_id(ctx, &self.batch_id)
                .await?
                .ok_or(JobConsumerError::MissingFixBatch(self.batch_id))?;
            batch.stamp_started(ctx).await?;
        }

        if self.fixes.is_empty() {
            return finish_batch(ctx, self.batch_id).await;
        }
        let fix_item = &self.fixes[0];

        let deleted_ctx = &ctx.clone_with_delete_visibility();
        // Get the workflow for the action we need to run.
        let component = Component::get_by_id(deleted_ctx, &fix_item.component_id)
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
        let resources = fix.run(ctx, run_id, workflow_prototype_id).await?;
        let completion_status: FixCompletionStatus = *fix
            .completion_status()
            .ok_or(FixError::EmptyCompletionStatus)?;

        // Upsert the relevant fix resolver.
        FixResolver::upsert(
            ctx,
            workflow_prototype_id,
            fix_item.attribute_value_id,
            Some(matches!(completion_status, FixCompletionStatus::Success)),
            *fix.id(),
        )
        .await?;

        let logs: Vec<_> = resources
            .iter()
            .flat_map(|r| &r.logs)
            .flat_map(|l| l.split('\n'))
            .map(|l| l.to_owned())
            .collect();

        // Inline dependent values propagation so we can run consecutive fixes that depend on the /root/resource from the previous fix
        todo!();
        // let attribute_value = Component::root_prop_child_attribute_value_for_component(
        //     ctx,
        //     *component.id(),
        //     RootPropChild::Resource,
        // )
        // .await?;

        // // Always retriggers confirmations, and propagates resource if it changed.
        // ctx.enqueue_blocking_job(DependentValuesUpdate::new(ctx, vec![*attribute_value.id()]))
        //     .await;

        WsEvent::fix_return(
            ctx,
            *fix.id(),
            self.batch_id,
            fix_item.attribute_value_id,
            fix_item.action.clone(),
            completion_status,
            logs,
        )
        .await?
        .publish_on_commit(ctx)
        .await?;

        if self.fixes.len() == 1 {
            finish_batch(ctx, self.batch_id).await?;
        } else {
            ctx.enqueue_job(FixesJob::new_iteration(
                ctx,
                self.fixes.iter().skip(1).cloned().collect(),
                self.batch_id,
            ))
            .await;
        }

        Ok(())
    }
}

impl TryFrom<JobInfo> for FixesJob {
    type Error = JobConsumerError;

    fn try_from(job: JobInfo) -> Result<Self, Self::Error> {
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

        Ok(Self {
            fixes: args.fixes,
            batch_id: args.batch_id,
            started: args.started,
            access_builder,
            visibility,
            job: Some(job),
        })
    }
}

async fn finish_batch(ctx: &DalContext, id: FixBatchId) -> JobConsumerResult<()> {
    // Mark the batch as completed.
    let mut batch = FixBatch::get_by_id(ctx, &id)
        .await?
        .ok_or(JobConsumerError::MissingFixBatch(id))?;
    let batch_completion_status = batch.stamp_finished(ctx).await?;
    WsEvent::fix_batch_return(ctx, *batch.id(), batch_completion_status)
        .await?
        .publish_on_commit(ctx)
        .await?;

    Component::run_all_confirmations(ctx).await?;
    Ok(())
}
