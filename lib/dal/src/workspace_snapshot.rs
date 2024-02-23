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
use content_store::{ContentHash, Store, StoreError};
use petgraph::prelude::*;
use petgraph::stable_graph::Edges;
use serde::{Deserialize, Serialize};
use si_data_pg::{PgError, PgRow};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::time::Instant;
use ulid::Ulid;

use crate::change_set_pointer::{ChangeSetPointer, ChangeSetPointerError, ChangeSetPointerId};
use crate::workspace_snapshot::conflict::Conflict;
use crate::workspace_snapshot::edge_weight::{
    EdgeWeight, EdgeWeightError, EdgeWeightKind, EdgeWeightKindDiscriminants,
};
use crate::workspace_snapshot::node_weight::category_node_weight::CategoryNodeKind;
use crate::workspace_snapshot::node_weight::NodeWeight;
use crate::workspace_snapshot::update::Update;
use crate::workspace_snapshot::vector_clock::VectorClockId;
use crate::{
    pk,
    workspace_snapshot::{graph::WorkspaceSnapshotGraphError, node_weight::NodeWeightError},
    DalContext, TransactionsError, WorkspaceSnapshotGraph,
};

use self::node_weight::{NodeWeightDiscriminants, OrderingNodeWeight};

const FIND_FOR_CHANGE_SET: &str =
    include_str!("queries/workspace_snapshot/find_for_change_set.sql");

