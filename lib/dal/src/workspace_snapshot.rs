//! This is the primary interface for business logic to interact with the graph. All interaction
//! should be done through one of the `Ext` traits that [`WorkspaceSnapshot`] implements to avoid
//! having code outside of the specific graph version implementation that requires having knowledge
//! of how the internals of that specific version of the graph work.

use std::{
    collections::HashSet,
    sync::{
        Arc,
        atomic::AtomicBool,
    },
};

use content_address::ContentAddress;
use graph::{
    RebaseBatch,
    WorkspaceSnapshotGraph,
    correct_transforms::correct_transforms,
    detector::Update,
    validator::connections::SocketConnection,
};
use node_weight::traits::CorrectTransformsError;
use petgraph::prelude::*;
use selector::WorkspaceSnapshotSelectorDiscriminants;
use serde::{
    Deserialize,
    Serialize,
};
use si_data_pg::PgError;
use si_events::{
    ContentHash,
    WorkspaceSnapshotAddress,
    ulid::Ulid,
    workspace_snapshot::Change,
};
use si_id::ApprovalRequirementDefinitionId;
use si_layer_cache::LayerDbError;
use si_split_graph::SplitGraphError;
use telemetry::prelude::*;
use thiserror::Error;
use tokio::{
    sync::{
        Mutex,
        RwLock,
        RwLockReadGuard,
        RwLockWriteGuard,
    },
    task::JoinError,
};

use self::node_weight::{
    NodeWeightDiscriminants,
    OrderingNodeWeight,
};
use crate::{
    ComponentError,
    ComponentId,
    DalContext,
    SchemaVariantError,
    SchemaVariantId,
    TransactionsError,
    WorkspaceError,
    WorkspaceSnapshotGraphVCurrent,
    attribute::{
        prototype::{
            AttributePrototypeError,
            argument::AttributePrototypeArgumentError,
        },
        value::AttributeValueError,
    },
    change_set::{
        ChangeSetError,
        ChangeSetId,
    },
    component::{
        ComponentResult,
        inferred_connection_graph::{
            InferredConnectionGraph,
            InferredConnectionGraphError,
        },
    },
    slow_rt::{
        self,
        SlowRuntimeError,
    },
    socket::{
        connection_annotation::ConnectionAnnotationError,
        input::InputSocketError,
    },
    workspace_snapshot::{
        edge_weight::{
            EdgeWeight,
            EdgeWeightKindDiscriminants,
        },
        graph::{
            LineageId,
            WorkspaceSnapshotGraphDiscriminants,
            WorkspaceSnapshotGraphError,
        },
        node_weight::{
            NodeWeight,
            NodeWeightError,
            category_node_weight::CategoryNodeKind,
        },
    },
};

pub mod content_address;
pub mod dependent_value_root;
pub mod edge_weight;
pub mod graph;
pub mod migrator;
pub mod node_weight;
pub mod selector;
pub mod split_snapshot;
pub mod traits;
pub mod update;

