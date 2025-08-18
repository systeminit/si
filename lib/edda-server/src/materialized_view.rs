use std::{
    collections::{
        BinaryHeap,
        HashSet,
    },
    time::Duration,
};

use dal::{
    ChangeSet,
    ChangeSetError,
    ComponentError,
    DalContext,
    SchemaVariantError,
    TransactionsError,
    WorkspaceSnapshotError,
    action::{
        ActionError,
        prototype::ActionPrototypeError,
    },
    cached_module::CachedModuleError,
    diagram::DiagramError,
    prop::PropError,
    workspace_snapshot::WorkspaceSnapshotSelector,
};
use frigg::{
    FriggError,
    FriggStore,
};
use serde_json::Value;
use si_events::{
    WorkspaceSnapshotAddress,
    materialized_view::BuildPriority,
    workspace_snapshot::Change,
};
use si_frontend_mv_types::{
    action::{
        ActionPrototypeViewList as ActionPrototypeViewListMv,
        ActionViewList as ActionViewListMv,
    },
    checksum::FrontendChecksum,
    component::{
        Component as ComponentMv,
        ComponentInList as ComponentInListMv,
        ComponentList as ComponentListMv,
        SchemaMembers,
        attribute_tree::AttributeTree as AttributeTreeMv,
        component_diff::ComponentDiff as ComponentDiffMv,
    },
    dependent_values::DependentValueComponentList as DependentValueComponentListMv,
    incoming_connections::{
        IncomingConnections as IncomingConnectionsMv,
        IncomingConnectionsList as IncomingConnectionsListMv,
        ManagementConnections as ManagementConnectionsMv,
    },
    index::{
        ChangeSetMvIndex,
        DeploymentMvIndex,
    },
    materialized_view::{
        MaterializedViewInventoryItem,
        materialized_view_definitions_checksum,
    },
    object::{
        FrontendObject,
        patch::{
            ChangesetIndexUpdate,
            ChangesetPatchBatch,
            ChangesetUpdateMeta,
            DeploymentIndexUpdate,
            DeploymentPatchBatch,
            DeploymentUpdateMeta,
            ObjectPatch,
            StreamingPatch,
        },
    },
    reference::{
        IndexReference,
        ReferenceKind,
    },
    schema_variant::{
        SchemaVariant as SchemaVariantMv,
        SchemaVariantCategories as SchemaVariantCategoriesMv,
    },
    view::{
        View as ViewMv,
        ViewComponentList as ViewComponentListMv,
        ViewList as ViewListMv,
    },
};
use si_id::{
    ChangeSetId,
    WorkspacePk,
};
use si_layer_cache::LayerDbError;
use telemetry::prelude::*;
use telemetry_utils::metric;
use thiserror::Error;
use tokio::{
    task::JoinSet,
    time::Instant,
};

use crate::updates::{
    EddaUpdates,
    EddaUpdatesError,
};

