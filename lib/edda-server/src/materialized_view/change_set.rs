use std::collections::HashSet;

use dal::{
    ChangeSet,
    DalContext,
    workspace_snapshot::WorkspaceSnapshotSelector,
};
use frigg::FriggStore;
use si_events::{
    WorkspaceSnapshotAddress,
    workspace_snapshot::Change,
};
use si_frontend_mv_types::{
    definition_checksum::materialized_view_definition_checksums,
    index::change_set::ChangeSetMvIndexV2,
    object::{
        FrontendObject,
        patch::{
            ChangesetIndexUpdate,
            ChangesetPatchBatch,
            ChangesetUpdateMeta,
            ObjectPatch,
        },
    },
    reference::{
        IndexReference,
        ReferenceKind,
    },
};
use si_id::{
    ChangeSetId,
    EntityId,
};
use telemetry::prelude::*;

use crate::{
    materialized_view::{
        MaterializedViewError,
        build_mv_inner,
    },
    updates::EddaUpdates,
};

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

    let definition_checksums = materialized_view_definition_checksums();

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
            && &pointer.definition_checksums == definition_checksums
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
            let index_update = ChangesetIndexUpdate::new(meta, pointer.index_checksum, None);
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
        si.edda.from_index_checksum = Empty,
        si.edda.to_index_checksum = Empty,
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
    let mv_index = ChangeSetMvIndexV2::new(snapshot_to_address.to_string(), index_entries);
    let mv_index_frontend_object = FrontendObject::try_from(mv_index)?;
    let from_index_checksum =
        from_index_checksum.map_or(mv_index_frontend_object.checksum.to_owned(), |check| check);
    let to_index_checksum = mv_index_frontend_object.checksum.to_owned();
    let meta = ChangesetUpdateMeta {
        workspace_id,
        change_set_id,
        from_index_checksum: from_index_checksum.to_owned(),
        to_index_checksum: to_index_checksum.to_owned(),
    };
    span.record("si.edda.from_index_checksum", from_index_checksum);
    span.record("si.edda.to_index_checksum", &to_index_checksum);
    let patch_batch = ChangesetPatchBatch::new(meta.clone(), patches);
    let change_set_mv_id = change_set_id.to_string();

    let index_update =
        ChangesetIndexUpdate::new(meta, mv_index_frontend_object.checksum.to_owned(), None);

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
        si.edda.mv.combined_changes.count = Empty,
        si.edda.mv.outdated_mv.kind_count = Empty,
        si.edda.from_index_checksum = Empty,
        si.edda.to_index_checksum = Empty,
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
    debug!("building for explicit changes: {:?}", changes);
    let span = current_span_for_instrument_at!("info");
    span.record("si.workspace.id", workspace_id.to_string());

    let (index_frontend_object, index_kv_revision) = frigg
        .get_change_set_index(ctx.workspace_pk()?, change_set_id)
        .await?
        .ok_or_else(|| MaterializedViewError::NoIndexForIncrementalBuild {
            workspace_pk: workspace_id,
            change_set_id,
        })?;
    let mv_index = match serde_json::from_value::<
        si_frontend_mv_types::index::change_set::ChangeSetMvIndexVersion,
    >(index_frontend_object.data.clone())
    {
        Ok(si_frontend_mv_types::index::change_set::ChangeSetMvIndexVersion::V1(_)) => {
            return build_all_mv_for_change_set(
                ctx,
                frigg,
                edda_updates,
                parallel_build_limit,
                Some(index_frontend_object.checksum.clone()),
                "fallback due to index upgrade",
            )
            .await;
        }
        Ok(si_frontend_mv_types::index::change_set::ChangeSetMvIndexVersion::V2(v2_index)) => {
            v2_index
        }
        Err(_) => {
            return build_all_mv_for_change_set(
                ctx,
                frigg,
                edda_updates,
                parallel_build_limit,
                Some(index_frontend_object.checksum.clone()),
                "fallback due to index parse error",
            )
            .await;
        }
    };

    // Always check for outdated definitions and combine with explicit changes
    let (outdated_changes, outdated_mv_kind_count) =
        get_changes_for_outdated_definitions(ctx, &mv_index).await?;
    debug!(
        "building for outdated definition changes: {:?}",
        outdated_changes
    );
    span.record("si.edda.mv.outdated_mv.kind_count", outdated_mv_kind_count);

    // Combine and deduplicate changes by entity_id
    let mut combined_changes_map: std::collections::HashMap<EntityId, Change> =
        std::collections::HashMap::new();

    // Add explicit changes
    for &change in changes {
        combined_changes_map.insert(change.entity_id, change);
    }

    // Add outdated definition changes (this will naturally deduplicate by entity_id)
    for change in outdated_changes {
        combined_changes_map.insert(change.entity_id, change);
    }

    let combined_changes: Vec<Change> = combined_changes_map.into_values().collect();
    span.record("si.edda.mv.combined_changes.count", combined_changes.len());

    debug!("combined deduplicated changes: {:?}", combined_changes);

    // If no changes after deduplication, we are done
    if combined_changes.is_empty() {
        debug!("No changes to process");
        return Ok(());
    }

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
        &combined_changes,
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

    let new_mv_index = ChangeSetMvIndexV2::new(to_snapshot_address.to_string(), index_entries);
    let new_mv_index_frontend_object = FrontendObject::try_from(new_mv_index)?;

    let patch = json_patch::diff(
        &index_frontend_object.data,
        &new_mv_index_frontend_object.data,
    );

    let to_index_checksum = new_mv_index_frontend_object.checksum.to_owned();
    let meta = ChangesetUpdateMeta {
        workspace_id,
        change_set_id,
        from_index_checksum: from_index_checksum.clone(),
        to_index_checksum: to_index_checksum.clone(),
    };
    let patch_batch = ChangesetPatchBatch::new(meta.clone(), patches);
    span.record("si.edda.from_index_checksum", &from_index_checksum);
    span.record("si.edda.to_index_checksum", &to_index_checksum);
    let index_patch = ObjectPatch {
        kind: ReferenceKind::ChangeSetMvIndex.to_string(),
        id: new_mv_index_frontend_object.id.clone(),
        from_checksum: from_index_checksum,
        to_checksum: to_index_checksum,
        patch,
    };

    let index_update = ChangesetIndexUpdate::new(
        meta,
        new_mv_index_frontend_object.checksum.to_owned(),
        Some(index_patch),
    );
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

