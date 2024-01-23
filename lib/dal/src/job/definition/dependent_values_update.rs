use std::{collections::HashMap, collections::HashSet, convert::TryFrom};

use async_trait::async_trait;

use serde::{Deserialize, Serialize};
use telemetry::prelude::*;
use tokio::task::JoinSet;

use crate::tasks::StatusReceiverClient;
use crate::tasks::StatusReceiverRequest;
use crate::{diagram, ComponentId};
use crate::{
    job::consumer::{
        JobConsumer, JobConsumerError, JobConsumerMetadata, JobConsumerResult, JobInfo,
    },
    job::producer::{JobProducer, JobProducerResult},
    AccessBuilder, AttributeValue, AttributeValueError, AttributeValueId, AttributeValueResult,
    DalContext, StandardModel, StatusUpdater, Visibility, WsEvent,
};
use crate::{FuncBindingReturnValue, InternalProvider};

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
        // TODO(nick,paulo,zack,jacob): ensure we do not _have_ to force non deleted visibility in the future.
        let visibility = visibility.to_non_deleted();

        Box::new(Self {
            attribute_values,
            access_builder,
            visibility,
            job: None,
        })
    }

    fn job_id(&self) -> Option<String> {
        self.job.as_ref().map(|j| j.id.clone())
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
        let council_subject =
            if let Some(subject_prefix) = ctx.nats_conn().metadata().subject_prefix() {
                format!("{subject_prefix}.council")
            } else {
                "council".to_string()
            };
        let jid = council_server::Id::from_string(&self.job_id().unwrap())?;
        let mut council = council_server::Client::new(
            ctx.nats_conn().clone(),
            &council_subject,
            jid,
            self.visibility().change_set_pk.into(),
        )
        .await?;
        let pub_council = council.clone_into_pub();

        let res = self.inner_run(ctx, &mut council, pub_council).await;

        council.bye().await?;

        res
    }
}