#[remain::sorted]
#[derive(Debug, Error)]
pub enum MaterializedViewError {
    #[error("Action error: {0}")]
    Action(#[from] ActionError),
    #[error("ActionPrototype error: {0}")]
    ActionPrototype(#[from] ActionPrototypeError),
    #[error("Cached module error: {0}")]
    CachedModule(#[from] CachedModuleError),
    #[error("Change Set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("Component error: {0}")]
    Component(#[from] ComponentError),
    #[error("Diagram error: {0}")]
    Diagram(#[from] DiagramError),
    #[error("edda updates error: {0}")]
    EddaUpdates(#[from] EddaUpdatesError),
    #[error("Frigg error: {0}")]
    Frigg(#[from] FriggError),
    #[error("Join error: {0}")]
    JoinError(#[from] tokio::task::JoinError),
    #[error("Layer DB error: {0}")]
    LayerDb(#[from] LayerDbError),
    #[error("materialized views error: {0}")]
    MaterializedViews(#[from] dal_materialized_views::Error),
    #[error(
        "No index for incremental build for workspace {workspace_pk} and change set {change_set_id}"
    )]
    NoIndexForIncrementalBuild {
        workspace_pk: WorkspacePk,
        change_set_id: ChangeSetId,
    },
    #[error("Prop error: {0}")]
    Prop(#[from] PropError),
    #[error("SchemaVariant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("serde_json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("Transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("WorkspaceSnapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

/// This function iterates active change sets that share the same snapshot address,
/// and looks for an [`ChangeSetMvIndex`] it can use. If it finds one, create a new index pointer to
/// the found [`ChangeSetMvIndex`] and return true. If none can be used, return false.
/// NOTE: The copy will fail if we try to reuse an index for a change set that already has an index pointer
#[instrument(
    name = "materialized_view.try_reuse_mv_index_for_new_change_set",
    level = "info",
    skip_all,
    fields(
        si.workspace.id = Empty,
        si.change_set.id = %ctx.change_set_id(),
        si.from_change_set.id = Empty,
    ),
)]
pub async fn try_reuse_mv_index_for_new_change_set(
    ctx: &DalContext,
    frigg: &FriggStore,
    edda_updates: &EddaUpdates,
    snapshot_address: WorkspaceSnapshotAddress,
) -> Result<bool, MaterializedViewError> {
    let span = current_span_for_instrument_at!("info");
    let workspace_id = ctx.workspace_pk()?;
    let change_set_id = ctx.change_set_id();
    span.record("si.workspace.id", workspace_id.to_string());
    // first find a change set Id in my workspace that has the same snapshot address that I do
    // do we care which change set we use? Assuming there's more than one, we could choose Head...
    let change_sets_using_snapshot = ChangeSet::list_active(ctx).await?;

    let definitions_checksum = materialized_view_definitions_checksum();

    for change_set in change_sets_using_snapshot {
        // found a match, so let's retrieve that MvIndex and put the same object as ours
        // If we're unable to parse the pointer for some reason, don't treat it as a hard error and just move on.
        let Ok(Some((pointer, _revision))) = frigg
            .get_change_set_index_pointer_value(workspace_id, change_set.id)
            .await
        else {
            // try the next one
            // no need error if this index was never built, it would get rebuilt when necessary
            continue;
        };

        if pointer.snapshot_address == snapshot_address.to_string()
            && pointer.definition_checksum == definitions_checksum
        {
            // found one, create a new index pointer to it!
            let change_set_mv_id = change_set_id.to_string();
            frigg
                .insert_change_set_index_key_for_existing_index(
                    workspace_id,
                    &change_set_mv_id,
                    pointer.clone(),
                )
                .await?;
            span.record("si.from_change_set.id", change_set.id.to_string());
            let meta = ChangesetUpdateMeta {
                workspace_id,
                change_set_id,
                from_index_checksum: pointer.clone().index_checksum.to_owned(), // These are the same because we're starting from current for the new change set
                to_index_checksum: pointer.clone().index_checksum,
            };
            let index_update = ChangesetIndexUpdate::new(meta, pointer.index_checksum);
            edda_updates
                .publish_change_set_index_update(index_update)
                .await?;
            return Ok(true);
        }
    }
    Ok(false)
}

/// This function first tries to copy and existing [`ChangeSetMvIndex`] if we find a valid one with the same snapshot address
/// If it cannot copy one, it builds all Materialized Views (MVs) for the change set in the [`DalContext`].
/// It assumes there is no existing [`ChangeSetMvIndex`] for the change set.
#[instrument(
    name = "materialized_view.new_change_set",
    level = "info",
    skip_all,
    fields(
        si.workspace.id = Empty,
        si.change_set.id = %ctx.change_set_id(),
    ),
)]
pub async fn reuse_or_rebuild_index_for_new_change_set(
    ctx: &DalContext,
    frigg: &FriggStore,
    edda_updates: &EddaUpdates,
    parallel_build_limit: usize,
    to_snapshot_address: WorkspaceSnapshotAddress,
) -> Result<(), MaterializedViewError> {
    let span = current_span_for_instrument_at!("info");
    span.record("si.workspace.id", ctx.workspace_pk()?.to_string());
    let did_copy_result =
        self::try_reuse_mv_index_for_new_change_set(ctx, frigg, edda_updates, to_snapshot_address)
            .await;

    match did_copy_result {
        // If we returned successfully, evaluate the response,
        Ok(did_copy) => {
            if did_copy {
                return Ok(());
            } else {
                // we did not copy anything, so we must rebuild from scratch (no from_snapshot_address this time)
                self::build_all_mv_for_change_set(
                    ctx,
                    frigg,
                    edda_updates,
                    parallel_build_limit,
                    None,
                    "initial build",
                )
                .await?
            }
        }
        Err(err) => {
            error!(si.error.message = ?err, "error copying existing index");
            // we did not copy anything, so we must rebuild from scratch (no from_snapshot_address this time)
            self::build_all_mv_for_change_set(
                ctx,
                frigg,
                edda_updates,
                parallel_build_limit,
                None,
                "initial build",
            )
            .await?
        }
    }
    Ok(())
}

/// This function builds all Materialized Views (MVs) for the change set in the [`DalContext`].
/// It assumes there is no existing [`ChangeSetMvIndex`] for the change set.
/// If we're rebuilding due to a moved snapshot (Edda got behind), then we pass in the [`from_snapshot_address`]
/// so that patches can be successfully processed.
/// If we're rebuilding on demand, pass [`None`] for [`from_snapshot_address`]
#[instrument(
    name = "materialized_view.build_all_mv_for_change_set",
    level = "info",
    skip_all,
    fields(
        si.workspace.id = Empty,
        si.change_set.id = %ctx.change_set_id(),
        si.materialized_view.reason = reason_message,
        si.edda.mv.count = Empty,
        si.edda.mv.avg_build_elapsed_ms = Empty,
        si.edda.mv.max_build_elapsed_ms = Empty,
        si.edda.mv.slowest_kind = Empty,
    ),
)]
pub async fn build_all_mv_for_change_set(
    ctx: &DalContext,
    frigg: &FriggStore,
    edda_updates: &EddaUpdates,
    parallel_build_limit: usize,
    from_index_checksum: Option<String>,
    reason_message: &'static str,
) -> Result<(), MaterializedViewError> {
    let span = current_span_for_instrument_at!("info");
    span.record("si.workspace.id", ctx.workspace_pk()?.to_string());

    // Pretend everything has changed, and build all MVs.
    let changes = map_all_nodes_to_change_objects(&ctx.workspace_snapshot()?)
        .instrument(tracing::info_span!(
            "materialized_view.build_all_mv_for_change_set.make_changes_for_all_nodes"
        ))
        .await?;

    let (
        frontend_objects,
        patches,
        build_count,
        build_total_elapsed,
        build_max_elapsed,
        build_slowest_mv_kind,
    ) = build_mv_inner(
        ctx,
        frigg,
        parallel_build_limit,
        edda_updates,
        ctx.workspace_pk()?,
        ctx.change_set_id(),
        &changes,
    )
    .await?;
    span.record("si.edda.mv.count", build_count);
    if build_count > 0 {
        span.record(
            "si.edda.mv.avg_build_elapsed_ms",
            build_total_elapsed.as_millis() / build_count,
        );
        span.record(
            "si.edda.mv.max_build_elapsed_ms",
            build_max_elapsed.as_millis(),
        );
        span.record("si.edda.mv.slowest_kind", build_slowest_mv_kind);
    }

    let mut index_entries: Vec<_> = frontend_objects.into_iter().map(Into::into).collect();
    index_entries.sort();
    let snapshot_to_address = ctx.workspace_snapshot()?.address().await;
    let workspace_id = ctx.workspace_pk()?;
    let change_set_id = ctx.change_set_id();
    debug!("index_entries {:?}", index_entries);
    let mv_index = ChangeSetMvIndex::new(snapshot_to_address.to_string(), index_entries);
    let mv_index_frontend_object = FrontendObject::try_from(mv_index)?;
    let meta = ChangesetUpdateMeta {
        workspace_id,
        change_set_id,
        from_index_checksum: from_index_checksum
            .map_or(mv_index_frontend_object.checksum.to_owned(), |check| check),
        to_index_checksum: mv_index_frontend_object.checksum.to_owned(),
    };
    let patch_batch = ChangesetPatchBatch::new(meta.clone(), patches);
    let change_set_mv_id = change_set_id.to_string();
    let index_update =
        ChangesetIndexUpdate::new(meta, mv_index_frontend_object.checksum.to_owned());

    frigg
        .put_change_set_index(
            ctx.workspace_pk()?,
            &change_set_mv_id,
            &mv_index_frontend_object,
        )
        .await?;

    edda_updates
        .publish_change_set_patch_batch(patch_batch)
        .await?;
    edda_updates
        .publish_change_set_index_update(index_update)
        .await?;

    Ok(())
}

pub async fn map_all_nodes_to_change_objects(
    snapshot: &WorkspaceSnapshotSelector,
) -> Result<Vec<Change>, MaterializedViewError> {
    Ok(snapshot
        .nodes()
        .await?
        .into_iter()
        .map(|node| Change {
            entity_id: node.id().into(),
            entity_kind: (&node).into(),
            merkle_tree_hash: node.merkle_tree_hash(),
        })
        .collect())
}

#[instrument(
    name = "materialized_view.build_mv_for_changes_in_change_set",
    level = "info",
    skip_all,
    fields(
        si.workspace.id = Empty,
        si.change_set.id = %change_set_id,
        si.snapshot_from_address = %from_snapshot_address,
        si.snapshot_to_address = %to_snapshot_address,
        si.edda_request.changes.count = changes.len(),
        si.edda.mv.count = Empty,
        si.edda.mv.avg_build_elapsed_ms = Empty,
        si.edda.mv.max_build_elapsed_ms = Empty,
        si.edda.mv.slowest_kind = Empty,
    )
)]
#[allow(clippy::too_many_arguments)]
pub async fn build_mv_for_changes_in_change_set(
    ctx: &DalContext,
    frigg: &FriggStore,
    edda_updates: &EddaUpdates,
    parallel_build_limit: usize,
    change_set_id: ChangeSetId,
    from_snapshot_address: WorkspaceSnapshotAddress,
    to_snapshot_address: WorkspaceSnapshotAddress,
    changes: &[Change],
) -> Result<(), MaterializedViewError> {
    let workspace_id = ctx.workspace_pk()?;
    debug!("building for changes: {:?}", changes);
    let span = current_span_for_instrument_at!("info");
    span.record("si.workspace.id", workspace_id.to_string());
    let (index_frontend_object, index_kv_revision) = frigg
        .get_change_set_index(ctx.workspace_pk()?, change_set_id)
        .await?
        .ok_or_else(|| MaterializedViewError::NoIndexForIncrementalBuild {
            workspace_pk: workspace_id,
            change_set_id,
        })?;
    let from_index_checksum = index_frontend_object.checksum;
    let (
        frontend_objects,
        patches,
        build_count,
        build_total_elapsed,
        build_max_elapsed,
        build_slowest_mv_kind,
    ) = build_mv_inner(
        ctx,
        frigg,
        parallel_build_limit,
        edda_updates,
        workspace_id,
        change_set_id,
        changes,
    )
    .await?;
    span.record("si.edda.mv.count", build_count);
    if build_count > 0 {
        span.record(
            "si.edda.mv.avg_build_elapsed_ms",
            build_total_elapsed.as_millis() / build_count,
        );
        span.record(
            "si.edda.mv.max_build_elapsed_ms",
            build_max_elapsed.as_millis(),
        );
        span.record("si.edda.mv.slowest_kind", build_slowest_mv_kind);
    }

    let mv_index: ChangeSetMvIndex = serde_json::from_value(index_frontend_object.data)?;
    let removal_checksum = "0".to_string();
    let removed_items: HashSet<(String, String)> = patches
        .iter()
        .filter_map(|patch| {
            if patch.to_checksum == removal_checksum {
                Some((patch.kind.clone(), patch.id.clone()))
            } else {
                None
            }
        })
        .collect();

    let mut index_entries: Vec<IndexReference> =
        frontend_objects.into_iter().map(Into::into).collect();
    let new_index_entries: HashSet<(String, String)> = index_entries
        .iter()
        .map(|index_entry| (index_entry.kind.clone(), index_entry.id.clone()))
        .collect();

    for index_entry in mv_index.mv_list {
        if !removed_items.contains(&(index_entry.kind.clone(), index_entry.id.clone()))
            && !new_index_entries.contains(&(index_entry.kind.clone(), index_entry.id.clone()))
        {
            index_entries.push(index_entry.clone());
        }
    }
    index_entries.sort();

    let new_mv_index = ChangeSetMvIndex::new(to_snapshot_address.to_string(), index_entries);

    let new_mv_index_frontend_object = FrontendObject::try_from(new_mv_index)?;
    let meta = ChangesetUpdateMeta {
        workspace_id,
        change_set_id,
        from_index_checksum,
        to_index_checksum: new_mv_index_frontend_object.checksum.to_owned(),
    };
    let patch_batch = ChangesetPatchBatch::new(meta.clone(), patches);
    let index_update =
        ChangesetIndexUpdate::new(meta, new_mv_index_frontend_object.checksum.to_owned());
    let change_set_mv_id = change_set_id.to_string();

    frigg
        .update_change_set_index(
            workspace_id,
            &change_set_mv_id,
            &new_mv_index_frontend_object,
            index_kv_revision,
        )
        .await?;

    edda_updates
        .publish_change_set_patch_batch(patch_batch)
        .await?;
    edda_updates
        .publish_change_set_index_update(index_update)
        .await?;

    Ok(())
}

/// This function builds all Materialized Views (MVs) this deployment environment
#[instrument(
    name = "materialized_view.build_all_mvs_for_deployment",
    level = "info",
    skip_all,
    fields(
        si.materialized_view.reason = reason_message,
        si.edda.mv.count = Empty,
        si.edda.mv.avg_build_elapsed_ms = Empty,
        si.edda.mv.max_build_elapsed_ms = Empty,
        si.edda.mv.slowest_kind = Empty,
    ),
)]
pub async fn build_all_mvs_for_deployment(
    ctx: &DalContext,
    frigg: &FriggStore,
    edda_updates: &EddaUpdates,
    _parallel_build_limit: usize,
    reason_message: &'static str,
) -> Result<(), MaterializedViewError> {
    // TODO create update and delete ops when needed
    let op = BuildMvOp::Create;

    let (Some(patch), Some(frontend_object)) = build_mv(
        op,
        ReferenceKind::CachedSchemas,
        dal_materialized_views::cached_schemas::assemble(ctx),
    )
    .await?
    else {
        return Ok(());
    };

    let index_reference = IndexReference::from(frontend_object.clone());

    // Make an index, store it with the deployment function
    let index_entries: Vec<_> = vec![index_reference];

    debug!("index_entries {:?}", index_entries);
    let mv_index = DeploymentMvIndex::new(index_entries);
    let mv_index_frontend_object = FrontendObject::try_from(mv_index)?;
    let from_index_checksum = mv_index_frontend_object.checksum.to_owned();
    let to_index_checksum = from_index_checksum.clone();
    let meta = DeploymentUpdateMeta {
        from_index_checksum,
        to_index_checksum,
    };
    let patch_batch = DeploymentPatchBatch::new(meta.clone(), vec![patch]);
    let index_update =
        DeploymentIndexUpdate::new(meta, mv_index_frontend_object.checksum.to_owned());

    // Store it on frigg
    frigg
        .put_deployment_index(&mv_index_frontend_object)
        .await?;
    frigg.insert_deployment_object(&frontend_object).await?;

    // publish updates
    edda_updates
        .publish_deployment_patch_batch(patch_batch)
        .await?;
    edda_updates
        .publish_deployment_index_update(index_update)
        .await?;

    Ok(())
}

