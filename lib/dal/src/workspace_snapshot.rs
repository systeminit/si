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

pub mod content_address;
pub mod edge_weight;
pub mod graph;
pub mod lamport_clock;
pub mod migrator;
pub mod node_weight;
pub mod update;
pub mod vector_clock;

use graph::correct_transforms::correct_transforms;
use graph::detect_updates::Update;
use graph::{RebaseBatch, WorkspaceSnapshotGraph};
use node_weight::traits::CorrectTransformsError;
use std::collections::HashSet;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use petgraph::prelude::*;
pub use petgraph::Direction;
use serde::{Deserialize, Serialize};
use si_data_pg::PgError;
use si_events::{ulid::Ulid, ContentHash, WorkspaceSnapshotAddress};
use si_layer_cache::LayerDbError;
use strum::IntoEnumIterator;
use telemetry::prelude::*;
use thiserror::Error;
use tokio::sync::{Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard};
use tokio::task::JoinError;

use self::node_weight::{NodeWeightDiscriminants, OrderingNodeWeight};
use crate::action::{Action, ActionError};
use crate::attribute::prototype::argument::{
    AttributePrototypeArgument, AttributePrototypeArgumentError, AttributePrototypeArgumentId,
};
use crate::change_set::{ChangeSetError, ChangeSetId};
use crate::component::inferred_connection_graph::InferredConnectionGraph;
use crate::slow_rt::{self, SlowRuntimeError};
use crate::workspace_snapshot::content_address::ContentAddressDiscriminants;
use crate::workspace_snapshot::edge_weight::{
    EdgeWeight, EdgeWeightKind, EdgeWeightKindDiscriminants,
};
use crate::workspace_snapshot::graph::LineageId;
use crate::workspace_snapshot::node_weight::category_node_weight::CategoryNodeKind;
use crate::workspace_snapshot::node_weight::NodeWeight;
use crate::{
    pk, AttributeValueId, Component, ComponentError, ComponentId, Workspace, WorkspaceError,
};
use crate::{
    workspace_snapshot::{graph::WorkspaceSnapshotGraphError, node_weight::NodeWeightError},
    DalContext, TransactionsError, WorkspaceSnapshotGraphV2,
};

pk!(NodeId);
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeInformation {
    pub node_weight_kind: NodeWeightDiscriminants,
    pub id: NodeId,
}

impl From<&NodeWeight> for NodeInformation {
    fn from(node_weight: &NodeWeight) -> Self {
        Self {
            node_weight_kind: node_weight.into(),
            id: node_weight.id().into(),
        }
    }
}

