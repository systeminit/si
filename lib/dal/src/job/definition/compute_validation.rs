use async_trait::async_trait;
use pinga_core::api_types::job_execution_request::JobArgsVCurrent;
use serde::{
    Deserialize,
    Serialize,
};
use si_id::{
    ChangeSetId,
    WorkspacePk,
};
use telemetry::prelude::*;
use tokio::task::JoinSet;

use crate::{
    AttributeValueId,
    ChangeSet,
    ChangeSetStatus,
    DalContext,
    job::consumer::{
        DalJob,
        JobCompletionState,
        JobConsumer,
        JobConsumerResult,
    },
    validation::{
        ValidationOutput,
        ValidationOutputNode,
    },
};

const VALIDATION_CONCURRENCY_LIMIT: usize = 20;

#[derive(Debug, Deserialize, Serialize)]
struct ComputeValidationArgs {
    attribute_values: Vec<AttributeValueId>,
}

impl From<ComputeValidation> for ComputeValidationArgs {
    fn from(value: ComputeValidation) -> Self {
        Self {
            attribute_values: value.attribute_value_ids,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ComputeValidation {
    workspace_id: WorkspacePk,
    change_set_id: ChangeSetId,
    attribute_value_ids: Vec<AttributeValueId>,
}

impl ComputeValidation {
    pub fn new(
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
        attribute_value_ids: Vec<AttributeValueId>,
    ) -> Box<Self> {
        Box::new(Self {
            workspace_id,
            change_set_id,
            attribute_value_ids,
        })
    }
}

impl DalJob for ComputeValidation {
    fn args(&self) -> JobArgsVCurrent {
        JobArgsVCurrent::Validation {
            attribute_value_ids: self.attribute_value_ids.clone(),
        }
    }

    fn workspace_id(&self) -> WorkspacePk {
        self.workspace_id
    }

    fn change_set_id(&self) -> ChangeSetId {
        self.change_set_id
    }
}

impl ComputeValidation {
    async fn perform_validation(
        ctx: DalContext,
        attribute_value_id: AttributeValueId,
        parent_span: Span,
    ) -> JobConsumerResult<()> {
        let maybe_validation =
            ValidationOutput::compute_for_attribute_value(&ctx, attribute_value_id, parent_span)
                .await?;

        ValidationOutputNode::upsert_or_wipe_for_attribute_value(
            &ctx,
            attribute_value_id,
            maybe_validation.clone(),
        )
        .await?;

        Ok(())
    }
}

#[async_trait]
impl JobConsumer for ComputeValidation {
    #[instrument(
        name = "compute_validation.run",
        skip_all,
        level = "info",
        fields(
            attribute_values = ?self.attribute_value_ids,
        )
    )]
    async fn run(&self, ctx: &mut DalContext) -> JobConsumerResult<JobCompletionState> {
        let span = current_span_for_instrument_at!("info");

        let change_set = ChangeSet::get_by_id(ctx, ctx.change_set_id()).await?;

        if change_set.status == ChangeSetStatus::Abandoned {
            info!("Validation enqueued for abandoned change set. Returning early");
            return Ok(JobCompletionState::Done);
        }

        let mut attribute_value_ids = {
            let workspace_snapshot = ctx.workspace_snapshot()?;

            let mut ids = Vec::with_capacity(self.attribute_value_ids.len());
            for av_id in &self.attribute_value_ids {
                // It's possible that one or more of the initial AttributeValueIds provided by the
                // enqueued ComputeValidation may have been removed from the snapshot between when
                // the CV job was created and when we're processing things now. This could happen
                // if there are other modifications to the snapshot before the CV job starts
                // executing, as the job always operates on the current state of the change set's
                // snapshot, not the state at the time the job was created.
                if !workspace_snapshot.node_exists(av_id).await {
                    debug!("Attribute Value {av_id} missing, skipping it in ComputeValidations");
                    continue;
                }

                ids.push(*av_id);
            }
            ids
        }
        .into_iter();

        let mut tasks = JoinSet::new();

        loop {
            if tasks.len() <= VALIDATION_CONCURRENCY_LIMIT {
                if let Some(av_id) = attribute_value_ids.next() {
                    tasks.spawn(Self::perform_validation(ctx.clone(), av_id, span.clone()));
                }
            };

            match tasks.join_next().await {
                Some(Ok(Ok(()))) => {}
                // Error from task, early return err
                Some(Ok(Err(job_consumer_err))) => return Err(job_consumer_err),
                // Join error from JoinSet, early return err
                Some(Err(join_err)) => return Err(join_err.into()),
                // JoinSet is empty, all work is complete
                None => break,
            }
        }

        ctx.commit().await?;
        Ok(JobCompletionState::Done)
    }
}
