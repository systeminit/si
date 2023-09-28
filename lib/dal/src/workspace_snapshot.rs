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

use chrono::{DateTime, Utc};
use petgraph::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use si_data_pg::{PgError, PgRow};
use telemetry::prelude::*;
use thiserror::Error;
use ulid::Ulid;

use crate::change_set_pointer::{ChangeSetPointer, ChangeSetPointerError, ChangeSetPointerId};
use crate::component::ComponentKind;
use crate::workspace_snapshot::conflict::Conflict;
use crate::workspace_snapshot::edge_weight::EdgeWeight;
use crate::workspace_snapshot::node_weight::NodeWeight;
use crate::workspace_snapshot::update::Update;
use crate::{
    pk,
    workspace_snapshot::{graph::WorkspaceSnapshotGraphError, node_weight::NodeWeightError},
    DalContext, TransactionsError, WorkspaceSnapshotGraph,
};

use self::content_address::ContentAddress;
use self::node_weight::ContentNodeWeight;

const FIND_FOR_CHANGE_SET: &str =
    include_str!("queries/workspace_snapshot/find_for_change_set.sql");

#[remain::sorted]
#[derive(Error, Debug)]
pub enum WorkspaceSnapshotError {
    #[error("ChangeSet error: {0}")]
    ChangeSet(#[from] ChangeSetPointerError),
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
    id: WorkspaceSnapshotId,
    created_at: DateTime<Utc>,
    snapshot: Value,
    #[serde(skip_serializing)]
    working_copy: Option<WorkspaceSnapshotGraph>,
}

impl TryFrom<PgRow> for WorkspaceSnapshot {
    type Error = WorkspaceSnapshotError;

    fn try_from(row: PgRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.try_get("id")?,
            created_at: row.try_get("created_at")?,
            snapshot: row.try_get("snapshot")?,
            working_copy: None,
        })
    }
}

impl WorkspaceSnapshot {
    pub async fn initial(
        ctx: &DalContext,
        change_set: &ChangeSetPointer,
    ) -> WorkspaceSnapshotResult<Self> {
        let snapshot = WorkspaceSnapshotGraph::new(change_set)?;
        Ok(Self::new_inner(ctx, snapshot).await?)
    }

    pub async fn write(&mut self, ctx: &DalContext) -> WorkspaceSnapshotResult<()> {
        let working_copy = self.working_copy()?;
        working_copy.cleanup();

        let object = Self::new_inner(ctx, working_copy.clone()).await?;

        self.id = object.id;
        self.created_at = object.created_at;
        self.snapshot = object.snapshot;
        Ok(())
    }

    /// This _private_ method crates a new, immutable [`WorkspaceSnapshot`] from a
    /// [`WorkspaceSnapshotGraph`].
    async fn new_inner(
        ctx: &DalContext,
        graph: WorkspaceSnapshotGraph,
    ) -> WorkspaceSnapshotResult<Self> {
        let serialized_snapshot = serde_json::to_value(graph)?;
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "INSERT INTO workspace_snapshots (snapshot) VALUES ($1) RETURNING *",
                &[&serialized_snapshot],
            )
            .await?;
        Ok(Self::try_from(row)?)
    }

    pub fn id(&self) -> WorkspaceSnapshotId {
        self.id
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
            .query_one(
                "SELECT * FROM workspace_snapshots WHERE id = $1",
                &[&workspace_snapshot_id],
            )
            .await?;
        Ok(Self::try_from(row)?)
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
        Ok(Self::try_from(row)?)
    }

    pub fn create_schema(
        &self,
        change_set: &ChangeSetPointer,
        name: impl AsRef<str>,
        component_kind: ComponentKind,
    ) -> WorkspaceSnapshotResult<Ulid> {
        let new_schema_id = change_set.generate_ulid()?;
        // let new_schema =
        // let schema_node_weight = NodeWeight::Content(ContentNodeWeight::new(
        //     change_set,
        //     new_schema_id,
        //     ContentAddress::Schema(content_hash),
        // )?);

        // self.working_copy()?.add_node()

        Ok(new_schema_id)
    }
}

#[cfg(test)]
mod test {
    use crate::component::ComponentKind;

    use super::*;
    use chrono::Utc;
    use pretty_assertions_sorted::assert_eq;
    use ulid::Ulid;

    fn simple_initial_workspace_snapshot() -> WorkspaceSnapshotResult<WorkspaceSnapshot> {
        let id = Ulid::new();
        let timestamp = Utc::now();
        // The `.map_err` is a little cursed, but `WorkspaceSnapshotGraphError` does do `From` for
        // `ChangeSetPointerError`, and `WorkspaceSnapshotError` has `From` for
        // `WorkspaceSnapshotGraphError`.
        let change_set =
            ChangeSetPointer::new_local().map_err(Into::<WorkspaceSnapshotGraphError>::into)?;
        let new_graph = WorkspaceSnapshotGraph::new(&change_set)?;

        serde_json::from_value(serde_json::json![
            {
                "id": id,
                "timestamp_created_at": timestamp,
                "timestamp_updated_at": timestamp,
                "snapshot": new_graph
            }
        ])
        .map_err(Into::into)
    }

    #[test]
    fn create_schema_variant_basic() {
        let workspace_snapshot = simple_initial_workspace_snapshot()
            .expect("Unable to get basic starting WorkspaceSnapshot");
        let change_set = ChangeSetPointer::new_local().expect("Unable to create ChangeSetPointer");

        let schema = workspace_snapshot
            .create_schema(&change_set, "First Schema", ComponentKind::Standard)
            .expect("Unable to create first schema");
        todo!();
    }
}
