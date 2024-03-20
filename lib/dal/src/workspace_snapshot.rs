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

use crate::workspace_snapshot::node_weight::CategoryNodeWeight;
use chrono::Utc;
use futures::executor;
use petgraph::visit::DfsEvent;
use si_events::NodeWeightAddress;
use si_layer_cache::db::node_weight::NodeWeightDb;
use si_layer_cache::persister::PersistStatus;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use tokio::task::JoinError;
use tokio::time::Instant;

use petgraph::prelude::*;
use si_data_pg::PgError;
use si_events::ContentHash;
use si_events::WorkspaceSnapshotAddress;
use telemetry::prelude::*;
use thiserror::Error;
use ulid::Ulid;

use crate::change_set_pointer::{ChangeSetId, ChangeSetPointer, ChangeSetPointerError};
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

#[remain::sorted]
#[derive(Error, Debug)]
pub enum WorkspaceSnapshotError {
    #[error("Category node not found: {0:?}")]
    CategoryNodeNotFound(NodeIndex),
    #[error("change set pointer error: {0}")]
    ChangeSetPointer(#[from] ChangeSetPointerError),
    #[error("change set pointer {0} has no workspace snapshot address")]
    ChangeSetPointerMissingWorkspaceSnapshotAddress(ChangeSetId),
    #[error("edge weight error: {0}")]
    EdgeWeight(#[from] EdgeWeightError),
    #[error("JoinError: {0:?}")]
    Join(#[from] JoinError),
    #[error("layer db error: {0}")]
    LayerDb(#[from] si_layer_cache::LayerDbError),
    #[error("missing content from store for id: {0}")]
    MissingContentFromStore(Ulid),
    #[error("monotonic error: {0}")]
    Monotonic(#[from] ulid::MonotonicError),
    #[error("NodeWeight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("NodeWeight not found at {0:?}")]
    NodeWeightMissing(NodeWeightAddress),
    #[error("Node with id {0} not found")]
    NodeWithIdNotFound(Ulid),
    #[error("si_data_pg error: {0}")]
    Pg(#[from] PgError),
    #[error("postcard error: {0}")]
    Postcard(#[from] postcard::Error),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("NodeIndex has too many Ordering children: {0:?}")]
    TooManyOrderingForNode(Ulid),
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
}

pub type WorkspaceSnapshotResult<T> = Result<T, WorkspaceSnapshotError>;

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
    node_weight_db: NodeWeightDb<NodeWeight>,
    events_actor: si_events::Actor,
    events_tenancy: si_events::Tenancy,
}

struct SnapshotReadGuard<'a> {
    read_only_graph: Arc<WorkspaceSnapshotGraph>,
    working_copy_read_guard: RwLockReadGuard<'a, Option<WorkspaceSnapshotGraph>>,
}

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
    #[instrument(level = "debug", skip_all)]
    pub async fn initial(
        ctx: &DalContext,
        change_set: &ChangeSetPointer,
    ) -> WorkspaceSnapshotResult<Self> {
        let root_node = Arc::new(NodeWeight::new_content(
            change_set,
            change_set.generate_ulid()?,
            content_address::ContentAddress::Root,
        )?);
        let root_id = root_node.id();
        let (node_address, _) = ctx
            .layer_db()
            .node_weight()
            .write(
                root_node.clone(),
                None,
                ctx.events_tenancy(),
                ctx.events_actor(),
            )
            .await?;

        let graph: WorkspaceSnapshotGraph = WorkspaceSnapshotGraph::new(root_node, node_address)?;

        let initial = Self {
            address: Arc::new(RwLock::new(WorkspaceSnapshotAddress::nil())),
            read_only_graph: Arc::new(graph),
            working_copy: Arc::new(RwLock::new(None)),
            node_weight_db: ctx.layer_db().node_weight().clone(),
            events_actor: ctx.events_actor(),
            events_tenancy: ctx.events_tenancy(),
        };

        // Create the category nodes under root.
        let component_node_id = initial
            .add_category_node(change_set, CategoryNodeKind::Component)
            .await?;
        let func_node_id = initial
            .add_category_node(change_set, CategoryNodeKind::Func)
            .await?;
        let action_batch_node_id = initial
            .add_category_node(change_set, CategoryNodeKind::ActionBatch)
            .await?;
        let schema_node_id = initial
            .add_category_node(change_set, CategoryNodeKind::Schema)
            .await?;
        let secret_node_id = initial
            .add_category_node(change_set, CategoryNodeKind::Secret)
            .await?;
        // Connect them to root.
        initial
            .add_edge(
                root_id,
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())?,
                action_batch_node_id,
            )
            .await?;
        initial
            .add_edge(
                root_id,
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())?,
                component_node_id,
            )
            .await?;
        initial
            .add_edge(
                root_id,
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())?,
                func_node_id,
            )
            .await?;
        initial
            .add_edge(
                root_id,
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())?,
                schema_node_id,
            )
            .await?;
        initial
            .add_edge(
                root_id,
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())?,
                secret_node_id,
            )
            .await?;
        initial.write(ctx, change_set.vector_clock_id()).await?;