macro_rules! spawn_build_mv_task {
    ($build_tasks:expr, $ctx:expr, $frigg:expr, $change:expr, $mv_id:expr, $mv:ty, $build_fn:expr $(,)?) => {
        let kind = <$mv as ::si_frontend_mv_types::materialized_view::MaterializedView>::kind();

        // Record the currently running number of MV build tasks so we can see just how parallel this actually
        // ends up being.
        metric!(counter.edda.mv_build = 1);
        $build_tasks.spawn(build_mv_for_graph_task(
            $ctx.clone(),
            $frigg.clone(),
            $change,
            $mv_id,
            kind,
            $build_fn,
        ));
    };
}

type BuildMvInnerReturn = (
    Vec<FrontendObject>,
    Vec<ObjectPatch>,
    u128,
    Duration,
    Duration,
    &'static str,
);

#[instrument(
    name = "materialized_view.build_mv_inner",
    level = "debug",
    skip_all,
    fields(
        si.workspace.id = %workspace_pk,
        si.change_set_id = %change_set_id,
    ),
)]
async fn build_mv_inner(
    ctx: &DalContext,
    frigg: &FriggStore,
    parallel_build_limit: usize,
    edda_updates: &EddaUpdates,
    workspace_pk: si_id::WorkspacePk,
    change_set_id: ChangeSetId,
    changes: &[Change],
) -> Result<BuildMvInnerReturn, MaterializedViewError> {
    let mut frontend_objects = Vec::new();
    let mut patches = Vec::new();
    let mut build_tasks = JoinSet::new();
    let mut queued_mv_builds = BinaryHeap::new();

    // Queue everything so we can let the priority queue determine the order everything is built.
    for &change in changes {
        for mv_inventory_item in ::inventory::iter::<MaterializedViewInventoryItem>() {
            if mv_inventory_item.should_build_for_change(change) {
                queued_mv_builds.push(QueuedBuildMvTask {
                    change,
                    mv_kind: mv_inventory_item.kind(),
                    priority: mv_inventory_item.build_priority(),
                });
            }
        }
    }

    let mut build_total_elapsed = Duration::from_nanos(0);
    let mut build_count: u128 = 0;
    let mut build_max_elapsed = Duration::from_nanos(0);
    let mut build_slowest_mv_kind: &str = "N/A";

    loop {
        // If there there aren't any queued builds waiting for the concurrency limit then
        // we've finished everything and are able to send off the collected
        // FrontendObjects, and ObjectPatches to update the index & send patches out over the
        // websocket.
        if queued_mv_builds.is_empty() && build_tasks.is_empty() {
            break;
        }

        // Spawn as many of the queued build tasks as we can, up to the concurrency limit.
        while !queued_mv_builds.is_empty() && build_tasks.len() < parallel_build_limit {
            let Some(QueuedBuildMvTask {
                change, mv_kind, ..
            }) = queued_mv_builds.pop()
            else {
                // This _really_ shouldn't ever return `None` as we just checked that
                // `queued_mv_builds` is not empty.
                break;
            };
            spawn_build_mv_task_for_change_and_mv_kind(
                &mut build_tasks,
                ctx,
                frigg,
                change,
                mv_kind,
                workspace_pk,
            )
            .await?
        }

        if let Some(join_result) = build_tasks.join_next().await {
            let (kind, mv_id, build_duration, execution_result) = join_result?;
            metric!(counter.edda.mv_build = -1);

            match execution_result {
                Ok((maybe_patch, maybe_frontend_object)) => {
                    // We need to make sure the frontend object is inserted into the store first so that
                    // a client can directly fetch it without racing against the object's insertion if the
                    // client does not already have the base object to apply the streaming patch to.
                    if let Some(frontend_object) = maybe_frontend_object {
                        frigg
                            .insert_workspace_object(workspace_pk, &frontend_object)
                            .await?;
                        frontend_objects.push(frontend_object);
                    }
                    if let Some(patch) = maybe_patch {
                        let streaming_patch = StreamingPatch::new(
                            workspace_pk,
                            change_set_id,
                            kind,
                            mv_id,
                            patch.from_checksum.clone(),
                            patch.to_checksum.clone(),
                            patch.patch.clone(),
                        );
                        edda_updates
                            .publish_streaming_patch(streaming_patch)
                            .await?;

                        debug!("Patch!: {:?}", patch);
                        patches.push(patch);
                    }
                    build_count += 1;
                    if build_duration > build_max_elapsed {
                        build_max_elapsed = build_duration;
                        build_slowest_mv_kind = kind.into();
                    }
                    build_total_elapsed += build_duration;
                }
                Err(err) => {
                    warn!(name = "mv_build_error", si.error.message = err.to_string(), kind = %kind, id = %mv_id);
                }
            }
        }
    }

    Ok((
        frontend_objects,
        patches,
        build_count,
        build_total_elapsed,
        build_max_elapsed,
        build_slowest_mv_kind,
    ))
}