impl DependentValuesUpdate {
    async fn inner_run(
        &self,
        ctx: &mut DalContext,
        council: &mut council_server::Client,
        pub_council: council_server::PubClient,
    ) -> JobConsumerResult<()> {
        // TODO(nick,paulo,zack,jacob): ensure we do not _have_ to do this in the future.
        ctx.update_without_deleted_visibility();

        let ctx_builder = ctx.to_builder();
        let mut status_updater = StatusUpdater::initialize(ctx).await;

        let mut dependency_graph =
            AttributeValue::dependent_value_graph(ctx, &self.attribute_values).await?;

        // The dependent_value_graph is read-only, so we can safely rollback the inner txns, to
        // make sure we don't hold open txns unnecessarily
        ctx.rollback().await?;

        // NOTE(nick,jacob): uncomment this for debugging.
        // Save printed output to a file and execute the following: "dot <file> -Tsvg -o <newfile>.svg"
        // println!("{}", dependency_graph_to_dot(ctx, &dependency_graph).await?);

        // Remove the `AttributeValueIds` from the list of values that are in the dependencies,
        // as we consider that one to have already been updated. This lets us check for
        // `AttributeValuesId`s where the list of *unsatisfied* dependencies is empty.
        let attribute_values_set: HashSet<AttributeValueId> =
            HashSet::from_iter(self.attribute_values.iter().cloned());
        let mut to_remove = Vec::new();
        for (id, val) in dependency_graph.iter_mut() {
            val.retain(|id| !attribute_values_set.contains(id));
            if val.is_empty() {
                to_remove.push(*id);
            }
        }

        for id in to_remove {
            dependency_graph.remove(&id);
        }

        debug!(?dependency_graph, "Generated dependency graph");

        if dependency_graph.is_empty() {
            return Ok(());
        }

        // Cache the original dependency graph to send the status receiver.
        let original_dependency_graph = dependency_graph.clone();

        council
            .register_dependency_graph(
                dependency_graph
                    .iter()
                    .map(|(key, value)| (key.into(), value.iter().map(Into::into).collect()))
                    .collect(),
            )
            .await?;

        let mut enqueued: Vec<AttributeValueId> = dependency_graph.keys().copied().collect();
        enqueued.extend(dependency_graph.values().flatten().copied());
        status_updater.values_queued(ctx, enqueued).await;

        // Status updater reads from the database and uses its own connection from the pg_pool to
        // do writes
        ctx.rollback().await?;

        let mut update_tasks = JoinSet::new();

        while !dependency_graph.is_empty() {
            match council.fetch_response().await? {
                Some(response) => match response {
                    council_server::Response::OkToProcess { node_ids } => {
                        debug!(?node_ids, job_id = ?self.job_id(), "Ok to start processing nodes");
                        for node_id in node_ids {
                            let id = AttributeValueId::from(node_id);

                            status_updater.values_running(ctx, vec![id]).await;
                            // Status updater reads from the database and uses its own connection
                            // from the pg_pool to do writes
                            ctx.rollback().await?;

                            let task_ctx = ctx_builder
                                .build(self.access_builder().build(self.visibility()))
                                .await?;

                            let attribute_value = AttributeValue::get_by_id(&task_ctx, &id)
                                .await?
                                .ok_or_else(|| {
                                    AttributeValueError::NotFound(id, self.visibility())
                                })?;
                            update_tasks.spawn(update_value(
                                task_ctx,
                                attribute_value,
                                pub_council.clone(),
                                Span::current(),
                            ));
                        }
                    }
                    council_server::Response::BeenProcessed { node_id } => {
                        debug!(?node_id, job_id = ?self.job_id(), "Node has been processed by a job");
                        let id = AttributeValueId::from(node_id);
                        dependency_graph.remove(&id);

                        // Send a completed status for this value and *remove* it from the hash
                        status_updater.values_completed(ctx, vec![id]).await;

                        WsEvent::change_set_written(ctx)
                            .await?
                            .publish_on_commit(ctx)
                            .await?;

                        // Publish the WsEvent
                        ctx.commit().await?;
                    }
                    council_server::Response::Failed { node_id } => {
                        debug!(?node_id, job_id = ?self.job_id(), "Node failed on another job");
                        let id = AttributeValueId::from(node_id);
                        dependency_graph.remove(&id);

                        // Send a completed status for this value and *remove* it from the hash
                        status_updater.values_completed(ctx, vec![id]).await;
                        // Status updater reads from the database and uses its own connection from
                        // the pg_pool to do writes
                        ctx.rollback().await?;
                    }
                    council_server::Response::Shutdown => break,
                },
                // FIXME: reconnect
                None => break, // Happens if subscriber has been unsubscribed or if connection is closed
            }

            WsEvent::change_set_written(ctx)
                .await?
                .publish_on_commit(ctx)
                .await?;

            // Publish the WsEvent now!
            ctx.commit().await?;

            // If we get `None` back from the `JoinSet` that means that there are no
            // further tasks in the `JoinSet` for us to wait on. This should only happen
            // after we've stopped adding new tasks to the `JoinSet`, which means either:
            //   * We have completely walked the initial graph, and have visited every
            //     node.
            //   * We've encountered a cycle that means we can no longer make any
            //     progress on walking the graph.
            // In both cases, there isn't anything more we can do, so we can stop looking
            // at the graph to find more work.
            while let Some(future_result) = update_tasks.join_next().await {
                // We get back a `Some<Result<Result<..>>>`. We've already unwrapped the
                // `Some`, the outermost `Result` is a `JoinError` to let us know if
                // anything went wrong in joining the task.
                match future_result {
                    // We have successfully updated a value
                    Ok(Ok(())) => {}
                    // There was an error (with our code) when updating the value
                    Ok(Err(err)) => {
                        warn!(error = ?err, "error updating value");
                        return Err(err);
                    }
                    // There was a Tokio JoinSet error when joining the task back (i.e. likely
                    // I/O error)
                    Err(err) => {
                        warn!(error = ?err, "error when joining update task");
                        return Err(err.into());
                    }
                }
            }
        }

        status_updater.finish(ctx).await;

        WsEvent::change_set_written(ctx)
            .await?
            .publish_on_commit(ctx)
            .await?;

        let client = StatusReceiverClient::new(ctx.nats_conn().clone()).await;
        if let Err(e) = client
            .publish(&StatusReceiverRequest {
                visibility: *ctx.visibility(),
                tenancy: *ctx.tenancy(),
                dependent_graph: original_dependency_graph,
            })
            .await
        {
            error!("could not publish status receiver request: {:?}", e);
        }

        ctx.commit().await?;

        Ok(())
    }
}

