//! Mostly everything is a node or an edge!

// #![warn(
//     missing_debug_implementations,
//     missing_docs,
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

pub mod edge;
pub mod lamport_clock;
pub mod node;
pub mod vector_clock;

use petgraph::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use si_data_pg::PgError;
use std::sync::{Arc, Mutex};
use thiserror::Error;
use ulid::{Generator, Ulid};

use crate::workspace_snapshot::edge::SnapshotEdge;
use crate::workspace_snapshot::node::{SnapshotNode, SnapshotNodeKind};
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

// FIXME(nick): remove this in favor of the real one.
pub type ChangeSetId = Ulid;

// FIXME(nick): remove this in favor of the real one.
pub struct ChangeSet {
    pub id: ChangeSetId,
    pub generator: Arc<Mutex<Generator>>,
}

pub type WorkspaceSnapshotId = Ulid;

pub type WorkspaceSnapshotGraph = StableGraph<SnapshotNode, SnapshotEdge>;

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceSnapshot {
    id: WorkspaceSnapshotId,
    #[serde(flatten)]
    timestamp: Timestamp,
    snapshot: Value,
}

impl WorkspaceSnapshot {
    pub async fn new(ctx: &DalContext) -> WorkspaceSnapshotResult<Self> {
        let mut snapshot: StableGraph<SnapshotNode, SnapshotEdge> =
            StableGraph::with_capacity(1, 0);
        snapshot.add_node(SnapshotNode::new(SnapshotNodeKind::Root));
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
