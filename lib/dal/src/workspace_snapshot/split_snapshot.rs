use std::{
    collections::{
        HashMap,
        HashSet,
    },
    sync::{
        Arc,
        atomic::AtomicBool,
    },
};

use async_trait::async_trait;
use petgraph::Direction::{
    self,
    Incoming,
    Outgoing,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_events::{
    ContentHash,
    Timestamp,
    WorkspaceSnapshotAddress,
    merkle_tree_hash::MerkleTreeHash,
    workspace_snapshot::{
        Change,
        EntityKind,
    },
};
use si_id::{
    ApprovalRequirementDefinitionId,
    AttributeValueId,
    ChangeSetId,
    ComponentId,
    EntityId,
    InputSocketId,
    PropId,
    SchemaId,
    SchemaVariantId,
    UserPk,
    ViewId,
    ulid::Ulid,
};
use si_layer_cache::LayerDbError;
use si_split_graph::{
    SplitGraph,
    SplitGraphNodeIndex,
    SplitGraphNodeWeight,
    SubGraph,
    SuperGraph,
    opt_zip::OptZip,
};
use strum::{
    EnumDiscriminants,
    EnumIter,
    EnumString,
    IntoEnumIterator,
};
use telemetry::prelude::*;
use tokio::{
    sync::{
        Mutex,
        RwLock,
        RwLockReadGuard,
        RwLockWriteGuard,
    },
    task::JoinSet,
    time::Instant,
};

use super::{
    CycleCheckGuard,
    DependentValueRoot,
    EntityKindExt,
    InferredConnectionsWriteGuard,
    InputSocketExt,
    SchemaVariantExt,
    WorkspaceSnapshotError,
    WorkspaceSnapshotResult,
    content_address::ContentAddressDiscriminants,
    graph::LineageId,
    node_weight::{
        CategoryNodeWeight,
        NodeWeight,
        NodeWeightError,
        category_node_weight::CategoryNodeKind,
    },
    traits::{
        approval_requirement::ApprovalRequirementExt,
        diagram::view::ViewExt,
        prop::PropExt,
        socket::input::input_socket_from_node_weight,
    },
};
use crate::{
    ComponentError,
    DalContext,
    EdgeWeight,
    EdgeWeightKind,
    EdgeWeightKindDiscriminants,
    InputSocket,
    NodeWeightDiscriminants,
    SchemaVariantError,
    approval_requirement::{
        ApprovalRequirement,
        ApprovalRequirementApprover,
        ApprovalRequirementDefinition,
    },
    component::{
        ComponentResult,
        inferred_connection_graph::InferredConnectionGraph,
    },
    layer_db_types::{
        ViewContent,
        ViewContentV1,
    },
    prop::PropResult,
    slow_rt,
    socket::input::InputSocketError,
};

pub type SplitSnapshotGraphV1 = SplitGraph<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>;
pub type SplitSnapshotGraphVCurrent = SplitSnapshotGraphV1;

pub type SubGraphV1 = SubGraph<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>;
pub type SubGraphVCurrent = SubGraphV1;

pub type UpdateV1 = si_split_graph::Update<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>;
pub type UpdateVCurrent = UpdateV1;

pub type SplitRebaseBatchV1 = Vec<UpdateV1>;
pub type SplitRebaseBatchVCurrent = SplitRebaseBatchV1;

#[derive(Serialize, Deserialize, Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, Serialize, Deserialize, EnumString, EnumIter))]
pub enum SplitSnapshotStorage {
    SuperGraph(SuperGraphVersion),
    SubGraphV1(SubGraphVersion),
}

#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, Serialize, Deserialize, EnumString, EnumIter))]
pub enum SplitSnapshotGraph {
    V1(SplitSnapshotGraphV1),
}

#[derive(Serialize, Deserialize, Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, Serialize, Deserialize, EnumString, EnumIter))]
pub enum SuperGraphVersion {
    V1(SuperGraph),
}

impl SuperGraphVersionDiscriminants {
    pub fn current_discriminant() -> Self {
        Self::V1
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, Serialize, Deserialize, EnumString, EnumIter))]
pub enum SubGraphVersion {
    V1(SubGraphV1),
}

impl SubGraphVersionDiscriminants {
    pub fn current() -> Self {
        Self::V1
    }
}

impl std::ops::Deref for SplitSnapshotGraph {
    type Target = SplitSnapshotGraphVCurrent;

    fn deref(&self) -> &Self::Target {
        self.inner()
    }
}

impl std::ops::DerefMut for SplitSnapshotGraph {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner_mut()
    }
}

impl SplitSnapshotGraph {
    pub fn inner(&self) -> &SplitSnapshotGraphVCurrent {
        match self {
            SplitSnapshotGraph::V1(inner) => inner,
        }
    }

    pub fn inner_mut(&mut self) -> &mut SplitSnapshotGraphVCurrent {
        match self {
            SplitSnapshotGraph::V1(inner) => inner,
        }
    }

    pub fn current_discriminant() -> SplitSnapshotGraphDiscriminants {
        SplitSnapshotGraphDiscriminants::iter()
            .next_back()
            .expect("Unable to get last element of an iterator guaranteed to have elements")
    }
}

#[must_use = "if unused the lock will be released immediately"]
struct SnapshotReadGuard<'a> {
    read_only_graph: Arc<SplitSnapshotGraph>,
    working_copy_read_guard: RwLockReadGuard<'a, Option<SplitSnapshotGraphVCurrent>>,
}

#[must_use = "if unused the lock will be released immediately"]
struct SnapshotWriteGuard<'a> {
    working_copy_write_guard: RwLockWriteGuard<'a, Option<SplitSnapshotGraphVCurrent>>,
}

impl std::ops::Deref for SnapshotReadGuard<'_> {
    type Target = SplitSnapshotGraphVCurrent;

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
    type Target = SplitSnapshotGraphVCurrent;

    fn deref(&self) -> &Self::Target {
        let option = &*self.working_copy_write_guard;
        option.as_ref().expect(
            "attempted to deref snapshot without copying contents into the mutable working copy",
        )
    }
}

