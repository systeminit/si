use std::collections::{
    HashMap,
    HashSet,
    VecDeque,
};

use dal::{
    ChangeSet,
    ChangeSetError,
    ComponentError,
    DalContext,
    SchemaVariantError,
    TransactionsError,
    Ulid,
    WorkspaceSnapshotError,
    action::{
        ActionError,
        prototype::ActionPrototypeError,
    },
    dependency_graph::DependencyGraph,
    diagram::DiagramError,
    prop::PropError,
    workspace_snapshot::WorkspaceSnapshotSelector,
};
use frigg::{
    FriggError,
    FriggStore,
};
use si_events::{
    WorkspaceSnapshotAddress,
    workspace_snapshot::{
        Change,
        Checksum,
    },
};
use si_frontend_mv_types::{
    MaterializedView,
    action::{
        ActionPrototypeViewList as ActionPrototypeViewListMv,
        ActionViewList as ActionViewListMv,
    },
    checksum::FrontendChecksum,
    component::{
        Component as ComponentMv,
        ComponentList as ComponentListMv,
    },
    incoming_connections::{
        IncomingConnections as IncomingConnectionsMv,
        IncomingConnectionsList as IncomingConnectionsListMv,
    },
    index::MvIndex,
    materialized_view::{
        MaterializedViewInventoryItem,
        materialized_view_definitions_checksum,
    },
    object::{
        FrontendObject,
        patch::{
            INDEX_UPDATE_KIND,
            IndexUpdate,
            ObjectPatch,
            PATCH_BATCH_KIND,
            PatchBatch,
            UpdateMeta,
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
use strum::IntoEnumIterator;
use telemetry::prelude::*;
use telemetry_utils::metric;
use thiserror::Error;
use tokio::task::JoinSet;

use crate::data_cache::{
    DataCache,
    DataCacheError,
};

/// Limit for how many spawned MV build tasks can exist before
/// waiting for existing tasks to finish before spawning another one.
const PARALLEL_BUILD_LIMIT: usize = 50;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum MaterializedViewError {
    #[error("Action error: {0}")]
    Action(#[from] ActionError),
    #[error("ActionPrototype error: {0}")]
    ActionPrototype(#[from] ActionPrototypeError),
    #[error("Change Set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("Component error: {0}")]
    Component(#[from] ComponentError),
    #[error("DataCache error: {0}")]
    DataCache(#[from] DataCacheError),
    #[error("Diagram error: {0}")]
    Diagram(#[from] DiagramError),
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
/// and looks for an [`MvIndex`] it can use. If it finds one, create a new index pointer to
/// the found [`MvIndex`] and return true. If none can be used, return false.
/// NOTE: The copy will fail if we try to reuse an index for a change set that already has an index pointer
#[instrument(
    name = "materialized_view.try_reuse_index_for_snapshot",
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
        let Some((pointer, _revision)) = frigg
            .get_index_pointer_value(workspace_id, change_set.id)
            .await?
        else {
            // try the next one
            // no need error if this index was never built, it would get rebuilt when necessary
            continue;
        };

        if pointer.snapshot_address == snapshot_address.to_string()
            && pointer.definition_checksum == definitions_checksum
        {
            // found one, create a new index pointer to it!
            // Note: we might want to consider validting the index here before we use it
            let change_set_mv_id = change_set_id.to_string();
            frigg
                .insert_index_key_for_existing_index(workspace_id, &change_set_mv_id, pointer)
                .await?;
            span.record("si.from_change_set.id", change_set_mv_id);
            let meta = UpdateMeta {
                workspace_id,
                change_set_id: Some(change_set_id),
                snapshot_from_address: None,
                snapshot_to_address: Some(snapshot_address),
            };
            let index_update = IndexUpdate {
                meta,
                kind: INDEX_UPDATE_KIND,
            };
            DataCache::publish_index_update(ctx, index_update).await?;
            return Ok(true);
        }
    }
    Ok(false)
}

/// This function first tries to copy and existing [`MvIndex`] if we find a valid one with the same snapshot address
/// If it cannot copy one, it builds all Materialized Views (MVs) for the change set in the [`DalContext`].
/// It assumes there is no existing [`MvIndex`] for the change set.
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
    to_snapshot_address: WorkspaceSnapshotAddress,
) -> Result<(), MaterializedViewError> {
    let span = current_span_for_instrument_at!("info");
    span.record("si.workspace.id", ctx.workspace_pk()?.to_string());
    let did_copy_result =
        self::try_reuse_mv_index_for_new_change_set(ctx, frigg, to_snapshot_address)
            .instrument(tracing::info_span!(
                "edda.change_set_processor_task.try_clone_index_from_snapshot"
            ))
            .await;

    match did_copy_result {
        // If we returned successfully, evaluate the response,
        Ok(did_copy) => {
            if did_copy {
                return Ok(());
            } else {
                // we did not copy anything, so we must rebuild from scratch (no from_snapshot_address this time)
                self::build_all_mv_for_change_set(ctx, frigg, None)
                    .instrument(tracing::info_span!(
                        "edda.change_set_processor_task.build_all_mv_for_change_set.initial_build"
                    ))
                    .await?
            }
        }
        Err(err) => {
            error!(si.error.message = ?err, "error copying existing index");
            // we did not copy anything, so we must rebuild from scratch (no from_snapshot_address this time)
            self::build_all_mv_for_change_set(ctx, frigg, None)
                .instrument(tracing::info_span!(
                    "edda.change_set_processor_task.build_all_mv_for_change_set.initial_build"
                ))
                .await?
        }
    }
    Ok(())
}

/// This function builds all Materialized Views (MVs) for the change set in the [`DalContext`].
/// It assumes there is no existing [`MvIndex`] for the change set.
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
    ),
)]
pub async fn build_all_mv_for_change_set(
    ctx: &DalContext,
    frigg: &FriggStore,
    snapshot_from_address: Option<WorkspaceSnapshotAddress>,
) -> Result<(), MaterializedViewError> {
    let span = current_span_for_instrument_at!("info");
    span.record("si.workspace.id", ctx.workspace_pk()?.to_string());

    // Pretend everything has changed, and build all MVs.
    let changes = map_all_nodes_to_change_objects(&ctx.workspace_snapshot()?)
        .instrument(tracing::info_span!(
            "materialized_view.build_all_mv_for_change_set.make_changes_for_all_nodes"
        ))
        .await?;

    let (frontend_objects, patches) = build_mv_inner(
        ctx,
        frigg,
        ctx.workspace_pk()?,
        ctx.change_set_id(),
        &changes,
    )
    .await?;

    let mut index_entries: Vec<_> = frontend_objects.into_iter().map(Into::into).collect();
    index_entries.sort();
    let snapshot_to_address = ctx.workspace_snapshot()?.address().await;
    let workspace_id = ctx.workspace_pk()?;
    let change_set_id = ctx.change_set_id();
    debug!("index_entries {:?}", index_entries);
    let mv_index = MvIndex::new(snapshot_to_address.to_string(), index_entries);
    let mv_index_frontend_object = FrontendObject::try_from(mv_index)?;
    let meta = UpdateMeta {
        workspace_id,
        change_set_id: Some(change_set_id),
        snapshot_from_address: snapshot_from_address.or(Some(snapshot_to_address)),
        snapshot_to_address: Some(snapshot_to_address),
    };
    let patch_batch = PatchBatch {
        meta,
        kind: PATCH_BATCH_KIND,
        patches,
    };
    let change_set_mv_id = change_set_id.to_string();
    let index_update = IndexUpdate {
        meta,
        kind: INDEX_UPDATE_KIND,
    };

    frigg
        .put_index(
            ctx.workspace_pk()?,
            &change_set_mv_id,
            &mv_index_frontend_object,
        )
        .await?;
    DataCache::publish_index_update(ctx, index_update).await?;
    DataCache::publish_patch_batch(ctx, patch_batch).await?;

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
        si.edda_request.changes = Empty,
    )
)]
pub async fn build_mv_for_changes_in_change_set(
    ctx: &DalContext,
    frigg: &FriggStore,
    change_set_id: ChangeSetId,
    from_snapshot_address: WorkspaceSnapshotAddress,
    to_snapshot_address: WorkspaceSnapshotAddress,
    changes: &[Change],
) -> Result<(), MaterializedViewError> {
    let workspace_id = ctx.workspace_pk()?;
    debug!("building for changes: {:?}", changes);
    let span = current_span_for_instrument_at!("info");
    span.record("si.workspace.id", workspace_id.to_string());
    span.record("si.edda_request.changes", format!("{:?}", changes));
    let (index_frontend_object, index_kv_revision) = frigg
        .get_index(ctx.workspace_pk()?, change_set_id)
        .await?
        .ok_or_else(|| MaterializedViewError::NoIndexForIncrementalBuild {
            workspace_pk: workspace_id,
            change_set_id,
        })?;

    let (frontend_objects, patches) =
        build_mv_inner(ctx, frigg, workspace_id, change_set_id, changes).await?;
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

    let new_mv_index = MvIndex::new(to_snapshot_address.to_string(), index_entries);

    let new_mv_index_frontend_object = FrontendObject::try_from(new_mv_index)?;
    let meta = UpdateMeta {
        workspace_id,
        change_set_id: Some(change_set_id),
        snapshot_from_address: Some(from_snapshot_address),
        snapshot_to_address: Some(to_snapshot_address),
    };
    let patch_batch = PatchBatch {
        meta,
        kind: PATCH_BATCH_KIND,
        patches,
    };
    let index_update = IndexUpdate {
        meta,
        kind: INDEX_UPDATE_KIND,
    };
    let change_set_mv_id = change_set_id.to_string();

    frigg
        .update_index(
            workspace_id,
            &change_set_mv_id,
            &new_mv_index_frontend_object,
            index_kv_revision,
        )
        .await?;
    DataCache::publish_index_update(ctx, index_update).await?;

    DataCache::publish_patch_batch(ctx, patch_batch).await?;

    Ok(())
}

macro_rules! spawn_build_mv_task {
    ($build_tasks:expr, $mv_task_ids: expr, $ctx:expr, $frigg:expr, $change:expr, $mv_id:expr, $mv:ty, $build_fn:expr $(,)?) => {
        let task_id = ::si_events::ulid::Ulid::new();
        let kind = <$mv as ::si_frontend_mv_types::materialized_view::MaterializedView>::kind();
        // Record the task ID of the MV build task we're about to spawn so we can track when all of the build
        // tasks for any given MV kind have finished.
        $mv_task_ids
            .entry(kind)
            .and_modify(|task_ids| {
                task_ids.insert(task_id);
            })
            .or_insert_with(|| [task_id].iter().copied().collect());
        // Record the currently running number of MV build tasks so we can see just how parallel this actually
        // ends up being.
        metric!(counter.edda.mv_build = 1);
        $build_tasks.spawn(build_mv_task(
            task_id,
            $ctx.clone(),
            $frigg.clone(),
            $change,
            $mv_id,
            kind,
            $build_fn,
        ));
    };
}

#[instrument(
    name = "materialized_view.build_mv_inner",
    level = "info",
    skip_all,
    fields(
        si.workspace.id = %workspace_pk,
        si.change_set_id = %change_set_id,
    ),
)]
async fn build_mv_inner(
    ctx: &DalContext,
    frigg: &FriggStore,
    workspace_pk: si_id::WorkspacePk,
    change_set_id: ChangeSetId,
    changes: &[Change],
) -> Result<(Vec<FrontendObject>, Vec<ObjectPatch>), MaterializedViewError> {
    let mut mv_dependency_graph = mv_dependency_graph()?;
    let mut frontend_objects = Vec::new();
    let mut patches = Vec::new();
    let mut build_tasks = JoinSet::new();
    let mut ready_to_start_mv_kinds: HashSet<ReferenceKind> =
        mv_dependency_graph.independent_ids().into_iter().collect();
    let mut mv_task_ids: HashMap<ReferenceKind, HashSet<Ulid>> = HashMap::new();
    let mut queued_mv_builds = VecDeque::new();
    loop {
        // If there aren't any ready to start MV kinds, there aren't any builds queued waiting
        // for the concurrency limit, and there aren't any currently running MV build tasks,
        // then we've finished everything we can and are able to send off the collected
        // FrontendObjects, and ObjectPatches to update the index & send patches out over the
        // websocket.
        if ready_to_start_mv_kinds.is_empty()
            && queued_mv_builds.is_empty()
            && mv_task_ids.values().all(|tasks| tasks.is_empty())
        {
            break;
        }

        // Spawn as many of the queued build tasks as we can, up to the concurrency limit.
        while !queued_mv_builds.is_empty() && build_tasks.len() < PARALLEL_BUILD_LIMIT {
            let Some(QueuedBuildMvTask { change, mv_kind }) = queued_mv_builds.pop_front() else {
                // This _really_ shouldn't ever return `None` as we just checked that
                // `queued_mv_builds` is not empty.
                break;
            };
            // This _really_ shouldn't ever return `Some`, but better to be paranoid than to
            // forget about pending work.
            if let Some(queued_build) = spawn_build_mv_task_for_change_and_mv_kind(
                &mut build_tasks,
                &mut mv_task_ids,
                ctx,
                frigg,
                change,
                mv_kind,
                change_set_id,
            )
            .await?
            {
                queued_mv_builds.push_back(queued_build);
                break;
            }
        }

        // If there aren't any ready_to start MV kinds, and there are currently building MVs,
        // or if there are queued MV builds waiting on the concurrency limit, then we don't
        // need to bother looping through the changes only to check against an empty list of
        // ready to start MVs, or to grow the queue even more. We should jump straight to
        // waiting for the next building MV to finish.
        if !ready_to_start_mv_kinds.is_empty() && queued_mv_builds.is_empty() {
            for &change in changes {
                for &mv_kind in &ready_to_start_mv_kinds {
                    if let Some(queued_build) = spawn_build_mv_task_for_change_and_mv_kind(
                        &mut build_tasks,
                        &mut mv_task_ids,
                        ctx,
                        frigg,
                        change,
                        mv_kind,
                        change_set_id,
                    )
                    .await?
                    {
                        queued_mv_builds.push_back(queued_build);
                    }
                }
            }

            // Now that these MV kinds have had their build tasks kicked off, we want to prevent kicking them off again
            // as MV build tasks finish, and we also don't want to free up the things that depend on them until _all_
            // of the started tasks for that MV kind have finished. Once the build tasks have finished, we'll remove the
            // MV kind from the graph, which will let the downstream MVs start.

            // We need to track which MV kinds actually started, as if an MV kind was ready, but no changes came in that
            // the MV kind cares about, we should remove it from the graph now.
            for &running_mv_kind in &ready_to_start_mv_kinds {
                // if the task has been kicked off, or it's queued, let's cycle on self.
                if mv_task_ids
                    .clone()
                    .into_keys()
                    .any(|build| build == running_mv_kind)
                    || queued_mv_builds
                        .iter()
                        .any(|build| build.mv_kind == running_mv_kind)
                {
                    mv_dependency_graph.cycle_on_self(running_mv_kind);
                }
                // otherwise, no work to do here, let's remove it and free up the downstream MV kinds
                else {
                    mv_dependency_graph.remove_id(running_mv_kind);
                }
            }
        }

        if let Some(join_result) = build_tasks.join_next().await {
            let (kind, mv_id, task_id, execution_result) = join_result?;
            metric!(counter.edda.mv_build = -1);

            let std::collections::hash_map::Entry::Occupied(mut mv_kind_task_ids) =
                mv_task_ids.entry(kind)
            else {
                error!(
                    id = %mv_id,
                    kind = %kind,
                    si.error.message = "Got a build task that finished for a kind we didn't know had started any builds.",
                );
                continue;
            };
            mv_kind_task_ids.get_mut().remove(&task_id);
            // If there are no more running tasks for this MV kind, then we can remove it from the dependency
            // graph to free up the next batch of MVs to run.
            if mv_kind_task_ids.get().is_empty() {
                mv_dependency_graph.remove_id(kind);
            }

            match execution_result {
                Ok((maybe_patch, maybe_frontend_object)) => {
                    if let Some(patch) = maybe_patch {
                        debug!("Patch!: {:?}", patch);
                        patches.push(patch);
                    }
                    if let Some(frontend_object) = maybe_frontend_object {
                        frontend_objects.push(frontend_object);
                    }
                }
                Err(err) => {
                    warn!(name = "mv_build_error", si.error.message = err.to_string(), kind = %kind, id = %mv_id);
                }
            }
        }
        ready_to_start_mv_kinds = mv_dependency_graph.independent_ids().into_iter().collect();
    }

    frigg
        .insert_objects(workspace_pk, frontend_objects.iter())
        .await?;

    Ok((frontend_objects, patches))
}