/// Wrapper around `AttributeValue.update_from_prototype_function(&ctx)` to get it to
/// play more nicely with being spawned into a `JoinSet`.
#[instrument(
    name = "dependent_values_update.update_value",
    parent = &parent_span,
    skip_all,
    level = "info",
    fields(
        attribute_value.id = %attribute_value.id(),
    )
)]
async fn update_value(
    ctx: DalContext,
    mut attribute_value: AttributeValue,
    council: council_server::PubClient,
    parent_span: Span,
) -> JobConsumerResult<()> {
    let update_result = attribute_value.update_from_prototype_function(&ctx).await;
    // We don't propagate the error up, because we want the rest of the nodes in the graph to make progress
    // if they are able to.
    if update_result.is_err() {
        error!(?update_result, attribute_value_id = %attribute_value.id(), "Error updating AttributeValue");
        council
            .failed_processing_value(attribute_value.id().into())
            .await?;
        ctx.rollback().await?;
    }

    // If this is for an internal provider corresponding to a root prop for the schema variant of an existing component,
    // then we want to update summary tables.
    if !attribute_value.context.is_component_unset()
        && !attribute_value.context.is_internal_provider_unset()
        && InternalProvider::is_for_root_prop(&ctx, attribute_value.context.internal_provider_id())
            .await
            .unwrap()
    {
        if let Some(fbrv) =
            FuncBindingReturnValue::get_by_id(&ctx, &attribute_value.func_binding_return_value_id())
                .await?
        {
            if let Some(component_value_json) = fbrv.unprocessed_value() {
                update_summary_tables(
                    &ctx,
                    component_value_json,
                    attribute_value.context.component_id(),
                )
                .await?;
            }
        }
    }

    ctx.commit().await?;

    if update_result.is_ok() {
        council.processed_value(attribute_value.id().into()).await?;
    }

    Ok(())
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
            "SELECT object FROM summary_qualification_update_v1($1, $2, $3, $4, $5, $6, $7, $8)",
            &[
                ctx.tenancy(),
                ctx.visibility(),
                &component_id,
                &name,
                &total,
                &warned,
                &succeeded,
                &failed,
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

#[allow(unused)]
async fn dependency_graph_to_dot(
    ctx: &DalContext,
    graph: &HashMap<AttributeValueId, Vec<AttributeValueId>>,
) -> AttributeValueResult<String> {
    let mut node_definitions = String::new();
    for attr_val_id in graph.keys() {
        let attr_val = AttributeValue::get_by_id(ctx, attr_val_id)
            .await?
            .ok_or_else(|| AttributeValueError::NotFound(*attr_val_id, *ctx.visibility()))?;
        let prop_id = attr_val.context.prop_id();
        let internal_provider_id = attr_val.context.internal_provider_id();
        let external_provider_id = attr_val.context.external_provider_id();
        let component_id = attr_val.context.component_id();
        node_definitions.push_str(&format!(
            "\"{attr_val_id}\"[label=\"\\lAttribute Value: {attr_val_id}\\n\\lProp: {prop_id}\\lInternal Provider: {internal_provider_id}\\lExternal Provider: {external_provider_id}\\lComponent: {component_id}\"];",
        ));
    }

    let mut node_graph = String::new();
    for (attr_val, inputs) in graph {
        let dependencies = format!(
            "{{{dep_list}}}",
            dep_list = inputs
                .iter()
                .map(|i| format!("\"{i}\""))
                .collect::<Vec<String>>()
                .join(" ")
        );
        let dependency_line = format!("{dependencies} -> \"{attr_val}\";",);
        node_graph.push_str(&dependency_line);
    }

    let dot_digraph = format!("digraph G {{{node_definitions}{node_graph}}}");

    Ok(dot_digraph)
}