impl std::ops::DerefMut for SnapshotWriteGuard<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let option: &mut Option<SplitSnapshotGraphVCurrent> = &mut self.working_copy_write_guard;
        &mut *option.as_mut().expect("attempted to DerefMut a snapshot without copying contents into the mutable working copy")
    }
}

#[derive(Debug, Clone)]
pub struct SplitSnapshot {
    address: Arc<Mutex<WorkspaceSnapshotAddress>>,
    read_only_graph: Arc<SplitSnapshotGraph>,
    working_copy: Arc<RwLock<Option<SplitSnapshotGraphVCurrent>>>,
    cycle_check: Arc<AtomicBool>,
    dvu_roots: Arc<Mutex<HashSet<DependentValueRoot>>>,
    inferred_connection_graph: Arc<RwLock<Option<InferredConnectionGraph>>>,
}

impl SplitSnapshot {
    pub async fn id(&self) -> WorkspaceSnapshotAddress {
        *self.address.lock().await
    }

    pub fn from_graph(graph: SplitSnapshotGraph) -> Self {
        Self {
            address: Arc::new(Mutex::new(WorkspaceSnapshotAddress::nil())),
            read_only_graph: Arc::new(graph),
            working_copy: Arc::new(RwLock::new(None)),
            cycle_check: Arc::new(AtomicBool::new(false)),
            dvu_roots: Arc::new(Mutex::new(HashSet::new())),
            inferred_connection_graph: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn subgraph_count(&self) -> usize {
        self.working_copy().await.subgraph_count()
    }

    fn add_category_nodes(graph: &mut SplitSnapshotGraphVCurrent) -> WorkspaceSnapshotResult<Ulid> {
        let mut view_category_id = Ulid::new();

        // Implementation of add_category_nodes
        for category_node_kind in CategoryNodeKind::iter() {
            let id = Ulid::new();
            let lineage_id = Ulid::new();

            if category_node_kind == CategoryNodeKind::View {
                view_category_id = id;
            }

            graph.add_or_replace_node(NodeWeight::Category(CategoryNodeWeight::new(
                id,
                lineage_id,
                category_node_kind,
            )))?;
            graph.add_edge(
                graph.root_id()?,
                EdgeWeight::new(EdgeWeightKind::new_use()),
                id,
            )?;
        }

        Ok(view_category_id)
    }

    async fn add_default_view(
        ctx: &DalContext,
        graph: &mut SplitSnapshotGraphVCurrent,
        view_category_id: Ulid,
    ) -> WorkspaceSnapshotResult<()> {
        let id = Ulid::new();
        let lineage_id = Ulid::new();

        let content = ViewContent::V1(ViewContentV1 {
            timestamp: Timestamp::now(),
            name: "DEFAULT".to_owned(),
        });

        let (content_address, _) = ctx.layer_db().cas().write(
            Arc::new(content.clone().into()),
            None,
            ctx.events_tenancy(),
            ctx.events_actor(),
        )?;

        let node_weight = NodeWeight::new_view(id, lineage_id, content_address);
        graph.add_or_replace_node(node_weight.clone())?;

        graph.add_edge(
            view_category_id,
            EdgeWeight::new(EdgeWeightKind::new_use_default()),
            id,
        )?;

        Ok(())
    }

    pub async fn initial(ctx: &DalContext, split_max: usize) -> WorkspaceSnapshotResult<Self> {
        let mut graph = SplitSnapshotGraphVCurrent::new(split_max);

        let view_category_id = Self::add_category_nodes(&mut graph)?;
        Self::add_default_view(ctx, &mut graph, view_category_id).await?;

        // We do not care about any field other than "working_copy" because
        // "write" will populate them using the assigned working copy.
        let initial = Self {
            address: Arc::new(Mutex::new(WorkspaceSnapshotAddress::nil())),
            read_only_graph: Arc::new(SplitSnapshotGraph::V1(graph)),
            working_copy: Arc::new(RwLock::new(None)),
            cycle_check: Arc::new(AtomicBool::new(false)),
            dvu_roots: Arc::new(Mutex::new(HashSet::new())),
            inferred_connection_graph: Arc::new(RwLock::new(None)),
        };

        initial.write(ctx).await?;

        Ok(initial)
    }

    pub async fn current_rebase_batch(
        &self,
    ) -> WorkspaceSnapshotResult<Option<SplitRebaseBatchVCurrent>> {
        let self_clone = self.clone();

        let updates = slow_rt::spawn(async move {
            let mut working_copy = self_clone.working_copy_mut().await;
            working_copy.cleanup_and_merkle_tree_hash();

            self_clone.read_only_graph.detect_updates(&working_copy)
        })?
        .await?;

        Ok((!updates.is_empty()).then_some(updates))
    }

    pub async fn detect_updates(
        &self,
        updated: &Self,
    ) -> WorkspaceSnapshotResult<SplitRebaseBatchVCurrent> {
        let self_clone = self.clone();
        let updated_clone = updated.clone();

        Ok(slow_rt::spawn(async move {
            self_clone
                .working_copy()
                .await
                .detect_updates(&*updated_clone.working_copy().await)
        })?
        .await?)
    }

    #[instrument(
        name = "split_snapshot.calculate_rebase_batch",
        level = "info",
        skip_all
    )]
    pub async fn calculate_rebase_batch(
        base_snapshot: Arc<Self>,
        updated_snapshot: Arc<Self>,
    ) -> WorkspaceSnapshotResult<Option<SplitRebaseBatchVCurrent>> {
        let updates =
            slow_rt::spawn(async move { base_snapshot.detect_updates(&updated_snapshot).await })?
                .await??;

        Ok((!updates.is_empty()).then_some(updates))
    }

