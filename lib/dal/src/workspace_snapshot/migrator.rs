use super::{
    graph::{
        WorkspaceSnapshotGraph, WorkspaceSnapshotGraphDiscriminants, WorkspaceSnapshotGraphError,
    },
    node_weight::{
        input_socket_node_weight::InputSocketNodeWeightError,
        schema_variant_node_weight::SchemaVariantNodeWeightError,
    },
};
use crate::workspace_snapshot::migrator::v4::migrate_v3_to_v4;
use crate::workspace_snapshot::node_weight::NodeWeightError;
use crate::{
    dependency_graph::DependencyGraph,
    workspace_snapshot::migrator::{v2::migrate_v1_to_v2, v3::migrate_v2_to_v3},
    ChangeSet, ChangeSetError, ChangeSetStatus, DalContext, Visibility, Workspace, WorkspaceError,
    WorkspaceSnapshot, WorkspaceSnapshotError,
};
use si_events::WorkspaceSnapshotAddress;
use si_layer_cache::LayerDbError;
use std::sync::Arc;
use telemetry::prelude::*;
use thiserror::Error;
use ulid::DecodeError;

pub mod v2;
pub mod v3;
pub mod v4;

#[derive(Error, Debug)]
#[remain::sorted]
pub enum SnapshotGraphMigratorError {
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("ulid decode error: {0}")]
    Decode(#[from] DecodeError),
    #[error("InputSocketNodeWeight error: {0}")]
    InputSocketNodeWeight(#[from] InputSocketNodeWeightError),
    #[error("layer db error: {0}")]
    LayerDb(#[from] LayerDbError),
    #[error("node weight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("SchemaVariantNodeWeight error: {0}")]
    SchemaVariantNodeWeight(#[from] SchemaVariantNodeWeightError),
    #[error("unexpected graph version {1:?} for snapshot {0}, cannot migrate")]
    UnexpectedGraphVersion(
        WorkspaceSnapshotAddress,
        WorkspaceSnapshotGraphDiscriminants,
    ),
    #[error("workspace error: {0}")]
    Workspace(#[from] WorkspaceError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
    #[error("workspace snapshot graph error: {0}")]
    WorkspaceSnapshotGraph(#[from] WorkspaceSnapshotGraphError),
}

pub type SnapshotGraphMigratorResult<T> = Result<T, SnapshotGraphMigratorError>;

pub struct SnapshotGraphMigrator;

impl SnapshotGraphMigrator {
    pub fn new() -> Self {
        Self
    }

    async fn should_migrate(&self, ctx: &DalContext) -> SnapshotGraphMigratorResult<bool> {
        Ok(
            if let Some(builtin_workspace) = Workspace::find_builtin(ctx).await? {
                builtin_workspace.snapshot_version()
                    != WorkspaceSnapshotGraph::current_discriminant()
            } else {
                false
            },
        )
    }

    #[instrument(skip(self, ctx))]
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

        info!(
            "Migrating {} snapshot(s)",
            change_set_graph.independent_ids().len(),
        );

        loop {
            let change_sets_to_migrate = change_set_graph.independent_ids();
            if change_sets_to_migrate.is_empty() {
                break;
            }

            for change_set_id in change_sets_to_migrate {
                let mut change_set = ChangeSet::find(ctx, change_set_id)
                    .await?
                    .ok_or(ChangeSetError::ChangeSetNotFound(change_set_id))?;
                if change_set.workspace_id.is_none() || change_set.status == ChangeSetStatus::Failed
                {
                    // These are broken/garbage change sets generated during migrations of the
                    // "universal" workspace/change set. They're not actually accessible via normal
                    // means, as we generally follow the chain starting at the workspace, and these
                    // aren't associated with any workspace.
                    change_set_graph.remove_id(change_set_id);
                    continue;
                }

                let snapshot_address = change_set.workspace_snapshot_address;
                trace!(
                    "Migrating snapshot {} for change set {} with base change set of {:?}",
                    snapshot_address,
                    change_set_id,
                    change_set.base_change_set_id,
                );

                let new_snapshot = match self.migrate_snapshot(ctx, snapshot_address).await {
                    Ok(new_snapshot) => new_snapshot,
                    Err(err) => {
                        let err_string = err.to_string();
                        if err_string.contains("missing from store for node")
                            || err_string.contains("workspace snapshot graph missing at address")
                        {
                            error!(error = ?err, "Migration error: {err_string}, marking change set {} for workspace {:?} as failed", change_set.id, change_set.workspace_id);
                            change_set
                                .update_status(ctx, ChangeSetStatus::Failed)
                                .await?;
                            change_set_graph.remove_id(change_set_id);
                            continue;
                        } else {
                            return Err(err)?;
                        }
                    }
                };

                let (new_snapshot_address, _) = ctx
                    .layer_db()
                    .workspace_snapshot()
                    .write(
                        Arc::new(new_snapshot),
                        None,
                        ctx.events_tenancy(),
                        ctx.events_actor(),
                    )
                    .await?;

                // NOTE(victor): The context that gets passed in does not have a workspace snapshot
                // on it, since its main purpose is to allow access to the services context.
                // We need to create a context for each migrated changeset here to run operations
                // that depend on the graph
                let mut ctx_after_migration =
                    ctx.clone_with_new_visibility(Visibility::from(change_set.id));
                let migrated_snapshot = WorkspaceSnapshot::find(ctx, new_snapshot_address).await?;
                ctx_after_migration.set_workspace_snapshot(migrated_snapshot);

                change_set
                    .update_pointer(&ctx_after_migration, new_snapshot_address)
                    .await?;
                trace!(
                    "Migrated snapshot {} for change set {} with base change set of {:?}",
                    snapshot_address,
                    change_set_id,
                    change_set.base_change_set_id,
                );

                change_set_graph.remove_id(change_set_id);
            }
        }

        info!("Migration finished, marking all workspaces as migrated to latest version");

        Workspace::set_snapshot_version_for_all_workspaces(
            ctx,
            WorkspaceSnapshotGraph::current_discriminant(),
        )
        .await?;

        Ok(())
    }

    #[instrument(skip(self, ctx))]
    pub async fn migrate_snapshot(
        &mut self,
        ctx: &DalContext,
        workspace_snapshot_address: WorkspaceSnapshotAddress,
    ) -> SnapshotGraphMigratorResult<WorkspaceSnapshotGraph> {
        let snapshot_bytes = ctx
            .layer_db()
            .workspace_snapshot()
            .read_bytes_from_durable_storage(&workspace_snapshot_address)
            .await?
            .ok_or(WorkspaceSnapshotError::WorkspaceSnapshotGraphMissing(
                workspace_snapshot_address,
            ))?;

        info!("snapshot is {} bytes", snapshot_bytes.len());

        let mut working_graph: WorkspaceSnapshotGraph =
            si_layer_cache::db::serialize::from_bytes(&snapshot_bytes)?;

        // Incrementally migrate the graph until we reach the newest version.
        loop {
            match working_graph {
                WorkspaceSnapshotGraph::Legacy => {
                    return Err(SnapshotGraphMigratorError::UnexpectedGraphVersion(
                        workspace_snapshot_address,
                        working_graph.into(),
                    ));
                }
                WorkspaceSnapshotGraph::V1(inner_graph) => {
                    working_graph = WorkspaceSnapshotGraph::V2(migrate_v1_to_v2(inner_graph)?);
                }
                WorkspaceSnapshotGraph::V2(inner_graph) => {
                    working_graph =
                        WorkspaceSnapshotGraph::V3(migrate_v2_to_v3(ctx, inner_graph).await?);
                }
                WorkspaceSnapshotGraph::V3(inner_graph) => {
                    working_graph =
                        WorkspaceSnapshotGraph::V4(migrate_v3_to_v4(ctx, inner_graph).await?);
                }
                WorkspaceSnapshotGraph::V4(_) => {
                    // Nothing to do, this is the newest version,
                    break;
                }
            }
        }

        Ok(working_graph)
    }
}

impl Default for SnapshotGraphMigrator {
    fn default() -> Self {
        Self::new()
    }
}