type BuildMvTaskResult = (
    ReferenceKind,
    String,
    Duration,
    Result<(Option<ObjectPatch>, Option<FrontendObject>), MaterializedViewError>,
);

async fn build_mv_for_graph_task<F, T, E>(
    ctx: DalContext,
    frigg: FriggStore,
    change: Change,
    mv_id: String,
    mv_kind: ReferenceKind,
    build_mv_future: F,
) -> BuildMvTaskResult
where
    F: Future<Output = Result<T, E>>,
    T: serde::Serialize + TryInto<FrontendObject> + FrontendChecksum,
    E: Into<MaterializedViewError>,
    MaterializedViewError: From<E>,
    MaterializedViewError: From<<T as TryInto<FrontendObject>>::Error>,
{
    debug!(kind = %mv_kind, id = %mv_id, "Building MV");
    let start = Instant::now();

    let result = build_mv_for_graph_task_inner(
        &ctx,
        &frigg,
        change,
        mv_id.clone(),
        mv_kind,
        build_mv_future,
    )
    .await;

    (mv_kind, mv_id, start.elapsed(), result)
}

type MvBuilderResult = Result<(Option<ObjectPatch>, Option<FrontendObject>), MaterializedViewError>;

pub enum BuildMvOp {
    Create,
    Delete { id: String, checksum: String },
    UpdateFrom { checksum: String, data: Value },
}

