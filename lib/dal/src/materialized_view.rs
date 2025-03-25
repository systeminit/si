use std::collections::HashSet;

use frigg::{Error as FriggError, FriggStore};
use si_events::{
    workspace_snapshot::{Change, Checksum},
    WorkspaceSnapshotAddress,
};
use si_frontend_types::{
    index::MvIndex,
    object::{
        patch::{ObjectPatch as FrontendObjectPatch, PatchBatch, PatchBatchMeta, PATCH_BATCH_KIND},
        FrontendObject,
    },
    reference::{IndexReference, ReferenceKind},
    view::{View as ViewMv, ViewList as ViewListMv},
    MaterializedView,
};
use si_id::{ChangeSetId, WorkspacePk};
use strum::IntoEnumIterator;
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    data_cache::{DataCache, DataCacheError},
    dependency_graph::DependencyGraph,
    diagram::DiagramError,
    DalContext, SchemaVariantError, TransactionsError, WorkspaceSnapshotError,
};

#[derive(Debug, Error)]
pub enum MaterializedViewError {
    #[error("DataCache error: {0}")]
    DataCache(#[from] DataCacheError),
    #[error("Diagram error: {0}")]
    Diagram(#[from] DiagramError),
    #[error("Frigg error: {0}")]
    Frigg(#[from] FriggError),
    #[error("No index for incremental build for workspace {workspace_pk} and change set {change_set_id}")]
    NoIndexForIncrementalBuild {
        workspace_pk: WorkspacePk,
        change_set_id: ChangeSetId,
    },
    #[error("SchemaVariant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("serde_json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("Transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("WorkspaceSnapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

/// This function builds all Materialized Views (MVs) for the change set in the [`DalContext`].
/// It assumes there is no existing [`MvIndex`] for the change set.
#[instrument(
    name = "materialized_view.build_all_mv_for_change_set",
    level = "debug",
    skip_all,
    fields(
        si.workspace.id = Empty,
        si.change_set.id = %ctx.change_set_id(),
    ),
)]
pub async fn build_all_mv_for_change_set(
    ctx: &DalContext,
    frigg: &FriggStore,
) -> Result<(), MaterializedViewError> {
    let span = current_span_for_instrument_at!("debug");
    span.record("si.workspace.id", ctx.workspace_pk()?.to_string());

    // Pretend everything has changed, and build all MVs.
    let changes = ctx
        .workspace_snapshot()?
        .map_all_nodes_to_change_objects()
        .instrument(tracing::info_span!("Generating Changes for all nodes"))
        .await?;

    let (frontend_objects, patches) = build_mv_inner(
        ctx,
        frigg,
        ctx.workspace_pk()?,
        ctx.change_set_id(),
        &changes,
    )
    .instrument(tracing::info_span!("Building MVs"))
    .await?;

    let index_entries = frontend_objects.into_iter().map(Into::into).collect();
    let mv_index = MvIndex::new(ctx.change_set_id(), index_entries);
    let mv_index_frontend_object = FrontendObject::try_from(mv_index)?;
    frigg
        .insert_object(ctx.workspace_pk()?, &mv_index_frontend_object)
        .await?;

    let patch_batch = PatchBatch {
        meta: PatchBatchMeta {
            workspace_id: ctx.workspace_pk()?,
            change_set_id: Some(ctx.change_set_id()),
            snapshot_from_address: None,
            snapshot_to_address: Some(ctx.workspace_snapshot()?.id().await),
        },
        kind: PATCH_BATCH_KIND,
        patches,
    };

    // This will fail if the index has already been created by something else.
    if let Err(error) = frigg
        .insert_index(ctx.workspace_pk()?, &mv_index_frontend_object)
        .await
    {
        warn!(
            "Problem updating MvIndex pointer for change set {}, skipping patch batch publish: {:?}",
            ctx.change_set_id(),
            error,
        );
        return Ok(());
    }

    DataCache::publish_patch_batch(ctx, patch_batch)
        .instrument(tracing::info_span!("Publishing patch batch"))
        .await?;

    Ok(())
}

#[instrument(
    name = "materialized_view.build_mv_for_changes_in_change_set",
    level = "debug",
    skip_all,
    fields(
        si.workspace.id = Empty,
        si.change_set.id = %change_set_id,
        si.snapshot_from_address = %from_snapshot_address,
        si.snapshot_to_address = %to_snapshot_address,)
)]
pub async fn build_mv_for_changes_in_change_set(
    ctx: &DalContext,
    frigg: &FriggStore,
    change_set_id: ChangeSetId,
    from_snapshot_address: WorkspaceSnapshotAddress,
    to_snapshot_address: WorkspaceSnapshotAddress,
    changes: &[Change],
) -> Result<(), MaterializedViewError> {
    let workspace_pk = ctx.workspace_pk()?;

    let span = current_span_for_instrument_at!("debug");
    span.record("si.workspace.id", workspace_pk.to_string());

    let (index_frontend_object, index_kv_revision) = frigg
        .get_index(ctx.workspace_pk()?, change_set_id)
        .await?
        .ok_or_else(|| MaterializedViewError::NoIndexForIncrementalBuild {
            workspace_pk,
            change_set_id,
        })?;

    let (frontend_objects, patches) =
        build_mv_inner(ctx, frigg, workspace_pk, change_set_id, changes).await?;

    let mv_index: MvIndex = serde_json::from_value(index_frontend_object.data)?;
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

    let new_mv_index = MvIndex::new(change_set_id, index_entries);

    let new_mv_index_frontend_object = FrontendObject::try_from(new_mv_index)?;
    frigg
        .insert_object(workspace_pk, &new_mv_index_frontend_object)
        .await?;

    let patch_batch = PatchBatch {
        meta: PatchBatchMeta {
            workspace_id: workspace_pk,
            change_set_id: Some(change_set_id),
            snapshot_from_address: Some(from_snapshot_address),
            snapshot_to_address: Some(to_snapshot_address),
        },
        kind: PATCH_BATCH_KIND,
        patches,
    };

    frigg
        .update_index(
            workspace_pk,
            &new_mv_index_frontend_object,
            index_kv_revision,
        )
        .await?;

    DataCache::publish_patch_batch(ctx, patch_batch).await?;

    Ok(())
}