type BuildMvTaskResult = (
    ReferenceKind,
    String,
    Ulid,
    Result<(Option<ObjectPatch>, Option<FrontendObject>), MaterializedViewError>,
);

async fn build_mv_task<F, T, E>(
    task_id: Ulid,
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
    let result = build_mv_task_inner(
        &ctx,
        &frigg,
        change,
        mv_id.clone(),
        mv_kind,
        build_mv_future,
    )
    .await;

    (mv_kind, mv_id, task_id, result)
}

type MvBuilderResult = Result<(Option<ObjectPatch>, Option<FrontendObject>), MaterializedViewError>;

async fn build_mv_task_inner<F, T, E>(
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
    let mv_kind:String = mv_kind.to_string();
     let (from_checksum, _previous_data) = if let Some(previous_version) = frigg
            .get_current_object(ctx.workspace_pk()?, ctx.change_set_id(), &mv_kind, &mv_id)
            .await?
        {
            (previous_version.checksum, previous_version.data)
        } else {
            // Object doesn't exist in Frigg either
            ("0".to_string(), serde_json::Value::Null)
        };
    if !ctx
        .workspace_snapshot()?
        .node_exists(change.entity_id)
        .await
    {
        // Object was removed
        Ok((
            Some(ObjectPatch {
                kind: mv_kind,
                id: mv_id,
                from_checksum,
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

        let kind = mv_kind;
        let (from_checksum, previous_data) = if let Some(previous_version) = frigg
            .get_current_object(ctx.workspace_pk()?, ctx.change_set_id(), &kind, &mv_id)
            .await?
        {
            (previous_version.checksum, previous_version.data)
        } else {
            // Object is new
            ("0".to_string(), serde_json::Value::Null)
        };

        if from_checksum == to_checksum {
            Ok((None, Some(frontend_object)))
        } else {
            Ok((
                Some(ObjectPatch {
                    kind,
                    id: mv_id,
                    from_checksum,
                    to_checksum,
                    patch: json_patch::diff(&previous_data, &mv_json),
                }),
                Some(frontend_object),
            ))
        }
    }
}

#[instrument(
    name = "materialized_view.mv_dependency_graph",
    level = "debug",
    skip_all
)]
fn mv_dependency_graph() -> Result<DependencyGraph<ReferenceKind>, MaterializedViewError> {
    let mut dependency_graph = DependencyGraph::new();

    for mv in ::inventory::iter::<MaterializedViewInventoryItem>() {
        for reference_kind in mv.reference_dependencies() {
            dependency_graph.id_depends_on(mv.kind(), *reference_kind);
        }
    }

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

/// The [`Change`] of the trigger entity that would have spawned a build task for the
/// `mv_kind`, if we hadn't already reached the concurrency limit for running build
/// tasks.
#[derive(Debug)]
struct QueuedBuildMvTask {
    pub change: Change,
    pub mv_kind: ReferenceKind,
}

async fn spawn_build_mv_task_for_change_and_mv_kind(
    build_tasks: &mut JoinSet<BuildMvTaskResult>,
    mv_task_ids: &mut HashMap<ReferenceKind, HashSet<Ulid>>,
    ctx: &DalContext,
    frigg: &FriggStore,
    change: Change,
    mv_kind: ReferenceKind,
    change_set_id: ChangeSetId,
) -> Result<Option<QueuedBuildMvTask>, MaterializedViewError> {
    match mv_kind {
        ReferenceKind::ActionPrototypeViewList => {
            let entity_mv_id = change.entity_id.to_string();

            let trigger_entity = <ActionPrototypeViewListMv as MaterializedView>::trigger_entity();
            if change.entity_kind != trigger_entity {
                return Ok(None);
            }

            if build_tasks.len() < PARALLEL_BUILD_LIMIT {
                spawn_build_mv_task!(
                    build_tasks,
                    mv_task_ids,
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
            } else {
                return Ok(Some(QueuedBuildMvTask { change, mv_kind }));
            }
        }
        ReferenceKind::ActionViewList => {
            let change_set_mv_id = change_set_id.to_string();

            let trigger_entity = <ActionViewListMv as MaterializedView>::trigger_entity();
            if change.entity_kind != trigger_entity {
                return Ok(None);
            }

            if build_tasks.len() < PARALLEL_BUILD_LIMIT {
                spawn_build_mv_task!(
                    build_tasks,
                    mv_task_ids,
                    ctx,
                    frigg,
                    change,
                    change_set_mv_id,
                    ActionViewListMv,
                    dal_materialized_views::action_view_list::assemble(ctx.clone()),
                );
            } else {
                return Ok(Some(QueuedBuildMvTask { change, mv_kind }));
            }
        }
        ReferenceKind::Component => {
            let entity_mv_id = change.entity_id.to_string();

            let trigger_entity = <ComponentMv as MaterializedView>::trigger_entity();
            if change.entity_kind != trigger_entity {
                return Ok(None);
            }

            if build_tasks.len() < PARALLEL_BUILD_LIMIT {
                spawn_build_mv_task!(
                    build_tasks,
                    mv_task_ids,
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
            } else {
                return Ok(Some(QueuedBuildMvTask { change, mv_kind }));
            }
        }
        ReferenceKind::ComponentList => {
            let change_set_mv_id = change_set_id.to_string();

            let trigger_entity = <ComponentListMv as MaterializedView>::trigger_entity();
            if change.entity_kind != trigger_entity {
                return Ok(None);
            }

            if build_tasks.len() < PARALLEL_BUILD_LIMIT {
                spawn_build_mv_task!(
                    build_tasks,
                    mv_task_ids,
                    ctx,
                    frigg,
                    change,
                    change_set_mv_id,
                    ComponentListMv,
                    dal_materialized_views::component_list::assemble(ctx.clone(),),
                );
            } else {
                return Ok(Some(QueuedBuildMvTask { change, mv_kind }));
            }
        }
        ReferenceKind::IncomingConnections => {
            let entity_mv_id = change.entity_id.to_string();

            let trigger_entity = <IncomingConnectionsMv as MaterializedView>::trigger_entity();
            if change.entity_kind != trigger_entity {
                return Ok(None);
            }

            if build_tasks.len() < PARALLEL_BUILD_LIMIT {
                spawn_build_mv_task!(
                    build_tasks,
                    mv_task_ids,
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
            } else {
                return Ok(Some(QueuedBuildMvTask { change, mv_kind }));
            }
        }
        ReferenceKind::IncomingConnectionsList => {
            let change_set_mv_id = change_set_id.to_string();

            let trigger_entity = <IncomingConnectionsListMv as MaterializedView>::trigger_entity();
            if change.entity_kind != trigger_entity {
                return Ok(None);
            }

            if build_tasks.len() < PARALLEL_BUILD_LIMIT {
                spawn_build_mv_task!(
                    build_tasks,
                    mv_task_ids,
                    ctx,
                    frigg,
                    change,
                    change_set_mv_id,
                    IncomingConnectionsListMv,
                    dal_materialized_views::incoming_connections_list::assemble(ctx.clone()),
                );
            } else {
                return Ok(Some(QueuedBuildMvTask { change, mv_kind }));
            }
        }
        ReferenceKind::SchemaVariant => {
            let entity_mv_id = change.entity_id.to_string();

            let trigger_entity = <SchemaVariantMv as MaterializedView>::trigger_entity();
            if change.entity_kind != trigger_entity {
                return Ok(None);
            }

            if build_tasks.len() < PARALLEL_BUILD_LIMIT {
                spawn_build_mv_task!(
                    build_tasks,
                    mv_task_ids,
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
            } else {
                return Ok(Some(QueuedBuildMvTask { change, mv_kind }));
            }
        }
        ReferenceKind::SchemaVariantCategories => {
            let change_set_mv_id = change_set_id.to_string();

            let trigger_entity = <SchemaVariantCategoriesMv as MaterializedView>::trigger_entity();
            if change.entity_kind != trigger_entity {
                return Ok(None);
            }

            if build_tasks.len() < PARALLEL_BUILD_LIMIT {
                spawn_build_mv_task!(
                    build_tasks,
                    mv_task_ids,
                    ctx,
                    frigg,
                    change,
                    change_set_mv_id,
                    SchemaVariantCategoriesMv,
                    dal_materialized_views::schema_variant_categories::assemble(ctx.clone()),
                );
            } else {
                return Ok(Some(QueuedBuildMvTask { change, mv_kind }));
            }
        }
        ReferenceKind::View => {
            let entity_mv_id = change.entity_id.to_string();

            let trigger_entity = <ViewMv as MaterializedView>::trigger_entity();
            if change.entity_kind != trigger_entity {
                return Ok(None);
            }

            if build_tasks.len() < PARALLEL_BUILD_LIMIT {
                spawn_build_mv_task!(
                    build_tasks,
                    mv_task_ids,
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
            } else {
                return Ok(Some(QueuedBuildMvTask { change, mv_kind }));
            }
        }
        ReferenceKind::ViewList => {
            let change_set_mv_id = change_set_id.to_string();

            let trigger_entity = <ViewListMv as MaterializedView>::trigger_entity();
            if change.entity_kind != trigger_entity {
                return Ok(None);
            }

            if build_tasks.len() < PARALLEL_BUILD_LIMIT {
                spawn_build_mv_task!(
                    build_tasks,
                    mv_task_ids,
                    ctx,
                    frigg,
                    change,
                    change_set_mv_id,
                    ViewListMv,
                    dal_materialized_views::view_list::assemble(ctx.clone()),
                );
            } else {
                return Ok(Some(QueuedBuildMvTask { change, mv_kind }));
            }
        }
        ReferenceKind::ViewComponentList => {
            let entity_mv_id = change.entity_id.to_string();

            let trigger_entity = <ViewComponentListMv as MaterializedView>::trigger_entity();
            if change.entity_kind != trigger_entity {
                return Ok(None);
            }

            if build_tasks.len() < PARALLEL_BUILD_LIMIT {
                spawn_build_mv_task!(
                    build_tasks,
                    mv_task_ids,
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
            } else {
                return Ok(Some(QueuedBuildMvTask { change, mv_kind }));
            }
        }

        // Building the `MvIndex` itself is handled separately as the logic depends
        // on whether we're doing an incremental build or a full build from scratch.
        ReferenceKind::MvIndex => {}

        // These `ReferenceKind` do not have associated `MaterializedView`s (yet?),
        // so we skip them.
        ReferenceKind::ChangeSetList | ReferenceKind::ChangeSetRecord => {}
    }

    Ok(None)
}