async fn build_mv_for_graph_task_inner<F, T, E>(
    ctx: &DalContext,
    frigg: &FriggStore,
    change: Change,
    mv_id: String,
    mv_kind: ReferenceKind,
    build_mv_future: F,
) -> MvBuilderResult
where
    F: Future<Output = Result<T, E>>,
    T: serde::Serialize + TryInto<FrontendObject> + FrontendChecksum,
    E: Into<MaterializedViewError>,
    MaterializedViewError: From<E>,
    MaterializedViewError: From<<T as TryInto<FrontendObject>>::Error>,
{
    let op = {
        let maybe_previous_version = frigg
            .get_current_workspace_object(
                ctx.workspace_pk()?,
                ctx.change_set_id(),
                &mv_kind.to_string(),
                &mv_id,
            )
            .await?;

        let exists_on_snapshot = ctx
            .workspace_snapshot()?
            .node_exists(change.entity_id)
            .await;

        if !exists_on_snapshot {
            let checksum = maybe_previous_version
                .map(|obj| obj.checksum)
                .unwrap_or_else(|| "0".to_string());

            BuildMvOp::Delete {
                id: mv_id,
                checksum,
            }
        } else if let Some(previous_version) = maybe_previous_version {
            BuildMvOp::UpdateFrom {
                checksum: previous_version.checksum,
                data: previous_version.data,
            }
        } else {
            BuildMvOp::Create
        }
    };

    build_mv(op, mv_kind, build_mv_future).await
}