    #[instrument(name = "split_snapshot.find", level = "debug", skip_all, fields())]
    pub async fn find(
        ctx: &DalContext,
        split_snapshot_supergraph_addr: WorkspaceSnapshotAddress,
    ) -> WorkspaceSnapshotResult<Self> {
        let snapshot = match ctx
            .layer_db()
            .split_snapshot_supergraph()
            .read_wait_for_memory(&split_snapshot_supergraph_addr)
            .await
        {
            Ok(supergraph) => {
                let supergraph = supergraph.ok_or(
                    WorkspaceSnapshotError::SplitSnapshotSuperGraphMissingAtAddress(
                        split_snapshot_supergraph_addr,
                    ),
                )?;

                let mut subgraphs = vec![];
                for &subgraph_address in supergraph.addresses() {
                    let subgraph_address = subgraph_address.into();
                    let subgraph = ctx
                        .layer_db()
                        .split_snapshot_subgraph()
                        .read_wait_for_memory(&subgraph_address)
                        .await?
                        .ok_or(
                            WorkspaceSnapshotError::SplitSnapshotSubGraphMissingAtAddress(
                                subgraph_address,
                            ),
                        )?;

                    // xxx: we have to make the splitgraph constructable from arcs, it will
                    // xxx: need to handle the copy-on-write behavior internally
                    subgraphs.push(subgraph.as_ref().clone());
                }

                Arc::new(SplitSnapshotGraph::V1(
                    SplitSnapshotGraphVCurrent::from_parts(supergraph.as_ref().clone(), subgraphs),
                ))
            }
            Err(err) => match err {
                LayerDbError::Postcard(_) => {
                    return Err(WorkspaceSnapshotError::WorkspaceSnapshotNotMigrated(
                        split_snapshot_supergraph_addr,
                    ));
                }
                err => Err(err)?,
            },
        };

        Ok(Self {
            address: Arc::new(Mutex::new(split_snapshot_supergraph_addr)),
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
        // hoping we don't get unlucky every time
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
                Err(
                    WorkspaceSnapshotError::SplitSnapshotSuperGraphMissingAtAddress(_)
                    | WorkspaceSnapshotError::SplitSnapshotSubGraphMissingAtAddress(_),
                ) => {
                    warn!(
                        "Unable to retrieve split snapshot {:?} for change set {:?}. Retries remaining: {}",
                        address, change_set_id, retries
                    );
                    tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
                    continue;
                }
                Err(e) => return Err(e),
            }
        }

        error!(
            "Retries exceeded trying to fetch split snapshot for change set {:?}",
            change_set_id
        );

        Err(WorkspaceSnapshotError::WorkspaceSnapshotNotFetched)
    }

    pub async fn write(
        &self,
        ctx: &DalContext,
    ) -> WorkspaceSnapshotResult<WorkspaceSnapshotAddress> {
        let self_clone = self.clone();
        let layer_db = ctx.layer_db().clone();
        let events_tenancy = ctx.events_tenancy();
        let events_actor = ctx.events_actor();

        let supergraph_address = slow_rt::spawn(async move {
            let mut working_copy = self_clone.working_copy_mut().await;
            let start = Instant::now();
            working_copy.cleanup_and_merkle_tree_hash();
            warn!("cleaned up working copy in {:?}", start.elapsed());

            warn!(
                "current addresses: {:?}",
                working_copy.supergraph().addresses()
            );

            let current_supergraph = working_copy.supergraph();
            let mut new_supergraph = SuperGraph::new(
                current_supergraph.split_max(),
                current_supergraph.root_index(),
                current_supergraph.external_source_map().clone(),
            );

            let mut join_set = JoinSet::new();

            let subgraph_indexes = OptZip::new(
                self_clone
                    .read_only_graph
                    .subgraphs()
                    .iter()
                    .enumerate()
                    .map(|(idx, _)| idx),
                working_copy
                    .subgraphs()
                    .iter()
                    .enumerate()
                    .map(|(idx, _)| idx),
            )
            .collect::<Vec<_>>();

            drop(working_copy);

            for (original_subgraph_idx, new_subgraph_idx) in subgraph_indexes {
                let self_clone_clone = self_clone.clone();
                let layer_db_clone = layer_db.clone();
                join_set.spawn(async move {
                    let start = Instant::now();
                    let original_subgraph = original_subgraph_idx.and_then(|index| {
                        self_clone_clone
                            .read_only_graph
                            .subgraphs()
                            .get(index)
                            .map(|subgraph| (index, subgraph))
                    });

                    let working_copy = self_clone_clone.working_copy().await;
                    let new_subgraph = match new_subgraph_idx {
                        Some(index) => working_copy
                            .subgraphs()
                            .get(index)
                            .map(|subgraph| (index, subgraph)),
                        None => None,
                    };

                    let subgraph_address_and_index = match (original_subgraph, new_subgraph) {
                        (Some((orig_idx, orig)), Some((_, working))) => {
                            if orig.root_node_merkle_tree_hash()
                                != working.root_node_merkle_tree_hash()
                            {
                                let (new_address, _) =
                                    layer_db_clone.split_snapshot_subgraph().write(
                                        Arc::new(working.clone()),
                                        None,
                                        events_tenancy,
                                        events_actor,
                                    )?;

                                warn!(
                                    "rewrote subgraph in {:?} new address {:?}",
                                    start.elapsed(),
                                    new_address
                                );
                                (orig_idx, new_address)
                            } else {
                                let subgraph_address: WorkspaceSnapshotAddress =
                                    match self_clone_clone
                                        .read_only_graph
                                        .supergraph()
                                        .address_for_subgraph(orig_idx)
                                    {
                                        Some(addr) => addr.into(),
                                        None => {
                                            let (new_address, _) =
                                                layer_db_clone.split_snapshot_subgraph().write(
                                                    Arc::new(working.clone()),
                                                    None,
                                                    events_tenancy,
                                                    events_actor,
                                                )?;

                                            new_address
                                        }
                                    };

                                (orig_idx, subgraph_address)
                            }
                        }
                        (None, Some((new_index, working))) => {
                            let (new_address, _) = layer_db_clone.split_snapshot_subgraph().write(
                                Arc::new(working.clone()),
                                None,
                                events_tenancy,
                                events_actor,
                            )?;

                            warn!(
                                "wrote new subgraph in {:?}, address: {:?}",
                                start.elapsed(),
                                new_address
                            );
                            (new_index, new_address)
                        }
                        (Some(_), None) => {
                            todo!("we've removed a subgraph")
                        }
                        (None, None) => unreachable!("opt zip will never produce this"),
                    };

                    Ok::<_, WorkspaceSnapshotError>(subgraph_address_and_index)
                });
            }

            let mut subgraph_addresses = vec![];
            // Join all returns in the order that futures complete, not the order they were spawned, so we have to sort
            for result in join_set.join_all().await {
                let address_and_index = result?;
                subgraph_addresses.push(address_and_index);
            }
            subgraph_addresses.sort_by_key(|(index, _)| *index);
            for (_, address) in subgraph_addresses {
                new_supergraph.add_subgraph_address(address.into());
            }

            warn!("new subgraph_addresses: {:?}", new_supergraph.addresses());

            let start = Instant::now();
            let (supergraph_address, _) = layer_db.split_snapshot_supergraph().write(
                Arc::new(new_supergraph),
                None,
                events_tenancy,
                events_actor,
            )?;
            warn!(
                "wrote supergraph in {:?}, new address: {:?}",
                start.elapsed(),
                supergraph_address
            );

            Ok::<WorkspaceSnapshotAddress, WorkspaceSnapshotError>(supergraph_address)
        })?
        .await??;

        *self.address.lock().await = supergraph_address;

        Ok(supergraph_address)
    }