#[remain::sorted]
#[derive(Error, Debug)]
pub enum WorkspaceSnapshotError {
    #[error("change set pointer error: {0}")]
    ChangeSetPointer(#[from] ChangeSetPointerError),
    #[error("edge weight error: {0}")]
    EdgeWeight(#[from] EdgeWeightError),
    #[error("missing content from store for id: {0}")]
    MissingContentFromStore(Ulid),
    #[error("monotonic error: {0}")]
    Monotonic(#[from] ulid::MonotonicError),
    #[error("NodeWeight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("si_data_pg error: {0}")]
    Pg(#[from] PgError),
    #[error("postcard error: {0}")]
    Postcard(#[from] postcard::Error),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("store error: {0}")]
    Store(#[from] StoreError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("could not acquire lock: {0}")]
    TryLock(#[from] tokio::sync::TryLockError),
    #[error("Unexpected edge source {0} for target {1} and edge weight type {0:?}")]
    UnexpectedEdgeSource(Ulid, Ulid, EdgeWeightKindDiscriminants),
    #[error("Unexpected edge target {0} for source {1} and edge weight type {0:?}")]
    UnexpectedEdgeTarget(Ulid, Ulid, EdgeWeightKindDiscriminants),
    #[error("Unexpected number of incoming edges of type {0:?} for node type {1:?} with id {2}")]
    UnexpectedNumberOfIncomingEdges(EdgeWeightKindDiscriminants, NodeWeightDiscriminants, Ulid),
    #[error("WorkspaceSnapshotGraph error: {0}")]
    WorkspaceSnapshotGraph(#[from] WorkspaceSnapshotGraphError),
    #[error("workspace snapshot graph missing")]
    WorkspaceSnapshotGraphMissing,
    #[error("no workspace snapshot was fetched for this dal context")]
    WorkspaceSnapshotNotFetched,
}

pub type WorkspaceSnapshotResult<T> = Result<T, WorkspaceSnapshotError>;

pk!(WorkspaceSnapshotId);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkspaceSnapshot {
    id: WorkspaceSnapshotId,
    created_at: DateTime<Utc>,
    #[serde(skip_serializing)]
    working_copy: WorkspaceSnapshotGraph,
}

impl TryFrom<PgRow> for WorkspaceSnapshot {
    type Error = WorkspaceSnapshotError;

    fn try_from(row: PgRow) -> Result<Self, Self::Error> {
        let start = Instant::now();
        let snapshot: Vec<u8> = row.try_get("snapshot")?;
        info!("snapshot copy into vec: {:?}", start.elapsed());
        let start = Instant::now();
        let working_copy = postcard::from_bytes(&snapshot)?;
        info!("snapshot deserialize: {:?}", start.elapsed());
        Ok(Self {
            id: row.try_get("id")?,
            created_at: row.try_get("created_at")?,
            working_copy,
        })
    }
}

#[allow(dead_code)]
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
        let mut graph: WorkspaceSnapshotGraph = WorkspaceSnapshotGraph::new(change_set)?;

        // Create the category nodes under root.
        let component_node_index =
            graph.add_category_node(change_set, CategoryNodeKind::Component)?;
        let func_node_index = graph.add_category_node(change_set, CategoryNodeKind::Func)?;
        let schema_node_index = graph.add_category_node(change_set, CategoryNodeKind::Schema)?;
        let secret_node_index = graph.add_category_node(change_set, CategoryNodeKind::Secret)?;

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
        graph.add_edge(
            graph.root(),
            EdgeWeight::new(change_set, EdgeWeightKind::Use)?,
            secret_node_index,
        )?;

        // We do not care about any field other than "working_copy" because "write" will populate
        // them using the assigned working copy.
        let mut initial = Self {
            id: WorkspaceSnapshotId::NONE,
            created_at: Utc::now(),
            working_copy: graph,
        };
        initial.write(ctx, change_set.vector_clock_id()).await?;

        Ok(initial)
    }

    pub async fn write(
        &mut self,
        ctx: &DalContext,
        vector_clock_id: VectorClockId,
    ) -> WorkspaceSnapshotResult<WorkspaceSnapshotId> {
        // Pull out the working copy and clean it up.
        let working_copy = self.working_copy_mut();
        working_copy.cleanup();

        // Mark everything left as seen.
        working_copy.mark_graph_seen(vector_clock_id)?;

        // Write out to the content store.
        ctx.content_store().lock().await.write().await?;

        // Stamp the new workspace snapshot.
        let serialized_snapshot = postcard::to_stdvec(&working_copy)?;
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "INSERT INTO workspace_snapshots (snapshot) VALUES ($1) RETURNING *",
                &[&serialized_snapshot],
            )
            .await?;
        let object = Self::try_from(row)?;

        // Reset relevant fields on self.
        self.id = object.id;
        self.created_at = object.created_at;

        Ok(self.id)
    }

    pub fn id(&self) -> WorkspaceSnapshotId {
        self.id
    }

    pub fn root(&self) -> WorkspaceSnapshotResult<NodeIndex> {
        Ok(self.working_copy.root())
    }

    fn working_copy_mut(&mut self) -> &mut WorkspaceSnapshotGraph {
        &mut self.working_copy
    }

    pub fn add_node(&mut self, node: NodeWeight) -> WorkspaceSnapshotResult<NodeIndex> {
        let new_node_index = self.working_copy.add_node(node)?;
        Ok(new_node_index)
    }

    pub fn add_ordered_node(
        &mut self,
        change_set: &ChangeSetPointer,
        node: NodeWeight,
    ) -> WorkspaceSnapshotResult<NodeIndex> {
        let new_node_index = self.working_copy.add_ordered_node(change_set, node)?;
        Ok(new_node_index)
    }

    pub fn update_content(
        &mut self,
        change_set: &ChangeSetPointer,
        id: Ulid,
        new_content_hash: ContentHash,
    ) -> WorkspaceSnapshotResult<()> {
        Ok(self
            .working_copy
            .update_content(change_set, id, new_content_hash)?)
    }

    pub fn add_edge(
        &mut self,
        from_node_id: impl Into<Ulid>,
        edge_weight: EdgeWeight,
        to_node_id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<EdgeIndex> {
        let from_node_index = self.working_copy.get_node_index_by_id(from_node_id)?;
        let to_node_index = self.working_copy.get_node_index_by_id(to_node_id)?;
        Ok(self
            .working_copy
            .add_edge(from_node_index, edge_weight, to_node_index)?)
    }

    // NOTE(nick): this should only be used by the rebaser and in specific scenarios where the
    // indices are definitely correct.
    pub fn add_edge_unchecked(
        &mut self,
        from_node_index: NodeIndex,
        edge_weight: EdgeWeight,
        to_node_index: NodeIndex,
    ) -> WorkspaceSnapshotResult<EdgeIndex> {
        Ok(self
            .working_copy
            .add_edge(from_node_index, edge_weight, to_node_index)?)
    }

    pub fn add_ordered_edge(
        &mut self,
        change_set: &ChangeSetPointer,
        from_node_id: impl Into<Ulid>,
        edge_weight: EdgeWeight,
        to_node_id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<EdgeIndex> {
        let from_node_index = self.working_copy.get_node_index_by_id(from_node_id)?;
        let to_node_index = self.working_copy.get_node_index_by_id(to_node_id)?;
        let (edge_index, _) = self.working_copy.add_ordered_edge(
            change_set,
            from_node_index,
            edge_weight,
            to_node_index,
        )?;
        Ok(edge_index)
    }

    pub async fn detect_conflicts_and_updates(
        &self,
        to_rebase_vector_clock_id: VectorClockId,
        onto_workspace_snapshot: &mut WorkspaceSnapshot,
        onto_vector_clock_id: VectorClockId,
    ) -> WorkspaceSnapshotResult<(Vec<Conflict>, Vec<Update>)> {
        Ok(self.working_copy.detect_conflicts_and_updates(
            to_rebase_vector_clock_id,
            &onto_workspace_snapshot.working_copy,
            onto_vector_clock_id,
        )?)
    }

    // NOTE(nick): this should only be used by the rebaser.
    pub fn edge_endpoints(
        &mut self,
        edge_index: EdgeIndex,
    ) -> WorkspaceSnapshotResult<(NodeIndex, NodeIndex)> {
        Ok(self.working_copy.edge_endpoints(edge_index)?)
    }

    pub fn import_subgraph(
        &mut self,
        other: &mut Self,
        root_index: NodeIndex,
    ) -> WorkspaceSnapshotResult<()> {
        Ok(self
            .working_copy
            .import_subgraph(&other.working_copy, root_index)?)
    }

    /// Calls [`WorkspaceSnapshotGraph::replace_references()`]
    pub fn replace_references(
        &mut self,
        original_node_index: NodeIndex,
    ) -> WorkspaceSnapshotResult<()> {
        Ok(self.working_copy.replace_references(original_node_index)?)
    }

    pub fn get_node_weight_by_id(
        &self,
        id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<&NodeWeight> {
        let node_idx = self.get_node_index_by_id(id)?;
        Ok(self.working_copy.get_node_weight(node_idx)?)
    }

    pub fn get_node_weight(&self, node_index: NodeIndex) -> WorkspaceSnapshotResult<&NodeWeight> {
        Ok(self.working_copy.get_node_weight(node_index)?)
    }

    pub fn find_equivalent_node(
        &self,
        id: Ulid,
        lineage_id: Ulid,
    ) -> WorkspaceSnapshotResult<Option<NodeIndex>> {
        Ok(self.working_copy.find_equivalent_node(id, lineage_id)?)
    }

    pub fn cleanup(&mut self) -> WorkspaceSnapshotResult<()> {
        self.working_copy.cleanup();
        Ok(())
    }

    pub fn nodes(&self) -> WorkspaceSnapshotResult<impl Iterator<Item = (&NodeWeight, NodeIndex)>> {
        Ok(self.working_copy.nodes())
    }

    pub fn edges(
        &self,
    ) -> WorkspaceSnapshotResult<impl Iterator<Item = (&EdgeWeight, NodeIndex, NodeIndex)>> {
        Ok(self.working_copy.edges())
    }

    pub fn dot(&self) {
        self.working_copy.dot();
    }

    pub fn tiny_dot_to_file(&self, suffix: Option<&str>) {
        self.working_copy.tiny_dot_to_file(suffix);
    }

    #[inline(always)]
    pub fn get_node_index_by_id(&self, id: impl Into<Ulid>) -> WorkspaceSnapshotResult<NodeIndex> {
        Ok(self.working_copy.get_node_index_by_id(id)?)
    }

    pub fn get_latest_node_index(
        &self,
        node_index: NodeIndex,
    ) -> WorkspaceSnapshotResult<NodeIndex> {
        Ok(self.working_copy.get_latest_node_idx(node_index)?)
    }

    #[instrument(skip_all)]
    pub async fn find(
        ctx: &DalContext,
        workspace_snapshot_id: WorkspaceSnapshotId,
    ) -> WorkspaceSnapshotResult<Self> {
        let start = tokio::time::Instant::now();
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "SELECT * FROM workspace_snapshots WHERE id = $1",
                &[&workspace_snapshot_id],
            )
            .await?;
        info!("data fetch: {:?}", start.elapsed());
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

    pub fn get_category_node(
        &self,
        source: Option<Ulid>,
        kind: CategoryNodeKind,
    ) -> WorkspaceSnapshotResult<Ulid> {
        let (category_node_id, _) = self.working_copy.get_category_node(source, kind)?;
        Ok(category_node_id)
    }

    pub fn edges_directed(
        &self,
        id: impl Into<Ulid>,
        direction: Direction,
    ) -> WorkspaceSnapshotResult<Edges<'_, EdgeWeight, Directed, u32>> {
        let node_index = self.working_copy.get_node_index_by_id(id)?;
        Ok(self.working_copy.edges_directed(node_index, direction))
    }

    pub fn edges_directed_for_edge_weight_kind(
        &self,
        id: impl Into<Ulid>,
        direction: Direction,
        edge_kind: EdgeWeightKindDiscriminants,
    ) -> WorkspaceSnapshotResult<Vec<petgraph::stable_graph::EdgeReference<'_, EdgeWeight>>> {
        let node_index = self.working_copy.get_node_index_by_id(id)?;

        Ok(self
            .working_copy
            .edges_directed_for_edge_weight_kind(node_index, direction, edge_kind))
    }

    pub fn edges_directed_by_index(
        &self,
        node_index: NodeIndex,
        direction: Direction,
    ) -> WorkspaceSnapshotResult<Edges<'_, EdgeWeight, Directed, u32>> {
        Ok(self.working_copy.edges_directed(node_index, direction))
    }

    pub fn incoming_sources_for_edge_weight_kind(
        &self,
        id: impl Into<Ulid>,
        edge_weight_kind_discrim: EdgeWeightKindDiscriminants,
    ) -> WorkspaceSnapshotResult<Vec<NodeIndex>> {
        Ok(self
            .edges_directed(id.into(), Direction::Incoming)?
            .filter_map(|edge_ref| {
                if edge_weight_kind_discrim == edge_ref.weight().kind().into() {
                    Some(edge_ref.source())
                } else {
                    None
                }
            })
            .collect())
    }

    pub fn outgoing_targets_for_edge_weight_kind(
        &self,
        id: impl Into<Ulid>,
        edge_weight_kind_discrim: EdgeWeightKindDiscriminants,
    ) -> WorkspaceSnapshotResult<Vec<NodeIndex>> {
        let id = id.into();
        Ok(self
            .edges_directed(id, Direction::Outgoing)?
            .filter_map(|edge_ref| {
                if edge_weight_kind_discrim == edge_ref.weight().kind().into() {
                    Some(edge_ref.target())
                } else {
                    None
                }
            })
            .collect())
    }

    pub fn outgoing_targets_for_edge_weight_kind_by_index(
        &self,
        node_index: NodeIndex,
        edge_weight_kind_discrim: EdgeWeightKindDiscriminants,
    ) -> WorkspaceSnapshotResult<Vec<NodeIndex>> {
        Ok(self
            .edges_directed_by_index(node_index, Direction::Outgoing)?
            .filter_map(|edge_ref| {
                if edge_weight_kind_discrim == edge_ref.weight().kind().into() {
                    Some(edge_ref.target())
                } else {
                    None
                }
            })
            .collect())
    }

    pub fn all_outgoing_targets(
        &self,
        id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<Vec<NodeWeight>> {
        let mut result = vec![];
        let target_idxs: Vec<NodeIndex> = self
            .edges_directed(id, Direction::Outgoing)?
            .map(|edge_ref| edge_ref.target())
            .collect();

        for target_idx in target_idxs {
            let node_weight = self.get_node_weight(target_idx)?;
            result.push(node_weight.to_owned());
        }

        Ok(result)
    }

    pub fn all_incoming_sources(
        &self,
        id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<Vec<NodeWeight>> {
        let mut result = vec![];
        let source_idxs: Vec<NodeIndex> = self
            .edges_directed(id, Direction::Incoming)?
            .map(|edge_ref| edge_ref.source())
            .collect();

        for source_idx in source_idxs {
            let node_weight = self.get_node_weight(source_idx)?;
            result.push(node_weight.to_owned());
        }

        Ok(result)
    }

    pub fn remove_incoming_edges_of_kind(
        &mut self,
        change_set: &ChangeSetPointer,
        target_id: impl Into<Ulid>,
        kind: EdgeWeightKindDiscriminants,
    ) -> WorkspaceSnapshotResult<()> {
        let target_id = target_id.into();

        let sources = self.incoming_sources_for_edge_weight_kind(target_id, kind)?;
        for source_node_idx in sources {
            let target_node_idx = self.get_node_index_by_id(target_id)?;
            self.remove_edge(change_set, source_node_idx, target_node_idx, kind)?;
        }

        Ok(())
    }

    pub fn remove_node_by_id(&mut self, id: impl Into<Ulid>) -> WorkspaceSnapshotResult<()> {
        let id: Ulid = id.into();
        let node_idx = self.get_node_index_by_id(id)?;
        self.working_copy.remove_node(node_idx);
        self.working_copy.remove_node_id(id);

        Ok(())
    }

    pub fn remove_edge(
        &mut self,
        change_set: &ChangeSetPointer,
        source_node_index: NodeIndex,
        target_node_index: NodeIndex,
        edge_kind: EdgeWeightKindDiscriminants,
    ) -> WorkspaceSnapshotResult<()> {
        Ok(self.working_copy.remove_edge(
            change_set,
            source_node_index,
            target_node_index,
            edge_kind,
        )?)
    }

    /// Perform [`Updates`](Update) using [`self`](WorkspaceSnapshot) as the "to rebase" graph and
    /// another [`snapshot`](WorkspaceSnapshot) as the "onto" graph.
    pub fn perform_updates(
        &mut self,
        to_rebase_change_set: &ChangeSetPointer,
        onto: &mut WorkspaceSnapshot,
        updates: &[Update],
    ) -> WorkspaceSnapshotResult<()> {
        Ok(self
            .working_copy
            .perform_updates(to_rebase_change_set, &onto.working_copy, updates)?)
    }

    /// Mark whether a prop can be used as an input to a function. Props below
    /// Maps and Arrays are not valid inputs. Must only be used when
    /// "finalizing" a schema variant!
    pub fn mark_prop_as_able_to_be_used_as_prototype_arg(
        &mut self,
        node_index: NodeIndex,
    ) -> WorkspaceSnapshotResult<()> {
        self.working_copy
            .update_node_weight(node_index, |node_weight| match node_weight {
                NodeWeight::Prop(prop_inner) => {
                    prop_inner.set_can_be_used_as_prototype_arg(true);
                    Ok(())
                }
                _ => Err(WorkspaceSnapshotGraphError::IncompatibleNodeTypes)?,
            })?;

        Ok(())
    }

    pub fn ordering_node_for_container(
        &self,
        id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<Option<OrderingNodeWeight>> {
        let idx = self.get_node_index_by_id(id)?;
        Ok(self.working_copy.ordering_node_for_container(idx)?)
    }

    pub fn ordered_children_for_node(
        &self,
        id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<Option<Vec<Ulid>>> {
        let idx = self.get_node_index_by_id(id.into())?;
        let mut result = vec![];
        Ok(
            if let Some(idxs) = self.working_copy.ordered_children_for_node(idx)? {
                for idx in idxs {
                    let id = self.get_node_weight(idx)?.id();
                    result.push(id);
                }
                Some(result)
            } else {
                None
            },
        )
    }
}
