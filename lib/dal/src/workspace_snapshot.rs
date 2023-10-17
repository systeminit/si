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

pub mod api;
pub mod conflict;
pub mod content_address;
pub mod edge_weight;
pub mod graph;
pub mod lamport_clock;
pub mod node_weight;
pub mod update;
pub mod vector_clock;

use chrono::{DateTime, Utc};
use content_store::{ContentHash, StoreError};
use petgraph::prelude::*;
use serde::{Deserialize, Serialize};
use si_cbor::CborError;
use si_data_pg::{PgError, PgRow};
use std::collections::HashMap;
use telemetry::prelude::*;
use thiserror::Error;
use ulid::Ulid;

use crate::change_set_pointer::{ChangeSetPointer, ChangeSetPointerError, ChangeSetPointerId};
use crate::workspace_snapshot::conflict::Conflict;
use crate::workspace_snapshot::edge_weight::{EdgeWeight, EdgeWeightError, EdgeWeightKind};
use crate::workspace_snapshot::node_weight::category_node_weight::CategoryNodeKind;
use crate::workspace_snapshot::node_weight::NodeWeight;
use crate::workspace_snapshot::update::Update;
use crate::workspace_snapshot::vector_clock::VectorClockId;
use crate::{
    pk,
    workspace_snapshot::{graph::WorkspaceSnapshotGraphError, node_weight::NodeWeightError},
    DalContext, TransactionsError, WorkspaceSnapshotGraph,
};
use crate::{AttributePrototypeId, AttributeValueId, PropId, PropKind};

const FIND_FOR_CHANGE_SET: &str =
    include_str!("queries/workspace_snapshot/find_for_change_set.sql");