    async fn working_copy(&self) -> SnapshotReadGuard<'_> {
        SnapshotReadGuard {
            read_only_graph: self.read_only_graph.clone(),
            working_copy_read_guard: self.working_copy.read().await,
        }
    }

    async fn working_copy_mut(&self) -> SnapshotWriteGuard<'_> {
        let mut working_copy = self.working_copy.write().await;
        if working_copy.is_none() {
            *working_copy = Some(self.read_only_graph.inner().clone());
        }
        SnapshotWriteGuard {
            working_copy_write_guard: working_copy,
        }
    }

    pub async fn root(&self) -> WorkspaceSnapshotResult<Ulid> {
        Ok(self.working_copy().await.root_id()?)
    }

    pub async fn generate_ulid(&self) -> WorkspaceSnapshotResult<Ulid> {
        // XXX: do we need to use the generator for monotonically increasing IDs? is that really necessary?
        Ok(Ulid::new())
    }

    pub async fn enable_cycle_check(&self) -> CycleCheckGuard {
        self.cycle_check
            .store(true, std::sync::atomic::Ordering::Relaxed);
        CycleCheckGuard::new(self.cycle_check.clone())
    }

    pub async fn disable_cycle_check(&self) {
        self.cycle_check
            .store(false, std::sync::atomic::Ordering::Relaxed);
    }

    pub async fn cycle_check(&self) -> bool {
        self.cycle_check.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub async fn is_acyclic_directed(&self) -> bool {
        self.working_copy().await.is_acyclic_directed()
    }

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
        let mut working_copy = self.working_copy_mut().await;

        match working_copy.node_weight_mut(id) {
            Some(node_weight_mut) => {
                node_weight_mut.new_content_hash(new_content_hash)?;
                working_copy.touch_node(id);
                Ok(())
            }
            None => Err(WorkspaceSnapshotError::NodeNotFoundAtId(id)),
        }
    }

    pub async fn add_edge(
        &self,
        from_id: impl Into<Ulid>,
        edge_weight: EdgeWeight,
        to_id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<()> {
        if self.cycle_check().await {
            self.working_copy_mut().await.add_edge_with_cycle_check(
                from_id.into(),
                edge_weight,
                to_id.into(),
            )?;
        } else {
            self.working_copy_mut()
                .await
                .add_edge(from_id.into(), edge_weight, to_id.into())?;
        }
        Ok(())
    }

    pub async fn add_ordered_edge(
        &self,
        from_id: impl Into<Ulid>,
        edge_weight: EdgeWeight,
        to_id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<()> {
        self.working_copy_mut().await.add_ordered_edge(
            from_id.into(),
            edge_weight,
            to_id.into(),
        )?;
        Ok(())
    }

    pub async fn detect_changes(
        &self,
        updated_snapshot: &Self,
    ) -> WorkspaceSnapshotResult<Vec<Change>> {
        Ok(self
            .working_copy()
            .await
            .detect_changes(&*updated_snapshot.working_copy().await)?)
    }

    pub async fn perform_updates(&self, updates: &[UpdateVCurrent]) -> WorkspaceSnapshotResult<()> {
        let self_clone = self.clone();
        let updates = updates.to_vec();
        Ok(slow_rt::spawn(async move {
            self_clone
                .working_copy_mut()
                .await
                .perform_updates(&updates)
        })?
        .await?)
    }

    pub async fn import_component_subgraph(
        &self,
        _other: &Arc<Self>,
        _component_id: ComponentId,
    ) -> WorkspaceSnapshotResult<()> {
        // XXX: Implement import component subgraph
        Ok(())
    }

    pub async fn raw_node_weight(
        &self,
        id: impl Into<Ulid>,
    ) -> Option<SplitGraphNodeWeight<NodeWeight>> {
        self.working_copy()
            .await
            .raw_node_weight(id.into())
            .cloned()
    }

    pub async fn get_node_weight(
        &self,
        id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<NodeWeight> {
        let id = id.into();
        self.get_node_weight_opt(id)
            .await
            .ok_or(WorkspaceSnapshotError::NodeNotFoundAtId(id))
    }

    pub async fn split_graph_node_index(&self, id: impl Into<Ulid>) -> Option<SplitGraphNodeIndex> {
        self.working_copy().await.node_id_to_index(id.into())
    }

    pub async fn get_node_weight_opt(&self, id: impl Into<Ulid>) -> Option<NodeWeight> {
        self.working_copy().await.node_weight(id.into()).cloned()
    }

    pub async fn cleanup(&self) -> WorkspaceSnapshotResult<()> {
        self.working_copy_mut().await.cleanup();
        Ok(())
    }

    pub async fn cleanup_and_merkle_tree_hash(&self) -> WorkspaceSnapshotResult<()> {
        self.working_copy_mut().await.cleanup_and_merkle_tree_hash();
        Ok(())
    }

    pub async fn nodes(&self) -> WorkspaceSnapshotResult<Vec<NodeWeight>> {
        Ok(self.working_copy().await.nodes().cloned().collect())
    }

    pub async fn edges(&self) -> WorkspaceSnapshotResult<Vec<(EdgeWeight, Ulid, Ulid)>> {
        Ok(self
            .working_copy()
            .await
            .edges()
            .map(|(weight, src, dst)| (weight.clone(), src, dst))
            .collect())
    }

    pub async fn node_exists(&self, id: impl Into<Ulid>) -> bool {
        self.working_copy()
            .await
            .node_id_to_index(id.into())
            .is_some()
    }

    pub async fn get_category_node_or_err(
        &self,
        source: Option<Ulid>,
        kind: CategoryNodeKind,
    ) -> WorkspaceSnapshotResult<Ulid> {
        self.get_category_node(source, kind)
            .await?
            .ok_or(WorkspaceSnapshotError::CategoryNodeNotFound(kind))
    }

    pub async fn get_category_node(
        &self,
        source: Option<Ulid>,
        kind: CategoryNodeKind,
    ) -> WorkspaceSnapshotResult<Option<Ulid>> {
        let working_copy = self.working_copy().await;
        let source_id = source.unwrap_or(working_copy.root_id()?);
        Ok(working_copy
            .edges_directed(source_id, Outgoing)?
            .find(
                |edge_ref| match working_copy.node_weight(edge_ref.target()) {
                    Some(NodeWeight::Category(category_node)) => category_node.kind() == kind,
                    _ => false,
                },
            )
            .map(|edge_ref| edge_ref.target()))
    }

    pub async fn edges_directed_debug(
        &self,
        id: impl Into<Ulid>,
        direction: Direction,
    ) -> WorkspaceSnapshotResult<Vec<(EdgeWeight, Ulid, Ulid)>> {
        Ok(self
            .working_copy()
            .await
            .edges_directed(id.into(), direction)?
            .map(|edge_ref| {
                (
                    edge_ref.weight().clone(),
                    edge_ref.source(),
                    edge_ref.target(),
                )
            })
            .collect())
    }

    pub async fn edges_directed(
        &self,
        id: impl Into<Ulid>,
        direction: Direction,
    ) -> WorkspaceSnapshotResult<Vec<(EdgeWeight, Ulid, Ulid)>> {
        Ok(self
            .working_copy()
            .await
            .edges_directed(id.into(), direction)?
            .map(|edge_ref| {
                (
                    edge_ref.weight().clone(),
                    edge_ref.source(),
                    edge_ref.target(),
                )
            })
            .collect())
    }

    pub async fn edges_directed_for_edge_weight_kind(
        &self,
        id: impl Into<Ulid>,
        direction: Direction,
        edge_kind: EdgeWeightKindDiscriminants,
    ) -> WorkspaceSnapshotResult<Vec<(EdgeWeight, Ulid, Ulid)>> {
        Ok(self
            .working_copy()
            .await
            .edges_directed_for_edge_weight_kind(id.into(), direction, edge_kind)?
            .map(|edge_ref| {
                (
                    edge_ref.weight().clone(),
                    edge_ref.source(),
                    edge_ref.target(),
                )
            })
            .collect())
    }

    pub async fn remove_all_edges(&self, id: impl Into<Ulid>) -> WorkspaceSnapshotResult<()> {
        // Removing all edges to and from a node is the same as removing the node
        // the remove node method will handle the bookkeeping necessary for recalculating the
        // merkle tree
        self.working_copy_mut().await.remove_node(id.into())?;
        Ok(())
    }

    pub async fn incoming_sources_for_edge_weight_kind(
        &self,
        id: impl Into<Ulid>,
        edge_weight_kind_discrim: EdgeWeightKindDiscriminants,
    ) -> WorkspaceSnapshotResult<Vec<Ulid>> {
        Ok(self
            .working_copy()
            .await
            .edges_directed_for_edge_weight_kind(id.into(), Incoming, edge_weight_kind_discrim)?
            .map(|edge_ref| edge_ref.source())
            .collect())
    }

    pub async fn source_opt(
        &self,
        id: impl Into<Ulid>,
        kind: EdgeWeightKindDiscriminants,
    ) -> WorkspaceSnapshotResult<Option<Ulid>> {
        Ok(self
            .working_copy()
            .await
            .directed_unique_neighbor_of_edge_weight_kind(id.into(), Incoming, kind)?)
    }

    pub async fn outgoing_targets_for_edge_weight_kind(
        &self,
        id: impl Into<Ulid>,
        edge_weight_kind_discrim: EdgeWeightKindDiscriminants,
    ) -> WorkspaceSnapshotResult<Vec<Ulid>> {
        Ok(self
            .working_copy()
            .await
            .edges_directed_for_edge_weight_kind(id.into(), Outgoing, edge_weight_kind_discrim)?
            .map(|edge_ref| edge_ref.target())
            .collect())
    }

    pub async fn all_outgoing_targets(
        &self,
        id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<Vec<NodeWeight>> {
        let working_copy = self.working_copy().await;
        let targets = working_copy
            .edges_directed(id.into(), Outgoing)?
            .filter_map(|edge_ref| working_copy.node_weight(edge_ref.target()))
            .cloned()
            .collect();

        Ok(targets)
    }

    pub async fn all_incoming_sources(
        &self,
        id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<Vec<NodeWeight>> {
        let working_copy = self.working_copy().await;
        let sources = working_copy
            .edges_directed(id.into(), Incoming)?
            .filter_map(|edge_ref| working_copy.node_weight(edge_ref.source()))
            .cloned()
            .collect();

        Ok(sources)
    }

    pub async fn remove_incoming_edges_of_kind(
        &self,
        target_id: impl Into<Ulid>,
        kind: EdgeWeightKindDiscriminants,
    ) -> WorkspaceSnapshotResult<()> {
        let target_id = target_id.into();
        let mut working_copy = self.working_copy_mut().await;
        let sources: Vec<_> = working_copy
            .edges_directed_for_edge_weight_kind(target_id, Incoming, kind)?
            .map(|edge_ref| edge_ref.source())
            .collect();

        for source_id in sources {
            working_copy.remove_edge(source_id, kind, target_id)?;
        }

        Ok(())
    }

    pub async fn get_edges_between_nodes(
        &self,
        from_node_id: Ulid,
        to_node_id: Ulid,
    ) -> WorkspaceSnapshotResult<Vec<EdgeWeight>> {
        Ok(self
            .working_copy()
            .await
            .edges_directed(from_node_id, Outgoing)?
            .filter(|edge_ref| edge_ref.target() == to_node_id)
            .map(|edge_ref| edge_ref.weight().clone())
            .collect())
    }

    pub async fn remove_node_by_id(&self, id: impl Into<Ulid>) -> WorkspaceSnapshotResult<()> {
        self.working_copy_mut().await.remove_node(id.into())?;
        Ok(())
    }

    pub async fn remove_edge(
        &self,
        source_id: impl Into<Ulid>,
        target_id: impl Into<Ulid>,
        edge_kind: EdgeWeightKindDiscriminants,
    ) -> WorkspaceSnapshotResult<()> {
        self.working_copy_mut()
            .await
            .remove_edge(source_id.into(), edge_kind, target_id.into())?;
        Ok(())
    }

    pub async fn find_edge(
        &self,
        from_id: impl Into<Ulid>,
        to_id: impl Into<Ulid>,
        edge_weight_kind: EdgeWeightKindDiscriminants,
    ) -> Option<EdgeWeight> {
        self.working_copy()
            .await
            .find_edge(from_id.into(), to_id.into(), edge_weight_kind)
            .cloned()
    }

    pub async fn update_node_id(
        &self,
        current_id: impl Into<Ulid>,
        new_id: impl Into<Ulid>,
        new_lineage_id: LineageId,
    ) -> WorkspaceSnapshotResult<()> {
        self.working_copy_mut().await.update_node_id(
            current_id.into(),
            new_id.into(),
            new_lineage_id,
        )?;

        Ok(())
    }

    pub async fn ordered_children_for_node(
        &self,
        id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotResult<Option<Vec<Ulid>>> {
        Ok(self.working_copy().await.ordered_children(id.into()))
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
        let component_id = component_id.into();
        let working_copy = self.working_copy().await;
        if working_copy.node_id_to_index(component_id).is_none() {
            return Err(ComponentError::NotFound(component_id.into()));
        }

        let sv_id = working_copy
            .edges_directed(component_id, Outgoing)?
            .find(|edge_ref| {
                matches!(edge_ref.weight().kind(), EdgeWeightKind::Use { .. })
                    && matches!(
                        working_copy.node_weight(edge_ref.target()),
                        Some(NodeWeight::SchemaVariant(_))
                    )
            })
            .map(|edge_ref| edge_ref.target().into())
            .ok_or(ComponentError::SchemaVariantNotFound(component_id.into()))?;

        Ok(sv_id)
    }

    pub async fn frame_contains_components(
        &self,
        component_id: ComponentId,
    ) -> ComponentResult<Vec<ComponentId>> {
        let component_id = component_id.into();
        let working_copy = self.working_copy().await;
        if working_copy.node_id_to_index(component_id).is_none() {
            return Err(ComponentError::NotFound(component_id.into()));
        }

        let contained: Vec<ComponentId> = working_copy
            .edges_directed(component_id, Outgoing)?
            .filter(|edge_ref| {
                matches!(edge_ref.weight().kind(), EdgeWeightKind::FrameContains)
                    && matches!(
                        working_copy.node_weight(edge_ref.target()),
                        Some(NodeWeight::Component(_))
                    )
            })
            .map(|edge_ref| edge_ref.target().into())
            .collect();

        Ok(contained)
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

    pub async fn revert(&self) {
        let mut working_copy = self.working_copy.write().await;
        if working_copy.is_some() {
            *working_copy = None;
        }
    }
}
#[async_trait]
impl ApprovalRequirementExt for SplitSnapshot {
    async fn new_definition(
        &self,
        _ctx: &DalContext,
        _entity_id: Ulid,
        _minimum_approvers_count: usize,
        _approvers: HashSet<ApprovalRequirementApprover>,
    ) -> WorkspaceSnapshotResult<ApprovalRequirementDefinitionId> {
        // XXX: implement
        Ok(ApprovalRequirementDefinitionId::new())
    }

    async fn remove_definition(
        &self,
        _approval_requirement_definition_id: ApprovalRequirementDefinitionId,
    ) -> WorkspaceSnapshotResult<()> {
        // XXX: implement
        Ok(())
    }

    async fn add_individual_approver_for_definition(
        &self,
        _ctx: &DalContext,
        _id: ApprovalRequirementDefinitionId,
        _user_id: UserPk,
    ) -> WorkspaceSnapshotResult<()> {
        Ok(())
    }

    async fn remove_individual_approver_for_definition(
        &self,
        _ctx: &DalContext,
        _id: ApprovalRequirementDefinitionId,
        _user_id: UserPk,
    ) -> WorkspaceSnapshotResult<()> {
        Ok(())
    }

    async fn approval_requirements_for_changes(
        &self,
        _ctx: &DalContext,
        _changes: &[Change],
    ) -> WorkspaceSnapshotResult<(Vec<ApprovalRequirement>, HashMap<EntityId, MerkleTreeHash>)>
    {
        Ok((Vec::new(), HashMap::new()))
    }

    async fn approval_requirement_definitions_for_entity_id_opt(
        &self,
        _ctx: &DalContext,
        _entity_id: EntityId,
    ) -> WorkspaceSnapshotResult<Option<Vec<ApprovalRequirementDefinition>>> {
        Ok(None)
    }

    async fn entity_id_for_approval_requirement_definition_id(
        &self,
        _id: ApprovalRequirementDefinitionId,
    ) -> WorkspaceSnapshotResult<EntityId> {
        Ok(EntityId::new())
    }

    async fn get_approval_requirement_definition_by_id(
        &self,
        _ctx: &DalContext,
        _id: ApprovalRequirementDefinitionId,
    ) -> WorkspaceSnapshotResult<ApprovalRequirementDefinition> {
        Ok(ApprovalRequirementDefinition::fake())
    }
}

#[async_trait]
impl InputSocketExt for SplitSnapshot {
    async fn get_input_socket(
        &self,
        ctx: &DalContext,
        id: InputSocketId,
    ) -> WorkspaceSnapshotResult<InputSocket> {
        let working_copy = self.working_copy().await;
        let node_weight = working_copy
            .node_weight(id.into())
            .ok_or(WorkspaceSnapshotError::NodeNotFoundAtId(id.into()))?;

        let input_socket_node_weight = match node_weight {
            NodeWeight::InputSocket(input_socket_node_weight) => input_socket_node_weight,
            unexpected => {
                return Err(NodeWeightError::UnexpectedNodeWeightVariant(
                    unexpected.into(),
                    NodeWeightDiscriminants::InputSocket,
                ))?;
            }
        };

        Ok(input_socket_from_node_weight(ctx, input_socket_node_weight)
            .await
            .map_err(Box::new)?)
    }

    async fn get_input_socket_by_name_opt(
        &self,
        ctx: &DalContext,
        name: &str,
        schema_variant_id: SchemaVariantId,
    ) -> WorkspaceSnapshotResult<Option<InputSocket>> {
        Ok(self
            .list_input_sockets(ctx, schema_variant_id)
            .await?
            .into_iter()
            .find(|socket| socket.name() == name))
    }

    async fn list_input_socket_ids_for_schema_variant(
        &self,
        schema_variant_id: SchemaVariantId,
    ) -> WorkspaceSnapshotResult<Vec<InputSocketId>> {
        let working_copy = self.working_copy().await;
        let result: Vec<_> = working_copy
            .edges_directed_for_edge_weight_kind(
                schema_variant_id.into(),
                Outgoing,
                EdgeWeightKindDiscriminants::Socket,
            )?
            .filter_map(
                |edge_ref| match working_copy.node_weight(edge_ref.target()) {
                    Some(NodeWeight::InputSocket(_)) => Some(edge_ref.target().into()),
                    _ => None,
                },
            )
            .collect();

        Ok(result)
    }

    async fn list_input_sockets(
        &self,
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> WorkspaceSnapshotResult<Vec<InputSocket>> {
        let working_copy = self.working_copy().await;
        let input_sockets = working_copy
            .edges_directed_for_edge_weight_kind(
                schema_variant_id.into(),
                Outgoing,
                EdgeWeightKindDiscriminants::Socket,
            )?
            .filter_map(
                |edge_ref| match working_copy.node_weight(edge_ref.target()) {
                    Some(NodeWeight::InputSocket(inner)) => Some(inner),
                    _ => None,
                },
            );

        let mut result = vec![];

        for input_socket_node_weight in input_sockets {
            result.push(
                input_socket_from_node_weight(ctx, input_socket_node_weight)
                    .await
                    .map_err(Box::new)?,
            );
        }

        Ok(result)
    }

    async fn all_attribute_value_ids_everywhere_for_input_socket_id(
        &self,
        input_socket_id: InputSocketId,
    ) -> WorkspaceSnapshotResult<Vec<AttributeValueId>> {
        let working_copy = self.working_copy().await;

        let result: Vec<_> = working_copy
            .edges_directed_for_edge_weight_kind(
                input_socket_id.into(),
                Incoming,
                EdgeWeightKindDiscriminants::Socket,
            )?
            .filter_map(
                |edge_ref| match working_copy.node_weight(edge_ref.source()) {
                    Some(NodeWeight::AttributeValue(_)) => Some(edge_ref.source().into()),
                    _ => None,
                },
            )
            .collect();

        Ok(result)
    }

    async fn component_attribute_value_id_for_input_socket_id(
        &self,
        input_socket_id: InputSocketId,
        component_id: ComponentId,
    ) -> WorkspaceSnapshotResult<AttributeValueId> {
        let working_copy = self.working_copy().await;

        let mut result = None;
        for socket_value_edge_ref in working_copy.edges_directed_for_edge_weight_kind(
            component_id.into(),
            Outgoing,
            EdgeWeightKindDiscriminants::SocketValue,
        )? {
            for _ in working_copy
                .edges_directed_for_edge_weight_kind(
                    socket_value_edge_ref.target(),
                    Outgoing,
                    EdgeWeightKindDiscriminants::Socket,
                )?
                .filter(|edge_ref| input_socket_id == edge_ref.target().into())
            {
                if result.is_some() {
                    return Err(Box::new(InputSocketError::FoundTooManyForInputSocketId(
                        input_socket_id,
                        component_id,
                    ))
                    .into());
                }
                result = Some(socket_value_edge_ref.target().into());
            }
        }

        if let Some(av_id) = result {
            Ok(av_id)
        } else {
            Err(
                Box::new(InputSocketError::MissingAttributeValueForComponent(
                    input_socket_id,
                    component_id,
                ))
                .into(),
            )
        }
    }

    async fn input_socket_id_find_for_attribute_value_id(
        &self,
        attribute_value_id: AttributeValueId,
    ) -> WorkspaceSnapshotResult<Option<InputSocketId>> {
        let working_copy = self.working_copy().await;

        let mut result = None;

        let av_as_ulid = attribute_value_id.into();
        for edge_ref in working_copy.edges_directed_for_edge_weight_kind(
            av_as_ulid,
            Outgoing,
            EdgeWeightKindDiscriminants::Socket,
        )? {
            if result.is_some() {
                return Err(Box::new(
                    InputSocketError::MultipleSocketsForAttributeValue(attribute_value_id),
                ))?;
            }

            result = Some(edge_ref.target().into());
        }

        Ok(result)
    }
}

#[async_trait]
impl SchemaVariantExt for SplitSnapshot {
    async fn schema_id_for_schema_variant_id(
        &self,
        schema_variant_id: SchemaVariantId,
    ) -> WorkspaceSnapshotResult<SchemaId> {
        let working_copy = self.working_copy().await;

        let sv_ulid = schema_variant_id.into();

        let mut schemas = working_copy
            .edges_directed_for_edge_weight_kind(
                sv_ulid,
                Incoming,
                EdgeWeightKindDiscriminants::Use,
            )?
            .filter_map(
                |edge_ref| match working_copy.node_weight(edge_ref.source()) {
                    Some(NodeWeight::Content(content))
                        if content.content_address_discriminants()
                            == ContentAddressDiscriminants::Schema =>
                    {
                        Some(edge_ref.source())
                    }
                    _ => None,
                },
            );

        let schema_id = schemas
            .next()
            .ok_or_else(|| Box::new(SchemaVariantError::SchemaNotFound(schema_variant_id)))?;

        if schemas.next().is_some() {
            return Err(Box::new(SchemaVariantError::MoreThanOneSchemaFound(
                schema_variant_id,
            )))?;
        }

        Ok(schema_id.into())
    }

    async fn schema_variant_add_edge_to_input_socket(
        &self,
        schema_variant_id: SchemaVariantId,
        input_socket_id: InputSocketId,
    ) -> WorkspaceSnapshotResult<()> {
        let mut working_copy = self.working_copy_mut().await;

        working_copy.add_edge(
            schema_variant_id.into(),
            EdgeWeight::new(EdgeWeightKind::Socket),
            input_socket_id.into(),
        )?;

        Ok(())
    }
}

#[async_trait]
impl EntityKindExt for SplitSnapshot {
    async fn get_entity_kind_for_id(&self, id: EntityId) -> WorkspaceSnapshotResult<EntityKind> {
        let id = id.into();
        Ok(self
            .working_copy()
            .await
            .node_weight(id)
            .ok_or(WorkspaceSnapshotError::NodeNotFoundAtId(id))?
            .into())
    }
}

#[async_trait]
impl ViewExt for SplitSnapshot {
    async fn view_remove(&self, view_id: ViewId) -> WorkspaceSnapshotResult<()> {
        // If there are any Components remaining in the View, this View _CANNOT_ be the only View they
        // are in. If this View is the only View _ANY_ of the items are in, we do not allow removal
        // of the View.

        // View --Use--> Geometry --Represents-->
        //   {Component, DiagramObject <--DiagramObject-- View (on canvas)}
        //
        // Component <--Represents-- Geometry <--Use-- View
        //
        // View (on canvas) --DiagramObject--> DiagramObject <--Represents-- Geometry <--Use-- View

        let mut working_copy = self.working_copy_mut().await;

        let mut would_be_orphaned_component_ids = Vec::new();

        let view_id: Ulid = view_id.into();

        for view_use_edge_ref in working_copy.edges_directed_for_edge_weight_kind(
            view_id,
            Outgoing,
            EdgeWeightKindDiscriminants::Use,
        )? {
            let geometry_node_id = view_use_edge_ref.target();

            // Find the "thing" this Geometry is representing, so we can make sure it is also
            // represented by at least one Geometry in another View.
            let Some(represented_thing_id) = working_copy
                .directed_unique_neighbor_of_edge_weight_kind(
                    geometry_node_id,
                    Outgoing,
                    EdgeWeightKindDiscriminants::Represents,
                )?
            else {
                continue;
            };

            let Some(represented_thing_node_weight) =
                working_copy.node_weight(represented_thing_id)
            else {
                continue;
            };

            if NodeWeightDiscriminants::Component != represented_thing_node_weight.into() {
                // Components _MUST_ be in another View for this View to be able to be removed.
                // Things with DiagramObjects (currently only Views) do not have to be part of
                // another View for this View to be able to be removed.
                continue;
            }

            let mut view_member_ids = HashSet::new();
            for represents_edge_ref in working_copy.edges_directed_for_edge_weight_kind(
                represented_thing_id,
                Incoming,
                EdgeWeightKindDiscriminants::Represents,
            )? {
                let geometry_id = represents_edge_ref.source();
                let geometry_view_id = working_copy
                    .directed_unique_neighbor_of_edge_weight_kind(
                        geometry_id,
                        Incoming,
                        EdgeWeightKindDiscriminants::Use,
                    )?
                    .ok_or(WorkspaceSnapshotError::NoEdgesOfKindFound(
                        geometry_id,
                        Incoming,
                        EdgeWeightKindDiscriminants::Use,
                    ))?;

                if geometry_view_id != view_id {
                    view_member_ids.insert(geometry_view_id);
                }
            }

            if view_member_ids.is_empty() {
                would_be_orphaned_component_ids.push(represented_thing_node_weight.id());
            }
        }

        if !would_be_orphaned_component_ids.is_empty() {
            return Err(WorkspaceSnapshotError::ViewRemovalWouldOrphanItems(
                would_be_orphaned_component_ids,
            ));
        }

        let mut nodes_to_delete = vec![view_id];

        if let Some(diagram_object_id) = working_copy.directed_unique_neighbor_of_edge_weight_kind(
            view_id,
            Outgoing,
            EdgeWeightKindDiscriminants::DiagramObject,
        )? {
            nodes_to_delete.extend(
                working_copy
                    .edges_directed(diagram_object_id, Incoming)?
                    .map(|edge_ref| edge_ref.source()),
            );
        }

        for node_id in nodes_to_delete {
            working_copy.remove_node(node_id)?;
        }

        Ok(())
    }

    async fn list_for_component_id(&self, id: ComponentId) -> WorkspaceSnapshotResult<Vec<ViewId>> {
        if !self.node_exists(id).await {
            return Ok(vec![]);
        }

        let mut view_ids_set = HashSet::new();
        let working_copy = self.working_copy().await;

        for represents_edge_ref in working_copy.edges_directed_for_edge_weight_kind(
            id.into(),
            Incoming,
            EdgeWeightKindDiscriminants::Represents,
        )? {
            if let Some(view_id) = working_copy.directed_unique_neighbor_of_edge_weight_kind(
                represents_edge_ref.source(),
                Incoming,
                EdgeWeightKindDiscriminants::Use,
            )? {
                view_ids_set.insert(view_id);
            }
        }

        Ok(view_ids_set.into_iter().map(Into::into).collect())
    }
}

#[async_trait]
impl PropExt for SplitSnapshot {
    async fn ts_type(&self, _prop_id: PropId) -> PropResult<String> {
        Ok("any".to_string())
    }
}
