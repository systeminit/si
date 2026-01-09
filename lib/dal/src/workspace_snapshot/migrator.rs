use std::collections::HashMap;

use si_events::WorkspaceSnapshotAddress;
use si_layer_cache::LayerDbError;
use si_split_graph::SplitGraphError;
use split_v1::migrate_v4_to_split_v1;
use telemetry::prelude::*;
use thiserror::Error;
use ulid::DecodeError;

use super::{
    graph::{
        WorkspaceSnapshotGraph,
        WorkspaceSnapshotGraphError,
    },
    node_weight::{
        input_socket_node_weight::InputSocketNodeWeightError,
        schema_variant_node_weight::SchemaVariantNodeWeightError,
    },
};
use crate::{
    ChangeSet,
    ChangeSetError,
    ChangeSetStatus,
    DalContext,
    TransactionsError,
    Workspace,
    WorkspaceError,
    WorkspaceSnapshotError,
    workspace::SnapshotVersion,
    workspace_snapshot::{
        node_weight::NodeWeightError,
        split_snapshot::{
            SplitSnapshot,
            SubGraphVersionDiscriminants,
            SuperGraphVersionDiscriminants,
        },
    },
};

pub mod split_v1;

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
    #[error("Migration from that graph version no longer supported")]
    MigrationUnsupported,
    #[error("node weight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("SchemaVariantNodeWeight error: {0}")]
    SchemaVariantNodeWeight(#[from] SchemaVariantNodeWeightError),
    #[error("split graph error: {0}")]
    SplitGraph(#[from] SplitGraphError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("unexpected failure to migrate snapshot {0}")]
    UnexpectedMigrationFailure(WorkspaceSnapshotAddress),
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

    #[instrument(skip(self, ctx))]
    pub async fn migrate_all(&mut self, ctx: &DalContext) -> SnapshotGraphMigratorResult<()> {
        let mut workspace_count = 0;
        let mut change_set_count = 0;

        let mut migration_map = HashMap::new();

        for mut workspace in Workspace::list_all(ctx).await? {
            if workspace.is_current_version_and_kind() {
                continue;
            }

            let open_change_sets =
                ChangeSet::list_active_for_workspace(ctx, *workspace.pk()).await?;

            info!(
                "Migrating {} snapshot(s) for {}",
                open_change_sets.len(),
                workspace.pk()
            );

            for change_set in open_change_sets {
                let mut change_set =
                    ChangeSet::get_by_id_across_workspaces(ctx, change_set.id).await?;

                if change_set.workspace_id.is_none() || change_set.status == ChangeSetStatus::Failed
                {
                    // These are broken/garbage change sets generated during migrations of the
                    // "universal" workspace/change set. They're not actually accessible via normal
                    // means, as we generally follow the chain starting at the workspace, and these
                    // aren't associated with any workspace.
                    continue;
                }

                // NOTE(victor): The context that gets passed in does not have a workspace snapshot
                // on it, since its main purpose is to allow access to the services context.
                // We need to create a context for each migrated changeset here to run operations
                // that depend on the graph
                let mut ctx_after_migration = ctx.clone_with_change_set(change_set.id);
                // TODO make sure that when we clone with a changeset id we also set changeset
                // (or there's no clone anymore and we always change it via following method)
                ctx_after_migration.set_change_set(change_set.clone())?;

                let snapshot_address = change_set.workspace_snapshot_address;

                let new_snapshot_address = match migration_map.get(&snapshot_address) {
                    Some(cached_addr) => *cached_addr,
                    None => match self
                        .migrate_snapshot(&ctx_after_migration, snapshot_address)
                        .await
                    {
                        Ok(addr) => addr,
                        Err(err) => {
                            let err_string = err.to_string();
                            if err_string.contains("missing from store for node")
                                || err_string
                                    .contains("workspace snapshot graph missing at address")
                            {
                                error!(
                                    error = ?err,
                                    "Migration error: {err_string}, marking change set {} for workspace {:?} as failed",
                                    change_set.id, change_set.workspace_id
                                );

                                change_set
                                    .update_status(ctx, ChangeSetStatus::Failed)
                                    .await?;
                                continue;
                            } else {
                                return Err(err)?;
                            }
                        }
                    },
                };

                let migrated_snapshot =
                    SplitSnapshot::find(&ctx_after_migration, new_snapshot_address).await?;
                ctx_after_migration.set_workspace_split_snapshot(migrated_snapshot);

                change_set
                    .update_pointer(&ctx_after_migration, new_snapshot_address)
                    .await?;

                migration_map.insert(snapshot_address, new_snapshot_address);
                change_set_count += 1;
            }

            workspace
                .set_snapshot_versions(
                    ctx,
                    SnapshotVersion::Split(SuperGraphVersionDiscriminants::V1),
                    Some(SubGraphVersionDiscriminants::V1),
                )
                .await?;
            workspace_count += 1;
        }

        info!("Migration finished: {workspace_count} workspaces, {change_set_count} change sets");

        Ok(())
    }

    #[instrument(skip(self, ctx))]
    pub async fn migrate_snapshot(
        &mut self,
        ctx: &DalContext,
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

        let change_set = ctx.change_set()?;

        info!(
            "Migrating snapshot {} for change set {} with base change set of {:?} ({} bytes)",
            workspace_snapshot_address,
            change_set.id,
            change_set.base_change_set_id,
            snapshot_bytes.len()
        );

        #[allow(clippy::large_enum_variant)]
        enum Graphs {
            Legacy(WorkspaceSnapshotGraph),
            Split {
                address: WorkspaceSnapshotAddress,
                #[allow(unused)]
                supergraph_version: SuperGraphVersionDiscriminants,
                #[allow(unused)]
                subgraph_version: SubGraphVersionDiscriminants,
            },
        }

        let mut working_graph: Graphs =
            Graphs::Legacy(si_layer_cache::db::serialize::from_bytes(&snapshot_bytes)?);

        // Incrementally migrate the graph until we reach the newest version.
        loop {
            match working_graph {
                Graphs::Legacy(WorkspaceSnapshotGraph::Legacy)
                | Graphs::Legacy(
                    WorkspaceSnapshotGraph::V1
                    | WorkspaceSnapshotGraph::V2
                    | WorkspaceSnapshotGraph::V3,
                ) => {
                    return Err(SnapshotGraphMigratorError::MigrationUnsupported);
                }
                Graphs::Legacy(WorkspaceSnapshotGraph::V4(v4_graph)) => {
                    working_graph = Graphs::Split {
                        address: migrate_v4_to_split_v1(ctx, v4_graph).await?,
                        supergraph_version: SuperGraphVersionDiscriminants::V1,
                        subgraph_version: SubGraphVersionDiscriminants::V1,
                    };
                }
                Graphs::Split { .. } => {
                    break;
                }
            }
        }

        let new_address = match working_graph {
            Graphs::Split { address, .. } => address,
            _ => {
                return Err(SnapshotGraphMigratorError::UnexpectedMigrationFailure(
                    workspace_snapshot_address,
                ));
            }
        };

        info!(
            "Migrated snapshot {} for change set {} to {}",
            workspace_snapshot_address, change_set.id, new_address
        );

        Ok(new_address)
    }
}

impl Default for SnapshotGraphMigrator {
    fn default() -> Self {
        Self::new()
    }
}
