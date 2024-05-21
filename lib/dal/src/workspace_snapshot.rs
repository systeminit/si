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

use futures::executor;
use si_pkg::KeyOrIndex;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::task::JoinError;

use petgraph::prelude::*;
pub use petgraph::Direction;
use serde::{Deserialize, Serialize};
use si_data_pg::PgError;
use si_events::{ulid::Ulid, ContentHash, WorkspaceSnapshotAddress};
use strum::IntoEnumIterator;
use telemetry::prelude::*;
use thiserror::Error;
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::action::{Action, ActionError};
use crate::change_set::{ChangeSet, ChangeSetError, ChangeSetId};
use crate::workspace_snapshot::conflict::Conflict;
use crate::workspace_snapshot::edge_weight::{
    EdgeWeight, EdgeWeightError, EdgeWeightKind, EdgeWeightKindDiscriminants,
};
use crate::workspace_snapshot::node_weight::category_node_weight::CategoryNodeKind;
use crate::workspace_snapshot::node_weight::NodeWeight;
use crate::workspace_snapshot::update::Update;
use crate::workspace_snapshot::vector_clock::VectorClockId;
use crate::{
    workspace_snapshot::{graph::WorkspaceSnapshotGraphError, node_weight::NodeWeightError},
    DalContext, TransactionsError, WorkspaceSnapshotGraph,
};

use self::node_weight::{NodeWeightDiscriminants, OrderingNodeWeight};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeInformation {
    pub index: NodeIndex,
    pub node_weight_kind: NodeWeightDiscriminants,
    pub id: Ulid,
}

#[remain::sorted]
#[derive(Error, Debug)]
pub enum WorkspaceSnapshotError {
    #[error("Action error: {0}")]
    Action(#[from] Box<ActionError>),
    #[error("could not find category node of kind: {0:?}")]
    CategoryNodeNotFound(CategoryNodeKind),
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("change set {0} has no workspace snapshot address")]
    ChangeSetMissingWorkspaceSnapshotAddress(ChangeSetId),
    #[error("edge weight error: {0}")]
    EdgeWeight(#[from] EdgeWeightError),
    #[error("join error: {0}")]
    Join(#[from] JoinError),
    #[error("layer db error: {0}")]
    LayerDb(#[from] si_layer_cache::LayerDbError),
    #[error("missing content from store for id: {0}")]
    MissingContentFromStore(Ulid),
    #[error("monotonic error: {0}")]
    Monotonic(#[from] ulid::MonotonicError),
    #[error("NodeWeight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("ordering not found for node with ordered children: {0}")]
    OrderingNotFound(Ulid),
    #[error("si_data_pg error: {0}")]
    Pg(#[from] PgError),
    #[error("postcard error: {0}")]
    Postcard(#[from] postcard::Error),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
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
    #[error("workspace snapshot graph missing at address: {0}")]
    WorkspaceSnapshotGraphMissing(WorkspaceSnapshotAddress),
    #[error("no workspace snapshot was fetched for this dal context")]
    WorkspaceSnapshotNotFetched,
    #[error("Unable to write workspace snapshot")]
    WorkspaceSnapshotNotWritten,
}

pub type WorkspaceSnapshotResult<T> = Result<T, WorkspaceSnapshotError>;

/// The workspace graph. The concurrency types used here to give us interior
/// mutability in the tokio run time are *not* sufficient to prevent data races
/// when operating on the same graph on different threads, since our graph
/// operations are not "atomic" and the graph *WILL* end up being read from
/// different threads while a write operation is still in progress if it is
/// shared between threads for modification. For example after a node is added
/// but *before* the edges necessary to place that node in the right spot in the
/// graph have been added. We need a more general solution here, but for now an
/// example of synchronization when accessing a snapshot across threads can be
/// found in [`crate::job::definition::DependentValuesUpdate`].
#[derive(Debug, Clone)]
pub struct WorkspaceSnapshot {
    address: Arc<RwLock<WorkspaceSnapshotAddress>>,
    //    _created_at: Arc<RwLock<DateTime<Utc>>>,
    /// When the snapshot is fetched from the layer cache (hopefully from memory), it comes back
    /// wrapped in an Arc to prevent cloning the graph (which can get quite large). Graph
    /// operations that never modify the graph will use this read-only copy *until* the graph is
    /// modified.
    read_only_graph: Arc<WorkspaceSnapshotGraph>,

    /// Before the graph is modified, the read_only_graph is copied into this RwLock, and all
    /// subsequent graph operations (both read and write) will need to acquire this lock in order
    /// to read or write to the graph. See the SnapshotReadGuard and SnapshotWriteGuard
    /// implemenations of Deref and DerefMut, and their construction in
    /// working_copy()/working_copy_mut()
    working_copy: Arc<RwLock<Option<WorkspaceSnapshotGraph>>>,

    /// Whether we should perform cycle checks on add edge operations
    cycle_check: Arc<AtomicBool>,
}

/// A pretty dumb attempt to make enabling the cycle check more ergonomic. This
/// will reset the cycle check to false on drop, if nothing else is holding onto
/// the cycle check besides the guard being dropped and the workspace snapshot.
/// We are not being completely atomic in our check, so in concurrent situations
/// we might turn off a cycle check that wants to stay enabled. However the main
/// purpose is to ensure we disable the cycle check automatically on early
/// returns (for errors, for example), and we only enable the cycle check in
/// very narrow situations. If we want to support concurrency here better we can
/// do so in the future.
#[must_use = "if unused the cycle check will be immediately disabled"]
pub struct CycleCheckGuard {
    cycle_check: Arc<AtomicBool>,
}

impl std::ops::Drop for CycleCheckGuard {
    fn drop(&mut self) {
        if Arc::strong_count(&self.cycle_check) <= 2 {
            self.cycle_check
                .store(false, std::sync::atomic::Ordering::Relaxed);
        }
    }
}

#[must_use = "if unused the lock will be released immediately"]
struct SnapshotReadGuard<'a> {
    read_only_graph: Arc<WorkspaceSnapshotGraph>,
    working_copy_read_guard: RwLockReadGuard<'a, Option<WorkspaceSnapshotGraph>>,
}

#[must_use = "if unused the lock will be released immediately"]
struct SnapshotWriteGuard<'a> {
    working_copy_write_guard: RwLockWriteGuard<'a, Option<WorkspaceSnapshotGraph>>,
}

impl<'a> std::ops::Deref for SnapshotReadGuard<'a> {
    type Target = WorkspaceSnapshotGraph;

    fn deref(&self) -> &Self::Target {
        if self.working_copy_read_guard.is_some() {
            let option = &*self.working_copy_read_guard;
            option.as_ref().expect("we confirmed it was some above")
        } else {
            &self.read_only_graph
        }
    }
}

impl<'a> std::ops::Deref for SnapshotWriteGuard<'a> {
    type Target = WorkspaceSnapshotGraph;

    fn deref(&self) -> &Self::Target {
        let option = &*self.working_copy_write_guard;
        option.as_ref().expect(
            "attempted to deref snapshot without copying contents into the mutable working copy",
        )
    }
}

impl<'a> std::ops::DerefMut for SnapshotWriteGuard<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let option = &mut *self.working_copy_write_guard;
        &mut *option.as_mut().expect("attempted to DerefMut a snapshot without copying contents into the mutable working copy")
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
    #[instrument(
        name = "workspace_snapshot.initial",
        level = "debug",
        skip_all,
        fields(
            si.change_set.id = %change_set.id,
        )
    )]
    pub async fn initial(
        ctx: &DalContext,
        change_set: &ChangeSet,
    ) -> WorkspaceSnapshotResult<Self> {
        let mut graph: WorkspaceSnapshotGraph = WorkspaceSnapshotGraph::new(change_set)?;

        // Create the category nodes under root.
        for category_node_kind in CategoryNodeKind::iter() {
            let category_node_index = graph.add_category_node(change_set, category_node_kind)?;
            graph.add_edge(
                graph.root(),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())?,
                category_node_index,
            )?;
        }

        // We do not care about any field other than "working_copy" because "write" will populate
        // them using the assigned working copy.
        let initial = Self {
            address: Arc::new(RwLock::new(WorkspaceSnapshotAddress::nil())),
            read_only_graph: Arc::new(graph),
            working_copy: Arc::new(RwLock::new(None)),
            cycle_check: Arc::new(AtomicBool::new(false)),
        };

        initial.write(ctx, change_set.vector_clock_id()).await?;

        Ok(initial)
    }

