use std::{collections::HashMap, collections::HashSet, convert::TryFrom};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use telemetry::prelude::*;
use tokio::task::JoinSet;

use crate::tasks::StatusReceiverClient;
use crate::tasks::StatusReceiverRequest;
use crate::{
    job::consumer::{
        JobConsumer, JobConsumerError, JobConsumerMetadata, JobConsumerResult, JobInfo,
    },
    job::producer::{JobMeta, JobProducer, JobProducerResult},
    AccessBuilder, AttributeValue, AttributeValueError, AttributeValueId, AttributeValueResult,
    DalContext, StandardModel, StatusUpdater, Visibility, WsEvent,
};

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
    fn args(&self) -> JobProducerResult<serde_json::Value> {
        Ok(serde_json::to_value(DependentValuesUpdateArgs::from(
            self.clone(),
        ))?)
    }

    fn meta(&self) -> JobProducerResult<JobMeta> {
        let mut custom = HashMap::new();
        custom.insert(
            "access_builder".to_string(),
            serde_json::to_value(self.access_builder)?,
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
        serde_json::to_string(self).expect("Cannot serialize DependentValueUpdate")
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
        let jid = council_server::Id::from_string(&self.job.as_ref().unwrap().id)?;
        let mut council = council_server::Client::new(
            ctx.nats_conn().clone(),
            &council_subject,
            jid,
            self.visibility().change_set_pk.into(),
        )
        .await?;
        let pub_council = council.clone_into_pub();

        match self.inner_run(ctx, &mut council, pub_council).await {
            Ok(res) => Ok(res),
            Err(e) => {
                council.bye().await?;
                Err(e)
            }
        }
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

        let ctx_builder = ctx.services_context().into_builder();
        let mut status_updater = StatusUpdater::initialize(ctx).await;

        // Avoid lingering transaction while we wait to create attribute values
        // Status updater reads from the database and uses its own connection from the pg_pool to
        // do writes
        ctx.rollback().await?;

        debug!(job_id = ?self.job_id(), "Waiting to create AttributeValues");
        if let council_server::client::State::Shutdown = council.wait_to_create_values().await? {
            return Ok(());
        }

        AttributeValue::create_dependent_values(ctx, &self.attribute_values)
            .instrument(debug_span!("Creating dependent attribute values", job_id = ?self.job_id()))
            .await?;

        // Creating dependent values creates records in the database that need to be viewed by
        // other connections/txns so we commit
        ctx.commit().await?;

        debug!(job_id = ?self.job_id(), "Transaction committed");

        council.finished_creating_values().await?;
        debug!(job_id = ?self.job_id(), "Finished creating values");

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
                            dependency_graph.remove(&id);

                            status_updater.values_running(ctx, vec![id]).await;
                            // Status updater reads from the database and uses its own connection
                            // from the pg_pool to do writes
                            ctx.rollback().await?;

                            let task_ctx = ctx_builder.build(self.access_builder().build(self.visibility())).await?;

                            let attribute_value = AttributeValue::get_by_id(&task_ctx, &id)
                                .await?
                                .ok_or_else(|| {
                                    AttributeValueError::NotFound(id, self.visibility())
                                })?;
                            update_tasks.spawn(update_value(
                                task_ctx,
                                attribute_value,
                                pub_council.clone(),
                                status_updater.clone(),
                            ));
                        }
                    }
                    council_server::Response::BeenProcessed { node_id } => {
                        debug!(?node_id, job_id = ?self.job_id(), "Node has been processed by another job");
                        let id = AttributeValueId::from(node_id);
                        dependency_graph.remove(&id);

                        // Send a completed status for this value and *remove* it from the hash
                        status_updater.values_running(ctx, vec![id]).await;
                        status_updater.values_completed(ctx, vec![id]).await;
                        // Status updater reads from the database and uses its own connection from
                        // the pg_pool to do writes
                        ctx.rollback().await?;
                    }
                    council_server::Response::Failed { node_id } => {
                        debug!(?node_id, job_id = ?self.job_id(), "Node failed on another job");
                        let id = AttributeValueId::from(node_id);
                        dependency_graph.remove(&id);

                        // Send a completed status for this value and *remove* it from the hash
                        status_updater.values_running(ctx, vec![id]).await;
                        status_updater.values_completed(ctx, vec![id]).await;
                        // Status updater reads from the database and uses its own connection from
                        // the pg_pool to do writes
                        ctx.rollback().await?;
                    }
                    // If we receive an OkToCreate here, it's because council is telling us that it's Ok to run
                    // `AttributeValue::create_dependent_values` after it has already told us to do that, and after
                    // we have told it that we've finished doing so. This should never be able to happen normally,
                    // as it breaks the protocol contract we have with council.
                    council_server::Response::OkToCreate => return Err(JobConsumerError::CouncilProtocol("Told to create values again after we've finished creating values. Multiple instances of council running?".to_string())),
                    council_server::Response::Shutdown => break,
                },
                // FIXME: reconnect
                None => break, // Happens if subscription has been unsubscribed or if connection is closed
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

                        council.bye().await?;
                        return Err(err);
                    }
                    // There was a Tokio JoinSet error when joining the task back (i.e. likely
                    // I/O error)
                    Err(err) => {
                        warn!(error = ?err, "error when joining update task");

                        council.bye().await?;
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

        council.bye().await?;

        Ok(())
    }
}

/// Wrapper around `AttributeValue.update_from_prototype_function(&ctx)` to get it to
/// play more nicely with being spawned into a `JoinSet`.
#[instrument(
    name = "dependent_values_update.update_value",
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
    mut status_updater: StatusUpdater,
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

    // Send a completed status for this value and *remove* it from the hash
    status_updater
        .values_completed(&ctx, vec![*attribute_value.id()])
        .await;

    WsEvent::change_set_written(&ctx)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    // Commit the updated attr value & publish the WsEvent
    ctx.commit().await?;

    if update_result.is_ok() {
        council.processed_value(attribute_value.id().into()).await?;
    }

    Ok(())
}

impl TryFrom<JobInfo> for DependentValuesUpdate {
    type Error = JobConsumerError;

    fn try_from(job: JobInfo) -> Result<Self, Self::Error> {
        if job.args().len() != 3 {
            return Err(JobConsumerError::InvalidArguments(
                r#"[{ "attribute_values": <Vec<AttributeValueId>> }, <AccessBuilder>, <Visibility>]"#
                    .to_string(),
                job.args().to_vec(),
            ));
        }
        let args: DependentValuesUpdateArgs = serde_json::from_value(job.args()[0].clone())?;
        let access_builder: AccessBuilder = serde_json::from_value(job.args()[1].clone())?;
        let visibility: Visibility = serde_json::from_value(job.args()[2].clone())?;

        Ok(Self {
            attribute_values: args.attribute_values,
            access_builder,
            visibility,
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