pub async fn build_mv_inner(
    ctx: &DalContext,
    frigg: &FriggStore,
    workspace_pk: si_id::WorkspacePk,
    change_set_id: ChangeSetId,
    changes: &[Change],
) -> Result<(Vec<FrontendObject>, Vec<FrontendObjectPatch>), MaterializedViewError> {
    let mut mv_dependency_graph = mv_dependency_graph()?;
    let mut frontend_objects = Vec::new();
    let mut patches = Vec::new();

    loop {
        let independent_mvs = mv_dependency_graph.independent_ids();
        if independent_mvs.is_empty() {
            break;
        }
        // We eagerly remove the independent MVs as we're going to bail out if anything errors,
        // instead of attempting to process the remaining MVs.
        for mv_kind in &independent_mvs {
            mv_dependency_graph.remove_id(*mv_kind);
        }

        for change in changes {
            for &mv_kind in &independent_mvs {
                match mv_kind {
                    ReferenceKind::View => {
                        let mv_id = change.entity_id.to_string();

                        let (maybe_patch, maybe_frontend_object) =
                            si_frontend_types_macros::build_mv!(
                                ctx,
                                frigg,
                                change,
                                mv_id,
                                si_frontend_types::view::View,
                                crate::diagram::view::View::as_frontend_type(
                                    ctx,
                                    si_events::ulid::Ulid::from(change.entity_id).into()
                                )
                                .await,
                            )
                            .await?;

                        if let Some(patch) = maybe_patch {
                            patches.push(patch);
                        }
                        if let Some(object) = maybe_frontend_object {
                            frontend_objects.push(object);
                        }
                    }
                    ReferenceKind::ViewList => {
                        let mv_id = change_set_id.to_string();

                        let (maybe_patch, maybe_frontend_object) =
                            si_frontend_types_macros::build_mv!(
                                ctx,
                                frigg,
                                change,
                                mv_id,
                                si_frontend_types::view::ViewList,
                                crate::diagram::view::View::as_frontend_list_type(ctx).await,
                            )
                            .await?;

                        if let Some(patch) = maybe_patch {
                            patches.push(patch);
                        }
                        if let Some(object) = maybe_frontend_object {
                            frontend_objects.push(object);
                        }
                    }
                    ReferenceKind::SchemaVariantCategories => {
                        let mv_id = change_set_id.to_string();

                        let (maybe_patch, maybe_frontend_object) =
                            si_frontend_types_macros::build_mv!(
                                ctx,
                                frigg,
                                change,
                                mv_id,
                                si_frontend_types::schema_variant::SchemaVariantCategories,
                                crate::schema::variant::SchemaVariant::as_frontend_list_type_by_category(
                                    ctx
                                )
                                .await,
                            )
                            .await?;

                        if let Some(patch) = maybe_patch {
                            patches.push(patch);
                        }
                        if let Some(object) = maybe_frontend_object {
                            frontend_objects.push(object);
                        }
                    }

                    // Building the `MvIndex` itself is handled separately as the logic depends
                    // on whether we're doing an incremental build or a full build from scratch.
                    ReferenceKind::MvIndex => continue,

                    // These `ReferenceKind` do not have associated `MaterializedView`s (yet?),
                    // so we skip them.
                    ReferenceKind::ChangeSetList | ReferenceKind::ChangeSetRecord => continue,
                }
            }
        }
    }

    frigg
        .insert_objects(workspace_pk, frontend_objects.iter())
        .await?;

    Ok((frontend_objects, patches))
}

macro_rules! add_reference_dependencies_to_dependency_graph {
    ($dependency_graph:expr, $mv_reference_kind:expr, $mv:ident $(,)?) => {
        for reference_kind in <$mv as MaterializedView>::reference_dependencies() {
            $dependency_graph.id_depends_on($mv_reference_kind, *reference_kind);
        }
    };
}

#[instrument(
    name = "materialized_view.mv_dependency_graph",
    level = "debug",
    skip_all
)]
fn mv_dependency_graph() -> Result<DependencyGraph<ReferenceKind>, MaterializedViewError> {
    let mut dependency_graph = DependencyGraph::new();

    // All `MaterializedView` types must be covered here for them to be built.
    add_reference_dependencies_to_dependency_graph!(dependency_graph, ReferenceKind::View, ViewMv);
    add_reference_dependencies_to_dependency_graph!(
        dependency_graph,
        ReferenceKind::ViewList,
        ViewListMv,
    );

    // The MvIndex depends on everything else, but doesn't define any
    // `MaterializedView::reference_dependencies()` directly.
    for reference_kind in ReferenceKind::iter() {
        // MvIndex can't depend on itself.
        if reference_kind == ReferenceKind::MvIndex {
            continue;
        }

        dependency_graph.id_depends_on(ReferenceKind::MvIndex, reference_kind);
    }

    Ok(dependency_graph)
}
