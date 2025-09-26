use std::{
    collections::{
        HashMap,
        VecDeque,
    },
    sync::Arc,
};

use axum::response::Json;
use dal::{
    AttributeValue,
    Component,
    DalContext,
};
use futures::future::try_join_all;
use itertools::Itertools as _;
use sdf_extract::PosthogEventTracker;
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::json;
use si_frontend_mv_types::{
    index::change_set::ChangeSetMvIndexVersion,
    object::FrontendObject,
    reference::IndexReference,
};
use si_id::{
    AttributeValueId,
    ChangeSetId,
    ComponentId,
    WorkspacePk,
};
use telemetry::prelude::*;
use utoipa::{
    self,
    ToSchema,
};

use super::ComponentsResult;
use crate::{
    extract::{
        FriggStore,
        change_set::ChangeSetDalContext,
    },
    service::v1::ComponentsError,
};

#[utoipa::path(
    post,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/components/search",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
    ),
    tag = "components",
    request_body = SearchComponentsSpikeV1Request,
    summary = "Complex search for components",
    responses(
        (status = 200, description = "Components retrieved successfully", body = SearchComponentsSpikeV1Response),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 404, description = "Component not found"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn search_components_spike(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    FriggStore(frigg): FriggStore,
    tracker: PosthogEventTracker,
    request: Json<SearchComponentsSpikeV1Request>,
    // request: Result<Json<SearchComponentsSpikeV1Request>, axum::extract::rejection::JsonRejection>,
) -> ComponentsResult<Json<SearchComponentsSpikeV1Response>> {
    let Json(query) = request;
    let query = Arc::new(query);
    let workspace_pk = ctx.workspace_pk()?;
    let change_set_id = ctx.change_set_id();

    // Retrieve and search through AttributeTree MVs
    let component_ids = match query.search_method {
        Some(SearchMethod::Sync) => {
            match_attribute_trees_sync(&frigg, workspace_pk, change_set_id, &query).await?
        }
        Some(SearchMethod::Async) => {
            match_attribute_trees_async(&frigg, workspace_pk, change_set_id, &query).await?
        }
        Some(SearchMethod::TryJoinAll) => {
            match_attribute_trees_try_join_all(&frigg, workspace_pk, change_set_id, &query).await?
        }
        Some(SearchMethod::TryJoinAllSpawn) => {
            match_attribute_trees_try_join_all_spawn(&frigg, workspace_pk, change_set_id, &query)
                .await?
        }
        Some(SearchMethod::LessDeserializing) => {
            match_attribute_trees_less_deserializing(&frigg, workspace_pk, change_set_id, &query)
                .await?
        }
        Some(SearchMethod::LessDeserializingSpawn) => {
            match_attribute_trees_less_deserializing_spawn(
                &frigg,
                workspace_pk,
                change_set_id,
                &query,
            )
            .await?
        }
        Some(SearchMethod::Spawn) | None => {
            match_attribute_trees_spawn(&frigg, workspace_pk, change_set_id, &query).await?
        }
        Some(SearchMethod::Graph) => match_components_graph(ctx, &query).await?,
        Some(SearchMethod::GraphSpawn) => match_components_graph_spawn(ctx, &query).await?,
        Some(SearchMethod::GraphSpawn2) => match_components_graph_spawn2(ctx, &query).await?,
        Some(SearchMethod::Petgraph) => match_components_petgraph(ctx, &query).await?,
        Some(SearchMethod::PetgraphSpawn) => match_components_petgraph_spawn(ctx, &query).await?,
        Some(SearchMethod::PetgraphWithAsync) => {
            match_components_petgraph_with_async(ctx, &query).await?
        }
    };

    tracker.track(
        ctx,
        "api_search_components_spike",
        json!({
            "query": serde_json::to_value(query)?,
            "result_count": component_ids.len(),
        }),
    );

    Ok(Json(SearchComponentsSpikeV1Response { component_ids }))
}

async fn attribute_tree_mvs(
    frigg: &frigg::FriggStore,
    workspace_pk: WorkspacePk,
    change_set_id: ChangeSetId,
) -> ComponentsResult<impl Iterator<Item = IndexReference>> {
    // Grab the index
    // TODO don't convert to JSON and immediately convert to struct--convert straight to struct
    let Some((index, _)) = frigg
        .get_change_set_index(workspace_pk, change_set_id)
        .await?
    else {
        return Err(ComponentsError::ChangeSetIndexNotFound(
            workspace_pk,
            change_set_id,
        ));
    };
    let all_mvs = match serde_json::from_value(index.data)? {
        ChangeSetMvIndexVersion::V1(v1_index) => v1_index.mv_list,
        ChangeSetMvIndexVersion::V2(v2_index) => v2_index.mv_list,
    };
    Ok(all_mvs.into_iter().filter(|mv| mv.kind == "AttributeTree"))
}

#[instrument(level = "info", skip_all)]
async fn match_attribute_trees_async(
    frigg: &frigg::FriggStore,
    workspace_pk: WorkspacePk,
    change_set_id: ChangeSetId,
    query: &Arc<SearchComponentsSpikeV1Request>,
) -> ComponentsResult<Vec<ComponentId>> {
    // Get all the MVs at once
    let tasks = attribute_tree_mvs(frigg, workspace_pk, change_set_id)
        .await?
        .map(|mv| async move {
            match_attribute_tree(frigg.clone(), workspace_pk, mv, query.clone()).await
        })
        .collect_vec();

    // Collect the results
    let mut results = vec![];
    for task in tasks {
        if let Some(component_id) = task.await? {
            results.push(component_id);
        }
    }
    Ok(results)
}

#[instrument(level = "info", skip_all)]
async fn match_attribute_trees_spawn(
    frigg: &frigg::FriggStore,
    workspace_pk: WorkspacePk,
    change_set_id: ChangeSetId,
    query: &Arc<SearchComponentsSpikeV1Request>,
) -> ComponentsResult<Vec<ComponentId>> {
    // Get all the MVs at once
    let tasks = attribute_tree_mvs(frigg, workspace_pk, change_set_id)
        .await?
        .map(|mv| {
            tokio::spawn(match_attribute_tree(
                frigg.clone(),
                workspace_pk,
                mv,
                query.clone(),
            ))
        })
        .collect_vec();

    // Collect the results
    let mut results = vec![];
    for task in tasks {
        if let Some(component_id) = task.await?? {
            results.push(component_id);
        }
    }
    Ok(results)
}

#[instrument(level = "info", skip_all)]
async fn match_attribute_trees_try_join_all(
    frigg: &frigg::FriggStore,
    workspace_pk: WorkspacePk,
    change_set_id: ChangeSetId,
    query: &Arc<SearchComponentsSpikeV1Request>,
) -> ComponentsResult<Vec<ComponentId>> {
    // Get all the MVs at once
    let results = try_join_all(
        attribute_tree_mvs(frigg, workspace_pk, change_set_id)
            .await?
            .map(|mv| async move {
                match_attribute_tree(frigg.clone(), workspace_pk, mv, query.clone()).await
            }),
    )
    .await?;

    // Collect the results
    Ok(results.into_iter().flatten().collect())
}

#[instrument(level = "info", skip_all)]
async fn match_attribute_trees_try_join_all_spawn(
    frigg: &frigg::FriggStore,
    workspace_pk: WorkspacePk,
    change_set_id: ChangeSetId,
    query: &Arc<SearchComponentsSpikeV1Request>,
) -> ComponentsResult<Vec<ComponentId>> {
    // Get all the MVs at once
    let results = try_join_all(
        attribute_tree_mvs(frigg, workspace_pk, change_set_id)
            .await?
            .map(|mv| {
                tokio::spawn(match_attribute_tree(
                    frigg.clone(),
                    workspace_pk,
                    mv,
                    query.clone(),
                ))
            }),
    )
    .await?;

    // Collect the results
    results.into_iter().flatten_ok().try_collect()
}

#[instrument(level = "info", skip_all)]
async fn match_attribute_trees_less_deserializing(
    frigg: &frigg::FriggStore,
    workspace_pk: WorkspacePk,
    change_set_id: ChangeSetId,
    query: &Arc<SearchComponentsSpikeV1Request>,
) -> ComponentsResult<Vec<ComponentId>> {
    // Collect the results
    let mut results = vec![];
    for mv in attribute_tree_mvs(frigg, workspace_pk, change_set_id).await? {
        if let Some(component_id) =
            match_attribute_tree_less_deserializing(frigg.clone(), workspace_pk, mv, query.clone())
                .await?
        {
            results.push(component_id);
        }
    }
    Ok(results)
}

#[instrument(level = "info", skip_all)]
async fn match_attribute_trees_less_deserializing_spawn(
    frigg: &frigg::FriggStore,
    workspace_pk: WorkspacePk,
    change_set_id: ChangeSetId,
    query: &Arc<SearchComponentsSpikeV1Request>,
) -> ComponentsResult<Vec<ComponentId>> {
    // Get all the MVs at once
    let results = try_join_all(
        attribute_tree_mvs(frigg, workspace_pk, change_set_id)
            .await?
            .map(|mv| {
                tokio::spawn(match_attribute_tree_less_deserializing(
                    frigg.clone(),
                    workspace_pk,
                    mv,
                    query.clone(),
                ))
            }),
    )
    .await?;

    // Collect the results
    results.into_iter().flatten_ok().try_collect()
}

#[instrument(level = "info", skip_all)]
async fn match_attribute_trees_sync(
    frigg: &frigg::FriggStore,
    workspace_pk: WorkspacePk,
    change_set_id: ChangeSetId,
    query: &Arc<SearchComponentsSpikeV1Request>,
) -> ComponentsResult<Vec<ComponentId>> {
    // Collect the results
    let mut results = vec![];
    for mv in attribute_tree_mvs(frigg, workspace_pk, change_set_id).await? {
        if let Some(component_id) =
            match_attribute_tree(frigg.clone(), workspace_pk, mv, query.clone()).await?
        {
            results.push(component_id);
        }
    }
    Ok(results)
}

#[instrument(level = "debug", skip_all, fields(id))]
async fn match_attribute_tree(
    frigg: frigg::FriggStore,
    workspace_pk: WorkspacePk,
    IndexReference { kind, id, checksum }: IndexReference,
    query: Arc<SearchComponentsSpikeV1Request>,
) -> ComponentsResult<Option<ComponentId>> {
    // TODO don't convert to JSON and immediately convert to struct--convert straight to struct
    let Some(FrontendObject { data, .. }) = frigg
        .get_workspace_object(workspace_pk, &kind, &id, &checksum)
        .await?
    else {
        return Err(ComponentsError::MvNotFound(kind, id, checksum));
    };
    let attribute_tree: AttributeTreeForSearch = serde_json::from_value(data)?;
    for av in attribute_tree.attribute_values.values() {
        if av.path.ends_with(&query.attr_name) {
            if av.value == query.attr_value {
                return Ok(Some(attribute_tree.id));
            }
        }
    }

    Ok(None)
}

#[instrument(level = "debug", skip_all, fields(id))]
async fn match_attribute_tree_less_deserializing(
    frigg: frigg::FriggStore,
    workspace_pk: WorkspacePk,
    IndexReference { kind, id, checksum }: IndexReference,
    query: Arc<SearchComponentsSpikeV1Request>,
) -> ComponentsResult<Option<ComponentId>> {
    // TODO don't convert to JSON and immediately convert to struct--convert straight to struct
    let Some(attribute_tree) = frigg
        .get_workspace_object_data::<AttributeTreeForSearch>(workspace_pk, &kind, &id, &checksum)
        .await?
    else {
        return Err(ComponentsError::MvNotFound(kind, id, checksum));
    };
    for av in attribute_tree.attribute_values.values() {
        if av.path.ends_with(&query.attr_name) {
            if av.value == query.attr_value {
                return Ok(Some(attribute_tree.id));
            }
        }
    }

    Ok(None)
}

async fn match_components_graph(
    ctx: &DalContext,
    query: &Arc<SearchComponentsSpikeV1Request>,
) -> ComponentsResult<Vec<ComponentId>> {
    let ctx = Arc::new(ctx.clone());
    let mut results = vec![];
    for component_id in Component::list_ids(&ctx).await? {
        if let Some(component_id) =
            match_component_graph(ctx.clone(), component_id, query.clone()).await?
        {
            results.push(component_id);
        }
    }
    Ok(results)
}

async fn match_components_graph_spawn(
    ctx: &DalContext,
    query: &Arc<SearchComponentsSpikeV1Request>,
) -> ComponentsResult<Vec<ComponentId>> {
    info!("match_components_graph_spawn start");
    let ctx = Arc::new(ctx.clone());
    let tasks = Component::list_ids(&ctx)
        .await?
        .into_iter()
        .map(|component_id| {
            tokio::spawn(match_component_graph(
                ctx.clone(),
                component_id,
                query.clone(),
            ))
        })
        .collect_vec();
    info!("match_components_graph_spawn spawned: {}", tasks.len());

    // Collect the results
    let mut results = vec![];
    for task in tasks {
        if let Some(component_id) = task.await?? {
            results.push(component_id);
        }
    }
    info!("match_components_graph_spawn finished");
    Ok(results)
}

async fn match_components_graph_spawn2(
    ctx: &DalContext,
    query: &Arc<SearchComponentsSpikeV1Request>,
) -> ComponentsResult<Vec<ComponentId>> {
    info!("match_components_graph_spawn2 start");
    let ctx = Arc::new(ctx.clone());
    let tasks = Component::list_ids(&ctx)
        .await?
        .into_iter()
        .map(|component_id| {
            info!("spawning for component_id: {}", component_id);
            let ctx = ctx.clone();
            let query = query.clone();
            tokio::spawn(async move { match_component_graph_spawn(ctx, component_id, query).await })
        });
    info!("match_components_graph_spawn2 spawned: {}", tasks.len());

    // Collect the results
    let mut results = vec![];
    for task in tasks {
        if let Some(component_id) = task.await?? {
            results.push(component_id);
        }
    }
    info!("match_components_graph_spawn2 finished");
    Ok(results)
}

async fn match_components_petgraph(
    ctx: &DalContext,
    query: &Arc<SearchComponentsSpikeV1Request>,
) -> ComponentsResult<Vec<ComponentId>> {
    let ctx = Arc::new(ctx.clone());
    let mut results = vec![];
    for component_id in Component::list_ids(&ctx).await? {
        if let Some(component_id) =
            match_component_petgraph(ctx.clone(), component_id, query.clone()).await?
        {
            results.push(component_id);
        }
    }
    Ok(results)
}

async fn match_components_petgraph_spawn(
    ctx: &DalContext,
    query: &Arc<SearchComponentsSpikeV1Request>,
) -> ComponentsResult<Vec<ComponentId>> {
    info!("match_components_petgraph_spawn start");
    let ctx = Arc::new(ctx.clone());
    let tasks = Component::list_ids(&ctx)
        .await?
        .into_iter()
        .map(|component_id| {
            tokio::spawn(match_component_petgraph(
                ctx.clone(),
                component_id,
                query.clone(),
            ))
        })
        .collect_vec();
    info!("match_components_petgraph_spawn spawned: {}", tasks.len());

    // Collect the results
    let mut results = vec![];
    for task in tasks {
        if let Some(component_id) = task.await?? {
            results.push(component_id);
        }
    }
    info!("match_components_petgraph_spawn finished");
    Ok(results)
}

async fn match_components_petgraph_with_async(
    ctx: &DalContext,
    query: &Arc<SearchComponentsSpikeV1Request>,
) -> ComponentsResult<Vec<ComponentId>> {
    let ctx = Arc::new(ctx.clone());
    let mut results = vec![];
    for component_id in Component::list_ids(&ctx).await? {
        if let Some(component_id) =
            match_component_petgraph_with_async(ctx.clone(), component_id, query.clone()).await?
        {
            results.push(component_id);
        }
    }
    Ok(results)
}

// 1. Try petgraph dfs
// 2. Try going through props first
async fn match_component_graph(
    ctx: Arc<DalContext>,
    component_id: ComponentId,
    query: Arc<SearchComponentsSpikeV1Request>,
) -> ComponentsResult<Option<ComponentId>> {
    info!("single match start {}", component_id);
    let root_attribute_value_id = Component::root_attribute_value_id(&ctx, component_id).await?;
    let mut work_queue = VecDeque::from([root_attribute_value_id]);
    while let Some(av_id) = work_queue.pop_front() {
        let key = AttributeValue::prop_name(&ctx, av_id).await?;
        // let key = match AttributeValue::get_key_or_index_of_child_entry(&ctx, av_id).await? {
        //     None => AttributeValue::prop_name(&ctx, av_id).await?,
        //     Some(KeyOrIndex::Index(index)) => index.to_string(),
        //     Some(KeyOrIndex::Key(key)) => key,
        // };
        if key == query.attr_name {
            // We found one! Now check the value
            let av = AttributeValue::get_by_id(&ctx, av_id).await?;
            if let Some(serde_json::Value::String(value)) = av.value(&ctx).await? {
                if value == query.attr_value {
                    return Ok(Some(component_id));
                }
            }
        }
        work_queue.extend(AttributeValue::child_av_ids(&ctx, av_id).await?);
    }
    info!("single match complete {}", component_id);
    Ok(None)
}

async fn match_component_graph_spawn(
    ctx: Arc<DalContext>,
    component_id: ComponentId,
    query: Arc<SearchComponentsSpikeV1Request>,
) -> ComponentsResult<Option<ComponentId>> {
    info!("single start {}", component_id);
    let root_attribute_value_id = Component::root_attribute_value_id(&ctx, component_id).await?;
    let mut attr_values = vec![];
    let mut work_queue = VecDeque::from([root_attribute_value_id]);
    while let Some(av_id) = work_queue.pop_front() {
        let key = AttributeValue::prop_name(&ctx, av_id).await?;
        // let key = match AttributeValue::get_key_or_index_of_child_entry(&ctx, av_id).await? {
        //     None => AttributeValue::prop_name(&ctx, av_id).await?,
        //     Some(KeyOrIndex::Index(index)) => index.to_string(),
        //     Some(KeyOrIndex::Key(key)) => key,
        // };
        if key == query.attr_name {
            // We found one! Now check the value
            let av = AttributeValue::get_by_id(&ctx, av_id).await?;
            let ctx = ctx.clone();
            attr_values.push(tokio::spawn(async move { av.value(&ctx).await }));
        }
        work_queue.extend(AttributeValue::child_av_ids(&ctx, av_id).await?);
    }

    for attr_value in attr_values {
        if let Some(serde_json::Value::String(value)) = attr_value.await?? {
            if value == query.attr_value {
                info!("single complete");
                return Ok(Some(component_id));
            }
        }
    }
    info!("single complete {}", component_id);
    Ok(None)
}

// 1. Try petgraph dfs
// 2. Try going through props first
async fn match_component_petgraph(
    ctx: Arc<DalContext>,
    component_id: ComponentId,
    query: Arc<SearchComponentsSpikeV1Request>,
) -> ComponentsResult<Option<ComponentId>> {
    for av in ctx
        .workspace_snapshot()?
        .matching_avs(component_id, &query.attr_name)
        .await?
    {
        info!("Matching AV in {}", component_id);
        let value = AttributeValue::fetch_value_from_store(&ctx, av.value).await?;
        info!("Got content {}", component_id);
        if let Some(serde_json::Value::String(value)) = value {
            if value == query.attr_value {
                return Ok(Some(component_id));
            }
        }
    }
    Ok(None)
}

async fn match_component_petgraph_with_async(
    ctx: Arc<DalContext>,
    component_id: ComponentId,
    query: Arc<SearchComponentsSpikeV1Request>,
) -> ComponentsResult<Option<ComponentId>> {
    for av in ctx
        .workspace_snapshot()?
        .matching_avs_with_async(component_id, &query.attr_name)
        .await?
    {
        info!("Matching AV in {}", component_id);
        let value = AttributeValue::fetch_value_from_store(&ctx, av.value).await?;
        info!("Got content {}", component_id);
        if let Some(serde_json::Value::String(value)) = value {
            if value == query.attr_value {
                return Ok(Some(component_id));
            }
        }
    }
    Ok(None)
}

/// A pared-down version of the AttributeTree MV, containing only the fields we need for searching.
/// That way we don't pay the cost of deserializing everything else.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AttributeTreeForSearch {
    pub id: ComponentId,
    pub attribute_values: HashMap<AttributeValueId, AttributeValueForSearch>,
    pub component_name: String,
    pub schema_name: String,
}

