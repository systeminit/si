use std::{
    collections::{HashMap, HashSet},
    convert::TryFrom,
};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use council_server::ManagementResponse;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, collections::HashSet, convert::TryFrom};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::task::{JoinError, JoinSet};

//use crate::tasks::StatusReceiverClient;
//use crate::tasks::StatusReceiverRequest;
use crate::{
    attribute::value::{
        dependent_value_graph::DependentValueGraph, AttributeValueError, PrototypeExecutionResult,
    },
    job::consumer::{
        JobConsumer, JobConsumerError, JobConsumerMetadata, JobConsumerResult, JobInfo,
    },
    job::producer::{JobProducer, JobProducerResult},
    AccessBuilder, AttributeValue, AttributeValueId, DalContext,
    /*WsEvent*/
    TransactionsError, /*StatusUpdater,*/
    Visibility,
};

#[remain::sorted]
#[derive(Debug, Error)]
pub enum DependentValueUpdateError {
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    #[error(transparent)]
    TokioTask(#[from] JoinError),
    #[error(transparent)]
    Transactions(#[from] TransactionsError),
}

pub type DependentValueUpdateResult<T> = Result<T, DependentValueUpdateError>;

#[derive(Debug, Deserialize, Serialize)]
struct DependentValuesUpdateArgs {
    attribute_values: Vec<AttributeValueId>,
}

impl From<DependentValuesUpdate> for DependentValuesUpdateArgs {
    fn from(value: DependentValuesUpdate) -> Self {
        Self {
            attribute_values: value.attribute_values,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct DependentValuesUpdate {
    attribute_values: Vec<AttributeValueId>,
    access_builder: AccessBuilder,
    visibility: Visibility,
    job: Option<JobInfo>,
}

impl DependentValuesUpdate {
    pub fn new(
        access_builder: AccessBuilder,
        visibility: Visibility,
        attribute_values: Vec<AttributeValueId>,
    ) -> Box<Self> {
        Box::new(Self {
            attribute_values,
            access_builder,
            visibility,
            job: None,
        })
    }
}

impl JobProducer for DependentValuesUpdate {
    fn arg(&self) -> JobProducerResult<serde_json::Value> {
        Ok(serde_json::to_value(DependentValuesUpdateArgs::from(
            self.clone(),
        ))?)
    }
}

impl JobConsumerMetadata for DependentValuesUpdate {
    fn type_name(&self) -> String {
        "DependentValuesUpdate".to_string()
    }

    fn access_builder(&self) -> AccessBuilder {
        self.access_builder
    }

    fn visibility(&self) -> Visibility {
        self.visibility
    }
}

#[async_trait]
impl JobConsumer for DependentValuesUpdate {
    #[instrument(
        name = "dependent_values_update.run",
        skip_all,
        level = "info",
        fields(
            attribute_values = ?self.attribute_values,
        )
    )]
    async fn run(&self, ctx: &mut DalContext) -> JobConsumerResult<()> {
        Ok(self.inner_run(ctx).await?)
    }
}

impl DependentValuesUpdate {
    async fn inner_run(&self, ctx: &mut DalContext) -> DependentValueUpdateResult<()> {
        let start = tokio::time::Instant::now();

        let mut dependency_graph =
            DependentValueGraph::for_values(ctx, self.attribute_values.clone()).await?;

        debug!(
            "DependentValueGraph calculation took: {:?}",
            start.elapsed()
        );

        // Remove the first set of independent_values since they should already have had their functions executed
        for value in dependency_graph.independent_values() {
            dependency_graph.remove_value(value);
        }

        let mut seen_ids = HashSet::new();
        let mut task_id_to_av_id = HashMap::new();
        let mut update_join_set = JoinSet::new();

        let mut independent_value_ids = dependency_graph.independent_values();

        loop {
            if independent_value_ids.is_empty() && task_id_to_av_id.is_empty() {
                break;
            }

            for attribute_value_id in &independent_value_ids {
                let attribute_value_id = attribute_value_id.to_owned(); // release our borrow

                if !seen_ids.contains(&attribute_value_id) {
                    let join_handle = update_join_set.spawn(
                        values_from_prototype_function_execution(ctx.clone(), attribute_value_id),
                    );
                    task_id_to_av_id.insert(join_handle.id(), attribute_value_id);
                    seen_ids.insert(attribute_value_id);
                }
            }

            // Wait for a task to finish
            if let Some(join_result) = update_join_set.join_next_with_id().await {
                let (task_id, execution_result) = join_result?;
                if let Some(finished_value_id) = task_id_to_av_id.remove(&task_id) {
                    match execution_result {
                        Ok(execution_values) => {
                            match AttributeValue::set_values_from_execution_result(
                                ctx,
                                finished_value_id,
                                execution_values,
                            )
                            .await
                            {
                                // Remove the value, so that any values that dependent on it will
                                // become independent values (once all other dependencies are removed)
                                Ok(_) => dependency_graph.remove_value(finished_value_id),
                                Err(err) => {
                                    error!("error setting values from executed prototype function for AttributeValue {finished_value_id}: {err}");
                                    dependency_graph.cycle_on_self(finished_value_id);
                                }
                            }
                        }
                        Err(err) => {
                            // By adding an outgoing edge from the failed node to itself it will
                            // never appear in the `independent_values` call above since that looks for
                            // nodes *without* outgoing edges. Thus we will never try to re-execute
                            // the function for this value, nor will we execute anything in the
                            // dependency graph connected to this value

                            error!("error executing prototype function for AttributeValue {finished_value_id}: {err}");
                            dependency_graph.cycle_on_self(finished_value_id);
                        }
                    }
                }
            }

            independent_value_ids = dependency_graph.independent_values();
        }

        debug!("DependentValuesUpdate took: {:?}", start.elapsed());

        ctx.commit().await?;

        Ok(())
    }
}

/// Wrapper around `AttributeValue.values_from_prototype_function_execution(&ctx)` to get it to
/// play more nicely with being spawned into a `JoinSet`.
#[instrument(
    name = "dependent_values_update.values_from_prototype_function_execution",
    skip_all,
    level = "info",
    fields(
        attribute_value.id = %attribute_value_id,
    )
)]
async fn values_from_prototype_function_execution(
    ctx: DalContext,
    attribute_value_id: AttributeValueId,
) -> DependentValueUpdateResult<PrototypeExecutionResult> {
    Ok(AttributeValue::execute_prototype_function(&ctx, attribute_value_id).await?)
}

#[instrument(
    name = "dependent_values_update.update_summary_tables",
    skip_all,
    level = "info",
    fields(
        component.id = %component_id,
    )
)]
async fn update_summary_tables(
    ctx: &DalContext,
    component_value_json: &serde_json::Value,
    component_id: ComponentId,
) -> JobConsumerResult<()> {
    // Qualification summary table - if we add more summary tables, this should be extracted to its
    // own method.
    let mut total: i64 = 0;
    let mut warned: i64 = 0;
    let mut succeeded: i64 = 0;
    let mut failed: i64 = 0;
    let mut name: String = String::new();
    let mut color: String = String::new();
    let mut component_type: String = String::new();
    let mut has_resource: bool = false;
    let mut deleted_at: Option<String> = None;
    let mut deleted_at_datetime: Option<DateTime<Utc>> = None;
    if let Some(ref deleted_at) = deleted_at {
        let deleted_at_datetime_inner: DateTime<Utc> = deleted_at.parse()?;
        deleted_at_datetime = Some(deleted_at_datetime_inner);
    }

    if let Some(component_name) = component_value_json.pointer("/si/name") {
        if let Some(component_name_str) = component_name.as_str() {
            name = String::from(component_name_str);
        }
    }

    if let Some(component_color) = component_value_json.pointer("/si/color") {
        if let Some(component_color_str) = component_color.as_str() {
            color = String::from(component_color_str);
        }
    }

    if let Some(component_type_json) = component_value_json.pointer("/si/type") {
        if let Some(component_type_str) = component_type_json.as_str() {
            component_type = String::from(component_type_str);
        }
    }

    if let Some(_resource) = component_value_json.pointer("/resource/payload") {
        has_resource = true;
    }

    if let Some(deleted_at_value) = component_value_json.pointer("/deleted_at") {
        if let Some(deleted_at_str) = deleted_at_value.as_str() {
            deleted_at = Some(deleted_at_str.into());
        }
    }

    if let Some(qualification_map_value) = component_value_json.pointer("/qualification") {
        if let Some(qualification_map) = qualification_map_value.as_object() {
            for qual_result_map_value in qualification_map.values() {
                if let Some(qual_result_map) = qual_result_map_value.as_object() {
                    if let Some(qual_result) = qual_result_map.get("result") {
                        if let Some(qual_result_string) = qual_result.as_str() {
                            total += 1;
                            match qual_result_string {
                                "success" => succeeded += 1,
                                "warning" => warned += 1,
                                "failure" => failed += 1,
                                &_ => (),
                            }
                        }
                    }
                }
            }
        }
    }
    let _row = ctx
        .txns()
        .await?
        .pg()
        .query_one(
            "SELECT object FROM summary_qualification_update_v2($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            &[
                ctx.tenancy(),
                ctx.visibility(),
                &component_id,
                &name,
                &total,
                &warned,
                &succeeded,
                &failed,
                &deleted_at_datetime,
            ],
        )
        .await?;

    diagram::summary_diagram::component_update(
        ctx,
        &component_id,
        name,
        color,
        component_type,
        has_resource,
        deleted_at,
    )
    .await?;

    WsEvent::component_updated(ctx, component_id)
        .await?
        .publish_on_commit(ctx)
        .await?;

    Ok(())
}

impl TryFrom<JobInfo> for DependentValuesUpdate {
    type Error = JobConsumerError;

    fn try_from(job: JobInfo) -> Result<Self, Self::Error> {
        let args = DependentValuesUpdateArgs::deserialize(&job.arg)?;
        Ok(Self {
            attribute_values: args.attribute_values,
            access_builder: job.access_builder,
            visibility: job.visibility,
            job: Some(job),
        })
    }
}
