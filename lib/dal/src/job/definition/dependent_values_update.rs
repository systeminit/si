use std::{collections::HashMap, collections::HashSet, convert::TryFrom, sync::Arc};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use telemetry::prelude::*;
use tokio::{sync::Mutex, task::JoinSet};

use crate::{
    job::consumer::{JobConsumer, JobConsumerError, JobConsumerResult, JobInfo},
    job::producer::{JobMeta, JobProducer, JobProducerResult},
    AccessBuilder, AttributeValue, AttributeValueError, AttributeValueId, AttributeValueResult,
    Component, DalContext, StandardModel, StatusUpdater, Visibility, WsEvent,
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
    single_transaction: bool,
}

impl DependentValuesUpdate {
    pub fn new(ctx: &DalContext, attribute_values: Vec<AttributeValueId>) -> Box<Self> {
        let access_builder = AccessBuilder::from(ctx.clone());
        let visibility = *ctx.visibility();

        Box::new(Self {
            attribute_values,
            access_builder,
            visibility,
            job: None,
            single_transaction: false,
        })
    }

    async fn clone_ctx_with_new_transactions(
        &self,
        ctx: &DalContext,
    ) -> JobConsumerResult<DalContext> {
        if self.single_transaction {
            Ok(ctx.clone())
        } else {
            Ok(ctx.clone_with_new_transactions().await?)
        }
    }

    async fn commit_and_continue(&self, ctx: DalContext) -> JobConsumerResult<DalContext> {
        if self.single_transaction {
            Ok(ctx)
        } else {
            Ok(ctx.commit_and_continue().await?)
        }
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
        serde_json::to_string(self).expect("Cannot serialize DependentValueUpdate")
    }
}

#[async_trait]
impl JobConsumer for DependentValuesUpdate {
    fn type_name(&self) -> String {
        "DependentValuesUpdate".to_string()
    }

    /// This method is a hack to support SyncProcessor in DependentValusUpdate, since we commit transactions mid job, and write to multiple ones
    /// The sync processor needs everything to run within a single transaction, so we check for it
    fn set_sync(&mut self) {
        self.single_transaction = true;
        let boxed = Box::new(self.clone()) as Box<dyn JobProducer + Send + Sync>;
        self.job = Some(boxed.try_into().unwrap());
    }

    fn access_builder(&self) -> AccessBuilder {
        self.access_builder.clone()
    }

    fn visibility(&self) -> Visibility {
        self.visibility
    }