/// A pared-down version of the AttributeTree MV, containing only the fields we need for searching.
/// That way we don't pay the cost of deserializing everything else.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AttributeValueForSearch {
    // pub id: AttributeValueId,
    // pub key: Option<String>,
    pub path: String,
    // pub prop_id: Option<PropId>,
    pub value: serde_json::Value,
    // pub external_sources: Option<Vec<ExternalSource>>, // this is the detail of where the subscriptions are from
    // pub is_controlled_by_ancestor: bool, // if ancestor of prop is set by dynamic func, ID of ancestor that sets it
    // pub is_controlled_by_dynamic_func: bool, // props driven by non-dynamic funcs have a statically set value
    // pub overridden: bool, // true if this prop has a different controlling func id than the default for this asset
    // pub validation: Option<ValidationOutput>,
    // pub secret: Option<Secret>,
    // TODO remove from here and frontend. Always false right now.
    // pub has_socket_connection: bool,
    // pub is_default_source: bool,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum SearchMethod {
    Sync,
    Async,
    Spawn,
    TryJoinAllSpawn,
    TryJoinAll,
    LessDeserializing,
    LessDeserializingSpawn,
    Petgraph,
    PetgraphSpawn,
    PetgraphWithAsync,
    Graph,
    GraphSpawn,
    GraphSpawn2,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SearchComponentsSpikeV1Request {
    #[schema(example = "Region")]
    attr_name: String,
    #[schema(example = "us-east-1")]
    attr_value: String,
    #[schema(example = "spawn")]
    search_method: Option<SearchMethod>,
}

#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SearchComponentsSpikeV1Response {
    #[schema(value_type = Vec<String>, example = json!(["01H9ZQD35JPMBGHH69BT0Q79AA", "01H9ZQD35JPMBGHH69BT0Q79BB", "01H9ZQD35JPMBGHH69BT0Q79CC"]))]
    component_ids: Vec<ComponentId>,
}