#[remain::sorted]
#[derive(Error, Debug)]
pub enum WorkspaceSnapshotError {
    #[error("Action error: {0}")]
    Action(#[from] Box<ActionError>),
    #[error("Attribute Prototype Argument: {0}")]
    AttributePrototypeArgument(#[from] Box<AttributePrototypeArgumentError>),
    #[error("could not find category node of kind: {0:?}")]
    CategoryNodeNotFound(CategoryNodeKind),
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("change set {0} has no workspace snapshot address")]
    ChangeSetMissingWorkspaceSnapshotAddress(ChangeSetId),
    #[error("Component error: {0}")]
    Component(#[from] Box<ComponentError>),
    #[error("error correcting transforms: {0}")]
    CorrectTransforms(#[from] CorrectTransformsError),
    #[error("join error: {0}")]
    Join(#[from] JoinError),
    #[error("layer db error: {0}")]
    LayerDb(#[from] si_layer_cache::LayerDbError),
    #[error("missing content from store for id: {0}")]
    MissingContentFromStore(Ulid),
    #[error("could not find a max vector clock for change set id {0}")]
    MissingVectorClockForChangeSet(ChangeSetId),
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
    #[error("recently seen clocks missing for change set id {0}")]
    RecentlySeenClocksMissing(ChangeSetId),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("slow runtime error: {0}")]
    SlowRuntime(#[from] SlowRuntimeError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("could not acquire lock: {0}")]
    TryLock(#[from] tokio::sync::TryLockError),
    #[error("unable to forward migrate snapshot: {0}")]
    UnableToForwardMigrateSnapshot(String),
    #[error("Unexpected edge source {0} for target {1} and edge weight type {0:?}")]
    UnexpectedEdgeSource(Ulid, Ulid, EdgeWeightKindDiscriminants),
    #[error("Unexpected edge target {0} for source {1} and edge weight type {0:?}")]
    UnexpectedEdgeTarget(Ulid, Ulid, EdgeWeightKindDiscriminants),
    #[error("Unexpected number of incoming edges of type {0:?} for node type {1:?} with id {2}")]
    UnexpectedNumberOfIncomingEdges(EdgeWeightKindDiscriminants, NodeWeightDiscriminants, Ulid),
    #[error("Workspace error: {0}")]
    Workspace(#[from] Box<WorkspaceError>),
    #[error("Tenancy missing Workspace")]
    WorkspaceMissing,
    #[error("WorkspaceSnapshotGraph error: {0}")]
    WorkspaceSnapshotGraph(#[from] WorkspaceSnapshotGraphError),
    #[error("workspace snapshot graph missing at address: {0}")]
    WorkspaceSnapshotGraphMissing(WorkspaceSnapshotAddress),
    #[error("no workspace snapshot was fetched for this dal context")]
    WorkspaceSnapshotNotFetched,
    #[error("workspace snapshot {0} is not yet migrated to the latest version")]
    WorkspaceSnapshotNotMigrated(WorkspaceSnapshotAddress),
    #[error("Unable to write workspace snapshot")]
    WorkspaceSnapshotNotWritten,
}

impl WorkspaceSnapshotError {
    pub fn is_node_with_id_not_found(&self) -> bool {
        matches!(
            self,
            Self::WorkspaceSnapshotGraph(
                crate::workspace_snapshot::graph::WorkspaceSnapshotGraphError::NodeWithIdNotFound(
                    _,
                ),
            )
        )
    }
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
    working_copy: Arc<RwLock<Option<WorkspaceSnapshotGraphV2>>>,

    /// Whether we should perform cycle checks on add edge operations
    cycle_check: Arc<AtomicBool>,

    /// A hashset to prevent adding duplicate roots to the workspace in a single edit session
    dvu_roots: Arc<Mutex<HashSet<Ulid>>>,

    /// A cached version of the inferred connection graph for this snapshot
    inferred_connection_graph: Arc<RwLock<Option<InferredConnectionGraph>>>,
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
    working_copy_read_guard: RwLockReadGuard<'a, Option<WorkspaceSnapshotGraphV2>>,
}

#[must_use = "if unused the lock will be released immediately"]
struct SnapshotWriteGuard<'a> {
    working_copy_write_guard: RwLockWriteGuard<'a, Option<WorkspaceSnapshotGraphV2>>,
}

impl<'a> std::ops::Deref for SnapshotReadGuard<'a> {
    type Target = WorkspaceSnapshotGraphV2;

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
    type Target = WorkspaceSnapshotGraphV2;

    fn deref(&self) -> &Self::Target {
        let option = &*self.working_copy_write_guard;
        option.as_ref().expect(
            "attempted to deref snapshot without copying contents into the mutable working copy",
        )
    }
}

impl<'a> std::ops::DerefMut for SnapshotWriteGuard<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let option: &mut Option<WorkspaceSnapshotGraphV2> = &mut self.working_copy_write_guard;
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
    #[instrument(name = "workspace_snapshot.initial", level = "debug", skip_all)]
    pub async fn initial(ctx: &DalContext) -> WorkspaceSnapshotResult<Self> {
        let mut graph: WorkspaceSnapshotGraphV2 = WorkspaceSnapshotGraphV2::new()?;

        // Create the category nodes under root.
        for category_node_kind in CategoryNodeKind::iter() {
            let id = graph.generate_ulid()?;
            let lineage_id = graph.generate_ulid()?;
            let category_node_index =
                graph.add_category_node(id, lineage_id, category_node_kind)?;
            graph.add_edge(
                graph.root(),
                EdgeWeight::new(EdgeWeightKind::new_use()),
                category_node_index,
            )?;
        }

        // We do not care about any field other than "working_copy" because
        // "write" will populate them using the assigned working copy.
        let initial = Self {
            address: Arc::new(RwLock::new(WorkspaceSnapshotAddress::nil())),
            read_only_graph: Arc::new(WorkspaceSnapshotGraph::V2(graph)),
            working_copy: Arc::new(RwLock::new(None)),
            cycle_check: Arc::new(AtomicBool::new(false)),
            dvu_roots: Arc::new(Mutex::new(HashSet::new())),
            inferred_connection_graph: Arc::new(RwLock::new(None)),
        };

        initial.write(ctx).await?;

        Ok(initial)
    }

    pub async fn generate_ulid(&self) -> WorkspaceSnapshotResult<Ulid> {
        Ok(self.working_copy_mut().await.generate_ulid()?)
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

    /// Calculates the set of updates for the current snapshot against its working copy
    pub async fn current_rebase_batch(&self) -> WorkspaceSnapshotResult<Option<RebaseBatch>> {
        let self_clone = self.clone();
        let updates = slow_rt::spawn(async move {
            let mut working_copy = self_clone.working_copy_mut().await;
            working_copy.cleanup();

            self_clone.read_only_graph.detect_updates(&working_copy)
        })?
        .await?;

        Ok((!updates.is_empty()).then_some(RebaseBatch::new(updates)))
    }

    /// Calculates the set of updates made to `updated_snapshot` against
    /// `base_snapshot`. For these updates to be correct, `updated_snapshot` must
    /// have already seen all the changes made to `base_snapshot`
    pub async fn calculate_rebase_batch(
        base_snapshot: Arc<WorkspaceSnapshot>,
        updated_snapshot: Arc<WorkspaceSnapshot>,
    ) -> WorkspaceSnapshotResult<Option<RebaseBatch>> {
        let updates = slow_rt::spawn(async move {
            let updates = base_snapshot.detect_updates(&updated_snapshot).await?;

            Ok::<Vec<Update>, WorkspaceSnapshotError>(updates)
        })?
        .await??;

        Ok((!updates.is_empty()).then_some(RebaseBatch::new(updates)))
    }

    /// Given the state of the graph in `self`, and a set of updates, transform
    /// those updates to ensure an incorrect graph is not created
    #[instrument(
        name = "workspace_snapshot.correct_transforms",
        level = "info",
        skip_all
    )]
    pub async fn correct_transforms(
        &self,
        updates: Vec<Update>,
        from_different_change_set: bool,
    ) -> WorkspaceSnapshotResult<Vec<Update>> {
        let self_clone = self.clone();
        Ok(slow_rt::spawn(async move {
            correct_transforms(
                &*self_clone.working_copy().await,
                updates,
                from_different_change_set,
            )
        })?
        .await??)
    }

    #[instrument(
        name = "workspace_snapshot.write",
        level = "debug",
        skip_all,
        fields(
            si.workspace_snapshot.address = Empty,
        )
    )]
    pub async fn write(
        &self,
        ctx: &DalContext,
    ) -> WorkspaceSnapshotResult<WorkspaceSnapshotAddress> {
        // Pull out the working copy and clean it up.
        let new_address = {
            // Everything needs to be pulled out here so we can throw it into
            // the closure that will run on the "slow runtime"
            let self_clone = self.clone();
            let layer_db = ctx.layer_db().clone();
            let events_tenancy = ctx.events_tenancy();
            let events_actor = ctx.events_actor();

            // The write includes a potentially expensive serialization
            // operation, so we throw it onto the "slow" runtime, the one not
            // listening for requests/processing a nats queue
            let new_address = slow_rt::spawn(async move {
                let mut working_copy = self_clone.working_copy_mut().await;
                working_copy.cleanup();

                let (new_address, _) = layer_db
                    .workspace_snapshot()
                    .write(
                        Arc::new(WorkspaceSnapshotGraph::V2(working_copy.clone())),
                        None,
                        events_tenancy,
                        events_actor,
                    )
                    .await?;

                Ok::<WorkspaceSnapshotAddress, WorkspaceSnapshotError>(new_address)
            })?
            .await??;

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

    #[instrument(name = "workspace_snapshot.working_copy", level = "trace", skip_all)]
    async fn working_copy(&self) -> SnapshotReadGuard<'_> {
        SnapshotReadGuard {
            read_only_graph: self.read_only_graph.clone(),
            working_copy_read_guard: self.working_copy.read().await,
        }
    }

    #[instrument(
        name = "workspace_snapshot.working_copy_mut",
        level = "trace",
        skip_all
    )]
    async fn working_copy_mut(&self) -> SnapshotWriteGuard<'_> {
        let mut working_copy = self.working_copy.write().await;
        if working_copy.is_none() {
            // Make a copy of the read only graph as our new working copy
            *working_copy = Some(self.read_only_graph.inner().clone());
        }
        SnapshotWriteGuard {
            working_copy_write_guard: working_copy,
        }
    }

    /// Discard all changes in the working copy and return the graph to the
    /// version fetched from the layer db
    pub async fn revert(&self) {
        if self.working_copy.read().await.is_some() {
            *self.working_copy.write().await = None
        }
    }

    pub async fn serialized(&self) -> WorkspaceSnapshotResult<Vec<u8>> {
        let graph = self.working_copy().await.clone();
        Ok(si_layer_cache::db::serialize::to_vec(
            &WorkspaceSnapshotGraph::V2(graph),
        )?)
    }

    pub async fn from_bytes(bytes: &[u8]) -> WorkspaceSnapshotResult<Self> {
        let graph: Arc<WorkspaceSnapshotGraph> = si_layer_cache::db::serialize::from_bytes(bytes)?;

        Ok(Self {
            address: Arc::new(RwLock::new(WorkspaceSnapshotAddress::nil())),
            read_only_graph: graph,
            working_copy: Arc::new(RwLock::new(None)),
            cycle_check: Arc::new(AtomicBool::new(false)),
            dvu_roots: Arc::new(Mutex::new(HashSet::new())),
            inferred_connection_graph: Arc::new(RwLock::new(None)),
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
    pub async fn add_ordered_node(&self, node: NodeWeight) -> WorkspaceSnapshotResult<NodeIndex> {
        let new_node_index = self.working_copy_mut().await.add_ordered_node(node)?;
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
        id: Ulid,
        new_content_hash: ContentHash,
    ) -> WorkspaceSnapshotResult<()> {
        Ok(self
            .working_copy_mut()
            .await
            .update_content(id, new_content_hash)?)
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
            slow_rt::spawn(async move {
                let mut working_copy = self_clone.working_copy_mut().await;
                working_copy.add_edge_with_cycle_check(from_node_index, edge_weight, to_node_index)
            })?
            .await??
        } else {
            self.working_copy_mut()
                .await
                .add_edge(from_node_index, edge_weight, to_node_index)?
        })
    }

    /// Add an edge to the graph, bypassing any cycle checks and using node
    /// indices directly.  Use with care, since node indices are only reliably
    /// if the graph has not yet been modified.
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
            from_node_index,
            edge_weight,
            to_node_index,
        )?;
        Ok(edge_index)
    }