    async fn run(&self, ctx: &DalContext) -> JobConsumerResult<()> {
        let mut ctx = self.clone_ctx_with_new_transactions(ctx).await?;

        let now = std::time::Instant::now();

        let status_updater = Arc::new(Mutex::new(StatusUpdater::initialize(&ctx).await?));

        let jid = council::Id::from_string(&self.job.as_ref().unwrap().id)?;
        let mut council = council::Client::new(
            ctx.nats_conn().clone(),
            ctx.council_subject_prefix(),
            jid,
            self.visibility().change_set_pk.into(),
        )
        .await?;
        let pub_council = council.clone_into_pub();

        if let council::client::State::Shutdown = council.wait_to_create_values().await? {
            return Ok(());
        }

        AttributeValue::create_dependent_values(&ctx, &self.attribute_values).await?;

        ctx = self.commit_and_continue(ctx).await?;

        council.finished_creating_values().await?;

        let mut dependency_graph =
            AttributeValue::dependent_value_graph(&ctx, &self.attribute_values).await?;

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

        info!(
            "DependentValuesUpdate for {:?}: dependency_graph {:?}",
            self.attribute_values, &dependency_graph
        );

        if dependency_graph.is_empty() {
            return Ok(());
        }

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
        status_updater
            .lock()
            .await
            .values_queued(&ctx, enqueued)
            .await?;

        ctx = self.commit_and_continue(ctx).await?;

        let mut update_tasks = JoinSet::new();

        while !dependency_graph.is_empty() {
            match council.fetch_response().await? {
                Some(response) => match response {
                    council::Response::OkToProcess { node_ids } => {
                        for node_id in node_ids {
                            let id = AttributeValueId::from(node_id);
                            dependency_graph.remove(&id);

                            status_updater
                                .lock()
                                .await
                                .values_running(&ctx, vec![id])
                                .await?;
                            ctx = self.commit_and_continue(ctx).await?;

                            let task_ctx = self.clone_ctx_with_new_transactions(&ctx).await?;
                            let attribute_value = AttributeValue::get_by_id(&task_ctx, &id)
                                .await?
                                .ok_or_else(|| {
                                    AttributeValueError::NotFound(id, self.visibility())
                                })?;
                            update_tasks
                                .build_task()
                                .name("AttributeValue.update_from_prototype_function")
                                .spawn(update_value(
                                    task_ctx,
                                    attribute_value,
                                    self.single_transaction,
                                    pub_council.clone(),
                                    status_updater.clone(),
                                ))?;
                        }
                    }
                    council::Response::BeenProcessed { node_id } => {
                        let id = AttributeValueId::from(node_id);
                        dependency_graph.remove(&id);

                        // Send a completed status for this value and *remove* it from the hash
                        status_updater
                            .lock()
                            .await
                            .values_running(&ctx, vec![id])
                            .await?;
                        status_updater
                            .lock()
                            .await
                            .values_completed(&ctx, vec![id])
                            .await?;

                        ctx = self.commit_and_continue(ctx).await?;
                    }
                    council::Response::OkToCreate => unreachable!(),
                    council::Response::Shutdown => break,
                },
                // FIXME: reconnect
                None => break, // Happens if subscription has been unsubscribed or if connection is closed
            }

            WsEvent::change_set_written(&ctx)
                .await?
                .publish(&ctx)
                .await?;
            ctx = self.commit_and_continue(ctx).await?;

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
                        return Err(err.into());
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

        ctx = self.commit_and_continue(ctx).await?;

        Arc::try_unwrap(status_updater)
            .unwrap()
            .into_inner()
            .finish(&ctx)
            .await?;

        WsEvent::change_set_written(&ctx)
            .await?
            .publish(&ctx)
            .await?;

        // FIXME(nick,paulo): this is wrong. What if the confirmation attribute values are affected,
        // but the resource attribute values are NOT? We are shit outta luck. This only looks at the
        // resource attribute values.
        let resource_attribute_values: HashSet<AttributeValueId> = HashSet::from_iter(
            Component::list_all_resource_implicit_internal_provider_attribute_values(&ctx)
                .await?
                .iter()
                .map(|av| *av.id()),
        );
        for maybe_resource_attribute_value_id in &self.attribute_values {
            if resource_attribute_values.contains(maybe_resource_attribute_value_id) {
                WsEvent::confirmations_updated(&ctx)
                    .await?
                    .publish(&ctx)
                    .await?;
                break;
            }
        }

        if !self.single_transaction {
            ctx.commit().await?;
        }

        let elapsed_time = now.elapsed();
        info!(
            "DependentValuesUpdate for {:?} took {:?}",
            &self.attribute_values, elapsed_time
        );

        council.bye().await?;

        Ok(())
    }
}

/// Wrapper around `AttributeValue.update_from_prototype_function(&ctx)` to get it to
/// play more nicely with being spawned into a `JoinSet`.
async fn update_value(
    ctx: DalContext,
    mut attribute_value: AttributeValue,
    single_transaction: bool,
    council: council::PubClient,
    status_updater: Arc<Mutex<StatusUpdater>>,
) -> AttributeValueResult<()> {
    info!("DependentValueUpdate {:?}: START", attribute_value.id());
    let start = std::time::Instant::now();
    attribute_value.update_from_prototype_function(&ctx).await?;
    info!(
        "DependentValueUpdate {:?}: DONE {:?}",
        attribute_value.id(),
        start.elapsed()
    );

    // Send a completed status for this value and *remove* it from the hash
    status_updater
        .lock()
        .await
        .values_completed(&ctx, vec![*attribute_value.id()])
        .await
        .map_err(Box::new)?;

    WsEvent::change_set_written(&ctx)
        .await?
        .publish(&ctx)
        .await?;
    if !single_transaction {
        ctx.commit().await?;
    }

    council.processed_value(attribute_value.id().into()).await?;

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
            single_transaction: false,
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