/// Helper function to determine which entities need MaterializedView rebuilds
/// due to outdated definition checksums. Returns synthetic Change objects
/// for those entities and the count of outdated MV kinds. This is inlined into build_mv_for_changes_in_change_set.
/// Only works with V2 indexes - V1 indexes should be handled by the caller with a fallback to build_all_mv_for_change_set.
async fn get_changes_for_outdated_definitions(
    ctx: &DalContext,
    mv_index: &si_frontend_mv_types::index::change_set::ChangeSetMvIndexV2,
) -> Result<(Vec<Change>, usize), MaterializedViewError> {
    // Get current definition checksums from the registry
    let current_definition_checksums =
        si_frontend_mv_types::definition_checksum::materialized_view_definition_checksums();

    // Get the stored definition checksums from the V2 index
    let existing_definition_checksums = mv_index.definition_checksums.clone();

    // Check which MV types have outdated definitions
    let outdated_mv_types: std::collections::HashSet<String> = current_definition_checksums
        .iter()
        .filter_map(|(mv_kind_str, current_checksum)| {
            let existing_checksum = existing_definition_checksums.get(mv_kind_str);
            match existing_checksum {
                Some(existing) if existing != current_checksum => Some(mv_kind_str.clone()),
                None => {
                    // New MV type
                    Some(mv_kind_str.clone())
                }
                _ => None, // Checksum matches, no update needed
            }
        })
        .collect();

    let outdated_mv_kind_count = outdated_mv_types.len();

    if outdated_mv_types.is_empty() {
        debug!("No outdated definitions found");
        return Ok((Vec::new(), 0));
    }

    debug!(
        "Found {} outdated MV definitions: {:?}",
        outdated_mv_types.len(),
        outdated_mv_types,
    );

    // Generate synthetic changes for all entities in the workspace snapshot,
    // similar to build_all_mv_for_change_set, but then filter to only include
    // entities that would trigger builds for the outdated MV types
    let all_synthetic_changes = map_all_nodes_to_change_objects(&ctx.workspace_snapshot()?).await?;

    // First, filter the inventory items to only those with outdated definitions
    let outdated_inventory_items: Vec<_> = ::inventory::iter::<
        si_frontend_mv_types::materialized_view::MaterializedViewInventoryItem,
    >()
    .filter(|item| {
        let mv_kind_str = item.kind().to_string();
        outdated_mv_types.contains(&mv_kind_str)
    })
    .collect();

    debug!(
        "Checking {} outdated inventory items out of {} total for {} changes",
        outdated_inventory_items.len(),
        ::inventory::iter::<si_frontend_mv_types::materialized_view::MaterializedViewInventoryItem>().count(),
        all_synthetic_changes.len(),
    );

    let mut filtered_changes = Vec::new();

    // For each synthetic change, check if it should trigger a build for any of the outdated MV types
    for &change in &all_synthetic_changes {
        for mv_inventory_item in &outdated_inventory_items {
            if mv_inventory_item.should_build_for_change(change) {
                filtered_changes.push(change);
                break; // No need to check other outdated MV types for this change
            }
        }
    }

    debug!(
        "Filtered {} changes from {} total for outdated MV types: {:?}",
        filtered_changes.len(),
        all_synthetic_changes.len(),
        outdated_mv_types,
    );

    Ok((filtered_changes, outdated_mv_kind_count))
}
