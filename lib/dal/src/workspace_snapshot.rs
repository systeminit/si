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

pub mod change_set;
pub mod conflict;
pub mod edge_weight;
pub mod graph;
pub mod lamport_clock;
pub mod node_weight;
pub mod update;
pub mod vector_clock;

use petgraph::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use si_data_pg::PgError;
use thiserror::Error;
use ulid::Ulid;

use crate::{
    workspace_snapshot::{graph::WorkspaceSnapshotGraphError, node_weight::NodeWeightError},
    DalContext, StandardModelError, Timestamp, TransactionsError, WorkspaceSnapshotGraph,
};
use change_set::{ChangeSet, ChangeSetError, ChangeSetId};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum WorkspaceSnapshotError {
    #[error("monotonic error: {0}")]
    Monotonic(#[from] ulid::MonotonicError),
    #[error("NodeWeight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("si_data_pg error: {0}")]
    Pg(#[from] PgError),
    #[error("poison error: {0}")]
    Poison(String),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("WorkspaceSnapshotGraph error: {0}")]
    WorkspaceSnapshotGraph(#[from] WorkspaceSnapshotGraphError),
    #[error("workspace snapshot graph missing")]
    WorkspaceSnapshotGraphMissing,
}

pub type WorkspaceSnapshotResult<T> = Result<T, WorkspaceSnapshotError>;

pub type WorkspaceSnapshotId = Ulid;

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceSnapshot {
    id: WorkspaceSnapshotId,
    #[serde(flatten)]
    timestamp: Timestamp,
    snapshot: Value,
    #[serde(skip_serializing)]
    working_copy: Option<WorkspaceSnapshotGraph>,
}

impl WorkspaceSnapshot {
    pub async fn new(ctx: &DalContext, change_set: &ChangeSet) -> WorkspaceSnapshotResult<Self> {
        let snapshot = WorkspaceSnapshotGraph::new(change_set)?;
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

    pub async fn write(mut self, ctx: &DalContext) -> WorkspaceSnapshotResult<Self> {
        let working_copy = self.working_copy()?;
        working_copy.cleanup();

        let serialized_snapshot = serde_json::to_value(working_copy.clone())?;
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

    fn working_copy(&mut self) -> WorkspaceSnapshotResult<&mut WorkspaceSnapshotGraph> {
        if self.working_copy.is_none() {
            self.working_copy = Some(serde_json::from_value(self.snapshot.clone())?);
        }
        self.working_copy
            .as_mut()
            .ok_or(WorkspaceSnapshotError::WorkspaceSnapshotGraphMissing)
    }
}