pub use dependent_value_root::DependentValueRoot;
pub use petgraph::Direction;
pub use selector::WorkspaceSnapshotSelector;
pub use si_id::WorkspaceSnapshotNodeId as NodeId;
pub use traits::{
    entity_kind::EntityKindExt,
    schema::variant::SchemaVariantExt,
    socket::input::InputSocketExt,
};

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
    #[error("AttributePrototype error: {0}")]
    AttributePrototype(#[from] Box<AttributePrototypeError>),
    #[error("Attribute Prototype Argument: {0}")]
    AttributePrototypeArgument(#[from] Box<AttributePrototypeArgumentError>),
    #[error("AttributeValue error: {0}")]
    AttributeValue(#[from] Box<AttributeValueError>),
    #[error("could not find category node of kind: {0:?}")]
    CategoryNodeNotFound(CategoryNodeKind),
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("change set {0} has no workspace snapshot address")]
    ChangeSetMissingWorkspaceSnapshotAddress(ChangeSetId),
    #[error("Component error: {0}")]
    Component(#[from] Box<ComponentError>),
    #[error("ConnectionAnnotation error: {0}")]
    ConnectionAnnotation(#[from] Box<ConnectionAnnotationError>),
    #[error("error correcting transforms: {0}")]
    CorrectTransforms(#[from] CorrectTransformsError),
    #[error("error correcting transforms split graph: {0}")]
    CorrectTransformsSplit(#[from] split_snapshot::corrections::CorrectTransformsError),
    #[error("Action would create a graph cycle")]
    CreateGraphCycle,
    #[error("cannot delete the default view")]
    DefaultViewDeletionAttempt,
    #[error("InferredConnectionGraph error: {0}")]
    InferredConnectionGraph(#[from] Box<InferredConnectionGraphError>),
    #[error("InputSocket error: {0}")]
    InputSocket(#[from] Box<InputSocketError>),
    #[error("join error: {0}")]
    Join(#[from] JoinError),
    #[error("layer db error: {0}")]
    LayerDb(#[from] si_layer_cache::LayerDbError),
    #[error(
        "missing content from content map for hash ({0}) and approval requirement definition ({1})"
    )]
    MissingContentFromContentMap(ContentHash, ApprovalRequirementDefinitionId),
    #[error("missing content from store for id: {0}")]
    MissingContentFromStore(Ulid),
    #[error("missing content from store for address: {0}")]
    MissingContentFromStoreForAddress(ContentAddress),
    #[error("could not find a max vector clock for change set id {0}")]
    MissingVectorClockForChangeSet(ChangeSetId),
    #[error("monotonic error: {0}")]
    Monotonic(#[from] ulid::MonotonicError),
    #[error("node not found at node id: {0}")]
    NodeNotFoundAtId(Ulid),
    #[error("node not found at node index: {0:?}")]
    NodeNotFoundAtIndex(NodeIndex),
    #[error("NodeWeight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("expected edges of kind {2:?} {1:?} to/from {0} but none found")]
    NoEdgesOfKindFound(Ulid, Direction, EdgeWeightKindDiscriminants),
    #[error("ordering not found for node with ordered children: {0}")]
    OrderingNotFound(Ulid),
    #[error("si_data_pg error: {0}")]
    Pg(#[from] PgError),
    #[error("postcard error: {0}")]
    Postcard(#[from] postcard::Error),
    #[error("recently seen clocks missing for change set id {0}")]
    RecentlySeenClocksMissing(ChangeSetId),
    #[error("SchemaVariant error: {0}")]
    SchemaVariant(#[from] Box<SchemaVariantError>),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("slow runtime error: {0}")]
    SlowRuntime(#[from] SlowRuntimeError),
    #[error("split graph error: {0}")]
    SplitGraph(#[from] SplitGraphError),
    #[error("split snapshot subgraph missing at index: {0}")]
    SplitSnapshotSubGraphAddressMissingAtIndex(usize),
    #[error("split snapshot subgraph missing at address: {0}")]
    SplitSnapshotSubGraphMissingAtAddress(WorkspaceSnapshotAddress),
    #[error("split snapshot supergraph missing at address: {0}")]
    SplitSnapshotSuperGraphMissingAtAddress(WorkspaceSnapshotAddress),
    #[error("Too many edges of kind {1} found with node id {0:?} as the source")]
    TooManyEdgesOfKind(Ulid, EdgeWeightKindDiscriminants),
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
    #[error("Unexpected snapshot kind: {0}")]
    UnexpectedSnapshotKind(WorkspaceSnapshotSelectorDiscriminants),
    #[error("Removing View would orphan items: {0:?}")]
    ViewRemovalWouldOrphanItems(Vec<Ulid>),
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

impl From<AttributeValueError> for WorkspaceSnapshotError {
    fn from(value: AttributeValueError) -> Self {
        Self::AttributeValue(Box::new(value))
    }
}

impl WorkspaceSnapshotError {
    pub fn is_node_with_id_not_found(&self) -> bool {
        matches!(
            self,
            Self::WorkspaceSnapshotGraph(WorkspaceSnapshotGraphError::NodeWithIdNotFound(_,),)
        )
    }
}

pub type WorkspaceSnapshotResult<T> = Result<T, WorkspaceSnapshotError>;

/// The workspace graph. The public interface for this is provided through the the various `Ext`
/// traits that are implemented for [`WorkspaceSnapshot`].
///
/// ## Internals
///
/// The concurrency types used here to give us interior mutability in the tokio run time are *not*
/// sufficient to prevent data races when operating on the same graph on different threads, since
/// our graph operations are not "atomic" and the graph *WILL* end up being read from different
/// threads while a write operation is still in progress if it is shared between threads for
/// modification. For example after a node is added but *before* the edges necessary to place that
/// node in the right spot in the graph have been added. We need a more general solution here, but
/// for now an example of synchronization when accessing a snapshot across threads can be found in
/// [`crate::job::definition::DependentValuesUpdate`].
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
    working_copy: Arc<RwLock<Option<WorkspaceSnapshotGraphVCurrent>>>,

    /// Whether we should perform cycle checks on add edge operations
    cycle_check: Arc<AtomicBool>,

    /// A hashset to prevent adding duplicate roots to the workspace in a single edit session
    dvu_roots: Arc<Mutex<HashSet<DependentValueRoot>>>,

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

impl CycleCheckGuard {
    pub(crate) fn new(cycle_check: Arc<AtomicBool>) -> Self {
        Self { cycle_check }
    }
}

impl Drop for CycleCheckGuard {
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
    working_copy_read_guard: RwLockReadGuard<'a, Option<WorkspaceSnapshotGraphVCurrent>>,
}

#[must_use = "if unused the lock will be released immediately"]
struct SnapshotWriteGuard<'a> {
    working_copy_write_guard: RwLockWriteGuard<'a, Option<WorkspaceSnapshotGraphVCurrent>>,
}

impl std::ops::Deref for SnapshotReadGuard<'_> {
    type Target = WorkspaceSnapshotGraphVCurrent;

    fn deref(&self) -> &Self::Target {
        if self.working_copy_read_guard.is_some() {
            let option = &*self.working_copy_read_guard;
            option.as_ref().expect("we confirmed it was some above")
        } else {
            &self.read_only_graph
        }
    }
}

impl std::ops::Deref for SnapshotWriteGuard<'_> {
    type Target = WorkspaceSnapshotGraphVCurrent;

    fn deref(&self) -> &Self::Target {
        let option = &*self.working_copy_write_guard;
        option.as_ref().expect(
            "attempted to deref snapshot without copying contents into the mutable working copy",
        )
    }
}

impl std::ops::DerefMut for SnapshotWriteGuard<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let option: &mut Option<WorkspaceSnapshotGraphVCurrent> =
            &mut self.working_copy_write_guard;
        &mut *option.as_mut().expect("attempted to DerefMut a snapshot without copying contents into the mutable working copy")
    }
}

#[must_use = "if unused the lock will be released immediately"]
pub struct InferredConnectionsReadGuard<'a> {
    inferred_connection_graph: RwLockReadGuard<'a, Option<InferredConnectionGraph>>,
}

#[must_use = "if unused the lock will be released immediately"]
pub struct InferredConnectionsWriteGuard<'a> {
    inferred_connection_graph: RwLockWriteGuard<'a, Option<InferredConnectionGraph>>,
}

impl std::ops::Deref for InferredConnectionsReadGuard<'_> {
    type Target = InferredConnectionGraph;

    fn deref(&self) -> &Self::Target {
        let maybe = &*self.inferred_connection_graph;
        maybe
            .as_ref()
            .expect("attempted to Deref inferred connection graph without creating it first")
    }
}

impl std::ops::Deref for InferredConnectionsWriteGuard<'_> {
    type Target = InferredConnectionGraph;

    fn deref(&self) -> &Self::Target {
        let maybe = &*self.inferred_connection_graph;
        maybe
            .as_ref()
            .expect("attempted to Deref inferred connection graph without creating it first")
    }
}

impl std::ops::DerefMut for InferredConnectionsWriteGuard<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let maybe: &mut Option<InferredConnectionGraph> = &mut self.inferred_connection_graph;
        &mut *maybe
            .as_mut()
            .expect("attempted to DerefMut inferred connection graph without creating it first")
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
        let graph: WorkspaceSnapshotGraphVCurrent =
            WorkspaceSnapshotGraphVCurrent::new(ctx).await?;

        // We do not care about any field other than "working_copy" because
        // "write" will populate them using the assigned working copy.
        let initial = Self {
            address: Arc::new(RwLock::new(WorkspaceSnapshotAddress::nil())),
            read_only_graph: Arc::new(WorkspaceSnapshotGraph::V4(graph)),
            working_copy: Arc::new(RwLock::new(None)),
            cycle_check: Arc::new(AtomicBool::new(false)),
            dvu_roots: Arc::new(Mutex::new(HashSet::new())),
            inferred_connection_graph: Arc::new(RwLock::new(None)),
        };

        initial.write(ctx).await?;

        Ok(initial)
    }

    pub fn read_only_graph_version(&self) -> WorkspaceSnapshotGraphDiscriminants {
        WorkspaceSnapshotGraphDiscriminants::from(&(*self.read_only_graph))
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
    #[instrument(
        name = "workspace_snapshot.current_rebase_batch",
        level = "info",
        skip_all
    )]
    pub async fn current_rebase_batch(&self) -> WorkspaceSnapshotResult<Option<RebaseBatch>> {
        let self_clone = self.clone();
        let updates = slow_rt::spawn(async move {
            let mut working_copy = self_clone.working_copy_mut().await;
            working_copy.cleanup_and_merkle_tree_hash()?;

            Ok::<Vec<Update>, WorkspaceSnapshotGraphError>(
                self_clone.read_only_graph.detect_updates(&working_copy),
            )
        })?
        .await??;

        Ok((!updates.is_empty()).then_some(RebaseBatch::new(updates)))
    }

    /// Calculates the set of updates made to `updated_snapshot` against
    /// `base_snapshot`. For these updates to be correct, `updated_snapshot`
    /// must have already seen all the changes made to `base_snapshot`, and both
    /// snapshots should have had their merkle tree hashes calculated with
    /// `Self::cleanup_and_merkle_tree_hash`.
    #[instrument(
        name = "workspace_snapshot.calculate_rebase_batch",
        level = "info",
        skip_all
    )]
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
        level = "debug",
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
        let span = current_span_for_instrument_at!("debug");

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
                working_copy.cleanup_and_merkle_tree_hash()?;

                let (new_address, _) = layer_db.workspace_snapshot().write(
                    Arc::new(WorkspaceSnapshotGraph::V4(working_copy.clone())),
                    None,
                    events_tenancy,
                    events_actor,
                )?;

                Ok::<WorkspaceSnapshotAddress, WorkspaceSnapshotError>(new_address)
            })?
            .await??;

            span.record("si.workspace_snapshot.address", new_address.to_string());

            new_address
        };

        // Note, we continue to use the working copy after this, even for reads, since otherwise
        // we'd have to replace the read_only_graph, which would require another thread-safe
        // interior mutability type to store the read only graph in.

        *self.address.write().await = new_address;

        Ok(new_address)
    }

    /// Write the read only graph to the layer db, unmodified. Useful for
    /// persisting a snapshot that has been deserialized via `Self::from_bytes`
    #[instrument(
        name = "workspace_snapshot.write_readonly_graph",
        level = "info",
        skip_all
    )]
    pub async fn write_readonly_graph(
        &self,
        ctx: &DalContext,
    ) -> WorkspaceSnapshotResult<WorkspaceSnapshotAddress> {
        let events_tenancy = ctx.events_tenancy();
        let events_actor = ctx.events_actor();

        let (address, _) = ctx.layer_db().workspace_snapshot().write(
            self.read_only_graph.clone(),
            None,
            events_tenancy,
            events_actor,
        )?;

        Ok(address)
    }

    pub async fn id(&self) -> WorkspaceSnapshotAddress {
        *self.address.read().await
    }

    pub async fn root(&self) -> WorkspaceSnapshotResult<Ulid> {
        let root_idx = self.working_copy().await.root();
        Ok(self.working_copy().await.get_node_weight(root_idx)?.id())
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
        let mut working_copy = self.working_copy.write().await;
        if working_copy.is_some() {
            *working_copy = None;
        }
    }

    pub async fn serialized(&self) -> WorkspaceSnapshotResult<Vec<u8>> {
        let graph = self.working_copy().await.clone();
        Ok(si_layer_cache::db::serialize::to_vec(&WorkspaceSnapshotGraph::V4(graph))?.0)
    }

    pub fn from_bytes(bytes: &[u8]) -> WorkspaceSnapshotResult<Self> {
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

    /// Adds this node to the graph, or replaces it if a node with the same id
    /// already exists.
    pub async fn add_or_replace_node(&self, node: NodeWeight) -> WorkspaceSnapshotResult<()> {
        self.working_copy_mut().await.add_or_replace_node(node)?;
        Ok(())
    }

    pub async fn add_ordered_node(&self, node: NodeWeight) -> WorkspaceSnapshotResult<()> {
        self.working_copy_mut().await.add_ordered_node(node)?;
        Ok(())
    }

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
    ) -> WorkspaceSnapshotResult<()> {
        let from_node_index = self
            .working_copy()
            .await
            .get_node_index_by_id(from_node_id)?;
        let to_node_index = self.working_copy().await.get_node_index_by_id(to_node_id)?;
        if self.cycle_check().await {
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
        }

        Ok(())
    }

    /// Add an edge to the graph, bypassing any cycle checks and using node
    /// indices directly.
    pub async fn add_edge_unchecked(
        &self,
        from_id: impl Into<Ulid>,
        edge_weight: EdgeWeight,
        to_id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<()> {
        let from_node_index = self.working_copy().await.get_node_index_by_id(from_id)?;
        let to_node_index = self.working_copy().await.get_node_index_by_id(to_id)?;
        self.working_copy_mut()
            .await
            .add_edge(from_node_index, edge_weight, to_node_index)?;

        Ok(())
    }

    pub async fn add_ordered_edge(
        &self,
        from_node_id: impl Into<Ulid>,
        edge_weight: EdgeWeight,
        to_node_id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<()> {
        let from_node_index = self
            .working_copy()
            .await
            .get_node_index_by_id(from_node_id)?;
        let to_node_index = self.working_copy().await.get_node_index_by_id(to_node_id)?;
        self.working_copy_mut().await.add_ordered_edge(
            from_node_index,
            edge_weight,
            to_node_index,
        )?;

        Ok(())
    }

    #[instrument(name = "workspace_snapshot.detect_updates", level = "debug", skip_all)]
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

    #[instrument(name = "workspace_snapshot.detect_changes", level = "debug", skip_all)]
    pub async fn detect_changes(
        &self,
        onto_workspace_snapshot: &WorkspaceSnapshot,
    ) -> WorkspaceSnapshotResult<Vec<Change>> {
        let self_clone = self.clone();
        let onto_clone = onto_workspace_snapshot.clone();

        Ok(slow_rt::spawn(async move {
            self_clone
                .working_copy()
                .await
                .detect_changes(&*onto_clone.working_copy().await)
        })?
        .await??)
    }

    /// Gives the exact node index endpoints of an edge.
    pub async fn edge_endpoints(
        &self,
        edge_index: EdgeIndex,
    ) -> WorkspaceSnapshotResult<(Ulid, Ulid)> {
        let (source_idx, target_idx) = self.working_copy().await.edge_endpoints(edge_index)?;

        let source_id = self
            .working_copy()
            .await
            .node_index_to_id(source_idx)
            .ok_or(WorkspaceSnapshotError::NodeNotFoundAtIndex(source_idx))?;
        let target_id = self
            .working_copy()
            .await
            .node_index_to_id(target_idx)
            .ok_or(WorkspaceSnapshotError::NodeNotFoundAtIndex(target_idx))?;

        Ok((source_id, target_id))
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

    pub async fn get_node_weight(
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

    pub async fn get_node_weight_opt(&self, id: impl Into<Ulid>) -> Option<NodeWeight> {
        let working_copy = self.working_copy().await;
        working_copy
            .get_node_index_by_id_opt(id)
            .and_then(|node_index| working_copy.get_node_weight_opt(node_index))
            .cloned()
    }

    /// Remove any nodes without incoming edges from the graph, and update the
    /// index tables. If you are about to persist the graph, or calculate
    /// updates based on this graph and another one, then you want to call
    /// `Self::cleanup_and_merkle_tree_hash` instead.
    pub async fn cleanup(&self) -> WorkspaceSnapshotResult<()> {
        self.working_copy_mut().await.cleanup();
        Ok(())
    }

    /// Remove any orphaned nodes from the graph, update indexes then
    /// recalculate the merkle tree hash based on the nodes touched. *ALWAYS*
    /// call this before persisting a snapshot, or calculating updates (it is
    /// called already in `Self::write` and `Self::calculate_rebase_batch`)
    pub async fn cleanup_and_merkle_tree_hash(&self) -> WorkspaceSnapshotResult<()> {
        let mut working_copy = self.working_copy_mut().await;

        working_copy.cleanup_and_merkle_tree_hash()?;

        Ok(())
    }

    #[instrument(name = "workspace_snapshot.nodes", level = "debug", skip_all, fields())]
    pub async fn nodes(&self) -> WorkspaceSnapshotResult<Vec<NodeWeight>> {
        Ok(self
            .working_copy()
            .await
            .nodes()
            .map(|(weight, _)| weight)
            .cloned()
            .collect())
    }

    #[instrument(name = "workspace_snapshot.edges", level = "debug", skip_all, fields())]
    pub async fn edges(&self) -> WorkspaceSnapshotResult<Vec<(EdgeWeight, Ulid, Ulid)>> {
        let working_copy = self.working_copy().await;
        Ok(working_copy
            .edges()
            .filter_map(|(weight, from, to)| {
                working_copy
                    .node_index_to_id(from)
                    .zip(working_copy.node_index_to_id(to))
                    .map(|(from_id, to_id)| (weight.to_owned(), from_id, to_id))
            })
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

    /// Write the read only snapshot to disk. *WARNING* can panic! Use only for debugging
    pub fn write_readonly_graph_to_disk(&self, file_suffix: &str) {
        self.read_only_graph.write_to_disk(file_suffix);
    }

    async fn get_node_index_by_id(
        &self,
        id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<NodeIndex> {
        Ok(self.working_copy().await.get_node_index_by_id(id)?)
    }

    async fn get_node_index_by_id_opt(&self, id: impl Into<Ulid>) -> Option<NodeIndex> {
        self.working_copy().await.get_node_index_by_id_opt(id)
    }

    pub async fn node_exists(&self, id: impl Into<Ulid>) -> bool {
        self.get_node_index_by_id_opt(id).await.is_some()
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

    pub async fn get_category_node_or_err(
        &self,
        kind: CategoryNodeKind,
    ) -> WorkspaceSnapshotResult<Ulid> {
        self.get_category_node(kind)
            .await?
            .ok_or(WorkspaceSnapshotError::CategoryNodeNotFound(kind))
    }

    pub async fn get_category_node(
        &self,
        kind: CategoryNodeKind,
    ) -> WorkspaceSnapshotResult<Option<Ulid>> {
        Ok(self
            .working_copy()
            .await
            .get_category_node(kind)?
            .map(|(category_node_id, _)| category_node_id))
    }

    pub async fn edges_directed(
        &self,
        id: impl Into<Ulid>,
        direction: Direction,
    ) -> WorkspaceSnapshotResult<Vec<(EdgeWeight, Ulid, Ulid)>> {
        let working_copy = self.working_copy().await;

        let node_index = working_copy.get_node_index_by_id(id)?;
        Ok(working_copy
            .edges_directed(node_index, direction)
            .filter_map(|edge_ref| {
                working_copy
                    .node_index_to_id(edge_ref.source())
                    .zip(working_copy.node_index_to_id(edge_ref.target()))
                    .map(|(source, target)| (edge_ref.weight().to_owned(), source, target))
            })
            .collect())
    }

    pub async fn edges_directed_for_edge_weight_kind(
        &self,
        id: impl Into<Ulid>,
        direction: Direction,
        edge_kind: EdgeWeightKindDiscriminants,
    ) -> WorkspaceSnapshotResult<Vec<(EdgeWeight, Ulid, Ulid)>> {
        let working_copy = self.working_copy().await;
        let node_index = working_copy.get_node_index_by_id(id)?;

        Ok(working_copy
            .edges_directed_for_edge_weight_kind(node_index, direction, edge_kind)
            .filter_map(|(edge_weight, source_idx, target_idx)| {
                working_copy
                    .node_index_to_id(source_idx)
                    .zip(working_copy.node_index_to_id(target_idx))
                    .map(|(source_id, target_id)| (edge_weight, source_id, target_id))
            })
            .collect())
    }

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

    pub async fn incoming_sources_for_edge_weight_kind(
        &self,
        id: impl Into<Ulid>,
        edge_weight_kind_discrim: EdgeWeightKindDiscriminants,
    ) -> WorkspaceSnapshotResult<Vec<Ulid>> {
        Ok(self
            .edges_directed(id.into(), Direction::Incoming)
            .await?
            .into_iter()
            .filter_map(|(edge_weight, source_id, _)| {
                if edge_weight_kind_discrim == edge_weight.kind().into() {
                    Some(source_id)
                } else {
                    None
                }
            })
            .collect())
    }

    pub async fn source_opt(
        &self,
        id: impl Into<Ulid>,
        edge_weight_kind_discrim: EdgeWeightKindDiscriminants,
    ) -> WorkspaceSnapshotResult<Option<Ulid>> {
        let index = self.get_node_index_by_id(id).await?;
        let working_copy = self.working_copy().await;
        Ok(
            match working_copy.source_opt(index, edge_weight_kind_discrim)? {
                Some(source_index) => Some(working_copy.get_node_weight(source_index)?.id()),
                None => None,
            },
        )
    }

    pub async fn outgoing_targets_for_edge_weight_kind(
        &self,
        id: impl Into<Ulid>,
        edge_weight_kind_discrim: EdgeWeightKindDiscriminants,
    ) -> WorkspaceSnapshotResult<Vec<Ulid>> {
        let id = id.into();
        Ok(self
            .edges_directed(id, Direction::Outgoing)
            .await?
            .into_iter()
            .filter_map(|(edge_weight, _, target_id)| {
                if edge_weight_kind_discrim == edge_weight.kind().into() {
                    Some(target_id)
                } else {
                    None
                }
            })
            .collect())
    }

    pub async fn all_outgoing_targets(
        &self,
        id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<Vec<NodeWeight>> {
        let mut result = vec![];
        let target_ids: Vec<_> = self
            .edges_directed(id, Direction::Outgoing)
            .await?
            .into_iter()
            .map(|(_, _, target_id)| target_id)
            .collect();

        for target_id in target_ids {
            let node_weight = self.get_node_weight(target_id).await?;
            result.push(node_weight.to_owned());
        }

        Ok(result)
    }

    pub async fn all_incoming_sources(
        &self,
        id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<Vec<NodeWeight>> {
        let mut result = vec![];
        let source_ids: Vec<Ulid> = self
            .edges_directed(id, Direction::Incoming)
            .await?
            .into_iter()
            .map(|(_, source_id, _)| source_id)
            .collect();

        for source_id in source_ids {
            let node_weight = self.get_node_weight(source_id).await?;
            result.push(node_weight.to_owned());
        }

        Ok(result)
    }

    pub async fn remove_incoming_edges_of_kind(
        &self,
        target_id: impl Into<Ulid>,
        kind: EdgeWeightKindDiscriminants,
    ) -> WorkspaceSnapshotResult<()> {
        let target_id = target_id.into();

        let sources = self
            .incoming_sources_for_edge_weight_kind(target_id, kind)
            .await?;
        for source_id in sources {
            self.remove_edge(source_id, target_id, kind).await?;
        }

        Ok(())
    }

    pub async fn remove_outgoing_edges_of_kind(
        &self,
        source_id: impl Into<Ulid>,
        kind: EdgeWeightKindDiscriminants,
    ) -> WorkspaceSnapshotResult<()> {
        let source_id = source_id.into();

        let targets = self
            .outgoing_targets_for_edge_weight_kind(source_id, kind)
            .await?;
        for target_id in targets {
            self.remove_edge(source_id, target_id, kind).await?;
        }

        Ok(())
    }

    pub async fn get_edges_between_nodes(
        &self,
        from_node_id: Ulid,
        to_node_id: Ulid,
    ) -> WorkspaceSnapshotResult<Vec<EdgeWeight>> {
        let edges = self
            .edges()
            .await?
            .into_iter()
            .filter_map(|(edge, node_from, node_to)| {
                if node_from == from_node_id && node_to == to_node_id {
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

    pub async fn remove_edge(
        &self,
        source_id: impl Into<Ulid>,
        target_id: impl Into<Ulid>,
        edge_kind: EdgeWeightKindDiscriminants,
    ) -> WorkspaceSnapshotResult<()> {
        let source_node_index = self.get_node_index_by_id(source_id).await?;
        let target_node_index = self.get_node_index_by_id(target_id).await?;

        self.working_copy_mut().await.remove_edge(
            source_node_index,
            target_node_index,
            edge_kind,
        )?;

        Ok(())
    }

    pub async fn find_edge(
        &self,
        from_id: impl Into<Ulid>,
        to_id: impl Into<Ulid>,
        edge_weight_kind: EdgeWeightKindDiscriminants,
    ) -> Option<EdgeWeight> {
        let working_copy = self.working_copy().await;

        let (from_idx, to_idx) = working_copy
            .get_node_index_by_id_opt(from_id)
            .zip(working_copy.get_node_index_by_id_opt(to_id))?; // `?` works on Option, too

        working_copy
            .find_edge(from_idx, to_idx, edge_weight_kind)
            .map(ToOwned::to_owned)
    }

    /// Perform [`Updates`](Update) using [`self`](WorkspaceSnapshot) as the "to rebase" graph and
    /// another [`snapshot`](WorkspaceSnapshot) as the "onto" graph.
    #[instrument(
        name = "workspace_snapshot.perform_updates",
        level = "debug",
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

    pub async fn update_node_id(
        &self,
        current_id: impl Into<Ulid>,
        new_id: impl Into<Ulid>,
        new_lineage_id: LineageId,
    ) -> WorkspaceSnapshotResult<()> {
        let idx = self.get_node_index_by_id(current_id).await?;
        self.working_copy_mut()
            .await
            .update_node_id(idx, new_id, new_lineage_id)?;

        Ok(())
    }

    pub async fn ordered_children_for_node(
        &self,
        id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<Option<Vec<Ulid>>> {
        let idx = self.get_node_index_by_id(id.into()).await?;
        let mut result = vec![];
        let working_copy = self.working_copy().await;
        Ok(
            if let Some(idxs) = working_copy.ordered_children_for_node(idx)? {
                for idx in idxs {
                    if let Some(id) = working_copy.node_index_to_id(idx) {
                        result.push(id);
                    }
                }
                Some(result)
            } else {
                None
            },
        )
    }

    pub async fn dvu_root_check(&self, root: DependentValueRoot) -> bool {
        // ensure we don't grow the graph unnecessarily by adding the same value
        // in a single edit session
        let mut dvu_roots = self.dvu_roots.lock().await;

        if dvu_roots.contains(&root) {
            true
        } else {
            dvu_roots.insert(root);
            false
        }
    }

    pub async fn schema_variant_id_for_component_id(
        &self,
        component_id: ComponentId,
    ) -> ComponentResult<SchemaVariantId> {
        self.working_copy()
            .await
            .schema_variant_id_for_component_id(component_id)
    }

    pub async fn frame_contains_components(
        &self,
        component_id: ComponentId,
    ) -> ComponentResult<Vec<ComponentId>> {
        self.working_copy()
            .await
            .frame_contains_components(component_id)
    }

    pub async fn inferred_connection_graph(
        &self,
        ctx: &DalContext,
    ) -> WorkspaceSnapshotResult<InferredConnectionsWriteGuard<'_>> {
        let mut inferred_connection_write_guard = self.inferred_connection_graph.write().await;
        if inferred_connection_write_guard.is_none() {
            *inferred_connection_write_guard =
                Some(InferredConnectionGraph::new(ctx).await.map_err(Box::new)?);
        }

        Ok(InferredConnectionsWriteGuard {
            inferred_connection_graph: inferred_connection_write_guard,
        })
    }

    pub async fn clear_inferred_connection_graph(&self) {
        let mut inferred_connection_write_guard = self.inferred_connection_graph.write().await;
        *inferred_connection_write_guard = None;
    }

    /// Validate the snapshot in the given DalContext
    pub async fn validate(
        &self,
    ) -> WorkspaceSnapshotResult<Vec<(graph::validator::ValidationIssue, String)>> {
        Ok(graph::validator::validate_graph_with_text(
            &self.working_copy().await,
        )?)
    }

    /// Get the connection migrations that are available for this snapshot.
    pub async fn connection_migrations(
        &self,
        inferred_connections: impl IntoIterator<Item = SocketConnection>,
    ) -> WorkspaceSnapshotResult<Vec<graph::validator::connections::ConnectionMigration>> {
        Ok(graph::validator::connections::connection_migrations(
            &self.working_copy().await,
            inferred_connections,
        ))
    }
}
