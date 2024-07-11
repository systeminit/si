use petgraph::{prelude::*, visit::IntoEdgeReferences};
use si_layer_cache::LayerDbError;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
use telemetry::prelude::*;
use thiserror::Error;

use super::{
    graph::{
        WorkspaceSnapshotGraph, WorkspaceSnapshotGraphDiscriminants, WorkspaceSnapshotGraphError,
    },
    vector_clock::{
        deprecated::{DeprecatedVectorClock, DeprecatedVectorClockId},
        HasVectorClocks, VectorClock,
    },
};
use crate::{
    dependency_graph::DependencyGraph,
    workspace_snapshot::{
        content_address::ContentAddress,
        graph::{deprecated::DeprecatedWorkspaceSnapshotGraph, LineageId},
        node_weight::NodeWeight,
    },
    ChangeSet, ChangeSetError, ChangeSetId, DalContext, EdgeWeight, Workspace, WorkspaceError,
    WorkspaceSnapshotError, WorkspaceSnapshotGraphV1,
};
use si_events::{ulid::Ulid, VectorClockId, WorkspaceSnapshotAddress};

#[derive(Error, Debug)]
#[remain::sorted]
pub enum SnapshotGraphMigratorError {
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("layer db error: {0}")]
    LayerDb(#[from] LayerDbError),
    #[error("workspace error: {0}")]
    Workspace(#[from] WorkspaceError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
    #[error("workspace snapshot graph error: {0}")]
    WorkspaceSnapshotGraph(#[from] WorkspaceSnapshotGraphError),
}

pub type SnapshotGraphMigratorResult<T> = Result<T, SnapshotGraphMigratorError>;

pub struct SnapshotGraphMigrator {
    vector_clock_map: HashMap<DeprecatedVectorClockId, VectorClockId>,
}

impl SnapshotGraphMigrator {
    pub fn new() -> Self {
        Self {
            vector_clock_map: HashMap::new(),
        }
    }

    async fn should_migrate(&self, ctx: &DalContext) -> SnapshotGraphMigratorResult<bool> {
        Ok(
            if let Some(builtin_workspace) = Workspace::find_builtin(ctx).await? {
                builtin_workspace.snapshot_version() != WorkspaceSnapshotGraphDiscriminants::V1
            } else {
                false
            },
        )
    }

    pub async fn migrate_all(&mut self, ctx: &DalContext) -> SnapshotGraphMigratorResult<()> {
        if !self.should_migrate(ctx).await? {
            debug!("Builtin workspace has been migrated. Not migrating snapshots to the latest");
            return Ok(());
        }

        let open_change_sets = ChangeSet::list_open_for_all_workspaces(ctx).await?;

        let mut change_set_graph = DependencyGraph::new();
        for change_set in open_change_sets {
            match change_set.base_change_set_id {
                Some(base_change_set_id) => {
                    change_set_graph.id_depends_on(change_set.id, base_change_set_id);
                }
                None => {
                    change_set_graph.add_id(change_set.id);
                }
            }
        }

        loop {
            let change_sets_to_migrate = change_set_graph.independent_ids();
            if change_sets_to_migrate.is_empty() {
                break;
            }

            for change_set_id in change_sets_to_migrate {
                let mut change_set = ChangeSet::find(ctx, change_set_id)
                    .await?
                    .ok_or(ChangeSetError::ChangeSetNotFound(change_set_id))?;

                if let Some(snapshot_address) = change_set.workspace_snapshot_address {
                    info!(
                        "Migrating snapshot {} for change set {} with base change set of {:?}",
                        snapshot_address, change_set_id, change_set.base_change_set_id,
                    );

                    let new_snapshot_address = self
                        .migrate_snapshot(ctx, change_set_id, snapshot_address)
                        .await?;

                    change_set.update_pointer(ctx, new_snapshot_address).await?;
                }

                change_set_graph.remove_id(change_set_id);
            }
        }

        info!("Migration finished, marking all workspaces as migrated to latest version");

        Workspace::set_snapshot_version_for_all_workspaces(
            ctx,
            WorkspaceSnapshotGraphDiscriminants::V1,
        )
        .await?;

        Ok(())
    }

    pub async fn migrate_snapshot(
        &mut self,
        ctx: &DalContext,
        change_set_id: ChangeSetId,
        workspace_snapshot_address: WorkspaceSnapshotAddress,
    ) -> SnapshotGraphMigratorResult<WorkspaceSnapshotAddress> {
        let snapshot_bytes = ctx
            .layer_db()
            .workspace_snapshot()
            .read_bytes_from_durable_storage(&workspace_snapshot_address)
            .await?
            .ok_or(WorkspaceSnapshotError::WorkspaceSnapshotGraphMissing(
                workspace_snapshot_address,
            ))?;

        let deprecated_graph: DeprecatedWorkspaceSnapshotGraph =
            si_layer_cache::db::serialize::from_bytes(&snapshot_bytes)?;

        let deprecated_graph_inner = &deprecated_graph.graph;

        let mut new_graph: StableDiGraph<NodeWeight, EdgeWeight> = StableDiGraph::with_capacity(
            deprecated_graph_inner.node_count(),
            deprecated_graph_inner.edge_count(),
        );

        let mut node_index_by_id: HashMap<Ulid, NodeIndex> = HashMap::new();
        let mut node_indices_by_lineage_id: HashMap<LineageId, HashSet<NodeIndex>> = HashMap::new();
        // This is just a place holder until we find the root when iterating the nodes
        let mut root_index = deprecated_graph.root_index;
        let mut old_graph_idx_to_id = HashMap::new();

        for deprecated_node_weight in deprecated_graph_inner.node_weights() {
            let first_seen_clock = deprecated_node_weight.vector_clock_first_seen();
            let recently_seen_clock = deprecated_node_weight.vector_clock_recently_seen();
            let write_clock = deprecated_node_weight.vector_clock_write();

            let mut node_weight: NodeWeight = deprecated_node_weight.clone().into();
            let id = node_weight.id();
            let lineage_id = node_weight.lineage_id();

            self.migrate_vector_clock(
                change_set_id,
                &first_seen_clock,
                node_weight.vector_clock_first_seen_mut(),
            );

            self.migrate_vector_clock(
                change_set_id,
                &recently_seen_clock,
                node_weight.vector_clock_recently_seen_mut(),
            );

            self.migrate_vector_clock(
                change_set_id,
                &write_clock,
                node_weight.vector_clock_write_mut(),
            );

            let is_root_node = if let NodeWeight::Content(content_node_weight) = &node_weight {
                matches!(content_node_weight.content_address(), ContentAddress::Root)
            } else {
                false
            };

            let new_idx = new_graph.add_node(node_weight);
            if is_root_node {
                root_index = new_idx;
            }

            if let Some(idx) = deprecated_graph.node_index_by_id.get(&id) {
                old_graph_idx_to_id.insert(idx, id);
            }

            node_index_by_id.insert(id, new_idx);
            node_indices_by_lineage_id
                .entry(lineage_id)
                .and_modify(|index_set| {
                    index_set.insert(new_idx);
                })
                .or_insert(HashSet::from([new_idx]));
        }

        for edge_ref in deprecated_graph_inner.edge_references() {
            let deprecated_edge_weight = edge_ref.weight();
            let mut new_edge_weight: EdgeWeight = deprecated_edge_weight.to_owned().into();

            self.migrate_vector_clock(
                change_set_id,
                &deprecated_edge_weight.vector_clock_first_seen,
                new_edge_weight.vector_clock_first_seen_mut(),
            );

            self.migrate_vector_clock(
                change_set_id,
                &deprecated_edge_weight.vector_clock_recently_seen,
                new_edge_weight.vector_clock_recently_seen_mut(),
            );

            self.migrate_vector_clock(
                change_set_id,
                &deprecated_edge_weight.vector_clock_write,
                new_edge_weight.vector_clock_write_mut(),
            );

            let source_idx = edge_ref.source();
            let target_idx = edge_ref.target();
            let source_id_in_new_graph = old_graph_idx_to_id.get(&source_idx).copied();
            let target_id_in_new_graph = old_graph_idx_to_id.get(&target_idx).copied();

            if let (Some(source_id), Some(target_id)) =
                (source_id_in_new_graph, target_id_in_new_graph)
            {
                let source_idx_in_new_graph = node_index_by_id.get(&source_id).copied();
                let target_idx_in_new_graph = node_index_by_id.get(&target_id).copied();

                if let (Some(new_source_idx), Some(new_target_idx)) =
                    (source_idx_in_new_graph, target_idx_in_new_graph)
                {
                    new_graph.add_edge(new_source_idx, new_target_idx, new_edge_weight);
                }
            }
        }

        let mut new_snapshot_graph = WorkspaceSnapshotGraphV1::new_from_parts(
            new_graph,
            node_index_by_id,
            node_indices_by_lineage_id,
            root_index,
        );

        new_snapshot_graph.recalculate_entire_merkle_tree_hash()?;

        let (migrated_address, _) = ctx
            .layer_db()
            .workspace_snapshot()
            .write(
                Arc::new(WorkspaceSnapshotGraph::V1(new_snapshot_graph)),
                None,
                ctx.events_tenancy(),
                ctx.events_actor(),
            )
            .await?;

        Ok(migrated_address)
    }

    fn migrate_vector_clock(
        &mut self,
        change_set_id: ChangeSetId,
        deprecated_clock: &DeprecatedVectorClock,
        new_vector_clock: &mut VectorClock,
    ) {
        for (deprecated_clock_id, lamport_clock) in &deprecated_clock.entries {
            match self.vector_clock_map.get(deprecated_clock_id) {
                Some(already_mapped_clock_id) => {
                    new_vector_clock.inc_to_max_of(*already_mapped_clock_id, lamport_clock.counter)
                }
                None => {
                    let new_vector_clock_id =
                        VectorClockId::new(change_set_id.into_inner(), ulid::Ulid(0));
                    new_vector_clock.inc_to_max_of(new_vector_clock_id, lamport_clock.counter);

                    self.vector_clock_map
                        .insert(*deprecated_clock_id, new_vector_clock_id);
                }
            }
        }
    }
}

impl Default for SnapshotGraphMigrator {
    fn default() -> Self {
        Self::new()
    }
}