        Ok(initial)
    }

    pub fn events_actor(&self) -> si_events::Actor {
        self.events_actor.clone()
    }

    pub fn events_tenancy(&self) -> si_events::Tenancy {
        self.events_tenancy
    }

    pub async fn mark_graph_seen(
        &self,
        vector_clock_id: VectorClockId,
    ) -> WorkspaceSnapshotResult<()> {
        let mut updates = vec![];
        let seen_at = Utc::now();
        for edge in self.working_copy_mut().await.graph_mut().edge_weights_mut() {
            edge.mark_seen_at(vector_clock_id, seen_at);
        }

        for node_index in self.working_copy().await.graph().node_indices() {
            let mut remote_node_weight = self.get_node_weight(node_index).await?.as_ref().clone();
            remote_node_weight.mark_seen_at(vector_clock_id, seen_at);
            let node_id = remote_node_weight.id();
            let (new_address, _) = self
                .node_weight_db
                .write(
                    Arc::new(remote_node_weight),
                    None,
                    self.events_tenancy(),
                    self.events_actor(),
                )
                .await?;
            updates.push((node_index, new_address, node_id));
        }
        for (index, address, node_id) in updates {
            self.working_copy_mut()
                .await
                .update_node_weight_address(index, address, node_id)?;
        }

        Ok(())
    }

    #[instrument(level = "debug", skip_all)]
    pub async fn write(
        &self,
        ctx: &DalContext,
        vector_clock_id: VectorClockId,
    ) -> WorkspaceSnapshotResult<WorkspaceSnapshotAddress> {
        // Pull out the working copy and clean it up.
        let new_address = {
            self.cleanup().await?;

            info!("after cleanup");
            // Mark everything left as seen.
            self.mark_graph_seen(vector_clock_id).await?;
            info!("after mark");

            let (new_address, status_reader) = ctx
                .layer_db()
                .workspace_snapshot()
                .write(
                    Arc::new(self.working_copy().await.clone()),
                    None,
                    ctx.events_tenancy(),
                    ctx.events_actor(),
                )
                .await?;

            if let PersistStatus::Error(e) = status_reader.get_status().await? {
                return Err(e)?;
            }

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

    async fn working_copy(&self) -> SnapshotReadGuard<'_> {
        SnapshotReadGuard {
            read_only_graph: self.read_only_graph.clone(),
            working_copy_read_guard: self.working_copy.read().await,
        }
    }

    async fn working_copy_mut(&self) -> SnapshotWriteGuard<'_> {
        if self.working_copy.read().await.is_none() {
            // Make a copy of the read only graph as our new working copy
            *self.working_copy.write().await = Some(self.read_only_graph.as_ref().clone());
        }

        SnapshotWriteGuard {
            working_copy_write_guard: self.working_copy.write().await,
        }
    }

    pub async fn add_node(&self, node: NodeWeight) -> WorkspaceSnapshotResult<NodeIndex> {
        let node_id = node.id();
        let node_arc = Arc::new(node);
        let (hash, _) = self
            .node_weight_db
            .write(
                node_arc.clone(),
                None,
                self.events_tenancy(),
                self.events_actor(),
            )
            .await?;

        let maybe_existing_node_index = self.get_node_index_by_id_opt(node_id).await;
        self.working_copy_mut().await.add_node(node_arc, hash)?;

        // If we are replacing an existing node, we need to replace all references to it
        if let Some(existing_node_index) = maybe_existing_node_index {
            self.replace_references(existing_node_index).await?;
        }

        self.get_node_index_by_id(node_id).await
    }

    pub async fn add_ordered_node(
        &self,
        change_set: &ChangeSetPointer,
        node: NodeWeight,
    ) -> WorkspaceSnapshotResult<NodeIndex> {
        let new_node_index = self.add_node(node).await?;
        let ordering_node_index = self
            .add_node(NodeWeight::Ordering(OrderingNodeWeight::new(change_set)?))
            .await?;
        let edge_index = self
            .add_edge_unchecked(
                new_node_index,
                EdgeWeight::new(change_set, EdgeWeightKind::Ordering)?,
                ordering_node_index,
            )
            .await?;
        let (source, _) = self.edge_endpoints(edge_index).await?;
        Ok(source)
    }

    pub async fn add_category_node(
        &self,
        change_set: &ChangeSetPointer,
        kind: CategoryNodeKind,
    ) -> WorkspaceSnapshotResult<Ulid> {
        let inner_weight = CategoryNodeWeight::new(change_set, kind)?;
        let node_id = inner_weight.id();
        self.add_node(NodeWeight::Category(inner_weight)).await?;
        Ok(node_id)
    }

    pub async fn update_content(
        &self,
        change_set: &ChangeSetPointer,
        id: Ulid,
        new_content_hash: ContentHash,
    ) -> WorkspaceSnapshotResult<()> {
        let node_weight_index = self.get_node_index_by_id(id).await?;
        let node_weight = self.get_node_weight(node_weight_index).await?;
        // we have to copy it to modify it
        let mut node_weight = node_weight.as_ref().clone();

        node_weight.increment_vector_clock(change_set)?;
        node_weight.new_content_hash(new_content_hash)?;

        self.add_node(node_weight).await?;

        Ok(())
    }

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
        Ok(self
            .working_copy_mut()
            .await
            .add_edge(from_node_index, edge_weight, to_node_index)?)
    }

    // NOTE(nick): this should only be used by the rebaser and in specific scenarios where the
    // indices are definitely correct.
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

    pub async fn add_ordered_edge(
        &self,
        change_set: &ChangeSetPointer,
        from_node_id: impl Into<Ulid>,
        edge_weight: EdgeWeight,
        to_node_id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<EdgeIndex> {
        let from_node_id = from_node_id.into();
        let to_node_id = to_node_id.into();

        let start = Instant::now();
        let new_edge_index = self.add_edge(from_node_id, edge_weight, to_node_id).await?;
        info!("add edge took {:?}", start.elapsed());

        // Find the ordering node of the "container" if there is one, and add the thing pointed to
        // by the `to_node_id` to the ordering. Also point the ordering node at the thing with
        // an `Ordinal` edge, so that Ordering nodes must be touched *after* the things they order
        // in a depth first search
        let start = Instant::now();
        if let Some(mut container_ordering_node) =
            self.ordering_node_for_container(from_node_id).await?
        {
            info!("fetched ordering node in {:?}", start.elapsed());

            let start = Instant::now();
            self.add_edge(
                container_ordering_node.id(),
                EdgeWeight::new(change_set, EdgeWeightKind::Ordinal)?,
                to_node_id,
            )
            .await?;
            info!("added ordinal edge in {:?}", start.elapsed());

            container_ordering_node.push_to_order(change_set, to_node_id)?;
            let start = Instant::now();
            self.add_node(NodeWeight::Ordering(container_ordering_node))
                .await?;
            info!("replaced ordering node in {:?}", start.elapsed());
            info!(
                "address map size: {}",
                self.working_copy().await.address_map_len()
            );
        };

        Ok(new_edge_index)
    }

    #[instrument(level = "debug", skip_all)]
    pub async fn detect_conflicts_and_updates(
        &self,
        to_rebase_vector_clock_id: VectorClockId,
        onto_workspace_snapshot: &WorkspaceSnapshot,
        onto_vector_clock_id: VectorClockId,
    ) -> WorkspaceSnapshotResult<(Vec<Conflict>, Vec<Update>)> {
        let onto_workspace_snapshot = onto_workspace_snapshot.clone();
        let self_clone = self.clone();
        let handle = tokio::task::spawn_blocking(move || {
            let onto_graph = executor::block_on(onto_workspace_snapshot.working_copy());
            let mut conflicts: Vec<Conflict> = Vec::new();
            let mut updates: Vec<Update> = Vec::new();
            if let Err(traversal_error) = petgraph::visit::depth_first_search(
                onto_graph.graph(),
                Some(onto_graph.root()),
                |event| {
                    self_clone.detect_conflicts_and_updates_process_dfs_event(
                        to_rebase_vector_clock_id,
                        &onto_workspace_snapshot,
                        onto_vector_clock_id,
                        event,
                        &mut conflicts,
                        &mut updates,
                    )
                },
            ) {
                return Err(WorkspaceSnapshotGraphError::GraphTraversal(traversal_error));
            };
            Ok((conflicts, updates))
        });

        let (conflicts, updates) = handle.await??;

        Ok((conflicts, updates))
    }

    #[allow(clippy::too_many_arguments)]
    #[instrument(level = "debug", skip_all)]
    fn detect_conflicts_and_updates_process_dfs_event(
        &self,
        to_rebase_vector_clock_id: VectorClockId,
        onto: &WorkspaceSnapshot,
        onto_vector_clock_id: VectorClockId,
        event: DfsEvent<NodeIndex>,
        conflicts: &mut Vec<Conflict>,
        updates: &mut Vec<Update>,
    ) -> Result<petgraph::visit::Control<()>, petgraph::visit::DfsEvent<NodeIndex>> {
        match event {
            DfsEvent::Discover(onto_node_index, _) => {
                let onto_node_weight = executor::block_on(onto.get_node_weight(onto_node_index))
                    .map_err(|err| {
                        error!(
                            "Unable to get NodeWeight for onto NodeIndex {:?}: {}",
                            onto_node_index, err,
                        );
                        event
                    })?;

                let onto_local_node_weight = executor::block_on(onto.working_copy())
                    .get_node_weight(onto_node_index)
                    .map_err(|err| {
                        error!(
                            "Unable to get graph local node weight for NodeIndex {:?} in onto, {}",
                            onto_node_index, err
                        );
                        event
                    })?;

                let mut to_rebase_node_indexes = HashSet::new();
                let onto_root_node_index = executor::block_on(onto.root()).map_err(|err| {
                    error!("Unable to get root index for onto: {}", err);
                    event
                })?;

                if onto_node_index == onto_root_node_index {
                    // There can only be one (valid/current) `ContentAddress::Root` at any
                    // given moment, and the `lineage_id` isn't really relevant as it's not
                    // globally stable (even though it is locally stable). This matters as we
                    // may be dealing with a `WorkspaceSnapshotGraph` that is coming to us
                    // externally from a module that we're attempting to import. The external
                    // `WorkspaceSnapshotGraph` will be `self`, and the "local" one will be
                    // `onto`.
                    to_rebase_node_indexes.insert(executor::block_on(self.root()).map_err(
                        |err| {
                            error!("Unable to get root index for self: {}", err);
                            event
                        },
                    )?);
                } else {
                    // Only retain node indexes... or indices... if they are part of the current
                    // graph. There may still be garbage from previous updates to the graph
                    // laying around.
                    let mut potential_to_rebase_node_indexes =
                        executor::block_on(self.working_copy())
                            .get_node_index_by_lineage(onto_node_weight.lineage_id());

                    potential_to_rebase_node_indexes.retain(|node_index| {
                        executor::block_on(self.working_copy()).has_path_to_root(*node_index)
                    });
                    to_rebase_node_indexes.extend(potential_to_rebase_node_indexes);

                    // TODO(nick): detect category nodes with a different lineage. We will likely
                    // need to check incoming edges in one graph and then look for outgoing edges in
                    // the other graph.
                    // // Since category nodes may be created from scratch from a different workspace,
                    // // they may have different lineage ids. We still want to consider the same
                    // // category kind as an equivalent node, even though it might have a different
                    // // lineage id.
                    // if let NodeWeight::Category(onto_category_node_weight) = onto_node_weight {
                    //     onto_category_node_weight
                    // }
                    //     let category_node_kind = onto_category_node_weight.kind();
                    //     let (_, to_rebase_category_node_index) =
                    //         self.get_category_node(Some(onto_category_node_weight.id()), category_node_kind).map_err(|err| {
                    //             error!(
                    //                 "Unable to get to rebase Category node for kind {:?} from onto {:?}: {}",
                    //                 onto_category_node_weight.kind(), onto, err,
                    //             );
                    //             event
                    //         })?;
                    //     to_rebase_node_indexes.insert(to_rebase_category_node_index);
                    // }
                }

                // We'll lazily populate these, since we don't know if we'll need it at all, and
                // we definitely don't want to be re-fetching this information inside the loop
                // below, as it will be identical every time.
                let mut onto_ordering_node = None;

                // If everything with the same `lineage_id` is identical, then we can prune the
                // graph traversal, and avoid unnecessary lookups/comparisons.
                let mut any_content_with_lineage_has_changed = false;

                for to_rebase_node_index in to_rebase_node_indexes {
                    let to_rebase_local_node_weight= executor::block_on(self.working_copy()).get_node_weight(to_rebase_node_index).map_err(
                        |err| {
                            error!("Unable to get graph local node weight for NodeIndex {:?} on self, {}", to_rebase_node_index, err);
                            event
                        })?;

                    let to_rebase_node_weight = executor::block_on(
                        self.get_node_weight(to_rebase_node_index),
                    )
                    .map_err(|err| {
                        error!(
                            "Unable to get to_rebase NodeWeight for NodeIndex {:?}: {}",
                            to_rebase_node_index, err,
                        );
                        event
                    })?;

                    if onto_local_node_weight.merkle_tree_hash()
                        == to_rebase_local_node_weight.merkle_tree_hash()
                    {
                        // If the merkle tree hashes are the same, then the entire sub-graph is
                        // identical, and we don't need to check any further.
                        debug!(
                            "onto {} and to rebase {} merkle tree hashes are the same",
                            onto_node_weight.merkle_tree_hash(),
                            to_rebase_node_weight.merkle_tree_hash(),
                        );
                        continue;
                    }
                    any_content_with_lineage_has_changed = true;

                    // Check if there's a difference in the node itself (and whether it is a
                    // conflict if there is a difference).
                    if onto_local_node_weight.address() != onto_local_node_weight.address() {
                        if to_rebase_node_weight
                            .vector_clock_write()
                            .is_newer_than(onto_node_weight.vector_clock_write())
                        {
                            // The existing node (`to_rebase`) has changes, but has already seen
                            // all of the changes in `onto`. There is no conflict, and there is
                            // nothing to update.
                        } else if onto_node_weight
                            .vector_clock_write()
                            .is_newer_than(to_rebase_node_weight.vector_clock_write())
                        {
                            // `onto` has changes, but has already seen all of the changes in
                            // `to_rebase`. There is no conflict, and we should update to use the
                            // `onto` node.
                            updates.push(Update::ReplaceSubgraph {
                                onto: onto_node_index,
                                to_rebase: to_rebase_node_index,
                            });
                        } else {
                            // There are changes on both sides that have not
                            // been seen by the other side; this is a conflict.
                            // There may also be other conflicts in the outgoing
                            // relationships, the downstream nodes, or both.

                            // If the nodes in question are ordering nodes, the
                            // conflict we care about is the ChildOrder
                            // conflict, and will have already been detected.
                            // The content on the ordering node is just the
                            // ordering of the edges, so what matters if there
                            // is a conflict in order, not if the hashes differ
                            // because there is an extra edge (but the rest of
                            // the edges are ordered the same)
                            if !matches!(
                                (onto_node_weight.as_ref(), to_rebase_node_weight.as_ref()),
                                (
                                    NodeWeight::Ordering(OrderingNodeWeight { .. }),
                                    NodeWeight::Ordering(OrderingNodeWeight { .. })
                                )
                            ) {
                                conflicts.push(Conflict::NodeContent {
                                    to_rebase: to_rebase_node_index,
                                    onto: onto_node_index,
                                });
                            }
                        }
                    }

                    if onto_ordering_node.is_none() {
                        onto_ordering_node = executor::block_on(
                            onto.ordering_node_for_container(onto_node_weight.id()),
                        )
                        .map_err(|_| event)?;
                    }
                    let to_rebase_ordering_node = executor::block_on(
                        self.ordering_node_for_container(to_rebase_node_weight.id()),
                    )
                    .map_err(|_| event)?;

                    match (&to_rebase_ordering_node, &onto_ordering_node) {
                        (None, None) => {
                            // Neither is ordered. The potential conflict could be because one
                            // or more elements changed, because elements were added/removed,
                            // or a combination of these.
                            //
                            // We need to check for all of these using the outgoing edges from
                            // the containers, since we can't rely on an ordering child to
                            // contain all the information to determine ordering/addition/removal.
                            //
                            // Eventually, this will only happen on the root node itself, since
                            // Objects, Maps, and Arrays should all have an ordering, for at
                            // least display purposes.
                            info!(
                                "Found what appears to be two unordered containers: onto {:?}, to_rebase {:?}",
                                onto_node_index, to_rebase_node_index,
                            );
                            info!(
                                "Comparing unordered containers: {:?}, {:?}",
                                onto_node_index, to_rebase_node_index
                            );

                            let (container_conflicts, container_updates) = executor::block_on(self
                                .find_unordered_container_membership_conflicts_and_updates(
                                    to_rebase_vector_clock_id,
                                    to_rebase_node_index,
                                    onto,
                                    onto_vector_clock_id,
                                    onto_node_index,
                                ))
                                .map_err(|err| {
                                    error!("Unable to find unordered container membership conflicts and updates for onto container NodeIndex {:?} and to_rebase container NodeIndex {:?}: {}", onto_node_index, to_rebase_node_index, err);
                                    event
                                })?;

                            updates.extend(container_updates);
                            conflicts.extend(container_conflicts);
                        }
                        (None, Some(_)) | (Some(_), None) => {
                            // We're trying to compare an ordered container with an unordered one,
                            // which isn't something that logically makes sense, so we've likely
                            // started comparing incompatible things.
                            warn!(
                                "Attempting to compare an ordered, and an unordered container: onto {:?}, to_rebase {:?}",
                                onto_node_index, to_rebase_node_index,
                            );
                            return Err(event);
                        }
                        (Some(to_rebase_ordering_node), Some(onto_ordering_node)) => {
                            info!(
                                "Comparing ordered containers: {:?}, {:?}",
                                onto_node_index, to_rebase_node_index
                            );
                            let (container_conflicts, container_updates) = executor::block_on(
                                self.find_ordered_container_membership_conflicts_and_updates(
                                    to_rebase_vector_clock_id,
                                    to_rebase_node_index,
                                    to_rebase_ordering_node,
                                    onto,
                                    onto_vector_clock_id,
                                    onto_node_index,
                                    onto_ordering_node,
                                ),
                            )
                            .map_err(|_| event)?;

                            updates.extend(container_updates);
                            conflicts.extend(container_conflicts);

                            return Ok(petgraph::visit::Control::Continue);
                        }
                    }
                }

                if any_content_with_lineage_has_changed {
                    // There was at least one thing with a merkle tree hash difference, so we need
                    // to examine further down the tree to see where the difference(s) are, and
                    // where there are conflicts, if there are any.
                    Ok(petgraph::visit::Control::Continue)
                } else {
                    // Everything to be rebased is identical, so there's no need to examine the
                    // rest of the tree looking for differences & conflicts that won't be there.
                    Ok(petgraph::visit::Control::Prune)
                }
            }
            DfsEvent::TreeEdge(_, _)
            | DfsEvent::BackEdge(_, _)
            | DfsEvent::CrossForwardEdge(_, _)
            | DfsEvent::Finish(_, _) => {
                // These events are all ignored, since we handle looking at edges as we encounter
                // the node(s) the edges are coming from (Outgoing edges).
                Ok(petgraph::visit::Control::Continue)
            }
        }
    }

    async fn find_unordered_container_membership_conflicts_and_updates(
        &self,
        to_rebase_vector_clock_id: VectorClockId,
        to_rebase_container_index: NodeIndex,
        onto: &WorkspaceSnapshot,
        onto_vector_clock_id: VectorClockId,
        onto_container_index: NodeIndex,
    ) -> WorkspaceSnapshotResult<(Vec<Conflict>, Vec<Update>)> {
        #[derive(Debug, Clone, Hash, PartialEq, Eq)]
        struct UniqueEdgeInfo {
            pub kind: EdgeWeightKind,
            pub target_lineage: Ulid,
        }

        #[derive(Debug, Clone)]
        struct EdgeInfo {
            pub source_node_index: NodeIndex,
            pub target_node_index: NodeIndex,
            pub edge_weight: EdgeWeight,
        }

        let mut updates = Vec::new();
        let mut conflicts = Vec::new();

        let mut to_rebase_edges = HashMap::<UniqueEdgeInfo, EdgeInfo>::new();
        for (edge_weight, source_node_index, target_node_index) in self
            .edges_directed_by_index(to_rebase_container_index, Outgoing)
            .await?
        {
            let target_node_weight = self.get_node_weight(target_node_index).await?;
            to_rebase_edges.insert(
                UniqueEdgeInfo {
                    kind: edge_weight.kind().clone(),
                    target_lineage: target_node_weight.lineage_id(),
                },
                EdgeInfo {
                    source_node_index,
                    target_node_index,
                    edge_weight,
                },
            );
        }

        let mut onto_edges = HashMap::<UniqueEdgeInfo, EdgeInfo>::new();
        for (edge_weight, source_node_index, target_node_index) in onto
            .edges_directed_by_index(onto_container_index, Outgoing)
            .await?
        {
            let target_node_weight = onto.get_node_weight(target_node_index).await?;
            onto_edges.insert(
                UniqueEdgeInfo {
                    kind: edge_weight.kind().clone(),
                    target_lineage: target_node_weight.lineage_id(),
                },
                EdgeInfo {
                    source_node_index,
                    target_node_index,
                    edge_weight,
                },
            );
        }

        let only_to_rebase_edges = {
            let mut unique_edges = to_rebase_edges.clone();
            for key in onto_edges.keys() {
                unique_edges.remove(key);
            }
            unique_edges
        };
        let only_onto_edges = {
            let mut unique_edges = onto_edges.clone();
            for key in to_rebase_edges.keys() {
                unique_edges.remove(key);
            }
            unique_edges
        };

        debug!("only to rebase edges: {:?}", &only_to_rebase_edges);
        debug!("only onto edges: {:?}", &only_onto_edges);

        let root_seen_as_of_onto = self
            .get_node_weight(self.root().await?)
            .await?
            .vector_clock_recently_seen()
            .entry_for(onto_vector_clock_id);

        let onto_last_saw_to_rebase = onto
            .get_node_weight(onto.root().await?)
            .await?
            .vector_clock_recently_seen()
            .entry_for(to_rebase_vector_clock_id);

        for only_to_rebase_edge_info in only_to_rebase_edges.values() {
            let to_rebase_item_weight = self
                .get_node_weight(only_to_rebase_edge_info.target_node_index)
                .await?;

            // If `onto` has never seen this edge, then it's new, and there are no conflicts, and
            // no updates.
            if only_to_rebase_edge_info
                .edge_weight
                .vector_clock_first_seen()
                .entry_for(to_rebase_vector_clock_id)
                <= onto_last_saw_to_rebase
            {
                if to_rebase_item_weight
                    .vector_clock_write()
                    .entry_for(to_rebase_vector_clock_id)
                    >= onto_last_saw_to_rebase
                {
                    // Item has been modified in `onto` (`onto` item write vector clock > "seen as
                    // of" for `onto` entry in `to_rebase` root): Conflict (ModifyRemovedItem)
                    conflicts.push(Conflict::ModifyRemovedItem(
                        only_to_rebase_edge_info.target_node_index,
                    ))
                } else {
                    // Item not modified & removed by `onto`: No conflict; Update::RemoveEdge
                    updates.push(Update::RemoveEdge {
                        source: only_to_rebase_edge_info.source_node_index,
                        destination: only_to_rebase_edge_info.target_node_index,
                        edge_kind: only_to_rebase_edge_info.edge_weight.kind().into(),
                    });
                }
            } else {
                info!(
                    "edge weight entry for to rebase vector clock id {:?} is older than onto last saw {:?}",
                    only_to_rebase_edge_info.edge_weight.vector_clock_first_seen().entry_for(to_rebase_vector_clock_id),
                    onto_last_saw_to_rebase,
                );
            }
        }

        // - Items unique to `onto`:
        for only_onto_edge_info in only_onto_edges.values() {
            let onto_item_weight = onto
                .get_node_weight(only_onto_edge_info.target_node_index)
                .await?;

            if let Some(onto_first_seen) = only_onto_edge_info
                .edge_weight
                .vector_clock_first_seen()
                .entry_for(onto_vector_clock_id)
            {
                // From "onto_first_seen", we know "when was the first time onto saw this edge?".
                match root_seen_as_of_onto {
                    Some(root_seen_as_of) if onto_first_seen <= root_seen_as_of => {}
                    _ => {
                        // Edge first seen by `onto` > "seen as of" on `to_rebase` graph for `onto`'s entry on
                        // root node: Item is new.
                        // Other case where item is new: the `to_rebase` has never seen anything from
                        // the `onto` change set. All the items are new.
                        updates.push(Update::NewEdge {
                            source: to_rebase_container_index,
                            destination: only_onto_edge_info.target_node_index,
                            edge_weight: only_onto_edge_info.edge_weight.clone(),
                        });
                    }
                }
            } else if let Some(root_seen_as_of) = root_seen_as_of_onto {
                if onto_item_weight
                    .vector_clock_write()
                    .has_entries_newer_than(root_seen_as_of)
                {
                    // Item write vector clock has entries > "seen as of" on `to_rebase` graph for
                    // `onto`'s entry on root node: Conflict (RemoveModifiedItem)
                    conflicts.push(Conflict::RemoveModifiedItem {
                        container: to_rebase_container_index,
                        removed_item: only_onto_edge_info.target_node_index,
                    });
                }
            }
            // Item removed by `to_rebase`: No conflict & no update necessary.
        }

        // - Sets same: No conflicts/updates
        Ok((conflicts, updates))
    }

    #[allow(clippy::too_many_arguments)]
    async fn find_ordered_container_membership_conflicts_and_updates(
        &self,
        to_rebase_vector_clock_id: VectorClockId,
        to_rebase_container_index: NodeIndex,
        to_rebase_ordering: &OrderingNodeWeight,
        onto: &WorkspaceSnapshot,
        onto_vector_clock_id: VectorClockId,
        onto_container_index: NodeIndex,
        onto_ordering: &OrderingNodeWeight,
    ) -> WorkspaceSnapshotResult<(Vec<Conflict>, Vec<Update>)> {
        let mut updates = Vec::new();
        let mut conflicts = Vec::new();

        if onto_ordering.order() == to_rebase_ordering.order() {
            // Both contain the same items, in the same order. No conflicts, and nothing
            // to update.
            return Ok((conflicts, updates));
        } else if onto_ordering
            .vector_clock_write()
            .is_newer_than(to_rebase_ordering.vector_clock_write())
        {
            let onto_ordering_set: HashSet<Ulid> = onto_ordering.order().iter().copied().collect();
            let to_rebase_ordering_set: HashSet<Ulid> =
                to_rebase_ordering.order().iter().copied().collect();
            let new_items: HashSet<Ulid> = onto_ordering_set
                .difference(&to_rebase_ordering_set)
                .copied()
                .collect();
            let removed_items: HashSet<Ulid> = to_rebase_ordering_set
                .difference(&onto_ordering_set)
                .copied()
                .collect();

            // Find which `other` container items have the new ordering IDs so we can add edges
            // from the `to_rebase` container to them (and create them in `to_rebase` if they don't
            // already exist).
            for onto_container_item_index in onto
                .working_copy()
                .await
                .graph()
                .neighbors_directed(onto_container_index, Outgoing)
            {
                let onto_container_item_weight =
                    onto.get_node_weight(onto_container_item_index).await?;
                if new_items.contains(&onto_container_item_weight.id()) {
                    for edge in onto
                        .working_copy()
                        .await
                        .graph()
                        .edges_connecting(onto_container_index, onto_container_item_index)
                    {
                        updates.push(Update::NewEdge {
                            source: to_rebase_container_index,
                            destination: onto_container_item_index,
                            edge_weight: edge.weight().clone(),
                        });
                    }
                }
            }

            // Remove the edges from the `to_rebase` container to the items removed in `onto`. We
            // don't need to worry about removing the items themselves as they will be garbage
            // collected when we drop all items that are not reachable from `to_rebase.root_index`
            // if they are no longer referenced by anything.
            for to_rebase_container_item_index in self
                .working_copy()
                .await
                .graph()
                .neighbors_directed(to_rebase_container_index, Outgoing)
            {
                let to_rebase_container_item_weight =
                    self.get_node_weight(to_rebase_container_item_index).await?;
                if removed_items.contains(&to_rebase_container_item_weight.id()) {
                    for edgeref in self
                        .working_copy()
                        .await
                        .graph()
                        .edges_connecting(to_rebase_container_index, to_rebase_container_item_index)
                    {
                        updates.push(Update::RemoveEdge {
                            source: edgeref.source(),
                            destination: edgeref.target(),
                            edge_kind: edgeref.weight().kind().into(),
                        });
                    }
                }
            }
        } else if to_rebase_ordering
            .vector_clock_write()
            .is_newer_than(onto_ordering.vector_clock_write())
        {
            // We already have everything in `onto` as part of `to_rebase`. Nothing needs
            // updating, and there are no conflicts.
        } else {
            // Both `onto` and `to_rebase` have changes that the other has not incorporated. We
            // need to find out what the changes are to see what needs to be updated, and what
            // conflicts.
            let onto_ordering_set: HashSet<Ulid> = onto_ordering.order().iter().copied().collect();
            let to_rebase_ordering_set: HashSet<Ulid> =
                to_rebase_ordering.order().iter().copied().collect();

            // Make sure that both `onto` and `to_rebase` have the same relative ordering for the
            // nodes they have in common. If they don't, then that means that the order changed on
            // at least one of them.
            let common_items: HashSet<Ulid> = onto_ordering_set
                .intersection(&to_rebase_ordering_set)
                .copied()
                .collect();
            let common_onto_items = {
                let mut items = onto_ordering.order().clone();
                items.retain(|i| common_items.contains(i));
                items
            };
            let common_to_rebase_items = {
                let mut items = to_rebase_ordering.order().clone();
                items.retain(|i| common_items.contains(i));
                items
            };
            if common_onto_items != common_to_rebase_items {
                let onto_ordering_index = onto.get_node_index_by_id(onto_ordering.id()).await?;
                let to_rebase_ordering_index =
                    self.get_node_index_by_id(to_rebase_ordering.id()).await?;
                conflicts.push(Conflict::ChildOrder {
                    onto: onto_ordering_index,
                    to_rebase: to_rebase_ordering_index,
                });
            }

            let only_onto_items: HashSet<Ulid> = onto_ordering_set
                .difference(&to_rebase_ordering_set)
                .copied()
                .collect();
            let only_to_rebase_items: HashSet<Ulid> = to_rebase_ordering_set
                .difference(&onto_ordering_set)
                .copied()
                .collect();

            let mut only_to_rebase_item_indexes = HashMap::new();
            for target_idx in self
                .working_copy()
                .await
                .graph()
                .neighbors_directed(to_rebase_container_index, Outgoing)
            {
                let dest_node_weight = self.get_node_weight(target_idx).await?;
                if only_to_rebase_items.contains(&dest_node_weight.id()) {
                    only_to_rebase_item_indexes.insert(dest_node_weight.id(), target_idx);
                }
            }

            for only_to_rebase_item in only_to_rebase_items {
                let only_to_rebase_item_index = *only_to_rebase_item_indexes
                    .get(&only_to_rebase_item)
                    .ok_or(WorkspaceSnapshotGraphError::NodeWithIdNotFound(
                        only_to_rebase_item,
                    ))?;
                for to_rebase_edgeref in self
                    .working_copy()
                    .await
                    .graph()
                    .edges_connecting(to_rebase_container_index, only_to_rebase_item_index)
                {
                    if to_rebase_edgeref
                        .weight()
                        .vector_clock_first_seen()
                        .entry_for(onto_vector_clock_id)
                        .is_none()
                    {
                        // `only_to_rebase_item` is new: Edge in `to_rebase` does not have a "First Seen" for `onto`.
                    } else if self
                        .get_node_weight(only_to_rebase_item_index)
                        .await?
                        .vector_clock_write()
                        .entry_for(to_rebase_vector_clock_id)
                        .is_some()
                    {
                        // Entry was deleted in `onto`. If we have also modified the entry, then
                        // there's a conflict.
                        conflicts.push(Conflict::ModifyRemovedItem(only_to_rebase_item_index));
                    } else {
                        // Entry was deleted in `onto`, and has not been modified in `to_rebase`:
                        // Remove the edge.
                        updates.push(Update::RemoveEdge {
                            source: to_rebase_edgeref.source(),
                            destination: to_rebase_edgeref.target(),
                            edge_kind: to_rebase_edgeref.weight().kind().into(),
                        });
                    }
                }
            }

            let mut only_onto_item_indexes = HashMap::new();
            for onto_edgeref in onto
                .working_copy()
                .await
                .edges_directed(onto_container_index, Outgoing)
            {
                let dest_node_weight = onto.get_node_weight(onto_edgeref.target()).await?;
                if only_onto_items.contains(&dest_node_weight.id()) {
                    only_onto_item_indexes.insert(dest_node_weight.id(), onto_edgeref.target());
                }
            }

            let onto_root_seen_as_of = self
                .get_node_weight(self.root().await?)
                .await?
                .vector_clock_recently_seen()
                .entry_for(onto_vector_clock_id);
            for only_onto_item in only_onto_items {
                let only_onto_item_index = *only_onto_item_indexes.get(&only_onto_item).ok_or(
                    WorkspaceSnapshotGraphError::NodeWithIdNotFound(only_onto_item),
                )?;
                for onto_edgeref in onto
                    .working_copy()
                    .await
                    .graph()
                    .edges_connecting(onto_container_index, only_onto_item_index)
                {
                    // `only_onto_item` is new:
                    //   - "First seen" of edge for `onto` > "Seen As Of" on root for `onto` in
                    //     `to_rebase`.
                    if let Some(onto_first_seen) = onto_edgeref
                        .weight()
                        .vector_clock_first_seen()
                        .entry_for(onto_vector_clock_id)
                    {
                        if let Some(root_seen_as_of) = onto_root_seen_as_of {
                            if onto_first_seen > root_seen_as_of {
                                // The edge for the item was created more recently than the last
                                // state we knew of from `onto`, which means that the item is
                                // "new". We can't have removed something that we didn't know
                                // existed in the first place.
                                updates.push(Update::NewEdge {
                                    source: to_rebase_container_index,
                                    destination: onto_edgeref.target(),
                                    edge_weight: onto_edgeref.weight().clone(),
                                });
                            }
                        }
                    } else if let Ok(onto_item_node_weight) =
                        onto.get_node_weight(only_onto_item_index).await
                    {
                        if let Some(root_seen_as_of) = onto_root_seen_as_of {
                            if onto_item_node_weight
                                .vector_clock_write()
                                .has_entries_newer_than(root_seen_as_of)
                            {
                                // The item removed in `to_rebase` has been modified in `onto`
                                // since we last knew the state of `onto`: This is a conflict, as
                                // we don't know if the removal is still intended given the new
                                // state of the item.
                                conflicts.push(Conflict::RemoveModifiedItem {
                                    container: to_rebase_container_index,
                                    removed_item: only_onto_item_index,
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok((conflicts, updates))
    }

    // NOTE(nick): this should only be used by the rebaser.
    #[instrument(level = "debug", skip_all)]
    pub async fn edge_endpoints(
        &self,
        edge_index: EdgeIndex,
    ) -> WorkspaceSnapshotResult<(NodeIndex, NodeIndex)> {
        Ok(self.working_copy_mut().await.edge_endpoints(edge_index)?)
    }

    /// Replace references should be called when a node has been changed and
    /// copied into the graph.  It will use the original_node_index to find the
    /// most up to date version of the new node, and replace all edges that
    /// point to that old node with edges pointing to the new node.  Because the
    /// graph is treated as an immutable, copy-on-write structure, this means
    /// walking up the graph to the root and copying all nodes that have edges
    /// that point to the original_node_index, and all nodes that have edges
    /// that point to *those* parent nodes, etc, until we've processed the
    /// entire parent tree of the original node.
    #[instrument(level = "trace", skip_all)]
    pub async fn replace_references(
        &self,
        original_node_index: NodeIndex,
    ) -> WorkspaceSnapshotResult<()> {
        let start = Instant::now();
        // Climb from the original node, up to root, rewriting outgoing edges
        // along the way. But we have to be sure to climb to root once for each
        // sibling node that we encounter as we walk up to root.
        let mut outer_queue = VecDeque::from([original_node_index]);

        while let Some(old_node_index) = outer_queue.pop_front() {
            let mut work_queue = VecDeque::from([old_node_index]);

            while let Some(old_node_index) = work_queue.pop_front() {
                for edge_ref in self
                    .working_copy()
                    .await
                    .graph()
                    .edges_directed(old_node_index, Direction::Incoming)
                {
                    work_queue.push_back(edge_ref.source());
                    outer_queue.push_back(edge_ref.source())
                }

                let latest_node_idx = self
                    .working_copy()
                    .await
                    .get_latest_node_idx(old_node_index)?;
                let new_node_idx = if latest_node_idx != old_node_index {
                    latest_node_idx
                } else {
                    self.copy_node_by_index(latest_node_idx).await?
                };

                // Find all outgoing edges weights and find the edge targets.
                let mut edges_to_create = Vec::new();
                for edge_ref in self
                    .working_copy()
                    .await
                    .graph()
                    .edges_directed(old_node_index, Outgoing)
                {
                    edges_to_create.push((
                        edge_ref.weight().clone(),
                        edge_ref.target(),
                        edge_ref.id(),
                    ));
                }

                // Make copies of these edges where the source is the new node index and the
                // destination is one of the following...
                // - If an entry exists in `old_to_new_node_indices` for the destination node index,
                //   use the value of the entry (the destination was affected by the replacement,
                //   and needs to use the new node index to reflect this).
                // - There is no entry in `old_to_new_node_indices`; use the same destination node
                //   index as the old edge (the destination was *NOT* affected by the replacement,
                //   and does not have any new information to reflect).
                for (edge_weight, destination_node_index, edge_idx) in edges_to_create {
                    // Need to directly add the edge, without going through `self.add_edge` to avoid
                    // infinite recursion, and because we're the place doing all the book keeping
                    // that we'd be interested in happening from `self.add_edge`.
                    let destination_node_index = self
                        .working_copy()
                        .await
                        .get_latest_node_idx(destination_node_index)?;

                    self.working_copy_mut()
                        .await
                        .graph_mut()
                        .remove_edge(edge_idx);

                    self.working_copy_mut().await.graph_mut().update_edge(
                        new_node_idx,
                        destination_node_index,
                        edge_weight,
                    );
                }

                self.working_copy_mut()
                    .await
                    .update_merkle_tree_hash(new_node_idx)?;
            }
        }

        self.working_copy_mut().await.update_root_index()?;

        info!("replace references took: {:?}", start.elapsed(),);

        Ok(())
    }

    async fn copy_node_by_index(
        &self,
        node_index: NodeIndex,
    ) -> WorkspaceSnapshotResult<NodeIndex> {
        let remote_node_weight = self.get_node_weight(node_index).await?;
        let local_weight = self.working_copy().await.get_node_weight(node_index)?;
        Ok(self
            .working_copy_mut()
            .await
            .add_node(remote_node_weight, local_weight.address())?)
    }

    #[instrument(level = "debug", skip_all)]
    pub async fn get_node_weight_by_address(
        &self,
        address: NodeWeightAddress,
    ) -> WorkspaceSnapshotResult<Arc<NodeWeight>> {
        self.node_weight_db
            .read(&address)
            .await?
            .ok_or(WorkspaceSnapshotError::NodeWeightMissing(address))
    }

    #[instrument(level = "debug", skip_all)]
    pub async fn get_node_weight_by_id(
        &self,
        id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<Arc<NodeWeight>> {
        let node_idx = self.get_node_index_by_id(id).await?;
        self.get_node_weight(node_idx).await
    }

    #[instrument(level = "debug", skip_all)]
    pub async fn get_node_weight(
        &self,
        node_index: NodeIndex,
    ) -> WorkspaceSnapshotResult<Arc<NodeWeight>> {
        let local_node_weight = self.working_copy().await.get_node_weight(node_index)?;
        self.node_weight_db
            .read(&local_node_weight.address())
            .await?
            .ok_or(WorkspaceSnapshotError::NodeWeightMissing(
                local_node_weight.address(),
            ))
    }

    #[instrument(level = "debug", skip_all)]
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

    #[instrument(level = "debug", skip_all)]
    pub async fn cleanup(&self) -> WorkspaceSnapshotResult<()> {
        let start = tokio::time::Instant::now();

        // We want to remove all of the "garbage" we've accumulated while operating on the graph.
        // Anything that is no longer reachable from the current `self.root_index` should be
        // removed as it is no longer referenced by anything in the current version of the graph.
        // Fortunately, we don't need to walk the graph to find out if something is reachable from
        // the root, since `has_path_connecting` is slow (depth-first search). Any node that does
        // *NOT* have any incoming edges (aside from the `self.root_index` node) is not reachable,
        // by definition. Finding the list of nodes with no incoming edges is very fast. If we
        // remove all nodes (that are not the `self.root_index` node) that do not have any
        // incoming edges, and we keep doing this until the only one left is the `self.root_index`
        // node, then all remaining nodes are reachable from `self.root_index`.
        let mut old_root_ids: HashSet<NodeIndex>;
        let root_index = self.root().await?;
        loop {
            old_root_ids = self
                .working_copy()
                .await
                .graph()
                .externals(Incoming)
                .filter(|node_idx| *node_idx != root_index)
                .collect();
            if old_root_ids.is_empty() {
                break;
            }

            for stale_node_index in &old_root_ids {
                self.working_copy_mut()
                    .await
                    .graph_mut()
                    .remove_node(*stale_node_index);
            }
        }
        info!("Removed stale NodeIndex: {:?}", start.elapsed());

        let node_addresses: HashSet<NodeWeightAddress> = self
            .working_copy()
            .await
            .graph()
            .node_weights()
            .map(|weight| weight.address())
            .collect();

        let mut remaining_node_ids = HashSet::new();
        for address in &node_addresses {
            let node_weight = self.get_node_weight_by_address(*address).await?;
            remaining_node_ids.insert(node_weight.id());
        }

        // After we retain the nodes, collect the remaining ids and indices.
        info!(
            "Got remaining node IDs: {} ({:?})",
            remaining_node_ids.len(),
            start.elapsed()
        );
        let remaining_node_indices: HashSet<NodeIndex> =
            self.working_copy().await.graph().node_indices().collect();
        info!(
            "Got remaining NodeIndex: {} ({:?})",
            remaining_node_indices.len(),
            start.elapsed()
        );

        // Cleanup the node index by id map.
        self.working_copy_mut()
            .await
            .retain_node_index_by_id(remaining_node_ids);
        info!("Removed stale node_index_by_id: {:?}", start.elapsed());

        // Cleanup the node indices by lineage id map.
        self.working_copy_mut()
            .await
            .retain_node_indices_by_lineage_id(remaining_node_indices);
        info!(
            "Removed stale node_indices_by_lineage_id: {:?}",
            start.elapsed()
        );

        self.working_copy_mut()
            .await
            .retain_id_by_node_addresses(node_addresses);
        info!("Removed stale id_by_node_address: {:?}", start.elapsed());

        Ok(())
    }

    #[instrument(level = "debug", skip_all)]
    pub async fn nodes(&self) -> WorkspaceSnapshotResult<Vec<(Arc<NodeWeight>, NodeIndex)>> {
        let mut result = vec![];

        for node in self.working_copy().await.nodes() {
            let node_weight = self.get_node_weight_by_address(node.address()).await?;
            let node_index = self.get_node_index_by_id(node_weight.id()).await?;
            result.push((node_weight.clone(), node_index));
        }

        Ok(result)
    }

    #[instrument(level = "debug", skip_all)]
    pub async fn edges(&self) -> WorkspaceSnapshotResult<Vec<(EdgeWeight, NodeIndex, NodeIndex)>> {
        Ok(self
            .working_copy()
            .await
            .edges()
            .map(|(weight, from, to)| (weight.to_owned(), from, to))
            .collect())
    }

    pub async fn import_subgraph(
        &self,
        other: &WorkspaceSnapshot,
        root_index: NodeIndex,
    ) -> WorkspaceSnapshotResult<()> {
        let mut dfs =
            petgraph::visit::DfsPostOrder::new(other.working_copy().await.graph(), root_index);
        while let Some(node_index_to_copy) = dfs.next(&other.working_copy().await.graph()) {
            let node_weight_to_copy = other
                .get_node_weight(node_index_to_copy)
                .await?
                .as_ref()
                .clone();
            let node_weight_id = node_weight_to_copy.id();
            let node_weight_lineage_id = node_weight_to_copy.lineage_id();

            // The following assumes there are no conflicts between "self" and "other". If there
            // are conflicts between them, we shouldn't be running updates.
            let node_index = if let Some(equivalent_node_index) = self
                .find_equivalent_node(node_weight_id, node_weight_lineage_id)
                .await?
            {
                let equivalent_node_weight = self.get_node_weight(equivalent_node_index).await?;
                if equivalent_node_weight
                    .vector_clock_write()
                    .is_newer_than(node_weight_to_copy.vector_clock_write())
                {
                    equivalent_node_index
                } else {
                    let new_node_index = self.add_node(node_weight_to_copy).await?;
                    self.working_copy()
                        .await
                        .get_latest_node_idx(new_node_index)?
                }
            } else {
                self.add_node(node_weight_to_copy).await?
            };

            for (edge_weight, _, target_idx) in other
                .edges_directed_by_index(node_index_to_copy, Outgoing)
                .await?
            {
                let target_id = other.get_node_weight(target_idx).await?.id();
                let latest_target = self.get_node_index_by_id(target_id).await?;
                self.working_copy_mut().await.graph_mut().update_edge(
                    node_index,
                    latest_target,
                    edge_weight,
                );
            }
        }

        Ok(())
    }

    // pub async fn dot(&self) {
    //     self.working_copy().await.dot();
    // }

    // pub async fn tiny_dot_to_file(&self, suffix: Option<&str>) {
    //     self.working_copy().await.tiny_dot_to_file(suffix);
    // }

    pub async fn get_node_index_by_id_opt(&self, id: impl Into<Ulid>) -> Option<NodeIndex> {
        self.working_copy().await.get_node_index_by_id_opt(id)
    }

    #[instrument(level = "debug", skip_all)]
    pub async fn get_node_index_by_id(
        &self,
        id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<NodeIndex> {
        Ok(self.working_copy().await.get_node_index_by_id(id)?)
    }

    #[instrument(level = "debug", skip_all)]
    pub async fn get_latest_node_index(
        &self,
        node_index: NodeIndex,
    ) -> WorkspaceSnapshotResult<NodeIndex> {
        Ok(self.working_copy().await.get_latest_node_idx(node_index)?)
    }

    #[instrument(level = "info", skip_all)]
    pub async fn find(
        ctx: &DalContext,
        workspace_snapshot_addr: WorkspaceSnapshotAddress,
    ) -> WorkspaceSnapshotResult<Self> {
        let start = tokio::time::Instant::now();

        let snapshot = ctx
            .layer_db()
            .workspace_snapshot()
            .read(&workspace_snapshot_addr)
            .await?
            .ok_or(WorkspaceSnapshotError::WorkspaceSnapshotGraphMissing(
                workspace_snapshot_addr,
            ))?;

        info!("snapshot fetch took: {:?}", start.elapsed());

        Ok(Self {
            address: Arc::new(RwLock::new(workspace_snapshot_addr)),
            read_only_graph: snapshot,
            working_copy: Arc::new(RwLock::new(None)),
            node_weight_db: ctx.layer_db().node_weight().clone(),
            events_tenancy: ctx.events_tenancy(),
            events_actor: ctx.events_actor(),
        })
    }

    pub async fn find_for_change_set(
        ctx: &DalContext,
        change_set_pointer_id: ChangeSetId,
    ) -> WorkspaceSnapshotResult<Self> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_opt(
                "SELECT workspace_snapshot_address FROM change_set_pointers WHERE id = $1",
                &[&change_set_pointer_id],
            )
            .await?
            .ok_or(
                WorkspaceSnapshotError::ChangeSetPointerMissingWorkspaceSnapshotAddress(
                    change_set_pointer_id,
                ),
            )?;

        let address: WorkspaceSnapshotAddress = row.try_get("workspace_snapshot_address")?;

        Self::find(ctx, address).await
    }

    #[instrument(level = "debug", skip_all)]
    pub async fn get_category_node(
        &self,
        source: Option<Ulid>,
        kind: CategoryNodeKind,
    ) -> WorkspaceSnapshotResult<Ulid> {
        let source_index = match source {
            Some(provided_source) => self.get_node_index_by_id(provided_source).await?,
            None => self.root().await?,
        };

        // TODO(nick): ensure that two target category nodes of the same kind don't exist for the
        // same source node.
        for (_, _, maybe_category_node_index) in
            self.edges_directed_by_index(source_index, Outgoing).await?
        {
            let maybe_category_node_weight =
                self.get_node_weight(maybe_category_node_index).await?;

            if let NodeWeight::Category(category_node_weight) = maybe_category_node_weight.as_ref()
            {
                if category_node_weight.kind() == kind {
                    return Ok(category_node_weight.id());
                }
            }
        }

        Err(WorkspaceSnapshotError::CategoryNodeNotFound(source_index))
    }

    #[instrument(level = "debug", skip_all)]
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

    #[instrument(level = "debug", skip_all)]
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

    #[instrument(level = "debug", skip_all)]
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

    #[instrument(level = "debug", skip_all)]
    pub async fn remove_all_edges(
        &self,
        change_set: &ChangeSetPointer,
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

    #[instrument(level = "debug", skip_all)]
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

    #[instrument(level = "debug", skip_all)]
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

    #[instrument(level = "debug", skip_all)]
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

    #[instrument(level = "debug", skip_all)]
    pub async fn all_outgoing_targets(
        &self,
        id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<Vec<Arc<NodeWeight>>> {
        let mut result = vec![];
        let target_idxs: Vec<NodeIndex> = self
            .edges_directed(id, Direction::Outgoing)
            .await?
            .into_iter()
            .map(|(_, _, target_idx)| target_idx)
            .collect();

        for target_idx in target_idxs {
            let node_weight = self.get_node_weight(target_idx).await?;
            result.push(node_weight);
        }

        Ok(result)
    }

    #[instrument(level = "debug", skip_all)]
    pub async fn all_incoming_sources(
        &self,
        id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<Vec<Arc<NodeWeight>>> {
        let mut result = vec![];
        let source_idxs: Vec<NodeIndex> = self
            .edges_directed(id, Direction::Incoming)
            .await?
            .into_iter()
            .map(|(_, source_idx, _)| source_idx)
            .collect();

        for source_idx in source_idxs {
            let node_weight = self.get_node_weight(source_idx).await?;
            result.push(node_weight);
        }

        Ok(result)
    }

    #[instrument(level = "debug", skip_all)]
    pub async fn remove_incoming_edges_of_kind(
        &self,
        change_set: &ChangeSetPointer,
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

    #[instrument(level = "debug", skip_all)]
    pub async fn remove_node_by_id(
        &self,
        change_set: &ChangeSetPointer,
        id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<()> {
        let id: Ulid = id.into();
        let node_idx = self.get_node_index_by_id(id).await?;
        self.remove_all_edges(change_set, id).await?;
        self.working_copy_mut().await.remove_node(node_idx);
        self.working_copy_mut().await.remove_node_id(id);

        Ok(())
    }

    #[instrument(level = "debug", skip_all)]
    pub async fn remove_edge(
        &self,
        change_set: &ChangeSetPointer,
        source_node_index: NodeIndex,
        target_node_index: NodeIndex,
        edge_kind: EdgeWeightKindDiscriminants,
    ) -> WorkspaceSnapshotResult<()> {
        let source_node_index = self
            .working_copy()
            .await
            .get_latest_node_idx(source_node_index)?;

        let source_node_id = self.get_node_weight(source_node_index).await?.id();

        let target_node_index = self
            .working_copy()
            .await
            .get_latest_node_idx(target_node_index)?;

        let target_node_id = self.get_node_weight(target_node_index).await?.id();

        self.working_copy_mut()
            .await
            .remove_edge(source_node_index, target_node_index, edge_kind);

        if let Some(mut ordering_node) = self.ordering_node_for_container(source_node_id).await? {
            // We only want to update the ordering of the container if we removed an edge to
            // one of the ordered relationships.
            if ordering_node.remove_from_order(change_set, target_node_id)? {
                let ordering_node_index = self.get_node_index_by_id(ordering_node.id()).await?;
                self.working_copy_mut().await.remove_edge(
                    ordering_node_index,
                    target_node_index,
                    EdgeWeightKindDiscriminants::Ordinal,
                );

                self.add_node(NodeWeight::Ordering(ordering_node)).await?;
            }
        }

        let source_node_index = self
            .working_copy()
            .await
            .get_latest_node_idx(source_node_index)?;

        let mut work_queue = VecDeque::from([source_node_index]);

        while let Some(node_index) = work_queue.pop_front() {
            self.working_copy_mut().await.update_merkle_tree_hash(
                // If we updated the ordering node, that means we've invalidated the container's
                // NodeIndex (new_source_node_index), so we need to find the new NodeIndex to be able
                // to update the container's merkle tree hash.
                node_index,
            )?;

            for edge_ref in self
                .working_copy()
                .await
                .edges_directed(node_index, Incoming)
            {
                work_queue.push_back(edge_ref.source());
            }
        }

        Ok(())
    }

    #[instrument(level = "debug", skip_all)]
    pub async fn remove_edge_for_ulids(
        &self,
        change_set: &ChangeSetPointer,
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
    #[instrument(level = "debug", skip_all)]
    pub async fn perform_updates(
        &self,
        to_rebase_change_set: &ChangeSetPointer,
        onto: &WorkspaceSnapshot,
        updates: &[Update],
    ) -> WorkspaceSnapshotResult<()> {
        for update in updates {
            match update {
                Update::NewEdge {
                    source,
                    destination,
                    edge_weight,
                } => {
                    let updated_source = self.working_copy().await.get_latest_node_idx(*source)?;
                    let updated_source_id = self.get_node_weight(updated_source).await?.id();
                    let destination = self
                        .find_in_self_or_create_using_onto(*destination, onto)
                        .await?;
                    let destination_id = self.get_node_weight(destination).await?.id();

                    self.add_edge(updated_source_id, edge_weight.clone(), destination_id)
                        .await?;
                }
                Update::RemoveEdge {
                    source,
                    destination,
                    edge_kind,
                } => {
                    let updated_source = self.working_copy().await.get_latest_node_idx(*source)?;
                    let destination = self
                        .working_copy()
                        .await
                        .get_latest_node_idx(*destination)?;
                    self.remove_edge(
                        to_rebase_change_set,
                        updated_source,
                        destination,
                        *edge_kind,
                    )
                    .await?;
                }
                Update::ReplaceSubgraph {
                    onto: onto_subgraph_root,
                    to_rebase: to_rebase_subgraph_root,
                } => {
                    let updated_to_rebase = self
                        .working_copy()
                        .await
                        .get_latest_node_idx(*to_rebase_subgraph_root)?;
                    self.find_in_self_or_create_using_onto(*onto_subgraph_root, onto)
                        .await?;
                    self.replace_references(updated_to_rebase).await?;
                }
            }
        }
        Ok(())
    }

    /// Find in self where self is the "to rebase" side or create using "onto".
    async fn find_in_self_or_create_using_onto(
        &self,
        unchecked: NodeIndex,
        onto: &WorkspaceSnapshot,
    ) -> WorkspaceSnapshotResult<NodeIndex> {
        let unchecked_local_node_weight = onto.working_copy().await.get_node_weight(unchecked)?;
        let unchecked_node_weight = onto.get_node_weight(unchecked).await?;

        let found_or_created = {
            let equivalent_node = if let Some(found) = self
                .working_copy()
                .await
                .find_latest_idx_in_self_from_other_idx(&*onto.working_copy().await, unchecked)?
            {
                Some(found)
            } else {
                self.working_copy().await.find_equivalent_node(
                    unchecked_node_weight.id(),
                    unchecked_node_weight.lineage_id(),
                )?
            };

            match equivalent_node {
                Some(found_equivalent_node) => {
                    let found_equivalent_node_weight = self
                        .working_copy()
                        .await
                        .get_node_weight(found_equivalent_node)?;
                    if found_equivalent_node_weight.merkle_tree_hash()
                        != unchecked_local_node_weight.merkle_tree_hash()
                    {
                        self.import_subgraph(onto, unchecked).await?;
                        self.working_copy()
                            .await
                            .find_latest_idx_in_self_from_other_idx(
                                &*onto.working_copy().await,
                                unchecked,
                            )?
                            .ok_or(WorkspaceSnapshotGraphError::NodeWeightNotFound)?
                    } else {
                        found_equivalent_node
                    }
                }
                None => {
                    self.import_subgraph(onto, unchecked).await?;
                    self.working_copy()
                        .await
                        .find_latest_idx_in_self_from_other_idx(
                            &*onto.working_copy().await,
                            unchecked,
                        )?
                        .ok_or(WorkspaceSnapshotGraphError::NodeWeightNotFound)?
                }
            }
        };
        Ok(found_or_created)
    }

    /// Returns an `Option<Vec<Ulid>>`. If there is an ordering node, then the
    /// return will be a [`Some`], where the [`Vec`] is populated with the
    /// [`Ulid`] of the nodes specified by the ordering node, in the order
    /// defined by the ordering node. If there is not an ordering node, then the
    /// return will be [`None`].
    #[instrument(level = "debug", skip_all)]
    pub async fn ordered_children_for_node(
        &self,
        id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<Option<Vec<Ulid>>> {
        Ok(
            if let Some(ordering_weight) = self.ordering_node_for_container(id).await? {
                for ordered_id in ordering_weight.order() {
                    // verify that ordered thing in order is actually in the graph
                    if self.get_node_index_by_id_opt(*ordered_id).await.is_none() {
                        return Err(WorkspaceSnapshotError::NodeWithIdNotFound(*ordered_id));
                    }
                }
                Some(ordering_weight.order().clone())
            } else {
                None
            },
        )
    }

    pub async fn ordering_node_for_container(
        &self,
        container_node_id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<Option<OrderingNodeWeight>> {
        let container_node_id = container_node_id.into();
        let mut ordering_nodes: Vec<OrderingNodeWeight> = vec![];
        for (edge, _, target) in self.edges_directed(container_node_id, Outgoing).await? {
            if edge.kind() == &EdgeWeightKind::Ordering {
                if let NodeWeight::Ordering(inner) = self.get_node_weight(target).await?.as_ref() {
                    ordering_nodes.push(inner.clone());
                }
            }
        }

        if ordering_nodes.len() > 1 {
            error!(
                "Too many ordering nodes found for container NodeId {:?}",
                container_node_id,
            );
            return Err(WorkspaceSnapshotError::TooManyOrderingForNode(
                container_node_id,
            ));
        }
        Ok(ordering_nodes.first().cloned())
    }

    pub async fn update_order(
        &mut self,
        change_set: &ChangeSetPointer,
        container_id: Ulid,
        new_order: Vec<Ulid>,
    ) -> WorkspaceSnapshotResult<()> {
        if let Some(mut ordering_node) = self.ordering_node_for_container(container_id).await? {
            ordering_node.set_order(change_set, new_order)?;
            self.add_node(NodeWeight::Ordering(ordering_node)).await?;
        }

        Ok(())
    }
}