pub async fn build_mv<F, T, E>(
    operation: BuildMvOp,
    mv_kind: ReferenceKind,
    build_mv_future: F,
) -> MvBuilderResult
where
    F: Future<Output = Result<T, E>>,
    T: serde::Serialize + TryInto<FrontendObject> + FrontendChecksum,
    E: Into<MaterializedViewError>,
    MaterializedViewError: From<E>,
    MaterializedViewError: From<<T as TryInto<FrontendObject>>::Error>,
{
    let kind = mv_kind.to_string();

    if let BuildMvOp::Delete { id, checksum } = operation {
        Ok((
            Some(ObjectPatch {
                kind,
                id,
                from_checksum: checksum,
                to_checksum: "0".to_string(),
                patch: json_patch::Patch(vec![json_patch::PatchOperation::Remove(
                    json_patch::RemoveOperation::default(),
                )]),
            }),
            None,
        ))
    } else {
        let mv = build_mv_future.await?;
        let mv_json = serde_json::to_value(&mv)?;
        let to_checksum = FrontendChecksum::checksum(&mv).to_string();
        let frontend_object: FrontendObject = mv.try_into()?;

        let (from_checksum, previous_data) = match operation {
            BuildMvOp::Create => ("0".to_string(), serde_json::Value::Null),
            BuildMvOp::UpdateFrom { checksum, data } => (checksum, data),
            BuildMvOp::Delete { .. } => unreachable!(),
        };

        if from_checksum == to_checksum {
            // If checksum does not change, return the object but no patch
            Ok((None, Some(frontend_object)))
        } else {
            Ok((
                Some(ObjectPatch {
                    kind,
                    id: frontend_object.id.clone(),
                    from_checksum,
                    to_checksum,
                    patch: json_patch::diff(&previous_data, &mv_json),
                }),
                Some(frontend_object),
            ))
        }
    }
}

