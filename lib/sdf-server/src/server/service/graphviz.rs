use std::collections::{HashMap, HashSet, VecDeque};

use axum::{extract::Query, response::Response, routing::get, Json, Router};
use dal::{
    schema::variant::SchemaVariantError, workspace_snapshot::{
        content_address::ContentAddressDiscriminants,
        edge_weight::EdgeWeightKindDiscriminants,
        node_weight::{NodeWeight, NodeWeightDiscriminants},
        WorkspaceSnapshotError,
    }, Component, ComponentError, SchemaVariant, SchemaVariantId, TransactionsError, Visibility
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use ulid::Ulid;

use crate::server::{
    extract::{AccessBuilder, HandlerContext},
    impl_default_error_into_response,
    state::AppState,
};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum GraphVizError {
    #[error(transparent)]
    Component(#[from] ComponentError),
    #[error(transparent)]
    ContextTransaction(#[from] TransactionsError),
    #[error("graph did not have a root node, although this is an unreachable state")]
    NoRootNode,
    #[error(transparent)]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("could not acquire lock: {0}")]
    TryLock(#[from] tokio::sync::TryLockError),
    #[error("workspace snapshot error")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

type GraphVizResult<T> = Result<T, GraphVizError>;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GraphVizRequest {
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SchemaVariantVizRequest {
    #[serde(flatten)]
    pub visibility: Visibility,
    pub schema_variant_id: SchemaVariantId,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GraphVizNode {
    id: Ulid,
    content_kind: Option<ContentAddressDiscriminants>,
    node_kind: NodeWeightDiscriminants,
    name: Option<String>,
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "kind")]
pub enum Direction {
    Outgoing,
    Incoming,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GraphVizEdge {
    from: Ulid,
    to: Ulid,
    edge_weight_kind: EdgeWeightKindDiscriminants,
    direction: Direction,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GraphVizResponse {
    pub nodes: Vec<GraphVizNode>,
    pub edges: Vec<GraphVizEdge>,
    pub root_node_id: Option<Ulid>,
}

pub async fn schema_variant(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<SchemaVariantVizRequest>,
) -> GraphVizResult<Json<GraphVizResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let mut func_nodes = vec![];
    let mut nodes = vec![];
    let mut edges = vec![];
    let mut added_nodes = HashSet::new();
    let mut added_edges = HashSet::new();
    let mut root_node_id: Option<Ulid> = None;

    let sv = SchemaVariant::get_by_id(&ctx, request.schema_variant_id).await?;
    let workspace_snapshot = ctx.workspace_snapshot()?;

    let sv_node: GraphVizNode = {
        let node_idx = workspace_snapshot
            .get_node_index_by_id(request.schema_variant_id)
            .await?;
        let sv_node_weight = workspace_snapshot.get_node_weight(node_idx).await?;

        added_nodes.insert(sv_node_weight.id());
        GraphVizNode {
            id: sv_node_weight.id(),
            content_kind: sv_node_weight.content_address_discriminants(),
            node_kind: sv_node_weight.into(),
            name: Some(sv.name().to_owned()),
        }
    };

    nodes.push(sv_node);

    // descend
    let mut work_queue: VecDeque<Ulid> = VecDeque::from([request.schema_variant_id.into()]);
    while let Some(id) = work_queue.pop_front() {
        for target in workspace_snapshot.all_outgoing_targets(id).await? {
            work_queue.push_back(target.id());
            if !added_edges.contains(&(id, target.id())) {
                added_edges.insert((id, target.id()));

                let rel_edges = workspace_snapshot
                    .get_edges_between_nodes(id, target.id())
                    .await?;
                rel_edges.iter().enumerate().for_each(|(_, edge)| {
                    edges.push(GraphVizEdge {
                        from: id,
                        to: target.id(),
                        edge_weight_kind: edge.kind().into(),
                        direction: Direction::Outgoing,
                    });
                });
            }
            let name = match &target {
                NodeWeight::Category(inner) => Some(inner.kind().to_string()),
                NodeWeight::Func(inner) => {
                    func_nodes.push(inner.id());
                    Some(inner.name().to_owned())
                }
                NodeWeight::Prop(inner) => Some(inner.name().to_owned()),
                _ => None,
            };

            if !added_nodes.contains(&target.id()) {
                added_nodes.insert(target.id());
                nodes.push(GraphVizNode {
                    id: target.id(),
                    content_kind: target.content_address_discriminants(),
                    node_kind: target.into(),
                    name,
                })
            }
        }
    }

    // ascend
    let mut work_queue: VecDeque<Ulid> = VecDeque::from([request.schema_variant_id.into()]);
    while let Some(id) = work_queue.pop_front() {
        let sources = workspace_snapshot.all_incoming_sources(id).await?;
        if sources.is_empty() {
            root_node_id = Some(id);
            continue;
        }

        for source in sources {
            work_queue.push_back(source.id());
            if !added_edges.contains(&(source.id(), id)) {
                added_edges.insert((source.id(), id));
                let rel_edges = workspace_snapshot
                    .get_edges_between_nodes(source.id(), id)
                    .await?;
                rel_edges.iter().enumerate().for_each(|(_, edge)| {
                    edges.push(GraphVizEdge {
                        from: source.id(),
                        to: id,
                        edge_weight_kind: edge.kind().into(),
                        direction: Direction::Outgoing,
                    });
                });
            }

            let name = match &source {
                NodeWeight::Category(inner) => Some(inner.kind().to_string()),
                NodeWeight::Func(inner) => Some(inner.name().to_owned()),
                NodeWeight::Prop(inner) => Some(inner.name().to_owned()),
                _ => None,
            };

            if !added_nodes.contains(&source.id()) {
                added_nodes.insert(source.id());
                nodes.push(GraphVizNode {
                    id: source.id(),
                    content_kind: source.content_address_discriminants(),
                    node_kind: source.into(),
                    name,
                })
            }
        }
    }

    // connect func_nodes to root
    for func_id in func_nodes {
        for user_node_idx in workspace_snapshot
            .incoming_sources_for_edge_weight_kind(func_id, EdgeWeightKindDiscriminants::Use)
            .await?
        {
            let user_node = workspace_snapshot
                .get_node_weight(user_node_idx)
                .await?
                .to_owned();

            if let NodeWeight::Category(cat_inner) = &user_node {
                let name = Some(cat_inner.kind().to_string());
                if !added_edges.contains(&(func_id, cat_inner.id())) {
                    added_edges.insert((func_id, cat_inner.id()));
                    let rel_edges = workspace_snapshot
                        .get_edges_between_nodes(cat_inner.id(), func_id)
                        .await?;
                    rel_edges.iter().enumerate().for_each(|(_, edge)| {
                        edges.push(GraphVizEdge {
                            from: cat_inner.id(),
                            to: func_id,
                            edge_weight_kind: edge.kind().into(),
                            direction: Direction::Outgoing,
                        });
                    });
                }
                if !added_nodes.contains(&cat_inner.id()) {
                    added_nodes.insert(cat_inner.id());
                    nodes.push(GraphVizNode {
                        id: cat_inner.id(),
                        content_kind: user_node.content_address_discriminants(),
                        node_kind: user_node.to_owned().into(),
                        name,
                    })
                }
                for cat_user_node_idx in workspace_snapshot
                    .incoming_sources_for_edge_weight_kind(
                        user_node.id(),
                        EdgeWeightKindDiscriminants::Use,
                    )
                    .await?
                {
                    let node_weight = workspace_snapshot
                        .get_node_weight(cat_user_node_idx)
                        .await?;
                    match node_weight
                        .get_content_node_weight_of_kind(ContentAddressDiscriminants::Root)
                    {
                        Ok(root_content) => {
                            if !added_edges.contains(&(cat_inner.id(), root_content.id())) {
                                added_edges.insert((cat_inner.id(), root_content.id()));
                                let rel_edges = workspace_snapshot
                                    .get_edges_between_nodes(root_content.id(), cat_inner.id())
                                    .await?;
                                rel_edges.iter().enumerate().for_each(|(_, edge)| {
                                    edges.push(GraphVizEdge {
                                        from: root_content.id(),
                                        to: cat_inner.id(),
                                        edge_weight_kind: edge.kind().into(),
                                        direction: Direction::Outgoing,
                                    });
                                });
                            }
                        }
                        _ => continue,
                    }
                }
            }
        }
    }

    let root_node_id = root_node_id.ok_or(GraphVizError::NoRootNode)?;

    Ok(Json(GraphVizResponse {
        nodes,
        edges,
        root_node_id: Some(root_node_id),
    }))
}

pub async fn components(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<GraphVizRequest>,
) -> GraphVizResult<Json<GraphVizResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let workspace_snapshot = ctx.workspace_snapshot()?;

    let mut func_nodes = vec![];
    let mut nodes = vec![];
    let mut edges = vec![];
    let mut added_nodes = HashSet::new();
    let mut added_edges = HashSet::new();

    let components = Component::list(&ctx).await?;

    for component in &components {
        if !added_nodes.contains(&component.id()) {
            added_nodes.insert(component.id());
        } else {
            continue;
        }

        let node_weight = workspace_snapshot.get_node_weight_by_id(component.id()).await?;
        let node  = GraphVizNode {
            id: node_weight.id(),
            content_kind: node_weight.content_address_discriminants(),
            node_kind: node_weight.into(),
            name: Some(component.name(&ctx).await?.to_owned()),
        };
        nodes.push(node);


        let mut work_queue: VecDeque<Ulid> = VecDeque::from([component.id().into()]);
        while let Some(id) = work_queue.pop_front() {
            for target in workspace_snapshot.all_outgoing_targets(id).await? {
                work_queue.push_back(target.id());
                if !added_edges.contains(&(id, target.id())) {
                    added_edges.insert((id, target.id()));

                    let rel_edges = workspace_snapshot
                    .get_edges_between_nodes(id, target.id())
                    .await?;
                    rel_edges.iter().enumerate().for_each(|(_, edge)| {
                        edges.push(GraphVizEdge {
                            from: id,
                            to: target.id(),
                            edge_weight_kind: edge.kind().into(),
                            direction: Direction::Outgoing,
                        });
                    });
                }

                // TODO encapsulate this in node weight logic
                let name = match &target {
                    NodeWeight::Category(inner) => Some(inner.kind().to_string()),
                    NodeWeight::Func(inner) => {
                        func_nodes.push(inner.id());
                        Some(inner.name().to_owned())
                    }
                    NodeWeight::Prop(inner) => Some(inner.name().to_owned()),
                    NodeWeight::Component(inner) => Some(Component::get_by_id(&ctx, inner.id().into()).await?.name(&ctx).await?),
                    _ => None,
                };

                if !added_nodes.contains(&target.id().into()) {
                    added_nodes.insert(target.id().into());
                    nodes.push(GraphVizNode {
                        id: target.id(),
                        content_kind: target.content_address_discriminants(),
                        node_kind: target.into(),
                        name,
                    })
                }
            }
        }
    }


    let response = GraphVizResponse {
        nodes,
        edges,
        root_node_id: None,
    };

    Ok(Json(response))
}

pub async fn nodes_edges(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<GraphVizRequest>,
) -> GraphVizResult<Json<GraphVizResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let workspace_snapshot = ctx.workspace_snapshot()?;

    let mut node_idx_to_id = HashMap::new();

    let root_node_idx = workspace_snapshot.root().await?;

    let mut nodes = vec![];

    for ( weight, idx ) in workspace_snapshot
        .nodes()
        .await?
        .into_iter() {
            node_idx_to_id.insert(idx, weight.id());
            let name = match &weight {
                NodeWeight::Category(inner) => Some(inner.kind().to_string()),
                NodeWeight::Func(inner) => Some(inner.name().to_owned()),
                NodeWeight::Prop(inner) => Some(inner.name().to_owned()),
                NodeWeight::Component(inner) => Some(Component::get_by_id(&ctx, inner.id().into()).await?.name(&ctx).await?),
                NodeWeight::AttributePrototypeArgument(inner) => Some(format!(
                    "APA Targets: {}",
                    inner
                        .targets()
                        .map(|targets| format!(
                            "\nsource: {}\nto: {}",
                            targets.source_component_id, targets.destination_component_id
                        ))
                        .unwrap_or("".to_string())
                )),
                NodeWeight::FuncArgument(inner) => Some(inner.name().to_owned()),
                _ => None,
            };
            nodes.push(GraphVizNode {
                id: weight.id(),
                content_kind: weight.content_address_discriminants(),
                node_kind: weight.into(),
                name,
            })
        };

    let edges = workspace_snapshot
        .edges()
        .await?
        .into_iter()
        .filter_map(|(edge_weight, from, to)| {
            match (node_idx_to_id.get(&from), node_idx_to_id.get(&to)) {
                (None, _) | (_, None) => None,
                (Some(&from), Some(&to)) => Some(GraphVizEdge {
                    from,
                    to,
                    edge_weight_kind: edge_weight.kind().into(),
                    direction: Direction::Outgoing,
                }),
            }
        })
        .collect();

    let response = GraphVizResponse {
        nodes,
        edges,
        root_node_id: Some(node_idx_to_id
            .get(&root_node_idx)
            .copied()
            .ok_or(GraphVizError::NoRootNode)?),
    };

    Ok(Json(response))
}

impl_default_error_into_response!(GraphVizError);

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/schema_variant", get(schema_variant))
        .route("/nodes_edges", get(nodes_edges))
        .route("/components", get(components))
}
