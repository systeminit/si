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

pub mod conflict;
pub mod content_address;
pub mod edge_weight;
pub mod graph;
pub mod lamport_clock;
pub mod node_weight;
pub mod update;
pub mod vector_clock;

use petgraph::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use si_data_pg::{PgError, PgRow};
use telemetry::prelude::*;
use thiserror::Error;
use ulid::Ulid;

use crate::change_set_pointer::{ChangeSetPointer, ChangeSetPointerError, ChangeSetPointerId};
use crate::workspace_snapshot::conflict::Conflict;
use crate::workspace_snapshot::edge_weight::EdgeWeight;
use crate::workspace_snapshot::node_weight::NodeWeight;
use crate::workspace_snapshot::update::Update;
use crate::{
    pk, standard_model,
    workspace_snapshot::{graph::WorkspaceSnapshotGraphError, node_weight::NodeWeightError},
    DalContext, StandardModelError, Timestamp, TransactionsError, WorkspaceSnapshotGraph,
};

const FIND: &str = include_str!("queries/workspace_snapshot/find.sql");
const FIND_FOR_CHANGE_SET: &str =
    include_str!("queries/workspace_snapshot/find_for_change_set.sql");

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

pk!(WorkspaceSnapshotId);

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceSnapshot {
    pub id: WorkspaceSnapshotId,
    #[serde(flatten)]
    timestamp: Timestamp,
    snapshot: Value,
    #[serde(skip_serializing)]
    working_copy: Option<WorkspaceSnapshotGraph>,
}

impl WorkspaceSnapshot {
    pub async fn initial(
        ctx: &DalContext,
        change_set: &ChangeSetPointer,
    ) -> WorkspaceSnapshotResult<Self> {
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

    pub async fn write(&mut self, ctx: &DalContext) -> WorkspaceSnapshotResult<()> {
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
        self.id = object.id;
        self.timestamp = object.timestamp;
        self.snapshot = object.snapshot;

        Ok(())
    }

    fn working_copy(&mut self) -> WorkspaceSnapshotResult<&mut WorkspaceSnapshotGraph> {
        if self.working_copy.is_none() {
            self.working_copy = Some(serde_json::from_value(self.snapshot.clone())?);
        }
        self.working_copy
            .as_mut()
            .ok_or(WorkspaceSnapshotError::WorkspaceSnapshotGraphMissing)
    }

    fn snapshot(&self) -> WorkspaceSnapshotResult<WorkspaceSnapshotGraph> {
        Ok(serde_json::from_value(self.snapshot.clone())?)
    }

    pub fn add_node(&mut self, node: NodeWeight) -> WorkspaceSnapshotResult<NodeIndex> {
        Ok(self.working_copy()?.add_node(node)?)
    }

    pub fn add_edge(
        &mut self,
        change_set: &ChangeSetPointer,
        from_node_index: NodeIndex,
        edge_weight: EdgeWeight,
        to_node_index: NodeIndex,
    ) -> WorkspaceSnapshotResult<EdgeIndex> {
        Ok(self
            .working_copy()?
            .add_edge(from_node_index, edge_weight, to_node_index)?)
    }

    pub async fn detect_conflicts_and_updates(
        &self,
        ctx: &DalContext,
        to_rebase_change_set: &ChangeSetPointer,
        onto_change_set: &ChangeSetPointer,
    ) -> WorkspaceSnapshotResult<(Vec<Conflict>, Vec<Update>)> {
        let onto: WorkspaceSnapshot = Self::find_for_change_set(ctx, onto_change_set.id).await?;
        Ok(self.snapshot()?.detect_conflicts_and_updates(
            to_rebase_change_set,
            &onto.snapshot()?,
            onto_change_set,
        )?)
    }

    #[instrument(skip_all)]
    pub async fn find(
        ctx: &DalContext,
        workspace_snapshot_id: WorkspaceSnapshotId,
    ) -> WorkspaceSnapshotResult<Self> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(FIND, &[&workspace_snapshot_id])
            .await?;
        Ok(standard_model::object_from_row(row)?)
    }

    #[instrument(skip_all)]
    pub async fn find_for_change_set(
        ctx: &DalContext,
        change_set_pointer_id: ChangeSetPointerId,
    ) -> WorkspaceSnapshotResult<Self> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(FIND_FOR_CHANGE_SET, &[&change_set_pointer_id])
            .await?;
        Ok(standard_model::object_from_row(row)?)
    }
}