/// The [`Change`] of the trigger entity that would have spawned a build task for the
/// `mv_kind`, if we hadn't already reached the concurrency limit for running build
/// tasks.
#[derive(Debug, Eq, PartialEq)]
struct QueuedBuildMvTask {
    pub change: Change,
    pub mv_kind: ReferenceKind,
    pub priority: BuildPriority,
}

impl Ord for QueuedBuildMvTask {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.priority.cmp(&other.priority) {
            std::cmp::Ordering::Equal => {
                // Within the same priority, items will be ordered by the triggering
                // entity ID. Doing `self.cmp(other)` means that the newer IDs are
                // be considered "larger", and items with the same priority will be
                // processed newest to oldest acording to when their ID was created.
                self.change.entity_id.cmp(&other.change.entity_id)
            }
            ord => ord,
        }
    }
}

impl PartialOrd for QueuedBuildMvTask {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

async fn spawn_build_mv_task_for_change_and_mv_kind(
    build_tasks: &mut JoinSet<BuildMvTaskResult>,
    ctx: &DalContext,
    frigg: &FriggStore,
    change: Change,
    mv_kind: ReferenceKind,
    workspace_pk: si_id::WorkspacePk,
) -> Result<(), MaterializedViewError> {
    match mv_kind {
        ReferenceKind::ActionPrototypeViewList => {
            let entity_mv_id = change.entity_id.to_string();

            spawn_build_mv_task!(
                build_tasks,
                ctx,
                frigg,
                change,
                entity_mv_id,
                ActionPrototypeViewListMv,
                dal_materialized_views::action_prototype_view_list::assemble(
                    ctx.clone(),
                    si_events::ulid::Ulid::from(change.entity_id).into()
                ),
            );
        }
        ReferenceKind::ActionViewList => {
            let workspace_mv_id = workspace_pk.to_string();

            spawn_build_mv_task!(
                build_tasks,
                ctx,
                frigg,
                change,
                workspace_mv_id,
                ActionViewListMv,
                dal_materialized_views::action_view_list::assemble(ctx.clone()),
            );
        }
        ReferenceKind::AttributeTree => {
            let entity_mv_id = change.entity_id.to_string();

            spawn_build_mv_task!(
                build_tasks,
                ctx,
                frigg,
                change,
                entity_mv_id,
                AttributeTreeMv,
                dal_materialized_views::component::attribute_tree::assemble(
                    ctx.clone(),
                    si_events::ulid::Ulid::from(change.entity_id).into(),
                ),
            );
        }
        ReferenceKind::ComponentInList => {
            let entity_mv_id = change.entity_id.to_string();

            spawn_build_mv_task!(
                build_tasks,
                ctx,
                frigg,
                change,
                entity_mv_id,
                ComponentInListMv,
                dal_materialized_views::component::assemble_in_list(
                    ctx.clone(),
                    si_events::ulid::Ulid::from(change.entity_id).into(),
                ),
            );
        }
        ReferenceKind::Component => {
            let entity_mv_id = change.entity_id.to_string();

            spawn_build_mv_task!(
                build_tasks,
                ctx,
                frigg,
                change,
                entity_mv_id,
                ComponentMv,
                dal_materialized_views::component::assemble(
                    ctx.clone(),
                    si_events::ulid::Ulid::from(change.entity_id).into(),
                ),
            );
        }
        ReferenceKind::ComponentDiff => {
            let entity_mv_id = change.entity_id.to_string();

            spawn_build_mv_task!(
                build_tasks,
                ctx,
                frigg,
                change,
                entity_mv_id,
                ComponentDiffMv,
                dal_materialized_views::component::component_diff::assemble(
                    ctx.clone(),
                    si_events::ulid::Ulid::from(change.entity_id).into(),
                ),
            );
        }
        ReferenceKind::ComponentList => {
            let workspace_mv_id = workspace_pk.to_string();

            spawn_build_mv_task!(
                build_tasks,
                ctx,
                frigg,
                change,
                workspace_mv_id,
                ComponentListMv,
                dal_materialized_views::component_list::assemble(ctx.clone()),
            );
        }
        ReferenceKind::DependentValueComponentList => {
            let workspace_mv_id = workspace_pk.to_string();

            spawn_build_mv_task!(
                build_tasks,
                ctx,
                frigg,
                change,
                workspace_mv_id,
                DependentValueComponentListMv,
                dal_materialized_views::dependent_value_component_list::assemble(ctx.clone()),
            );
        }
        ReferenceKind::IncomingConnections => {
            let entity_mv_id = change.entity_id.to_string();

            spawn_build_mv_task!(
                build_tasks,
                ctx,
                frigg,
                change,
                entity_mv_id,
                IncomingConnectionsMv,
                dal_materialized_views::incoming_connections::assemble(
                    ctx.clone(),
                    si_events::ulid::Ulid::from(change.entity_id).into(),
                ),
            );
        }
        ReferenceKind::ManagementConnections => {
            let entity_mv_id = change.entity_id.to_string();

            spawn_build_mv_task!(
                build_tasks,
                ctx,
                frigg,
                change,
                entity_mv_id,
                ManagementConnectionsMv,
                dal_materialized_views::incoming_connections::assemble_management(
                    ctx.clone(),
                    si_events::ulid::Ulid::from(change.entity_id).into(),
                ),
            );
        }
        ReferenceKind::IncomingConnectionsList => {
            let workspace_mv_id = workspace_pk.to_string();

            spawn_build_mv_task!(
                build_tasks,
                ctx,
                frigg,
                change,
                workspace_mv_id,
                IncomingConnectionsListMv,
                dal_materialized_views::incoming_connections_list::assemble(ctx.clone()),
            );
        }
        ReferenceKind::SchemaMembers => {
            let entity_mv_id = change.entity_id.to_string();

            spawn_build_mv_task!(
                build_tasks,
                ctx,
                frigg,
                change,
                entity_mv_id,
                SchemaMembers,
                dal_materialized_views::component::assemble_schema_members(
                    ctx.clone(),
                    si_events::ulid::Ulid::from(change.entity_id).into()
                ),
            );
        }
        ReferenceKind::SchemaVariant => {
            let entity_mv_id = change.entity_id.to_string();

            spawn_build_mv_task!(
                build_tasks,
                ctx,
                frigg,
                change,
                entity_mv_id,
                SchemaVariantMv,
                dal_materialized_views::schema_variant::assemble(
                    ctx.clone(),
                    si_events::ulid::Ulid::from(change.entity_id).into()
                ),
            );
        }
        ReferenceKind::SchemaVariantCategories => {
            let workspace_mv_id = workspace_pk.to_string();

            spawn_build_mv_task!(
                build_tasks,
                ctx,
                frigg,
                change,
                workspace_mv_id,
                SchemaVariantCategoriesMv,
                dal_materialized_views::schema_variant_categories::assemble(ctx.clone()),
            );
        }
        ReferenceKind::View => {
            let entity_mv_id = change.entity_id.to_string();

            spawn_build_mv_task!(
                build_tasks,
                ctx,
                frigg,
                change,
                entity_mv_id,
                ViewMv,
                dal_materialized_views::view::assemble(
                    ctx.clone(),
                    si_events::ulid::Ulid::from(change.entity_id).into()
                )
            );
        }
        ReferenceKind::ViewList => {
            let workspace_mv_id = workspace_pk.to_string();

            spawn_build_mv_task!(
                build_tasks,
                ctx,
                frigg,
                change,
                workspace_mv_id,
                ViewListMv,
                dal_materialized_views::view_list::assemble(ctx.clone()),
            );
        }
        ReferenceKind::ViewComponentList => {
            let entity_mv_id = change.entity_id.to_string();

            spawn_build_mv_task!(
                build_tasks,
                ctx,
                frigg,
                change,
                entity_mv_id,
                ViewComponentListMv,
                dal_materialized_views::view_component_list::assemble(
                    ctx.clone(),
                    si_events::ulid::Ulid::from(change.entity_id).into(),
                ),
            );
        }

        // Building the `MvIndex` itself is handled separately as the logic depends
        // on whether we're doing an incremental build or a full build from scratch.
        ReferenceKind::ChangeSetMvIndex | ReferenceKind::DeploymentMvIndex => {}

        // These `ReferenceKind` do not have associated `MaterializedView`s (yet?),
        // so we skip them.
        ReferenceKind::ChangeSetList | ReferenceKind::ChangeSetRecord => {}

        ReferenceKind::CachedSchemas => {
            error!("Trying to build cached schemas via the graph task.")
        }
    }

    Ok(())
}