#[remain::sorted]
#[derive(Error, Debug)]
pub enum WorkspaceSnapshotError {
    #[error("attribute prototype {0} is missing a function edge")]
    AttributePrototypeMissingFunction(AttributePrototypeId),
    #[error("attribute value {0} missing prop edge when one was expected")]
    AttributeValueMissingPropEdge(AttributeValueId),
    #[error("attribute value {0} missing prototype")]
    AttributeValueMissingPrototype(AttributeValueId),
    #[error("cbor error: {0}")]
    Cbor(#[from] CborError),
    #[error("change set pointer error: {0}")]
    ChangeSetPointer(#[from] ChangeSetPointerError),
    #[error("edge weight error: {0}")]
    EdgeWeight(#[from] EdgeWeightError),
    #[error("cannot insert for prop kind: {0}")]
    InsertionForInvalidPropKind(PropKind),
    #[error("cannot find intrinsic func {0}")]
    IntrinsicFuncNotFound(String),
    #[error("missing content from store for id: {0}")]
    MissingContentFromStore(Ulid),
    #[error("monotonic error: {0}")]
    Monotonic(#[from] ulid::MonotonicError),
    #[error("NodeWeight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("NodeWeight mismatch, expected {0:?} to be {1}")]
    NodeWeightMismatch(NodeIndex, String),
    #[error("si_data_pg error: {0}")]
    Pg(#[from] PgError),
    #[error("poison error: {0}")]
    Poison(String),
    #[error("Array or map prop missing element prop: {0}")]
    PropMissingElementProp(PropId),
    #[error("Array or map prop has more than one child prop: {0}")]
    PropMoreThanOneChild(PropId),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("store error: {0}")]
    Store(#[from] StoreError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("Type mismatch, expected prop kind {0} got {1}")]
    TypeMismatch(PropKind, String),
    #[error("unexpected graph layout: {0}")]
    UnexpectedGraphLayout(&'static str),
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
    snapshot: Vec<u8>,
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

pub(crate) fn serde_value_to_string_type(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Bool(_) => "bool",
        serde_json::Value::Null => "null",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::Object(_) => "object",
        serde_json::Value::String(_) => "string",
    }
    .into()
}

impl WorkspaceSnapshot {
    pub async fn initial(
        ctx: &DalContext,
        change_set: &ChangeSetPointer,
    ) -> WorkspaceSnapshotResult<Self> {
        let mut graph = WorkspaceSnapshotGraph::new(change_set)?;

        // Create the category nodes under root.
        let component_node_index =
            graph.add_category_node(change_set, CategoryNodeKind::Component)?;
        let func_node_index = graph.add_category_node(change_set, CategoryNodeKind::Func)?;
        let schema_node_index = graph.add_category_node(change_set, CategoryNodeKind::Schema)?;

        // Connect them to root.
        graph.add_edge(
            graph.root(),
            EdgeWeight::new(change_set, EdgeWeightKind::Use)?,
            component_node_index,
        )?;
        graph.add_edge(
            graph.root(),
            EdgeWeight::new(change_set, EdgeWeightKind::Use)?,
            func_node_index,
        )?;
        graph.add_edge(
            graph.root(),
            EdgeWeight::new(change_set, EdgeWeightKind::Use)?,
            schema_node_index,
        )?;

        Self::new_inner(ctx, graph).await
    }

    pub async fn write(
        &mut self,
        ctx: &DalContext,
        vector_clock_id: VectorClockId,
    ) -> WorkspaceSnapshotResult<()> {
        // Pull out the working copy and clean it up.
        let working_copy = self.working_copy()?;
        working_copy.cleanup();

        // Mark everything left as seen.
        working_copy.mark_graph_seen(vector_clock_id)?;

        // Stamp the new workspace snapshot.
        let object = Self::new_inner(ctx, working_copy.clone()).await?;

        // Reset relevant fields on self.
        self.id = object.id;
        self.created_at = object.created_at;
        self.snapshot = object.snapshot;
        self.working_copy = None;
        Ok(())
    }

    /// This _private_ method crates a new, immutable [`WorkspaceSnapshot`] from a
    /// [`WorkspaceSnapshotGraph`].
    async fn new_inner(
        ctx: &DalContext,
        graph: WorkspaceSnapshotGraph,
    ) -> WorkspaceSnapshotResult<Self> {
        let serialized_snapshot = si_cbor::encode(&graph)?;
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "INSERT INTO workspace_snapshots (snapshot) VALUES ($1) RETURNING *",
                &[&serialized_snapshot],
            )
            .await?;
        Self::try_from(row)
    }

    pub fn id(&self) -> WorkspaceSnapshotId {
        self.id
    }

    fn working_copy(&mut self) -> WorkspaceSnapshotResult<&mut WorkspaceSnapshotGraph> {
        if self.working_copy.is_none() {
            self.working_copy = Some(si_cbor::decode(&self.snapshot)?);
        }
        self.working_copy
            .as_mut()
            .ok_or(WorkspaceSnapshotError::WorkspaceSnapshotGraphMissing)
    }

    pub fn root(&mut self) -> WorkspaceSnapshotResult<NodeIndex> {
        Ok(self.working_copy()?.root())
    }

    // TODO(nick): replace this with the api.
    pub fn add_node(&mut self, node: NodeWeight) -> WorkspaceSnapshotResult<NodeIndex> {
        let new_node_index = self.working_copy()?.add_node(node)?;
        Ok(new_node_index)
    }

    // TODO(nick): replace this with the api.
    pub fn update_content(
        &mut self,
        change_set: &ChangeSetPointer,
        id: Ulid,
        new_content_hash: ContentHash,
    ) -> WorkspaceSnapshotResult<()> {
        Ok(self
            .working_copy()?
            .update_content(change_set, id, new_content_hash)?)
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
        &mut self,
        to_rebase_vector_clock_id: VectorClockId,
        onto_workspace_snapshot: &mut WorkspaceSnapshot,
        onto_vector_clock_id: VectorClockId,
    ) -> WorkspaceSnapshotResult<(Vec<Conflict>, Vec<Update>)> {
        Ok(self.working_copy()?.detect_conflicts_and_updates(
            to_rebase_vector_clock_id,
            onto_workspace_snapshot.working_copy()?,
            onto_vector_clock_id,
        )?)
    }

    pub fn get_edge_by_index_stableish(
        &mut self,
        edge_index: EdgeIndex,
    ) -> WorkspaceSnapshotResult<EdgeWeight> {
        Ok(self
            .working_copy()?
            .get_edge_by_index_stableish(edge_index)?)
    }

    pub fn import_subgraph(
        &mut self,
        other: &mut Self,
        root_index: NodeIndex,
    ) -> WorkspaceSnapshotResult<HashMap<NodeIndex, NodeIndex>> {
        let updated_indices = self
            .working_copy()?
            .import_subgraph(other.working_copy()?, root_index)?;
        Ok(updated_indices)
    }

    pub fn replace_references(
        &mut self,
        original_node_index: NodeIndex,
        new_node_index: NodeIndex,
    ) -> WorkspaceSnapshotResult<()> {
        Ok(self
            .working_copy()?
            .replace_references(original_node_index, new_node_index)?)
    }

    pub fn get_node_weight(
        &mut self,
        node_index: NodeIndex,
    ) -> WorkspaceSnapshotResult<&NodeWeight> {
        Ok(self.working_copy()?.get_node_weight(node_index)?)
    }

    pub fn find_equivalent_node(
        &mut self,
        id: Ulid,
        lineage_id: Ulid,
    ) -> WorkspaceSnapshotResult<Option<NodeIndex>> {
        Ok(self.working_copy()?.find_equivalent_node(id, lineage_id)?)
    }

    pub fn dot(&mut self) {
        self.working_copy()
            .expect("failed on accessing or creating a working copy")
            .dot();
    }

    pub fn get_node_index_by_id(&mut self, id: Ulid) -> WorkspaceSnapshotResult<NodeIndex> {
        Ok(self.working_copy()?.get_node_index_by_id(id)?)
    }

    pub fn add_edge_from_root(
        &mut self,
        change_set: &ChangeSetPointer,
        destination: NodeIndex,
    ) -> WorkspaceSnapshotResult<EdgeIndex> {
        let root = self.working_copy()?.root();
        let new_edge = self.working_copy()?.add_edge(
            root,
            EdgeWeight::new(change_set, EdgeWeightKind::Use)?,
            destination,
        )?;
        Ok(new_edge)
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
        Self::try_from(row)
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
        Self::try_from(row)
    }
}
