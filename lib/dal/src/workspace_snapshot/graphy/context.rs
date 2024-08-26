use super::{GraphyEdgeRef, GraphyError, GraphyNode, GraphyNodeId, GraphyNodeRef, GraphyResult, Root};
use crate::{
    workspace_snapshot::{node_weight::NodeWeight, SnapshotReadGuard, WorkspaceSnapshot},
    DalContext, EdgeWeight,
};
use petgraph::{
    prelude::StableGraph,
    stable_graph::{EdgeReference, NodeIndex},
};
use si_events::ulid::Ulid;
use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

pub struct GraphyContext<'a> {
    pub ctx: &'a DalContext,
    pub workspace_snapshot: Arc<WorkspaceSnapshot>,
    graph: SnapshotReadGuard,
}

impl<'a> GraphyContext<'a> {
    pub async fn new(ctx: &'a DalContext) -> GraphyResult<Self> {
        Ok(Self::from_snapshot(ctx, ctx.workspace_snapshot()?).await)
    }
    pub async fn from_snapshot(
        ctx: &'a DalContext,
        workspace_snapshot: Arc<WorkspaceSnapshot>,
    ) -> Self {
        let graph = workspace_snapshot.working_copy().await;
        Self {
            ctx,
            workspace_snapshot,
            graph,
        }
    }
    pub fn petgraph(&'a self) -> &'a StableGraph<NodeWeight, EdgeWeight> {
        self.graph.graph()
    }
    pub fn root(&'a self) -> Root<'a> {
        Root::as_node(self.node_ref(self.graph.root()))
    }
    pub fn node_ref_by_id(&'a self, id: impl Into<Ulid>) -> GraphyResult<GraphyNodeRef<'a>> {
        Ok(self.node_ref(self.node_id_to_index(id)?))
    }
    pub fn node_by_id<Id: GraphyNodeId>(&'a self, id: Id) -> GraphyResult<Id::Node<'a>> {
        Ok(Id::Node::as_node(self.node_ref_by_id(id.into())?))
    }
    pub(super) fn node_ref(&'a self, index: NodeIndex) -> GraphyNodeRef<'a> {
        GraphyNodeRef { graph: self, index }
    }
    pub(super) fn edge_ref(&'a self, edge: EdgeReference<'a, EdgeWeight>) -> GraphyEdgeRef<'a> {
        GraphyEdgeRef { graph: self, edge }
    }
    pub(super) fn node_index_to_id(&self, index: NodeIndex) -> GraphyResult<Ulid> {
        self.graph
            .node_index_to_id(index)
            .ok_or(GraphyError::NodeIdNotFound(index))
    }
    pub(crate) fn node_id_to_index(&self, id: impl Into<Ulid>) -> GraphyResult<NodeIndex> {
        let id = id.into();
        Ok(self.graph.get_node_index_by_id(id)?)
    }
}

#[allow(dead_code)]
pub struct GraphyContextMut<'a> {
    pub ctx: &'a mut DalContext,
    pub workspace_snapshot: Arc<WorkspaceSnapshot>,
    graph: SnapshotReadGuard,
}

impl<'a> Deref for GraphyContext<'a> {
    type Target = &'a DalContext;
    fn deref(&self) -> &Self::Target {
        &self.ctx
    }
}
impl<'a> Deref for GraphyContextMut<'a> {
    type Target = DalContext;
    fn deref(&self) -> &Self::Target {
        &self.ctx
    }
}
impl<'a> AsRef<DalContext> for GraphyContext<'a> {
    fn as_ref(&self) -> &DalContext {
        &self.ctx
    }
}
impl<'a> AsRef<DalContext> for GraphyContextMut<'a> {
    fn as_ref(&self) -> &DalContext {
        &self.ctx
    }
}
impl<'a> AsMut<DalContext> for GraphyContextMut<'a> {
    fn as_mut(&mut self) -> &mut DalContext {
        self.ctx
    }
}
impl<'a> DerefMut for GraphyContextMut<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.ctx
    }
}
