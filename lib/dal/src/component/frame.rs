use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumString};
use telemetry::prelude::*;
use thiserror::Error;
use ulid::Ulid;

use crate::diagram::{EdgeId, NodeId};
use crate::workspace_snapshot::edge_weight::{EdgeWeight, EdgeWeightError, EdgeWeightKind};
use crate::workspace_snapshot::WorkspaceSnapshotError;
use crate::{
    Component, ComponentError, ComponentId, ComponentType, DalContext, TransactionsError, User,
};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum FrameError {
    #[error("component error: {0}")]
    Component(#[from] ComponentError),
    #[error("edge weight error: {0}")]
    EdgeWeight(#[from] EdgeWeightError),
    #[error("parent is not a frame (child id: {0}) (parent id: {1})")]
    ParentIsNotAFrame(ComponentId, ComponentId),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

pub type FrameResult<T> = Result<T, FrameError>;

/// A unit struct containing logic for working with frames.
pub struct Frame;

impl Frame {
    /// Provides the ability to attach a child [`Component`] to a parent frame.
    pub async fn attach_child_to_parent(
        ctx: &DalContext,
        parent_id: ComponentId,
        child_id: ComponentId,
    ) -> FrameResult<()> {
        let parent = Component::get_by_id(ctx, parent_id).await?;
        let parent_type = parent.get_type(ctx).await?;

        let (source_id, destination_id) = match parent_type {
            ComponentType::AggregationFrame => {
                unimplemented!("aggregation frames are untested in the new engine")
            }
            ComponentType::Component => {
                return Err(FrameError::ParentIsNotAFrame(child_id, parent_id))
            }
            ComponentType::ConfigurationFrameDown => (parent_id, child_id),
            ComponentType::ConfigurationFrameUp => (child_id, parent_id),
        };

        Self::attach_child_to_parent_symbolic(ctx, parent_id, child_id).await?;

        Component::connect_all(ctx, source_id, destination_id).await?;

        // TODO deal with deeply nested frames (connect to all ancestor valid sockets too)
        // TODO deal with connecting frames to frames (go through children and connect to ancestors)

        Ok(())
    }

    async fn attach_child_to_parent_symbolic(
        ctx: &DalContext,
        parent_id: ComponentId,
        child_id: ComponentId,
    ) -> FrameResult<()> {
        let mut workspace_snapshot = ctx.workspace_snapshot()?.write().await;
        let change_set = ctx.change_set_pointer()?;

        workspace_snapshot.add_edge(
            parent_id,
            EdgeWeight::new(change_set, EdgeWeightKind::FrameContains)?,
            child_id,
        )?;

        Ok(())
    }
}

// TODO(nick): replace once the switchover is complete.
#[remain::sorted]
#[derive(
    Deserialize, Serialize, Debug, PartialEq, Eq, Clone, Display, EnumString, AsRefStr, Copy,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum EdgeKind {
    Configuration,
    Symbolic,
}

// TODO(nick): replace once the switchover is complete.
pub type SocketId = Ulid;

// TODO(nick): replace once the switchover is complete.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Vertex {
    pub node_id: NodeId,
    pub socket_id: SocketId,
}

// TODO(nick): replace once the switchover is complete.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Connection {
    pub id: EdgeId,
    pub classification: EdgeKind,
    pub source: Vertex,
    pub destination: Vertex,
    pub created_by: Option<User>,
    pub deleted_by: Option<User>,
}
