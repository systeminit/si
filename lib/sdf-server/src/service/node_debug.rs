use axum::{extract::Query, routing::get, Json, Router};
use dal::{
    workspace_snapshot::{node_weight::NodeWeight, Direction},
    EdgeWeight, TransactionsError, Visibility, WorkspaceSnapshotError,
};
use serde::{Deserialize, Serialize};
use telemetry::prelude::*;
use thiserror::Error;
use ulid::Ulid;

use super::impl_default_error_into_response;
use crate::{
    extract::{AccessBuilder, HandlerContext},
    AppState,
};

#[remain::sorted]
#[derive(Debug, Error)]
pub enum NodeDebugError {
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

pub type NodeDebugResult<T> = std::result::Result<T, NodeDebugError>;

impl_default_error_into_response!(NodeDebugError);

pub fn routes() -> Router<AppState> {
    Router::new().route("/", get(node_debug))
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NodeDebugRequest {
    pub id: Ulid,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EdgeDebugInfo {
    pub edge_weight: EdgeWeight,
    pub other_node: NodeWeight,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NodeDebugResponse {
    node: NodeWeight,
    incoming_edges: Vec<EdgeDebugInfo>,
    outgoing_edges: Vec<EdgeDebugInfo>,
}

async fn node_debug(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<NodeDebugRequest>,
) -> NodeDebugResult<Json<NodeDebugResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let node_id = request.id;
    let mut incoming_edges = vec![];
    let mut outgoing_edges = vec![];

    let snapshot = ctx.workspace_snapshot()?;
    let node = snapshot.get_node_weight_by_id(node_id).await?;

    for (edge_weight, source_idx, _) in snapshot
        .edges_directed(node_id, Direction::Incoming)
        .await?
    {
        let other_node = snapshot.get_node_weight(source_idx).await?;
        incoming_edges.push(EdgeDebugInfo {
            edge_weight,
            other_node,
        });
    }

    for (edge_weight, _, target_idx) in snapshot
        .edges_directed(node_id, Direction::Outgoing)
        .await?
    {
        let other_node = snapshot.get_node_weight(target_idx).await?;
        outgoing_edges.push(EdgeDebugInfo {
            edge_weight,
            other_node,
        });
    }

    Ok(Json(NodeDebugResponse {
        node,
        incoming_edges,
        outgoing_edges,
    }))
}