    /// Enables cycle checks on calls to [`Self::add_edge`]. Does not force
    /// cycle checks for every [`WorkspaceSnapshotGrpah::add_edge`] operation if
    /// there is a consumer of [`WorkspaceSnapshotGraph`] that calls add_edge
    /// directly. Note that you must hang on to the returned guard or the cycle
    /// check will be disabled immediately
    pub async fn enable_cycle_check(&self) -> CycleCheckGuard {
        self.cycle_check
            .store(true, std::sync::atomic::Ordering::Relaxed);
        CycleCheckGuard {
            cycle_check: self.cycle_check.clone(),
        }
    }

    pub async fn disable_cycle_check(&self) {
        self.cycle_check
            .store(false, std::sync::atomic::Ordering::Relaxed);
    }

    pub async fn cycle_check(&self) -> bool {
        self.cycle_check.load(std::sync::atomic::Ordering::Relaxed)
    }

    #[instrument(
        name = "workspace_snapshot.write",
        level = "debug",
        skip_all,
        fields(
            si.vector_clock.id = %vector_clock_id,
            si.workspace_snapshot.address = Empty,
        )
    )]
    pub async fn write(
        &self,
        ctx: &DalContext,
        vector_clock_id: VectorClockId,
    ) -> WorkspaceSnapshotResult<WorkspaceSnapshotAddress> {
        // Pull out the working copy and clean it up.
        let new_address = {
            let mut working_copy = self.working_copy_mut().await;
            working_copy.cleanup();

            // Mark everything left as seen.
            working_copy.mark_graph_seen(vector_clock_id)?;

            let (new_address, _) = ctx
                .layer_db()
                .workspace_snapshot()
                .write(
                    Arc::new(working_copy.clone()),
                    None,
                    ctx.events_tenancy(),
                    ctx.events_actor(),
                )
                .await?;
            Span::current().record("si.workspace_snapshot.address", new_address.to_string());

            new_address
        };

        // Note, we continue to use the working copy after this, even for reads, since otherwise
        // we'd have to replace the read_only_graph, which would require another thread-safe
        // interior mutability type to store the read only graph in.

        *self.address.write().await = new_address;

        Ok(new_address)
    }

    pub async fn id(&self) -> WorkspaceSnapshotAddress {
        *self.address.read().await
    }

    pub async fn root(&self) -> WorkspaceSnapshotResult<NodeIndex> {
        Ok(self.working_copy().await.root())
    }

    #[instrument(name = "workspace_snapshot.working_copy", level = "debug", skip_all)]
    async fn working_copy(&self) -> SnapshotReadGuard<'_> {
        SnapshotReadGuard {
            read_only_graph: self.read_only_graph.clone(),
            working_copy_read_guard: self.working_copy.read().await,
        }
    }

    #[instrument(
        name = "workspace_snapshot.working_copy_mut",
        level = "debug",
        skip_all
    )]
    async fn working_copy_mut(&self) -> SnapshotWriteGuard<'_> {
        if self.working_copy.read().await.is_none() {
            // Make a copy of the read only graph as our new working copy
            *self.working_copy.write().await = Some(self.read_only_graph.as_ref().clone());
        }

        SnapshotWriteGuard {
            working_copy_write_guard: self.working_copy.write().await,
        }
    }

    pub async fn serialized(&self) -> WorkspaceSnapshotResult<Vec<u8>> {
        Ok(si_layer_cache::db::serialize::to_vec(
            &self.working_copy().await.clone(),
        )?)
    }

    pub async fn from_bytes(bytes: &[u8]) -> WorkspaceSnapshotResult<Self> {
        let graph: WorkspaceSnapshotGraph = si_layer_cache::db::serialize::from_bytes(bytes)?;

        Ok(Self {
            address: Arc::new(RwLock::new(WorkspaceSnapshotAddress::nil())),
            read_only_graph: Arc::new(graph),
            working_copy: Arc::new(RwLock::new(None)),
            cycle_check: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Returns `true` if the graph does not have a cycle in it. This operation
    /// is relatively expensive but necessary to prevent infinite loops.
    pub async fn is_acyclic_directed(&self) -> bool {
        self.working_copy().await.is_acyclic_directed()
    }

    pub async fn add_node(&self, node: NodeWeight) -> WorkspaceSnapshotResult<NodeIndex> {
        let new_node_index = self.working_copy_mut().await.add_node(node)?;
        Ok(new_node_index)
    }

    #[instrument(
        name = "workspace_snapshot.add_ordered_node",
        level = "debug",
        skip_all,
        fields()
    )]
    pub async fn add_ordered_node(
        &self,
        change_set: &ChangeSet,
        node: NodeWeight,
    ) -> WorkspaceSnapshotResult<NodeIndex> {
        let new_node_index = self
            .working_copy_mut()
            .await
            .add_ordered_node(change_set, node)?;
        Ok(new_node_index)
    }

    #[instrument(
        name = "workspace_snapshot.update_content",
        level = "debug",
        skip_all,
        fields()
    )]
    pub async fn update_content(
        &self,
        change_set: &ChangeSet,
        id: Ulid,
        new_content_hash: ContentHash,
    ) -> WorkspaceSnapshotResult<()> {
        Ok(self
            .working_copy_mut()
            .await
            .update_content(change_set, id, new_content_hash)?)
    }

    #[instrument(
        name = "workspace_snapshot.add_edge",
        level = "debug",
        skip_all,
        fields()
    )]
    pub async fn add_edge(
        &self,
        from_node_id: impl Into<Ulid>,
        edge_weight: EdgeWeight,
        to_node_id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<EdgeIndex> {
        let from_node_index = self
            .working_copy()
            .await
            .get_node_index_by_id(from_node_id)?;
        let to_node_index = self.working_copy().await.get_node_index_by_id(to_node_id)?;
        Ok(if self.cycle_check().await {
            let self_clone = self.clone();
            tokio::task::spawn_blocking(move || {
                let mut working_copy = executor::block_on(self_clone.working_copy_mut());
                working_copy.add_edge_with_cycle_check(from_node_index, edge_weight, to_node_index)
            })
            .await??
        } else {
            self.working_copy_mut()
                .await
                .add_edge(from_node_index, edge_weight, to_node_index)?
        })
    }

    // NOTE(nick): this should only be used by the rebaser and in specific scenarios where the
    // indices are definitely correct.
    #[instrument(
        name = "workspace_snapshot.add_edge_unchecked",
        level = "debug",
        skip_all,
        fields()
    )]
    pub async fn add_edge_unchecked(
        &self,
        from_node_index: NodeIndex,
        edge_weight: EdgeWeight,
        to_node_index: NodeIndex,
    ) -> WorkspaceSnapshotResult<EdgeIndex> {
        Ok(self
            .working_copy_mut()
            .await
            .add_edge(from_node_index, edge_weight, to_node_index)?)
    }

    #[instrument(
        name = "workspace_snapshot.add_ordered_edge",
        level = "debug",
        skip_all,
        fields()
    )]
    pub async fn add_ordered_edge(
        &self,
        change_set: &ChangeSet,
        from_node_id: impl Into<Ulid>,
        edge_weight: EdgeWeight,
        to_node_id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<EdgeIndex> {
        let from_node_index = self
            .working_copy()
            .await
            .get_node_index_by_id(from_node_id)?;
        let to_node_index = self.working_copy().await.get_node_index_by_id(to_node_id)?;
        let (edge_index, _) = self.working_copy_mut().await.add_ordered_edge(
            change_set,
            from_node_index,
            edge_weight,
            to_node_index,
        )?;
        Ok(edge_index)
    }

    #[instrument(
        name = "workspace_snapshot.detect_conflicts_and_updates",
        level = "debug",
        skip_all,
        fields()
    )]
    pub async fn detect_conflicts_and_updates(
        &self,
        to_rebase_vector_clock_id: VectorClockId,
        onto_workspace_snapshot: &WorkspaceSnapshot,
        onto_vector_clock_id: VectorClockId,
    ) -> WorkspaceSnapshotResult<(Vec<Conflict>, Vec<Update>)> {
        let self_clone = self.clone();
        let onto_clone = onto_workspace_snapshot.clone();

        Ok(tokio::task::spawn_blocking(move || {
            executor::block_on(self_clone.working_copy()).detect_conflicts_and_updates(
                to_rebase_vector_clock_id,
                &executor::block_on(onto_clone.working_copy()),
                onto_vector_clock_id,
            )
        })
        .await??)
    }

    // NOTE(nick): this should only be used by the rebaser.
    // NOTE(fnichol): ...it isn't though, at least right now... p.s. hey Nick!
    #[instrument(
        name = "workspace_snapshot.edge_endpoints",
        level = "debug",
        skip_all,
        fields()
    )]
    pub async fn edge_endpoints(
        &self,
        edge_index: EdgeIndex,
    ) -> WorkspaceSnapshotResult<(NodeIndex, NodeIndex)> {
        Ok(self.working_copy_mut().await.edge_endpoints(edge_index)?)
    }

    #[instrument(
        name = "workspace_snapshot.import_subgraph",
        level = "debug",
        skip_all,
        fields()
    )]
    pub async fn import_subgraph(
        &self,
        other: &mut Self,
        root_index: NodeIndex,
    ) -> WorkspaceSnapshotResult<()> {
        Ok(self
            .working_copy_mut()
            .await
            .import_subgraph(&*other.working_copy().await, root_index)?)
    }

    /// Calls [`WorkspaceSnapshotGraph::replace_references()`]
    #[instrument(
        name = "workspace_snapshot.replace_references",
        level = "debug",
        skip_all,
        fields()
    )]
    pub async fn replace_references(
        &self,
        original_node_index: NodeIndex,
    ) -> WorkspaceSnapshotResult<()> {
        Ok(self
            .working_copy_mut()
            .await
            .replace_references(original_node_index)?)
    }

    #[instrument(
        name = "workspace_snapshot.get_node_weight_by_id",
        level = "debug",
        skip_all,
        fields()
    )]
    pub async fn get_node_weight_by_id(
        &self,
        id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<NodeWeight> {
        let node_idx = self.get_node_index_by_id(id).await?;
        Ok(self
            .working_copy()
            .await
            .get_node_weight(node_idx)?
            .to_owned())
    }

    #[instrument(
        name = "workspace_snapshot.get_node_weight",
        level = "debug",
        skip_all,
        fields()
    )]
    pub async fn get_node_weight(
        &self,
        node_index: NodeIndex,
    ) -> WorkspaceSnapshotResult<NodeWeight> {
        Ok(self
            .working_copy()
            .await
            .get_node_weight(node_index)?
            .to_owned())
    }

    #[instrument(
        name = "workspace_snapshot.find_equivalent_node",
        level = "debug",
        skip_all,
        fields()
    )]
    pub async fn find_equivalent_node(
        &self,
        id: Ulid,
        lineage_id: Ulid,
    ) -> WorkspaceSnapshotResult<Option<NodeIndex>> {
        Ok(self
            .working_copy()
            .await
            .find_equivalent_node(id, lineage_id)?)
    }

    #[instrument(
        name = "workspace_snapshot.cleanup",
        level = "debug",
        skip_all,
        fields()
    )]
    pub async fn cleanup(&self) -> WorkspaceSnapshotResult<()> {
        self.working_copy_mut().await.cleanup();
        Ok(())
    }

    #[instrument(name = "workspace_snapshot.nodes", level = "debug", skip_all, fields())]
    pub async fn nodes(&self) -> WorkspaceSnapshotResult<Vec<(NodeWeight, NodeIndex)>> {
        Ok(self
            .working_copy()
            .await
            .nodes()
            .map(|(weight, index)| (weight.to_owned(), index))
            .collect())
    }

    #[instrument(name = "workspace_snapshot.edges", level = "debug", skip_all, fields())]
    pub async fn edges(&self) -> WorkspaceSnapshotResult<Vec<(EdgeWeight, NodeIndex, NodeIndex)>> {
        Ok(self
            .working_copy()
            .await
            .edges()
            .map(|(weight, from, to)| (weight.to_owned(), from, to))
            .collect())
    }

    pub async fn dot(&self) {
        self.working_copy().await.dot();
    }

    pub async fn tiny_dot_to_file(&self, suffix: Option<&str>) {
        self.working_copy().await.tiny_dot_to_file(suffix);
    }

    #[instrument(
        name = "workspace_snapshot.get_node_index_by_id",
        level = "debug",
        skip_all,
        fields()
    )]
    pub async fn get_node_index_by_id(
        &self,
        id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<NodeIndex> {
        Ok(self.working_copy().await.get_node_index_by_id(id)?)
    }

    #[instrument(
        name = "workspace_snapshot.try_get_node_index_by_id",
        level = "debug",
        skip_all,
        fields()
    )]
    pub async fn try_get_node_index_by_id(
        &self,
        id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<Option<NodeIndex>> {
        Ok(self.working_copy().await.try_get_node_index_by_id(id)?)
    }

    #[instrument(
        name = "workspace_snapshot.get_latest_node_index",
        level = "debug",
        skip_all,
        fields()
    )]
    pub async fn get_latest_node_index(
        &self,
        node_index: NodeIndex,
    ) -> WorkspaceSnapshotResult<NodeIndex> {
        Ok(self.working_copy().await.get_latest_node_idx(node_index)?)
    }

    #[instrument(name = "workspace_snapshot.find", level = "debug", skip_all, fields())]
    pub async fn find(
        ctx: &DalContext,
        workspace_snapshot_addr: WorkspaceSnapshotAddress,
    ) -> WorkspaceSnapshotResult<Self> {
        let start = tokio::time::Instant::now();

        let snapshot = ctx
            .layer_db()
            .workspace_snapshot()
            .read_wait_for_memory(&workspace_snapshot_addr)
            .await?
            .ok_or(WorkspaceSnapshotError::WorkspaceSnapshotGraphMissing(
                workspace_snapshot_addr,
            ))?;

        debug!("snapshot fetch took: {:?}", start.elapsed());

        Ok(Self {
            address: Arc::new(RwLock::new(workspace_snapshot_addr)),
            read_only_graph: snapshot,
            working_copy: Arc::new(RwLock::new(None)),
            cycle_check: Arc::new(AtomicBool::new(false)),
        })
    }

    pub async fn find_for_change_set(
        ctx: &DalContext,
        change_set_id: ChangeSetId,
    ) -> WorkspaceSnapshotResult<Self> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_opt(
                "SELECT workspace_snapshot_address FROM change_set_pointers WHERE id = $1",
                &[&change_set_id],
            )
            .await?
            .ok_or(
                WorkspaceSnapshotError::ChangeSetMissingWorkspaceSnapshotAddress(change_set_id),
            )?;

        let address: WorkspaceSnapshotAddress = row.try_get("workspace_snapshot_address")?;

        Self::find(ctx, address).await
    }

    #[instrument(
        name = "workspace_snapshot.get_category_node",
        level = "debug",
        skip_all,
        fields()
    )]
    pub async fn get_category_node_or_err(
        &self,
        source: Option<Ulid>,
        kind: CategoryNodeKind,
    ) -> WorkspaceSnapshotResult<Ulid> {
        self.get_category_node(source, kind)
            .await?
            .ok_or(WorkspaceSnapshotError::CategoryNodeNotFound(kind))
    }

    #[instrument(
        name = "workspace_snapshot.get_category_node_opt",
        level = "debug",
        skip_all,
        fields()
    )]
    pub async fn get_category_node(
        &self,
        source: Option<Ulid>,
        kind: CategoryNodeKind,
    ) -> WorkspaceSnapshotResult<Option<Ulid>> {
        Ok(self
            .working_copy()
            .await
            .get_category_node(source, kind)?
            .map(|(category_node_id, _)| category_node_id))
    }

    #[instrument(
        name = "workspace_snapshot.edges_directed",
        level = "debug",
        skip_all,
        fields()
    )]
    pub async fn edges_directed(
        &self,
        id: impl Into<Ulid>,
        direction: Direction,
    ) -> WorkspaceSnapshotResult<Vec<(EdgeWeight, NodeIndex, NodeIndex)>> {
        let node_index = self.working_copy().await.get_node_index_by_id(id)?;
        Ok(self
            .working_copy()
            .await
            .edges_directed(node_index, direction)
            .map(|edge_ref| {
                (
                    edge_ref.weight().to_owned(),
                    edge_ref.source(),
                    edge_ref.target(),
                )
            })
            .collect())
    }

    #[instrument(
        name = "workspace_snapshot.edges_directed_for_edge_weight_kind",
        level = "debug",
        skip_all,
        fields()
    )]
    pub async fn edges_directed_for_edge_weight_kind(
        &self,
        id: impl Into<Ulid>,
        direction: Direction,
        edge_kind: EdgeWeightKindDiscriminants,
    ) -> WorkspaceSnapshotResult<Vec<(EdgeWeight, NodeIndex, NodeIndex)>> {
        let node_index = self.working_copy().await.get_node_index_by_id(id)?;

        Ok(self
            .working_copy()
            .await
            .edges_directed_for_edge_weight_kind(node_index, direction, edge_kind))
    }

    #[instrument(
        name = "workspace_snapshot.edges_directed_by_index",
        level = "debug",
        skip_all,
        fields()
    )]
    pub async fn edges_directed_by_index(
        &self,
        node_index: NodeIndex,
        direction: Direction,
    ) -> WorkspaceSnapshotResult<Vec<(EdgeWeight, NodeIndex, NodeIndex)>> {
        Ok(self
            .working_copy()
            .await
            .edges_directed(node_index, direction)
            .map(|edge_ref| {
                (
                    edge_ref.weight().to_owned(),
                    edge_ref.source(),
                    edge_ref.target(),
                )
            })
            .collect())
    }

    #[instrument(
        name = "workspace_snapshot.remove_all_edges",
        level = "debug",
        skip_all,
        fields()
    )]
    pub async fn remove_all_edges(
        &self,
        change_set: &ChangeSet,
        id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<()> {
        let id = id.into();
        for (edge_weight, source, target) in self.edges_directed(id, Direction::Outgoing).await? {
            self.remove_edge(change_set, source, target, edge_weight.kind().into())
                .await?;
        }
        for (edge_weight, source, target) in self.edges_directed(id, Direction::Incoming).await? {
            self.remove_edge(change_set, source, target, edge_weight.kind().into())
                .await?;
        }
        Ok(())
    }

    #[instrument(
        name = "workspace_snapshot.incoming_sources_for_edge_weight_kind",
        level = "debug",
        skip_all,
        fields()
    )]
    pub async fn incoming_sources_for_edge_weight_kind(
        &self,
        id: impl Into<Ulid>,
        edge_weight_kind_discrim: EdgeWeightKindDiscriminants,
    ) -> WorkspaceSnapshotResult<Vec<NodeIndex>> {
        Ok(self
            .edges_directed(id.into(), Direction::Incoming)
            .await?
            .into_iter()
            .filter_map(|(edge_weight, source_idx, _)| {
                if edge_weight_kind_discrim == edge_weight.kind().into() {
                    Some(source_idx)
                } else {
                    None
                }
            })
            .collect())
    }

    #[instrument(
        name = "workspace_snapshot.outgoing_targets_for_edge_weight_kind",
        level = "debug",
        skip_all,
        fields()
    )]
    pub async fn outgoing_targets_for_edge_weight_kind(
        &self,
        id: impl Into<Ulid>,
        edge_weight_kind_discrim: EdgeWeightKindDiscriminants,
    ) -> WorkspaceSnapshotResult<Vec<NodeIndex>> {
        let id = id.into();
        Ok(self
            .edges_directed(id, Direction::Outgoing)
            .await?
            .into_iter()
            .filter_map(|(edge_weight, _, target_idx)| {
                if edge_weight_kind_discrim == edge_weight.kind().into() {
                    Some(target_idx)
                } else {
                    None
                }
            })
            .collect())
    }

    #[instrument(
        name = "workspace_snapshot.outgoing_targets_for_edge_weight_kind_by_index",
        level = "debug",
        skip_all,
        fields()
    )]
    pub async fn outgoing_targets_for_edge_weight_kind_by_index(
        &self,
        node_index: NodeIndex,
        edge_weight_kind_discrim: EdgeWeightKindDiscriminants,
    ) -> WorkspaceSnapshotResult<Vec<NodeIndex>> {
        Ok(self
            .edges_directed_by_index(node_index, Direction::Outgoing)
            .await?
            .into_iter()
            .filter_map(|(edge_weight, _, target_idx)| {
                if edge_weight_kind_discrim == edge_weight.kind().into() {
                    Some(target_idx)
                } else {
                    None
                }
            })
            .collect())
    }

    #[instrument(
        name = "workspace_snapshot.all_outgoing_targets",
        level = "debug",
        skip_all,
        fields()
    )]
    pub async fn all_outgoing_targets(
        &self,
        id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<Vec<NodeWeight>> {
        let mut result = vec![];
        let target_idxs: Vec<NodeIndex> = self
            .edges_directed(id, Direction::Outgoing)
            .await?
            .into_iter()
            .map(|(_, _, target_idx)| target_idx)
            .collect();

        for target_idx in target_idxs {
            let node_weight = self.get_node_weight(target_idx).await?;
            result.push(node_weight.to_owned());
        }

        Ok(result)
    }

    #[instrument(
        name = "workspace_snapshot.all_incoming_sources",
        level = "debug",
        skip_all,
        fields()
    )]
    pub async fn all_incoming_sources(
        &self,
        id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<Vec<NodeWeight>> {
        let mut result = vec![];
        let source_idxs: Vec<NodeIndex> = self
            .edges_directed(id, Direction::Incoming)
            .await?
            .into_iter()
            .map(|(_, source_idx, _)| source_idx)
            .collect();

        for source_idx in source_idxs {
            let node_weight = self.get_node_weight(source_idx).await?;
            result.push(node_weight.to_owned());
        }

        Ok(result)
    }

    #[instrument(
        name = "workspace_snapshot.remove_incoming_edges_of_kind",
        level = "debug",
        skip_all,
        fields()
    )]
    pub async fn remove_incoming_edges_of_kind(
        &self,
        change_set: &ChangeSet,
        target_id: impl Into<Ulid>,
        kind: EdgeWeightKindDiscriminants,
    ) -> WorkspaceSnapshotResult<()> {
        let target_id = target_id.into();

        let sources = self
            .incoming_sources_for_edge_weight_kind(target_id, kind)
            .await?;
        for source_node_idx in sources {
            let target_node_idx = self.get_node_index_by_id(target_id).await?;
            self.remove_edge(change_set, source_node_idx, target_node_idx, kind)
                .await?;
        }

        Ok(())
    }

    pub async fn get_edges_between_nodes(
        &self,
        from_node_id: Ulid,
        to_node_id: Ulid,
    ) -> WorkspaceSnapshotResult<Vec<EdgeWeight>> {
        let from_node_index = self.get_node_index_by_id(from_node_id).await?;

        let to_node_index = self.get_node_index_by_id(to_node_id).await?;
        let edges = self
            .edges()
            .await?
            .into_iter()
            .filter_map(|(edge, node_from, node_to)| {
                if node_from == from_node_index && node_to == to_node_index {
                    Some(edge)
                } else {
                    None
                }
            })
            .collect();
        Ok(edges)
    }

    #[instrument(
        name = "workspace_snapshot.remove_node_by_id",
        level = "debug",
        skip_all,
        fields()
    )]
    pub async fn remove_node_by_id(
        &self,
        change_set: &ChangeSet,
        id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<()> {
        let id: Ulid = id.into();
        let node_idx = self.get_node_index_by_id(id).await?;
        self.remove_all_edges(change_set, id).await?;
        self.working_copy_mut().await.remove_node(node_idx);
        self.working_copy_mut().await.remove_node_id(id);

        Ok(())
    }

    #[instrument(
        name = "workspace_snapshot.remove_edge",
        level = "debug",
        skip_all,
        fields()
    )]
    pub async fn remove_edge(
        &self,
        change_set: &ChangeSet,
        source_node_index: NodeIndex,
        target_node_index: NodeIndex,
        edge_kind: EdgeWeightKindDiscriminants,
    ) -> WorkspaceSnapshotResult<()> {
        Ok(self.working_copy_mut().await.remove_edge(
            change_set,
            source_node_index,
            target_node_index,
            edge_kind,
        )?)
    }

    #[instrument(
        name = "workspace_snapshot.remove_edge_for_ulids",
        level = "debug",
        skip_all,
        fields()
    )]
    pub async fn remove_edge_for_ulids(
        &self,
        change_set: &ChangeSet,
        source_node_id: impl Into<Ulid>,
        target_node_id: impl Into<Ulid>,
        edge_kind: EdgeWeightKindDiscriminants,
    ) -> WorkspaceSnapshotResult<()> {
        let source_node_index = self
            .working_copy()
            .await
            .get_node_index_by_id(source_node_id)?;
        let target_node_index = self
            .working_copy()
            .await
            .get_node_index_by_id(target_node_id)?;
        self.remove_edge(change_set, source_node_index, target_node_index, edge_kind)
            .await
    }

    /// Perform [`Updates`](Update) using [`self`](WorkspaceSnapshot) as the "to rebase" graph and
    /// another [`snapshot`](WorkspaceSnapshot) as the "onto" graph.
    #[instrument(
        name = "workspace_snapshot.perform_updates",
        level = "debug",
        skip_all,
        fields()
    )]
    pub async fn perform_updates(
        &self,
        to_rebase_change_set: &ChangeSet,
        onto: &WorkspaceSnapshot,
        updates: &[Update],
    ) -> WorkspaceSnapshotResult<()> {
        Ok(self.working_copy_mut().await.perform_updates(
            to_rebase_change_set,
            &*onto.working_copy().await,
            updates,
        )?)
    }

    /// Mark whether a prop can be used as an input to a function. Props below
    /// Maps and Arrays are not valid inputs. Must only be used when
    /// "finalizing" a schema variant!
    #[instrument(
        name = "workspace_snapshot.mark_prop_as_able_to_be_used_as_prototype_arg",
        level = "debug",
        skip_all,
        fields()
    )]
    pub async fn mark_prop_as_able_to_be_used_as_prototype_arg(
        &self,
        node_index: NodeIndex,
    ) -> WorkspaceSnapshotResult<()> {
        self.working_copy_mut()
            .await
            .update_node_weight(node_index, |node_weight| match node_weight {
                NodeWeight::Prop(prop_inner) => {
                    prop_inner.set_can_be_used_as_prototype_arg(true);
                    Ok(())
                }
                _ => Err(WorkspaceSnapshotGraphError::IncompatibleNodeTypes)?,
            })?;

        Ok(())
    }

    #[instrument(
        name = "workspace_snapshot.ordering_node_for_container",
        level = "debug",
        skip_all,
        fields()
    )]
    pub async fn ordering_node_for_container(
        &self,
        id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<Option<OrderingNodeWeight>> {
        let idx = self.get_node_index_by_id(id).await?;
        Ok(self.working_copy().await.ordering_node_for_container(idx)?)
    }

    #[instrument(
        name = "workspace_snapshot.ordered_children_for_node",
        level = "debug",
        skip_all,
        fields()
    )]
    pub async fn ordered_children_for_node(
        &self,
        id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<Option<Vec<Ulid>>> {
        let idx = self.get_node_index_by_id(id.into()).await?;
        let mut result = vec![];
        Ok(
            if let Some(idxs) = self.working_copy().await.ordered_children_for_node(idx)? {
                for idx in idxs {
                    let id = self.get_node_weight(idx).await?.id();
                    result.push(id);
                }
                Some(result)
            } else {
                None
            },
        )
    }

    #[instrument(
        name = "workspace_snapshot.index_or_key_of_child_entry",
        level = "debug",
        skip_all,
        fields()
    )]
    pub async fn index_or_key_of_child_entry(
        &self,
        id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<Option<KeyOrIndex>> {
        // First, let's see if this node has an incoming, ordinal edge which means
        // it has an index, in this case, it's an element in an Array
        let maybe_id = id.into();
        let maybe_ordering_node = self
            .incoming_sources_for_edge_weight_kind(maybe_id, EdgeWeightKindDiscriminants::Ordinal)
            .await
            .map_or(None, |node| node.first().cloned());

        if let Some(maybe_ordering_node) = maybe_ordering_node {
            // there's an ordering node, so let's grab the edgeweight which includes a
            // a vec with the ids for all of the ordered children.
            let order_node_weight = self
                .get_node_weight(maybe_ordering_node)
                .await?
                .get_ordering_node_weight()?;
            let index = order_node_weight.get_index_for_id(maybe_id)?;

            return Ok(Some(KeyOrIndex::Index(index)));
        }

        // now let's see if we have a child entry for a Map
        let maybe_containing_node = self
            .edges_directed_for_edge_weight_kind(
                maybe_id,
                Direction::Incoming,
                EdgeWeightKindDiscriminants::Contain,
            )
            .await
            .map_or(None, |node| node.first().cloned());
        if let Some((edge_weight, _, _)) = maybe_containing_node {
            // grab the key from the edge weight
            if let EdgeWeightKind::Contain(Some(contain_key)) = edge_weight.kind() {
                return Ok(Some(KeyOrIndex::Key(contain_key.to_string())));
            }
        }
        Ok(None)
    }

    /// Returns whether or not any Actions were dispatched.
    pub async fn dispatch_actions(ctx: &DalContext) -> WorkspaceSnapshotResult<bool> {
        let mut did_dispatch = false;
        for dispatchable_ation_id in Action::eligible_to_dispatch(ctx).await.map_err(Box::new)? {
            Action::dispatch_action(ctx, dispatchable_ation_id)
                .await
                .map_err(Box::new)?;
            did_dispatch = true;
        }

        Ok(did_dispatch)
    }

    async fn find_existing_dependent_value_root(
        &self,
        change_set: &ChangeSet,
        value_id: Ulid,
    ) -> WorkspaceSnapshotResult<(Ulid, Option<Ulid>)> {
        let dv_category_id = match self
            .get_category_node(None, CategoryNodeKind::DependentValueRoots)
            .await?
        {
            Some(dv_category_id) => dv_category_id,
            None => {
                let mut working_copy = self.working_copy_mut().await;
                let root_idx = working_copy.root();
                let category_node_idx = working_copy
                    .add_category_node(change_set, CategoryNodeKind::DependentValueRoots)?;
                working_copy.add_edge(
                    root_idx,
                    EdgeWeight::new(change_set, EdgeWeightKind::new_use())?,
                    category_node_idx,
                )?;

                working_copy.get_node_weight(category_node_idx)?.id()
            }
        };

        for dv_node_idx in self
            .outgoing_targets_for_edge_weight_kind(dv_category_id, EdgeWeightKindDiscriminants::Use)
            .await?
        {
            let dv_value_node_weight = self
                .get_node_weight(dv_node_idx)
                .await?
                .get_dependent_value_root_node_weight()?;

            if value_id == dv_value_node_weight.value_id() {
                return Ok((dv_category_id, Some(dv_value_node_weight.id())));
            }
        }

        Ok((dv_category_id, None))
    }

    pub async fn add_dependent_value_root(
        &self,
        change_set: &ChangeSet,
        value_id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<()> {
        let value_id = value_id.into();
        let (dv_category_id, existing_value_root_id) = self
            .find_existing_dependent_value_root(change_set, value_id)
            .await?;

        match existing_value_root_id {
            Some(value_root_id) => {
                let existing_value_index = self.get_node_index_by_id(value_root_id).await?;
                let value_root_node_weight = self
                    .get_node_weight_by_id(value_root_id)
                    .await?
                    .get_dependent_value_root_node_weight()?
                    .touch(change_set)?;
                self.add_node(NodeWeight::DependentValueRoot(value_root_node_weight))
                    .await?;
                self.replace_references(existing_value_index).await?;
            }
            None => {
                let new_dependent_value_node =
                    NodeWeight::new_dependent_value_root(change_set, value_id)?;
                let new_dv_node_id = new_dependent_value_node.id();
                self.add_node(new_dependent_value_node).await?;

                self.add_edge(
                    dv_category_id,
                    EdgeWeight::new(change_set, EdgeWeightKind::new_use())?,
                    new_dv_node_id,
                )
                .await?;
            }
        }

        Ok(())
    }

    pub async fn remove_dependent_value_root(
        &self,
        change_set: &ChangeSet,
        value_id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<()> {
        let value_id = value_id.into();
        let (_, existing_value_id) = self
            .find_existing_dependent_value_root(change_set, value_id)
            .await?;

        if let Some(existing_id) = existing_value_id {
            self.remove_node_by_id(change_set, existing_id).await?;
        }

        Ok(())
    }

    pub async fn has_dependent_value_roots(&self) -> WorkspaceSnapshotResult<bool> {
        Ok(
            match self
                .get_category_node(None, CategoryNodeKind::DependentValueRoots)
                .await?
            {
                Some(dv_category_id) => !self
                    .outgoing_targets_for_edge_weight_kind(
                        dv_category_id,
                        EdgeWeightKindDiscriminants::Use,
                    )
                    .await?
                    .is_empty(),
                None => false,
            },
        )
    }

    /// Removes all the dependent value nodes from the category and returns the value_ids
    pub async fn take_dependent_values(
        &self,
        change_set: &ChangeSet,
    ) -> WorkspaceSnapshotResult<Vec<Ulid>> {
        let dv_category_id = match self
            .get_category_node(None, CategoryNodeKind::DependentValueRoots)
            .await?
        {
            Some(cat_id) => cat_id,
            None => {
                return Ok(vec![]);
            }
        };

        let mut value_ids = vec![];
        let mut pending_removes = vec![];

        for dv_node_idx in self
            .outgoing_targets_for_edge_weight_kind(dv_category_id, EdgeWeightKindDiscriminants::Use)
            .await?
        {
            let dv_value_node_weight = self
                .get_node_weight(dv_node_idx)
                .await?
                .get_dependent_value_root_node_weight()?;
            value_ids.push(dv_value_node_weight.value_id());
            pending_removes.push(dv_value_node_weight.id());
        }

        for to_remove_id in pending_removes {
            self.remove_node_by_id(change_set, to_remove_id).await?;
        }

        Ok(value_ids)
    }

    /// List all the `value_ids` from the dependent value nodes in the category.
    pub async fn list_dependent_value_value_ids(&self) -> WorkspaceSnapshotResult<Vec<Ulid>> {
        let dv_category_id = match self
            .get_category_node(None, CategoryNodeKind::DependentValueRoots)
            .await?
        {
            Some(cat_id) => cat_id,
            None => {
                return Ok(vec![]);
            }
        };

        let mut value_ids = vec![];
        for dv_node_idx in self
            .outgoing_targets_for_edge_weight_kind(dv_category_id, EdgeWeightKindDiscriminants::Use)
            .await?
        {
            let dv_value_node_weight = self
                .get_node_weight(dv_node_idx)
                .await?
                .get_dependent_value_root_node_weight()?;
            value_ids.push(dv_value_node_weight.value_id());
        }

        Ok(value_ids)
    }
}