    #[instrument(
        name = "workspace_snapshot.detect_updates",
        level = "debug",
        skip_all,
        fields()
    )]
    pub async fn detect_updates(
        &self,
        onto_workspace_snapshot: &WorkspaceSnapshot,
    ) -> WorkspaceSnapshotResult<Vec<Update>> {
        let self_clone = self.clone();
        let onto_clone = onto_workspace_snapshot.clone();

        Ok(slow_rt::spawn(async move {
            self_clone
                .working_copy()
                .await
                .detect_updates(&*onto_clone.working_copy().await)
        })?
        .await?)
    }

    /// Gives the exact node index endpoints of an edge. Use with care, since
    /// node indexes can't be relied on after modifications to the graph.
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
        name = "workspace_snapshot.import_component_subgraph",
        level = "debug",
        skip_all
    )]
    pub async fn import_component_subgraph(
        &self,
        other: &Self,
        component_id: ComponentId,
    ) -> WorkspaceSnapshotResult<()> {
        let component_node_index = other.read_only_graph.get_node_index_by_id(component_id)?;
        Ok(self
            .working_copy_mut()
            .await
            .import_component_subgraph(&other.read_only_graph, component_node_index)?)
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

    /// Write the entire graph to a file in dot format for debugging. *WARNING*:
    /// Can panic! Don't use in production code paths.
    pub async fn tiny_dot_to_file(&self, suffix: Option<&str>) {
        self.working_copy().await.tiny_dot_to_file(suffix);
    }

    /// Write a subgraph of the graph to a file in dot format for debugging.
    /// *WARNING*: Can panic! Use only for debugging.
    pub async fn tiny_dot_subgraph(&self, subgraph_root: impl Into<Ulid>, suffix: Option<&str>) {
        let subgraph_root_idx = self
            .get_node_index_by_id(subgraph_root)
            .await
            .expect("unable to find node index for subgraph root");

        if let Some(subgraph) = self.working_copy().await.subgraph(subgraph_root_idx) {
            subgraph.tiny_dot_to_file(suffix);
        }
    }

    /// Write the snapshot to disk. *WARNING* can panic! Use only for debugging
    pub async fn write_working_copy_to_disk(&self, file_suffix: &str) {
        self.working_copy().await.write_to_disk(file_suffix);
    }

    pub fn write_readonly_graph_to_disk(&self, file_suffix: &str) {
        self.read_only_graph.write_to_disk(file_suffix);
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
    pub async fn try_get_node_index_by_id(&self, id: impl Into<Ulid>) -> Option<NodeIndex> {
        self.working_copy().await.get_node_index_by_id_opt(id)
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
        let snapshot = match ctx
            .layer_db()
            .workspace_snapshot()
            .read_wait_for_memory(&workspace_snapshot_addr)
            .await
        {
            Ok(snapshot) => snapshot.ok_or(
                WorkspaceSnapshotError::WorkspaceSnapshotGraphMissing(workspace_snapshot_addr),
            )?,
            Err(err) => match err {
                LayerDbError::Postcard(_) => {
                    return Err(WorkspaceSnapshotError::WorkspaceSnapshotNotMigrated(
                        workspace_snapshot_addr,
                    ));
                }
                err => Err(err)?,
            },
        };

        Ok(Self {
            address: Arc::new(RwLock::new(workspace_snapshot_addr)),
            read_only_graph: snapshot,
            working_copy: Arc::new(RwLock::new(None)),
            cycle_check: Arc::new(AtomicBool::new(false)),
            dvu_roots: Arc::new(Mutex::new(HashSet::new())),
            inferred_connection_graph: Arc::new(RwLock::new(None)),
        })
    }

    pub async fn find_for_change_set(
        ctx: &DalContext,
        change_set_id: ChangeSetId,
    ) -> WorkspaceSnapshotResult<Self> {
        // There's a race between finding which address to retrieve and actually retrieving it
        // where it's possible for the content at the address to be garbage collected, and no
        // longer be retrievable. We'll re-fetch which snapshot address to use, and will retry,
        // hoping we don't get unlucky every time.
        let mut retries: u8 = 5;

        while retries > 0 {
            retries -= 1;

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

            match Self::find(ctx, address).await {
                Ok(snapshot) => return Ok(snapshot),
                Err(WorkspaceSnapshotError::WorkspaceSnapshotGraphMissing(_)) => {
                    warn!(
                        "Unable to retrieve snapshot {:?} for change set {:?}. Retries remaining: {}",
                        address, change_set_id, retries
                    );
                    tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
                    continue;
                }
                Err(e) => return Err(e),
            }
        }

        error!(
            "Retries exceeded trying to fetch workspace snapshot for change set {:?}",
            change_set_id
        );
        Err(WorkspaceSnapshotError::WorkspaceSnapshotNotFetched)
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
    pub async fn remove_all_edges(&self, id: impl Into<Ulid>) -> WorkspaceSnapshotResult<()> {
        let id = id.into();
        for (edge_weight, source, target) in self.edges_directed(id, Direction::Outgoing).await? {
            self.remove_edge(source, target, edge_weight.kind().into())
                .await?;
        }
        for (edge_weight, source, target) in self.edges_directed(id, Direction::Incoming).await? {
            self.remove_edge(source, target, edge_weight.kind().into())
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
        target_id: impl Into<Ulid>,
        kind: EdgeWeightKindDiscriminants,
    ) -> WorkspaceSnapshotResult<()> {
        let target_id = target_id.into();

        let sources = self
            .incoming_sources_for_edge_weight_kind(target_id, kind)
            .await?;
        for source_node_idx in sources {
            let target_node_idx = self.get_node_index_by_id(target_id).await?;
            self.remove_edge(source_node_idx, target_node_idx, kind)
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
    pub async fn remove_node_by_id(&self, id: impl Into<Ulid>) -> WorkspaceSnapshotResult<()> {
        let id: Ulid = id.into();
        let node_idx = self.get_node_index_by_id(id).await?;
        self.remove_all_edges(id).await?;
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
        source_node_index: NodeIndex,
        target_node_index: NodeIndex,
        edge_kind: EdgeWeightKindDiscriminants,
    ) -> WorkspaceSnapshotResult<()> {
        Ok(self.working_copy_mut().await.remove_edge(
            source_node_index,
            target_node_index,
            edge_kind,
        )?)
    }

    pub async fn find_edge(
        &self,
        from_id: impl Into<Ulid>,
        to_id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<Option<EdgeWeight>> {
        let working_copy = self.working_copy().await;

        let from_idx = working_copy.get_node_index_by_id(from_id)?;
        let to_idx = working_copy.get_node_index_by_id(to_id)?;

        Ok(working_copy
            .find_edge(from_idx, to_idx)
            .map(ToOwned::to_owned))
    }

    #[instrument(
        name = "workspace_snapshot.remove_edge_for_ulids",
        level = "debug",
        skip_all,
        fields()
    )]
    pub async fn remove_edge_for_ulids(
        &self,
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
        self.remove_edge(source_node_index, target_node_index, edge_kind)
            .await
    }

    /// Perform [`Updates`](Update) using [`self`](WorkspaceSnapshot) as the "to rebase" graph and
    /// another [`snapshot`](WorkspaceSnapshot) as the "onto" graph.
    #[instrument(
        name = "workspace_snapshot.perform_updates",
        level = "info",
        skip_all,
        fields()
    )]
    pub async fn perform_updates(&self, updates: &[Update]) -> WorkspaceSnapshotResult<()> {
        let self_clone = self.clone();
        let updates = updates.to_vec();
        Ok(slow_rt::spawn(async move {
            self_clone
                .working_copy_mut()
                .await
                .perform_updates(&updates)
        })?
        .await??)
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
        name = "workspace_snapshot.update_node_id",
        level = "debug",
        skip_all,
        fields()
    )]
    pub async fn update_node_id(
        &self,
        current_id: impl Into<Ulid>,
        new_id: impl Into<Ulid>,
        new_lineage_id: LineageId,
    ) -> WorkspaceSnapshotResult<()> {
        let idx = self.get_node_index_by_id(current_id).await?;
        self.working_copy_mut()
            .await
            .update_node_id(idx, new_id, new_lineage_id)
            .await?;

        self.replace_references(idx).await?;

        Ok(())
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
        name = "workspace_snapshot.components_added_relative_to_base",
        level = "info",
        skip_all
    )]
    pub async fn components_added_relative_to_base(
        &self,
        ctx: &DalContext,
    ) -> WorkspaceSnapshotResult<Vec<ComponentId>> {
        let mut new_component_ids = Vec::new();
        let base_change_set_id = if let Some(change_set_id) = ctx.change_set()?.base_change_set_id {
            change_set_id
        } else {
            return Ok(new_component_ids);
        };

        // Even though the default change set for a workspace can have a base change set, we don't
        // want to consider anything as new/modified/removed when looking at the default change
        // set.
        let workspace = Workspace::get_by_pk_or_error(
            ctx,
            &ctx.tenancy()
                .workspace_pk()
                .ok_or(WorkspaceSnapshotError::WorkspaceMissing)?,
        )
        .await
        .map_err(Box::new)?;
        if workspace.default_change_set_id() == ctx.change_set_id() {
            return Ok(new_component_ids);
        }

        let base_snapshot = WorkspaceSnapshot::find_for_change_set(ctx, base_change_set_id).await?;

        let updates = base_snapshot
            .read_only_graph
            .detect_updates(&self.read_only_graph);

        for update in &updates {
            match update {
                Update::RemoveEdge { .. } | Update::ReplaceNode { .. } | Update::NewEdge { .. } => {
                }
                Update::NewNode { node_weight } => {
                    if let NodeWeight::Component(inner) = node_weight {
                        if base_snapshot
                            .read_only_graph
                            .get_node_index_by_id_opt(inner.id)
                            .is_none()
                        {
                            new_component_ids.push(inner.id.into())
                        }
                    }
                }
            }
        }

        Ok(new_component_ids)
    }

    #[instrument(
        name = "workspace_snapshot.components_removed_relative_to_base",
        level = "debug",
        skip_all
    )]
    pub async fn components_removed_relative_to_base(
        &self,
        ctx: &DalContext,
    ) -> WorkspaceSnapshotResult<Vec<ComponentId>> {
        let mut removed_component_ids = Vec::new();
        let base_change_set_id = if let Some(change_set_id) = ctx.change_set()?.base_change_set_id {
            change_set_id
        } else {
            return Ok(removed_component_ids);
        };

        // Even though the default change set for a workspace can have a base change set, we don't
        // want to consider anything as new/modified/removed when looking at the default change
        // set.
        let workspace = Workspace::get_by_pk_or_error(
            ctx,
            &ctx.tenancy()
                .workspace_pk()
                .ok_or(WorkspaceSnapshotError::WorkspaceMissing)?,
        )
        .await
        .map_err(Box::new)?;
        if workspace.default_change_set_id() == ctx.change_set_id() {
            return Ok(removed_component_ids);
        }

        let base_snapshot = WorkspaceSnapshot::find_for_change_set(ctx, base_change_set_id).await?;
        let component_category_id = if let Some((category_id, _)) = base_snapshot
            .read_only_graph
            .get_category_node(None, CategoryNodeKind::Component)?
        {
            category_id
        } else {
            // Can't have any new Components if there's no Component category node to put them
            // under.
            return Ok(removed_component_ids);
        };

        let updates = base_snapshot
            .read_only_graph
            .detect_updates(&self.read_only_graph);

        let mut added_component_ids = HashSet::new();

        for update in &updates {
            match update {
                Update::ReplaceNode { .. } | Update::NewNode { .. } => {
                    /* Updates unused for determining if a Component is removed in regard to the updates */
                }
                Update::NewEdge {
                    source,
                    destination,
                    edge_weight: _,
                } => {
                    // get updates that add an edge from the Components category to a component, which implies component creation
                    if source.id != component_category_id.into()
                        || destination.node_weight_kind != NodeWeightDiscriminants::Component
                    {
                        continue;
                    }

                    added_component_ids.insert(ComponentId::from(Ulid::from(destination.id)));
                }
                Update::RemoveEdge {
                    source,
                    destination,
                    edge_kind: _,
                } => {
                    // get updates that remove an edge from the Components category to a component, which implies component deletion
                    if source.id != component_category_id.into()
                        || destination.node_weight_kind != NodeWeightDiscriminants::Component
                    {
                        continue;
                    }

                    removed_component_ids.push(ComponentId::from(Ulid::from(destination.id)));
                }
            }
        }

        // Filter out ComponentIds that have both been deleted and created, since that implies an upgrade and not a real deletion
        Ok(removed_component_ids
            .into_iter()
            .filter(|id| !added_component_ids.contains(id))
            .collect())
    }

    #[instrument(
        name = "workspace_snapshot.socket_edges_added_relative_to_base",
        level = "debug",
        skip_all
    )]
    pub async fn socket_edges_added_relative_to_base(
        &self,
        ctx: &DalContext,
    ) -> WorkspaceSnapshotResult<Vec<AttributePrototypeArgumentId>> {
        let mut new_attribute_prototype_argument_ids = Vec::new();
        let base_change_set_id = if let Some(change_set_id) = ctx.change_set()?.base_change_set_id {
            change_set_id
        } else {
            return Ok(new_attribute_prototype_argument_ids);
        };

        // Even though the default change set for a workspace can have a base change set, we don't
        // want to consider anything as new/modified/removed when looking at the default change
        // set.
        let workspace = Workspace::get_by_pk_or_error(
            ctx,
            &ctx.tenancy()
                .workspace_pk()
                .ok_or(WorkspaceSnapshotError::WorkspaceMissing)?,
        )
        .await
        .map_err(Box::new)?;
        if workspace.default_change_set_id() == ctx.change_set_id() {
            return Ok(new_attribute_prototype_argument_ids);
        }

        let base_snapshot = WorkspaceSnapshot::find_for_change_set(ctx, base_change_set_id).await?;

        let updates = base_snapshot
            .read_only_graph
            .detect_updates(&self.read_only_graph);

        for update in &updates {
            match update {
                Update::NewNode { .. } | Update::RemoveEdge { .. } | Update::ReplaceNode { .. } => {
                    // Updates unused for determining if a socket to socket connection (in frontend
                    // terms) is new.
                }
                Update::NewEdge {
                    source: _source,
                    destination,
                    edge_weight: _,
                } => {
                    if destination.node_weight_kind
                        != NodeWeightDiscriminants::AttributePrototypeArgument
                    {
                        // We're interested in new AttributePrototypeArguments as they represent
                        // the connection between sockets. (The input socket has the output
                        // socket as one of its function arguments.)
                        continue;
                    }

                    let prototype_argument = AttributePrototypeArgument::get_by_id(
                        ctx,
                        AttributePrototypeArgumentId::from(Ulid::from(destination.id)),
                    )
                    .await
                    .map_err(Box::new)?;
                    if prototype_argument.targets().is_some() {
                        new_attribute_prototype_argument_ids.push(prototype_argument.id());
                    } else {
                        // If the AttributePrototypeArgument doesn't have targets, then it's
                        // not for a socket to socket connection.
                        continue;
                    }
                }
            }
        }

        Ok(new_attribute_prototype_argument_ids)
    }

    #[instrument(
        name = "workspace_snapshot.socket_edges_removed_relative_to_base",
        level = "debug",
        skip_all
    )]
    pub async fn socket_edges_removed_relative_to_base(
        &self,
        ctx: &DalContext,
    ) -> WorkspaceSnapshotResult<Vec<AttributePrototypeArgumentId>> {
        let mut removed_attribute_prototype_argument_ids = Vec::new();

        // Even though the default change set for a workspace can have a base change set, we don't
        // want to consider anything as new/modified/removed when looking at the default change
        // set.
        let workspace = Workspace::get_by_pk_or_error(
            ctx,
            &ctx.tenancy()
                .workspace_pk()
                .ok_or(WorkspaceSnapshotError::WorkspaceMissing)?,
        )
        .await
        .map_err(Box::new)?;
        if workspace.default_change_set_id() == ctx.change_set_id() {
            return Ok(removed_attribute_prototype_argument_ids);
        }

        let base_change_set_ctx = ctx.clone_with_base().await?;
        let base_change_set_ctx = &base_change_set_ctx;

        // * For each Component being removed (all edges to/from removed components should also
        //   show as removed):
        let removed_component_ids: HashSet<ComponentId> = self
            .components_removed_relative_to_base(ctx)
            .await?
            .iter()
            .copied()
            .collect();
        let remaining_component_ids: HashSet<ComponentId> = Component::list(ctx)
            .await
            .map_err(Box::new)?
            .iter()
            .map(Component::id)
            .collect();
        for removed_component_id in &removed_component_ids {
            let base_change_set_component =
                Component::get_by_id(base_change_set_ctx, *removed_component_id)
                    .await
                    .map_err(Box::new)?;

            // * Get incoming edges
            for incoming_connection in base_change_set_component
                .incoming_connections(base_change_set_ctx)
                .await
                .map_err(Box::new)?
            {
                //* Interested in:
                //  * Edge is coming from a Component being removed
                //  * Edge is coming from a Component that exists in current change set
                if removed_component_ids.contains(&incoming_connection.from_component_id)
                    || remaining_component_ids.contains(&incoming_connection.from_component_id)
                {
                    removed_attribute_prototype_argument_ids
                        .push(incoming_connection.attribute_prototype_argument_id);
                }
            }

            //* Get outgoing edges
            for outgoing_connection in base_change_set_component
                .outgoing_connections(base_change_set_ctx)
                .await
                .map_err(Box::new)?
            {
                //  * Interested in:
                //    * Edge is going to a Component being removed
                //    * Edge is going to a Component that exists in current change set
                if removed_component_ids.contains(&outgoing_connection.to_component_id)
                    || remaining_component_ids.contains(&outgoing_connection.to_component_id)
                {
                    removed_attribute_prototype_argument_ids
                        .push(outgoing_connection.attribute_prototype_argument_id);
                }
            }
        }

        // * For each removed AttributePrototypeArgument (removed edge connects two Components
        //   that have not been removed):
        let base_snapshot = base_change_set_ctx.workspace_snapshot()?;
        let updates = base_snapshot
            .read_only_graph
            .detect_updates(&self.read_only_graph);

        for update in updates {
            match update {
                Update::ReplaceNode { .. } | Update::NewEdge { .. } | Update::NewNode { .. } => {
                    /* Updates unused for determining if a connection between sockets has been removed */
                }
                Update::RemoveEdge {
                    source: _,
                    destination,
                    edge_kind,
                } => {
                    if edge_kind != EdgeWeightKindDiscriminants::PrototypeArgument {
                        continue;
                    }
                    let attribute_prototype_argument = AttributePrototypeArgument::get_by_id(
                        base_change_set_ctx,
                        AttributePrototypeArgumentId::from(Ulid::from(destination.id)),
                    )
                    .await
                    .map_err(Box::new)?;

                    // * Interested in all of them that have targets (connecting two Components
                    //   via sockets).
                    if attribute_prototype_argument.targets().is_some() {
                        removed_attribute_prototype_argument_ids
                            .push(attribute_prototype_argument.id());
                    }
                }
            }
        }

        Ok(removed_attribute_prototype_argument_ids)
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
                let id = working_copy.generate_ulid()?;
                let lineage_id = working_copy.generate_ulid()?;
                let category_node_idx = working_copy.add_category_node(
                    id,
                    lineage_id,
                    CategoryNodeKind::DependentValueRoots,
                )?;
                working_copy.add_edge(
                    root_idx,
                    EdgeWeight::new(EdgeWeightKind::new_use()),
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
        value_id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<()> {
        let value_id = value_id.into();

        // ensure we don't grow the graph unnecessarily by adding the same value
        // in a single edit session
        {
            let mut dvu_roots = self.dvu_roots.lock().await;
            if dvu_roots.contains(&value_id) {
                return Ok(());
            }
            dvu_roots.insert(value_id);
        }

        let (dv_category_id, _) = self.find_existing_dependent_value_root(value_id).await?;

        let id = self.generate_ulid().await?;
        let lineage_id = self.generate_ulid().await?;

        let new_dependent_value_node =
            NodeWeight::new_dependent_value_root(id, lineage_id, value_id);
        let new_dv_node_id = new_dependent_value_node.id();
        self.add_node(new_dependent_value_node).await?;

        self.add_edge(
            dv_category_id,
            EdgeWeight::new(EdgeWeightKind::new_use()),
            new_dv_node_id,
        )
        .await?;

        Ok(())
    }

    pub async fn remove_dependent_value_root(
        &self,
        value_id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<()> {
        let value_id = value_id.into();
        let (_, existing_value_id) = self.find_existing_dependent_value_root(value_id).await?;

        if let Some(existing_id) = existing_value_id {
            self.remove_node_by_id(existing_id).await?;
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
    pub async fn take_dependent_values(&self) -> WorkspaceSnapshotResult<Vec<Ulid>> {
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
            self.remove_node_by_id(to_remove_id).await?;
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

    /// There are cases where we want to cache this connection graph to avoid calculating
    /// it more than once for the workspace. In those cases, the graph will be `Some` here.
    pub async fn get_cached_inferred_connection_graph(
        &self,
    ) -> RwLockReadGuard<Option<InferredConnectionGraph>> {
        self.inferred_connection_graph.read().await
    }

    pub async fn set_cached_inferred_connection_graph(
        &self,
        maybe_graph: Option<InferredConnectionGraph>,
    ) {
        *self.inferred_connection_graph.write().await = maybe_graph;
    }

    /// If this node is associated to a single av, return it
    pub async fn associated_attribute_value_id(
        &self,
        node_weight: NodeWeight,
    ) -> WorkspaceSnapshotResult<Option<AttributeValueId>> {
        let mut this_node_weight = node_weight;
        while let Some(edge_kind) = match &this_node_weight {
            NodeWeight::AttributeValue(av) => return Ok(Some(av.id().into())),
            NodeWeight::AttributePrototypeArgument(_) => {
                Some(EdgeWeightKindDiscriminants::PrototypeArgument)
            }
            NodeWeight::Ordering(_) => Some(EdgeWeightKindDiscriminants::Ordering),

            NodeWeight::Content(content) => match content.content_address_discriminants() {
                ContentAddressDiscriminants::AttributePrototype => {
                    Some(EdgeWeightKindDiscriminants::Prototype)
                }
                ContentAddressDiscriminants::StaticArgumentValue => {
                    Some(EdgeWeightKindDiscriminants::PrototypeArgumentValue)
                }
                ContentAddressDiscriminants::ValidationOutput => {
                    Some(EdgeWeightKindDiscriminants::ValidationOutput)
                }

                ContentAddressDiscriminants::ActionPrototype
                | ContentAddressDiscriminants::Component
                | ContentAddressDiscriminants::DeprecatedAction
                | ContentAddressDiscriminants::DeprecatedActionBatch
                | ContentAddressDiscriminants::DeprecatedActionRunner
                | ContentAddressDiscriminants::Func
                | ContentAddressDiscriminants::FuncArg
                | ContentAddressDiscriminants::InputSocket
                | ContentAddressDiscriminants::JsonValue
                | ContentAddressDiscriminants::Module
                | ContentAddressDiscriminants::OutputSocket
                | ContentAddressDiscriminants::Prop
                | ContentAddressDiscriminants::Root
                | ContentAddressDiscriminants::Schema
                | ContentAddressDiscriminants::SchemaVariant
                | ContentAddressDiscriminants::Secret
                | ContentAddressDiscriminants::ValidationPrototype => None,
            },

            NodeWeight::Action(_)
            | NodeWeight::ActionPrototype(_)
            | NodeWeight::Category(_)
            | NodeWeight::Component(_)
            | NodeWeight::DependentValueRoot(_)
            | NodeWeight::Func(_)
            | NodeWeight::FuncArgument(_)
            | NodeWeight::Prop(_)
            | NodeWeight::Secret(_) => None,
        } {
            let next_node_idxs = self
                .incoming_sources_for_edge_weight_kind(this_node_weight.id(), edge_kind)
                .await?;

            this_node_weight = match next_node_idxs.first() {
                Some(&next_node_idx) if next_node_idxs.len() == 1 => {
                    self.get_node_weight(next_node_idx).await?
                }
                _ => {
                    return Err(WorkspaceSnapshotError::UnexpectedNumberOfIncomingEdges(
                        edge_kind,
                        NodeWeightDiscriminants::from(&this_node_weight),
                        this_node_weight.id(),
                    ))
                }
            };
        }

        Ok(None)
    }
}
