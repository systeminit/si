//! Mostly everything is a node or an edge!

// #![warn(
//     missing_debug_implementations,
//     missing_docs,
//     rust_2018_idioms,
//     unreachable_pub,
//     bad_style,
//     dead_code,
//     improper_ctypes,
//     non_shorthand_field_patterns,
//     no_mangle_generic_items,
//     overflowing_literals,
//     path_statements,
//     patterns_in_fns_without_body,
//     private_in_public,
//     unconditional_recursion,
//     unused,
//     unused_allocation,
//     unused_comparisons,
//     unused_parens,
//     while_true,
//     clippy::missing_panics_doc
// )]

use petgraph::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use si_data_pg::PgError;
use thiserror::Error;
use ulid::Ulid;

use crate::{DalContext, StandardModelError, Timestamp, TransactionsError};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum WorkspaceSnapshotError {
    #[error("si_data_pg error: {0}")]
    Pg(#[from] PgError),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
}

pub type WorkspaceSnapshotResult<T> = Result<T, WorkspaceSnapshotError>;

pub type WorkspaceSnapshotId = Ulid;

pub type WorkspaceSnapshotGraph = StableGraph<SiNode, SiEdge>;

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceSnapshot {
    id: WorkspaceSnapshotId,
    #[serde(flatten)]
    timestamp: Timestamp,
    snapshot: Value,
}

impl WorkspaceSnapshot {
    pub async fn new(ctx: &DalContext) -> WorkspaceSnapshotResult<Self> {
        let mut snapshot: StableGraph<SiNode, SiEdge> = StableGraph::with_capacity(1, 0);
        snapshot.add_node(SiNode::new(SiNodeKind::Root));
        let serialized_snapshot = serde_json::to_value(&snapshot)?;

        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "SELECT workspace_snapshot_create_v1($1) AS object",
                &[&serialized_snapshot],
            )
            .await?;
        let json: Value = row.try_get("object")?;
        let object: WorkspaceSnapshot = serde_json::from_value(json)?;
        Ok(object)
    }

    pub fn snapshot(&self) -> WorkspaceSnapshotResult<WorkspaceSnapshotGraph> {
        Ok(serde_json::from_value(self.snapshot.clone())?)
    }
}

pub type SiNodeId = Ulid;

#[derive(Debug, Serialize, Deserialize)]
pub struct SiNode {
    pub kind: SiNodeKind,
    pub id: SiNodeId,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SiNodeKind {
    Root,
}

impl SiNode {
    pub fn new(kind: SiNodeKind) -> SiNode {
        SiNode {
            kind,
            id: SiNodeId::new(),
        }
    }
}

pub type SiEdgeId = Ulid;

#[derive(Debug, Serialize, Deserialize)]
pub struct SiEdge {
    pub kind: SiEdgeKind,
    pub id: SiEdgeId,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SiEdgeKind {
    Uses,
}
