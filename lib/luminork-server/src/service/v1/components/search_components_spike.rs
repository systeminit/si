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
    EdgeWeightKind,
    EdgeWeightKindDiscriminants,
    PropId,
    workspace_snapshot::{
        WorkspaceSnapshotSelector,
        content_address::ContentAddress,
        graph::WorkspaceSnapshotGraphResult,
        node_weight::{
            AttributeValueNodeWeight,
            PropNodeWeight,
        },
    },
};
use futures::future::try_join_all;
use itertools::Itertools as _;
use petgraph::prelude::*;
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
        Some(SearchMethod::Petgraph) => {
            let graph = ctx.workspace_snapshot()?.graph().clone();
            match_components_petgraph(ctx, &query, PetgraphSyncImpl(graph)).await?
        }
        Some(SearchMethod::PetgraphId) => {
            let graph = ctx.workspace_snapshot()?.graph().clone();
            match_components_petgraph(ctx, &query, PetgraphSyncIdImpl(graph)).await?
        }
        Some(SearchMethod::PetgraphRef) => {
            let graph = ctx.workspace_snapshot()?.graph().clone();
            match_components_petgraph(ctx, &query, PetgraphSyncImpl(graph)).await?
        }
        Some(SearchMethod::PetgraphRefId) => {
            let graph = ctx.workspace_snapshot()?.graph().clone();
            match_components_petgraph(ctx, &query, PetgraphSyncIdImpl(graph)).await?
        }
        Some(SearchMethod::PetgraphAsync) => {
            let graph = ctx.workspace_snapshot()?.graph().clone();
            match_components_petgraph(ctx, &query, PetgraphAsyncImpl(graph)).await?
        }
        Some(SearchMethod::PetgraphAsyncId) => {
            let graph = ctx.workspace_snapshot()?.graph().clone();
            match_components_petgraph(ctx, &query, PetgraphAsyncIdImpl(graph)).await?
        }
        Some(SearchMethod::PetgraphCtx) => {
            let arc_ctx = Arc::new(ctx.clone());
            match_components_petgraph(ctx, &query, PetgraphSyncImpl(arc_ctx)).await?
        }
        Some(SearchMethod::PetgraphIdCtx) => {
            let arc_ctx = Arc::new(ctx.clone());
            match_components_petgraph(&ctx, &query, PetgraphSyncIdImpl(arc_ctx)).await?
        }
        Some(SearchMethod::PetgraphRefCtx) => {
            let arc_ctx = Arc::new(ctx.clone());
            match_components_petgraph(&ctx, &query, PetgraphSyncImpl(arc_ctx)).await?
        }
        Some(SearchMethod::PetgraphRefIdCtx) => {
            let arc_ctx = Arc::new(ctx.clone());
            match_components_petgraph(&ctx, &query, PetgraphSyncIdImpl(arc_ctx)).await?
        }
        Some(SearchMethod::PetgraphAsyncCtx) => {
            let arc_ctx = Arc::new(ctx.clone());
            match_components_petgraph(&ctx, &query, PetgraphAsyncImpl(arc_ctx)).await?
        }
        Some(SearchMethod::PetgraphAsyncIdCtx) => {
            let arc_ctx = Arc::new(ctx.clone());
            match_components_petgraph(&ctx, &query, PetgraphAsyncIdImpl(arc_ctx)).await?
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

async fn match_components_petgraph<T: MatchComponent>(
    ctx: &DalContext,
    query: &Arc<SearchComponentsSpikeV1Request>,
    matcher: T,
) -> ComponentsResult<Vec<ComponentId>> {
    let ctx = Arc::new(ctx.clone());
    let mut results = vec![];
    for component_id in Component::list_ids(&ctx).await? {
        if let Some(component_id) = matcher
            .clone()
            .match_component(ctx.clone(), component_id, query.clone())
            .await?
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

#[derive(Clone)]
pub struct PetgraphSync(Arc<dal::WorkspaceSnapshotGraph>);

impl PetgraphSync {
    fn graph(&self) -> &dal::WorkspaceSnapshotGraph {
        &self.0
    }

    pub fn root(self, component_id: ComponentId) -> WorkspaceSnapshotGraphResult<NodeIndex> {
        let component = self.graph().get_node_index_by_id(component_id)?;
        self.graph().target(component, EdgeWeightKind::Root)
    }

    pub fn children(self, av: NodeIndex) -> WorkspaceSnapshotGraphResult<Vec<NodeIndex>> {
        Ok(self
            .graph()
            .targets(av, EdgeWeightKindDiscriminants::Contain)
            .collect())
    }

    pub fn av_prop_name(self, av: NodeIndex) -> WorkspaceSnapshotGraphResult<String> {
        let prop = self.clone().av_prop(av)?;
        self.prop_name(prop)
    }
    pub fn av_prop(self, av: NodeIndex) -> WorkspaceSnapshotGraphResult<NodeIndex> {
        self.graph().target(av, EdgeWeightKind::Prop)
    }
    pub fn prop_name(self, prop: NodeIndex) -> WorkspaceSnapshotGraphResult<String> {
        Ok(self.prop_node_weight(prop)?.name)
    }
    pub fn prop_node_weight(self, prop: NodeIndex) -> WorkspaceSnapshotGraphResult<PropNodeWeight> {
        Ok(self.graph().get_node_weight(prop)?.get_prop_node_weight()?)
    }

    pub fn av_node_weight(
        self,
        av: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<AttributeValueNodeWeight> {
        Ok(self
            .graph()
            .get_node_weight(av)?
            .get_attribute_value_node_weight()?)
    }

    pub async fn av_value(
        self,
        av: NodeIndex,
        ctx: Arc<DalContext>,
    ) -> WorkspaceSnapshotGraphResult<Option<String>> {
        let av_node_weight = self.av_node_weight(av)?;
        let value = AttributeValue::fetch_value_from_store(&ctx, av_node_weight.value).await?;
        if let Some(serde_json::Value::String(value)) = value {
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }
}

#[derive(Clone)]
pub struct PetgraphAsync(Arc<dal::WorkspaceSnapshotGraph>);

impl PetgraphAsync {
    fn graph(&self) -> &dal::WorkspaceSnapshotGraph {
        &self.0
    }

    pub async fn root(self, component_id: ComponentId) -> WorkspaceSnapshotGraphResult<NodeIndex> {
        let component = self.graph().get_node_index_by_id(component_id)?;
        self.graph().target(component, EdgeWeightKind::Root)
    }

    pub async fn children(self, av: NodeIndex) -> WorkspaceSnapshotGraphResult<Vec<NodeIndex>> {
        Ok(self
            .graph()
            .targets(av, EdgeWeightKindDiscriminants::Contain)
            .collect())
    }

    pub async fn av_prop_name(self, av: NodeIndex) -> WorkspaceSnapshotGraphResult<String> {
        let prop = self.clone().av_prop(av).await?;
        self.prop_name(prop).await
    }
    pub async fn av_prop(self, av: NodeIndex) -> WorkspaceSnapshotGraphResult<NodeIndex> {
        self.graph().target(av, EdgeWeightKind::Prop)
    }
    pub async fn prop_name(self, prop: NodeIndex) -> WorkspaceSnapshotGraphResult<String> {
        Ok(self.prop_node_weight(prop).await?.name)
    }
    pub async fn prop_node_weight(
        self,
        prop: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<PropNodeWeight> {
        Ok(self.graph().get_node_weight(prop)?.get_prop_node_weight()?)
    }

    pub async fn av_node_weight(
        self,
        av: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<AttributeValueNodeWeight> {
        Ok(self
            .graph()
            .get_node_weight(av)?
            .get_attribute_value_node_weight()?)
    }

    pub async fn av_value(
        self,
        av: NodeIndex,
        ctx: Arc<DalContext>,
    ) -> WorkspaceSnapshotGraphResult<Option<String>> {
        let av_node_weight = self.av_node_weight(av).await?;
        let value = AttributeValue::fetch_value_from_store(&ctx, av_node_weight.value).await?;
        if let Some(serde_json::Value::String(value)) = value {
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }
}

#[derive(Clone)]
pub struct PetgraphAsyncCtx(Arc<DalContext>);

impl PetgraphAsyncCtx {
    fn graph(&self) -> Arc<dal::WorkspaceSnapshotGraph> {
        let Ok(workspace_snapshot) = self.0.workspace_snapshot() else {
            panic!("Workspace snapshot not found");
        };
        workspace_snapshot.graph().clone()
    }

    pub async fn root(self, component_id: ComponentId) -> WorkspaceSnapshotGraphResult<NodeIndex> {
        let component = self.graph().get_node_index_by_id(component_id)?;
        self.graph().target(component, EdgeWeightKind::Root)
    }

    pub async fn children(self, av: NodeIndex) -> WorkspaceSnapshotGraphResult<Vec<NodeIndex>> {
        Ok(self
            .graph()
            .targets(av, EdgeWeightKindDiscriminants::Contain)
            .collect())
    }

    pub async fn av_prop_name(self, av: NodeIndex) -> WorkspaceSnapshotGraphResult<String> {
        let prop = self.clone().av_prop(av).await?;
        self.prop_name(prop).await
    }
    pub async fn av_prop(self, av: NodeIndex) -> WorkspaceSnapshotGraphResult<NodeIndex> {
        self.graph().target(av, EdgeWeightKind::Prop)
    }
    pub async fn prop_name(self, prop: NodeIndex) -> WorkspaceSnapshotGraphResult<String> {
        Ok(self.prop_node_weight(prop).await?.name)
    }
    pub async fn prop_node_weight(
        self,
        prop: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<PropNodeWeight> {
        Ok(self.graph().get_node_weight(prop)?.get_prop_node_weight()?)
    }

    pub async fn av_node_weight(
        self,
        av: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<AttributeValueNodeWeight> {
        Ok(self
            .graph()
            .get_node_weight(av)?
            .get_attribute_value_node_weight()?)
    }

    pub async fn av_value(
        self,
        av: NodeIndex,
        ctx: Arc<DalContext>,
    ) -> WorkspaceSnapshotGraphResult<Option<String>> {
        let av_node_weight = self.av_node_weight(av).await?;
        let value = AttributeValue::fetch_value_from_store(&ctx, av_node_weight.value).await?;
        if let Some(serde_json::Value::String(value)) = value {
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }
}

pub trait PetgraphRef: Clone + Send + Sync {
    type Target: std::ops::Deref<Target = dal::WorkspaceSnapshotGraph> + Clone + Send + Sync;
    fn graph(&self) -> Self::Target;
}
impl PetgraphRef for Arc<DalContext> {
    type Target = WorkspaceSnapshotSelectorDeref;
    fn graph(&self) -> Self::Target {
        WorkspaceSnapshotSelectorDeref(self.workspace_snapshot().unwrap())
    }
}
impl PetgraphRef for Arc<dal::WorkspaceSnapshotGraph> {
    type Target = Arc<dal::WorkspaceSnapshotGraph>;
    fn graph(&self) -> Self::Target {
        self.clone()
    }
}
// So we can deref to the graph directly from workspace_snapshot()
#[derive(Clone)]
pub struct WorkspaceSnapshotSelectorDeref(WorkspaceSnapshotSelector);
impl std::ops::Deref for WorkspaceSnapshotSelectorDeref {
    type Target = dal::WorkspaceSnapshotGraph;
    fn deref(&self) -> &Self::Target {
        &self.0.graph()
    }
}

#[derive(Clone, derive_more::Deref)]
pub struct PetgraphSyncRefImpl<
    R: std::ops::Deref<Target = dal::WorkspaceSnapshotGraph> + Clone + Send + Sync,
>(R);
impl<R: std::ops::Deref<Target = dal::WorkspaceSnapshotGraph> + Clone + Send + Sync> From<R>
    for PetgraphSyncRefImpl<R>
{
    fn from(from: R) -> Self {
        Self(from)
    }
}
impl<R: std::ops::Deref<Target = dal::WorkspaceSnapshotGraph> + Clone + Send + Sync>
    PetgraphSyncRefImpl<R>
{
    pub fn root(&self, component: ComponentId) -> WorkspaceSnapshotGraphResult<NodeIndex> {
        let component = self.get_node_index_by_id(component)?;
        self.target(component, EdgeWeightKind::Root)
    }

    pub fn children(
        &self,
        av: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<impl Iterator<Item = NodeIndex>> {
        Ok(self.targets(av, EdgeWeightKindDiscriminants::Contain))
    }

    pub fn av_prop_name(&self, av: NodeIndex) -> WorkspaceSnapshotGraphResult<&str> {
        let prop = self.av_prop(av)?;
        self.prop_name(prop)
    }
    pub fn av_prop(&self, av: NodeIndex) -> WorkspaceSnapshotGraphResult<NodeIndex> {
        self.target(av, EdgeWeightKind::Prop)
    }
    pub fn prop_name(&self, prop: NodeIndex) -> WorkspaceSnapshotGraphResult<&str> {
        let prop_node = self.prop_node_weight(prop)?;
        Ok(&prop_node.name)
    }
    pub fn prop_node_weight(
        &self,
        prop: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<&PropNodeWeight> {
        Ok(self.get_node_weight(prop)?.as_prop_node_weight()?)
    }

    pub fn av_node_weight(
        &self,
        av: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<&AttributeValueNodeWeight> {
        Ok(self.get_node_weight(av)?.as_attribute_value_node_weight()?)
    }

    pub async fn av_value(
        &self,
        av: NodeIndex,
        ctx: &DalContext,
    ) -> WorkspaceSnapshotGraphResult<Option<String>> {
        let av_node = self.av_node_weight(av)?;
        self.fetch_value(&av_node.value, ctx).await
    }
    pub async fn fetch_value(
        &self,
        value: &Option<ContentAddress>,
        ctx: &DalContext,
    ) -> WorkspaceSnapshotGraphResult<Option<String>> {
        let value = AttributeValue::fetch_value_from_store(ctx, value.clone()).await?;
        if let Some(serde_json::Value::String(value)) = value {
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }
}

#[derive(Clone, derive_more::Deref)]
pub struct PetgraphSyncRefIdImpl<
    R: std::ops::Deref<Target = dal::WorkspaceSnapshotGraph> + Clone + Send + Sync,
>(PetgraphSyncRefImpl<R>);
impl<R: std::ops::Deref<Target = dal::WorkspaceSnapshotGraph> + Clone + Send + Sync> From<R>
    for PetgraphSyncRefIdImpl<R>
{
    fn from(from: R) -> Self {
        Self(from.into())
    }
}
impl<R: std::ops::Deref<Target = dal::WorkspaceSnapshotGraph> + Clone + Send + Sync>
    PetgraphSyncRefIdImpl<R>
{
    fn id<T: From<dal::Ulid>>(&self, index: NodeIndex) -> T {
        self.0.node_index_to_id(index).unwrap().into()
    }
    fn index(&self, id: impl Into<dal::Ulid>) -> NodeIndex {
        self.0.get_node_index_by_id(id).unwrap()
    }

    pub fn root(&self, component: ComponentId) -> WorkspaceSnapshotGraphResult<AttributeValueId> {
        Ok(self.id(self.0.root(component)?))
    }

    pub fn children(
        &self,
        av: AttributeValueId,
    ) -> WorkspaceSnapshotGraphResult<impl Iterator<Item = AttributeValueId>> {
        Ok(self.0.children(self.index(av))?.map(|av| self.id(av)))
    }

    pub fn av_prop_name(&self, av: AttributeValueId) -> WorkspaceSnapshotGraphResult<&str> {
        let prop = self.av_prop(av)?;
        self.prop_name(prop)
    }
    pub fn av_prop(&self, av: AttributeValueId) -> WorkspaceSnapshotGraphResult<PropId> {
        Ok(self.id(self.0.av_prop(self.index(av))?))
    }
    pub fn prop_name(&self, prop: PropId) -> WorkspaceSnapshotGraphResult<&str> {
        let prop_node = self.prop_node_weight(prop)?;
        Ok(&prop_node.name)
    }
    pub fn prop_node_weight(&self, prop: PropId) -> WorkspaceSnapshotGraphResult<&PropNodeWeight> {
        Ok(self
            .0
            .get_node_weight(self.index(prop))?
            .as_prop_node_weight()?)
    }

    pub fn av_node_weight(
        &self,
        av: AttributeValueId,
    ) -> WorkspaceSnapshotGraphResult<&AttributeValueNodeWeight> {
        Ok(self
            .0
            .get_node_weight(self.index(av))?
            .as_attribute_value_node_weight()?)
    }

    pub async fn av_value(
        &self,
        av: AttributeValueId,
        ctx: &DalContext,
    ) -> WorkspaceSnapshotGraphResult<Option<String>> {
        let av_node = self.av_node_weight(av)?;
        self.fetch_value(&av_node.value, ctx).await
    }
    pub async fn fetch_value(
        &self,
        value: &Option<ContentAddress>,
        ctx: &DalContext,
    ) -> WorkspaceSnapshotGraphResult<Option<String>> {
        let value = AttributeValue::fetch_value_from_store(ctx, value.clone()).await?;
        if let Some(serde_json::Value::String(value)) = value {
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }
}

#[derive(Clone, derive_more::From)]
pub struct PetgraphSyncImpl<R: PetgraphRef>(R);
impl<R: PetgraphRef> PetgraphSyncImpl<R> {
    fn inner(&self) -> PetgraphSyncRefImpl<R::Target> {
        PetgraphSyncRefImpl(self.0.graph())
    }

    pub fn root(self, component: ComponentId) -> WorkspaceSnapshotGraphResult<NodeIndex> {
        self.inner().root(component)
    }

    pub fn children(self, av: NodeIndex) -> WorkspaceSnapshotGraphResult<Vec<NodeIndex>> {
        Ok(self.inner().children(av)?.collect())
    }

    pub fn av_prop_name(self, av: NodeIndex) -> WorkspaceSnapshotGraphResult<String> {
        let prop = self.clone().av_prop(av)?;
        self.prop_name(prop)
    }
    pub fn av_prop(self, av: NodeIndex) -> WorkspaceSnapshotGraphResult<NodeIndex> {
        self.inner().av_prop(av)
    }
    pub fn prop_name(self, prop: NodeIndex) -> WorkspaceSnapshotGraphResult<String> {
        let prop_node = self.prop_node_weight(prop)?;
        Ok(prop_node.name)
    }
    pub fn prop_node_weight(self, prop: NodeIndex) -> WorkspaceSnapshotGraphResult<PropNodeWeight> {
        self.inner().prop_node_weight(prop).cloned()
    }

    pub fn av_node_weight(
        self,
        av: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<AttributeValueNodeWeight> {
        self.inner().av_node_weight(av).cloned()
    }

    pub async fn av_value(
        self,
        av: NodeIndex,
        ctx: &DalContext,
    ) -> WorkspaceSnapshotGraphResult<Option<String>> {
        let av_node = self.clone().av_node_weight(av)?;
        self.fetch_value(&av_node.value, ctx).await
    }
    pub async fn fetch_value(
        self,
        value: &Option<ContentAddress>,
        ctx: &DalContext,
    ) -> WorkspaceSnapshotGraphResult<Option<String>> {
        self.inner().fetch_value(value, ctx).await
    }
}

#[derive(Clone, derive_more::From)]
pub struct PetgraphSyncIdImpl<R: PetgraphRef>(R);
impl<R: PetgraphRef> PetgraphSyncIdImpl<R> {
    fn inner(self) -> PetgraphSyncRefIdImpl<R::Target> {
        self.0.graph().into()
    }

    pub fn root(self, component: ComponentId) -> WorkspaceSnapshotGraphResult<AttributeValueId> {
        self.inner().root(component)
    }

    pub fn children(
        self,
        av: AttributeValueId,
    ) -> WorkspaceSnapshotGraphResult<Vec<AttributeValueId>> {
        Ok(self.inner().children(av)?.collect())
    }

    pub fn av_prop_name(self, av: AttributeValueId) -> WorkspaceSnapshotGraphResult<String> {
        let prop = self.clone().av_prop(av)?;
        self.prop_name(prop)
    }
    pub fn av_prop(self, av: AttributeValueId) -> WorkspaceSnapshotGraphResult<PropId> {
        self.inner().av_prop(av)
    }
    pub fn prop_name(self, prop: PropId) -> WorkspaceSnapshotGraphResult<String> {
        let prop_node = self.prop_node_weight(prop)?;
        Ok(prop_node.name)
    }
    pub fn prop_node_weight(self, prop: PropId) -> WorkspaceSnapshotGraphResult<PropNodeWeight> {
        self.inner().prop_node_weight(prop).cloned()
    }

    pub fn av_node_weight(
        self,
        av: AttributeValueId,
    ) -> WorkspaceSnapshotGraphResult<AttributeValueNodeWeight> {
        self.inner().av_node_weight(av).cloned()
    }

    pub async fn av_value(
        self,
        av: AttributeValueId,
        ctx: &DalContext,
    ) -> WorkspaceSnapshotGraphResult<Option<String>> {
        let av_node = self.clone().av_node_weight(av)?;
        self.fetch_value(&av_node.value, ctx).await
    }
    pub async fn fetch_value(
        self,
        value: &Option<ContentAddress>,
        ctx: &DalContext,
    ) -> WorkspaceSnapshotGraphResult<Option<String>> {
        self.inner().fetch_value(value, ctx).await
    }
}

#[derive(Clone, derive_more::From)]
pub struct PetgraphAsyncImpl<R: PetgraphRef>(R);
impl<R: PetgraphRef> PetgraphAsyncImpl<R> {
    fn inner(self) -> PetgraphSyncImpl<R> {
        self.0.into()
    }

    pub async fn root(self, component: ComponentId) -> WorkspaceSnapshotGraphResult<NodeIndex> {
        self.inner().root(component)
    }

    pub async fn children(self, av: NodeIndex) -> WorkspaceSnapshotGraphResult<Vec<NodeIndex>> {
        Ok(self.inner().children(av)?)
    }

    pub async fn av_prop_name(self, av: NodeIndex) -> WorkspaceSnapshotGraphResult<String> {
        let prop = self.clone().av_prop(av).await?;
        self.prop_name(prop).await
    }
    pub async fn av_prop(self, av: NodeIndex) -> WorkspaceSnapshotGraphResult<NodeIndex> {
        self.inner().av_prop(av)
    }
    pub async fn prop_name(self, prop: NodeIndex) -> WorkspaceSnapshotGraphResult<String> {
        let prop_node = self.prop_node_weight(prop).await?;
        Ok(prop_node.name)
    }
    pub async fn prop_node_weight(
        self,
        prop: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<PropNodeWeight> {
        self.inner().prop_node_weight(prop)
    }

    pub async fn av_node_weight(
        self,
        av: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<AttributeValueNodeWeight> {
        self.inner().av_node_weight(av)
    }

    pub async fn av_value(
        self,
        av: NodeIndex,
        ctx: &DalContext,
    ) -> WorkspaceSnapshotGraphResult<Option<String>> {
        let av_node = self.clone().av_node_weight(av).await?;
        self.fetch_value(&av_node.value, ctx).await
    }
    pub async fn fetch_value(
        self,
        value: &Option<ContentAddress>,
        ctx: &DalContext,
    ) -> WorkspaceSnapshotGraphResult<Option<String>> {
        self.inner().fetch_value(value, ctx).await
    }
}

#[derive(Clone, derive_more::From)]
pub struct PetgraphAsyncIdImpl<R: PetgraphRef>(R);
impl<R: PetgraphRef> PetgraphAsyncIdImpl<R> {
    fn inner(self) -> PetgraphSyncIdImpl<R> {
        self.0.into()
    }

    pub async fn root(
        self,
        component: ComponentId,
    ) -> WorkspaceSnapshotGraphResult<AttributeValueId> {
        self.inner().root(component)
    }

    pub async fn children(
        self,
        av: AttributeValueId,
    ) -> WorkspaceSnapshotGraphResult<Vec<AttributeValueId>> {
        Ok(self.inner().children(av)?)
    }

    pub async fn av_prop_name(self, av: AttributeValueId) -> WorkspaceSnapshotGraphResult<String> {
        let prop = self.clone().av_prop(av).await?;
        self.prop_name(prop).await
    }
    pub async fn av_prop(self, av: AttributeValueId) -> WorkspaceSnapshotGraphResult<PropId> {
        self.inner().av_prop(av)
    }
    pub async fn prop_name(self, prop: PropId) -> WorkspaceSnapshotGraphResult<String> {
        let prop_node = self.prop_node_weight(prop).await?;
        Ok(prop_node.name)
    }
    pub async fn prop_node_weight(
        self,
        prop: PropId,
    ) -> WorkspaceSnapshotGraphResult<PropNodeWeight> {
        self.inner().prop_node_weight(prop)
    }

    pub async fn av_node_weight(
        self,
        av: AttributeValueId,
    ) -> WorkspaceSnapshotGraphResult<AttributeValueNodeWeight> {
        self.inner().av_node_weight(av)
    }

    pub async fn av_value(
        self,
        av: AttributeValueId,
        ctx: &DalContext,
    ) -> WorkspaceSnapshotGraphResult<Option<String>> {
        let av_node = self.clone().av_node_weight(av).await?;
        self.fetch_value(&av_node.value, ctx).await
    }
    pub async fn fetch_value(
        self,
        value: &Option<ContentAddress>,
        ctx: &DalContext,
    ) -> WorkspaceSnapshotGraphResult<Option<String>> {
        self.inner().fetch_value(value, ctx).await
    }
}

#[async_trait::async_trait]
pub trait MatchComponent: Clone {
    async fn match_component(
        self,
        ctx: Arc<DalContext>,
        component_id: ComponentId,
        query: Arc<SearchComponentsSpikeV1Request>,
    ) -> ComponentsResult<Option<ComponentId>>;
}

#[async_trait::async_trait]
impl<R: std::ops::Deref<Target = dal::WorkspaceSnapshotGraph> + Clone + Send + Sync> MatchComponent
    for PetgraphSyncRefImpl<R>
{
    async fn match_component(
        self,
        ctx: Arc<DalContext>,
        component_id: ComponentId,
        query: Arc<SearchComponentsSpikeV1Request>,
    ) -> ComponentsResult<Option<ComponentId>> {
        let root = self.root(component_id)?;
        let mut work_queue = VecDeque::from_iter(self.clone().children(root)?);
        while let Some(av) = work_queue.pop_front() {
            let prop_name = self.av_prop_name(av)?;
            if prop_name == query.attr_name {
                if let Some(value) = self.av_value(av, &ctx).await? {
                    if value == query.attr_value {
                        return Ok(Some(component_id));
                    }
                }
            }
            work_queue.extend(self.children(av)?);
        }
        Ok(None)
    }
}

#[async_trait::async_trait]
impl<R: std::ops::Deref<Target = dal::WorkspaceSnapshotGraph> + Clone + Send + Sync> MatchComponent
    for PetgraphSyncRefIdImpl<R>
{
    async fn match_component(
        self,
        ctx: Arc<DalContext>,
        component_id: ComponentId,
        query: Arc<SearchComponentsSpikeV1Request>,
    ) -> ComponentsResult<Option<ComponentId>> {
        let root = self.root(component_id)?;
        let mut work_queue = VecDeque::from_iter(self.children(root)?);
        while let Some(av) = work_queue.pop_front() {
            let prop_name = self.av_prop_name(av)?;
            if prop_name == query.attr_name {
                if let Some(value) = self.av_value(av, &ctx).await? {
                    if value == query.attr_value {
                        return Ok(Some(component_id));
                    }
                }
            }
            work_queue.extend(self.children(av)?);
        }
        Ok(None)
    }
}

#[async_trait::async_trait]
impl<R: PetgraphRef> MatchComponent for PetgraphSyncImpl<R> {
    async fn match_component(
        self,
        ctx: Arc<DalContext>,
        component_id: ComponentId,
        query: Arc<SearchComponentsSpikeV1Request>,
    ) -> ComponentsResult<Option<ComponentId>> {
        let root = self.clone().root(component_id)?;
        let mut work_queue = VecDeque::from(self.clone().children(root)?);
        while let Some(av) = work_queue.pop_front() {
            let prop_name = self.clone().av_prop_name(av)?;
            if prop_name == query.attr_name {
                if let Some(value) = self.clone().av_value(av, &ctx).await? {
                    if value == query.attr_value {
                        return Ok(Some(component_id));
                    }
                }
            }
            work_queue.extend(self.clone().children(av)?);
        }
        Ok(None)
    }
}

#[async_trait::async_trait]
impl<R: PetgraphRef> MatchComponent for PetgraphSyncIdImpl<R> {
    async fn match_component(
        self,
        ctx: Arc<DalContext>,
        component_id: ComponentId,
        query: Arc<SearchComponentsSpikeV1Request>,
    ) -> ComponentsResult<Option<ComponentId>> {
        let root = self.clone().root(component_id)?;
        let mut work_queue = VecDeque::from_iter(self.clone().children(root)?);
        while let Some(av) = work_queue.pop_front() {
            let prop_name = self.clone().av_prop_name(av)?;
            if prop_name == query.attr_name {
                if let Some(value) = self.clone().av_value(av, &ctx).await? {
                    if value == query.attr_value {
                        return Ok(Some(component_id));
                    }
                }
            }
            work_queue.extend(self.clone().children(av)?);
        }
        Ok(None)
    }
}

#[async_trait::async_trait]
impl<R: PetgraphRef> MatchComponent for PetgraphAsyncImpl<R> {
    async fn match_component(
        self,
        ctx: Arc<DalContext>,
        component_id: ComponentId,
        query: Arc<SearchComponentsSpikeV1Request>,
    ) -> ComponentsResult<Option<ComponentId>> {
        let root = self.clone().root(component_id).await?;
        let mut work_queue = VecDeque::from_iter(self.clone().children(root).await?);
        while let Some(av) = work_queue.pop_front() {
            let prop_name = self.clone().av_prop_name(av).await?;
            if prop_name == query.attr_name {
                if let Some(value) = self.clone().av_value(av, &ctx).await? {
                    if value == query.attr_value {
                        return Ok(Some(component_id));
                    }
                }
            }
            work_queue.extend(self.clone().children(av).await?);
        }
        Ok(None)
    }
}

#[async_trait::async_trait]
impl<R: PetgraphRef> MatchComponent for PetgraphAsyncIdImpl<R> {
    async fn match_component(
        self,
        ctx: Arc<DalContext>,
        component_id: ComponentId,
        query: Arc<SearchComponentsSpikeV1Request>,
    ) -> ComponentsResult<Option<ComponentId>> {
        let root = self.clone().root(component_id).await?;
        let mut work_queue = VecDeque::from(self.clone().children(root).await?);
        while let Some(av) = work_queue.pop_front() {
            let prop_name = self.clone().av_prop_name(av).await?;
            if prop_name == query.attr_name {
                if let Some(value) = self.clone().av_value(av, &ctx).await? {
                    if value == query.attr_value {
                        return Ok(Some(component_id));
                    }
                }
            }
            work_queue.extend(self.clone().children(av).await?);
        }
        Ok(None)
    }
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
    PetgraphId,
    PetgraphRef,
    PetgraphRefId,
    PetgraphAsync,
    PetgraphAsyncId,
    PetgraphCtx,
    PetgraphIdCtx,
    PetgraphRefCtx,
    PetgraphRefIdCtx,
    PetgraphAsyncCtx,
    PetgraphAsyncIdCtx,
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
