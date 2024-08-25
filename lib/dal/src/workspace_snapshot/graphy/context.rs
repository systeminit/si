use std::{ops::{Deref, DerefMut}, sync::Arc};
use si_events::ulid::Ulid;
use crate::{DalContext, EdgeWeight};
use super::*;
use super::super::{node_weight::NodeWeight, SnapshotReadGuard, WorkspaceSnapshot};

pub struct GraphyContext<'a> {
    pub ctx: &'a DalContext,
    pub workspace_snapshot: Arc<WorkspaceSnapshot>,
    graph: SnapshotReadGuard,
}

impl<'a> GraphyContext<'a> {
    pub async fn new(ctx: &'a DalContext) -> GraphyResult<Self> {
        Ok(Self::from_snapshot(ctx, ctx.workspace_snapshot()?).await)
    }
    pub async fn from_snapshot(ctx: &'a DalContext, workspace_snapshot: Arc<WorkspaceSnapshot>) -> Self {
        let graph = workspace_snapshot.working_copy().await;
        Self { ctx, workspace_snapshot, graph }
    }
    pub fn petgraph(&'a self) -> &'a StableGraph<NodeWeight, EdgeWeight> {
        self.graph.graph()
    }
    pub fn root(&'a self) -> Root<'a> {
        Root(self.node(self.graph.root()))
    }
    pub(super) fn node(&'a self, index: NodeIndex) -> GraphyNode<'a> {
        GraphyNode { graph: self, index }
    }
    // pub(super) fn node_index(&self, id: impl Into<Ulid>) -> GraphyResult<NodeIndex> {
    //     let id = id.into();
    //     Ok(self.graph.get_node_index_by_id(id)?)
    // }
    pub(super) fn node_index_to_id(&self, index: NodeIndex) -> GraphyResult<Ulid> {
        self.graph.node_index_to_id(index).ok_or(GraphyError::NodeIdNotFound(index))
    }
    // pub(super) fn node_by_id(&self, id: impl Into<Ulid>) -> GraphyResult<GraphyNode> {
    //     Ok(self.node(self.node_index(id)?))
    // }
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

