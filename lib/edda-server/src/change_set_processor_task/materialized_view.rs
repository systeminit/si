use std::collections::{
    HashMap,
    HashSet,
};

use dal::{
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
    data_cache::{
        DataCache,
        DataCacheError,
    },
    dependency_graph::DependencyGraph,
    diagram::DiagramError,
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
use si_frontend_types::{
    MaterializedView,
    action::{
        ActionPrototypeViewList as ActionPrototypeViewListMv,
        ActionViewList as ActionViewListMv,
    },
    index::MvIndex,
    newhotness::{
        attribute_tree::AttributeTree as AttributeTreeMv,
        component::{
            Component as ComponentMv,
            ComponentList as ComponentListMv,
        },
    },
    object::{
        FrontendObject,
        patch::{
            ObjectPatch as FrontendObjectPatch,
            PATCH_BATCH_KIND,
            PatchBatch,
            PatchBatchMeta,
        },
    },
    reference::{
        IndexReference,
        ReferenceKind,
    },
    schema_variant::SchemaVariantCategories as SchemaVariantCategoriesMv,
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
use strum::IntoEnumIterator;
use telemetry::prelude::*;
use telemetry_utils::metric;
use thiserror::Error;
use tokio::task::JoinSet;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum MaterializedViewError {
    #[error("Action error: {0}")]
    Action(#[from] ActionError),
    #[error("ActionPrototype error: {0}")]
    ActionPrototype(#[from] ActionPrototypeError),
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
    #[error("materialized views error: {0}")]
    MaterializedViews(#[from] dal_materialized_views::Error),
    #[error(
        "No index for incremental build for workspace {workspace_pk} and change set {change_set_id}"
    )]
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

    let index_entries = frontend_objects.into_iter().map(Into::into).collect();
    info!("index_entries {:?}", index_entries);
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
            snapshot_to_address: Some(ctx.workspace_snapshot()?.address().await),
        },
        kind: PATCH_BATCH_KIND,
        patches,
    };

    frigg
        .put_index(ctx.workspace_pk()?, &mv_index_frontend_object)
        .await?;

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
    let workspace_pk = ctx.workspace_pk()?;

    let span = current_span_for_instrument_at!("info");
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

macro_rules! spawn_build_mv_task {
    ($build_tasks:expr, $mv_task_ids: expr, $ctx:expr, $frigg:expr, $change:expr, $mv_id:expr, $mv:ty, $build_fn:expr $(,)?) => {
        let task_id = ::si_events::ulid::Ulid::new();
        let kind = <$mv as ::si_frontend_types::materialized_view::MaterializedView>::kind();
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
) -> Result<(Vec<FrontendObject>, Vec<FrontendObjectPatch>), MaterializedViewError> {
    let mut mv_dependency_graph = mv_dependency_graph()?;
    let mut frontend_objects = Vec::new();
    let mut patches = Vec::new();
    let mut build_tasks = JoinSet::new();
    let mut ready_to_start_mv_kinds: HashSet<ReferenceKind> =
        mv_dependency_graph.independent_ids().into_iter().collect();
    let mut mv_task_ids: HashMap<ReferenceKind, HashSet<Ulid>> = HashMap::new();

    loop {
        // If there aren't any ready to start MV kinds, and there aren't any currently running
        // MV build tasks, then we've finished everything we can and are able to send off the
        // collected FrontendObjects, and ObjectPatches to update the index & send patches out
        // over the websocket.
        if ready_to_start_mv_kinds.is_empty() && mv_task_ids.values().all(|tasks| tasks.is_empty())
        {
            break;
        }

        // If there aren't any ready_to start MV kinds, and there are currently building MVs,
        // then we don't need to bother looping through the changes only to check against an
        // empty list of ready to start MVs. We should jump straight to waiting for the next
        // building MV to finish.
        if !ready_to_start_mv_kinds.is_empty() {
            for &change in changes {
                for &mv_kind in &ready_to_start_mv_kinds {
                    match mv_kind {
                        ReferenceKind::ActionPrototypeViewList => {
                            let mv_id = change.entity_id.to_string();

                            let trigger_entity = <si_frontend_types::action::prototype::ActionPrototypeViewList as si_frontend_types::materialized_view::MaterializedView>::trigger_entity();
                            if change.entity_kind != trigger_entity {
                                continue;
                            }

                            spawn_build_mv_task!(
                                build_tasks,
                                mv_task_ids,
                                ctx,
                                frigg,
                                change,
                                mv_id,
                                si_frontend_types::action::prototype::ActionPrototypeViewList,
                                dal::action::prototype::ActionPrototype::as_frontend_list_type(
                                    ctx.clone(),
                                    si_events::ulid::Ulid::from(change.entity_id).into()
                                ),
                            );
                        }
                        ReferenceKind::ActionViewList => {
                            let mv_id = change_set_id.to_string();

                            let trigger_entity = <si_frontend_types::action::ActionViewList as si_frontend_types::materialized_view::MaterializedView>::trigger_entity();
                            if change.entity_kind != trigger_entity {
                                continue;
                            }

                            spawn_build_mv_task!(
                                build_tasks,
                                mv_task_ids,
                                ctx,
                                frigg,
                                change,
                                mv_id,
                                si_frontend_types::action::ActionViewList,
                                dal::action::Action::as_frontend_list_type(ctx.clone()),
                            );
                        }
                        ReferenceKind::AttributeTree => {
                            let mv_id = change.entity_id.to_string();

                            let trigger_entity = <si_frontend_types::newhotness::attribute_tree::AttributeTree as si_frontend_types::materialized_view::MaterializedView>::trigger_entity();
                            if change.entity_kind != trigger_entity {
                                continue;
                            }

                            spawn_build_mv_task!(
                                build_tasks,
                                mv_task_ids,
                                ctx,
                                frigg,
                                change,
                                mv_id,
                                si_frontend_types::newhotness::attribute_tree::AttributeTree,
                                dal_materialized_views::attribute_tree::as_frontend_type(
                                    ctx.clone(),
                                    si_events::ulid::Ulid::from(change.entity_id).into(),
                                ),
                            );
                        }
                        ReferenceKind::Component => {
                            let mv_id = change.entity_id.to_string();

                            let trigger_entity = <si_frontend_types::newhotness::component::Component as si_frontend_types::materialized_view::MaterializedView>::trigger_entity();
                            if change.entity_kind != trigger_entity {
                                continue;
                            }

                            spawn_build_mv_task!(
                                build_tasks,
                                mv_task_ids,
                                ctx,
                                frigg,
                                change,
                                mv_id,
                                si_frontend_types::newhotness::component::Component,
                                dal_materialized_views::component::as_frontend_type(
                                    ctx.clone(),
                                    si_events::ulid::Ulid::from(change.entity_id).into(),
                                ),
                            );
                        }
                        ReferenceKind::ComponentList => {
                            let mv_id = change.entity_id.to_string();

                            let trigger_entity = <si_frontend_types::newhotness::component::ComponentList as si_frontend_types::materialized_view::MaterializedView>::trigger_entity();
                            if change.entity_kind != trigger_entity {
                                continue;
                            }

                            spawn_build_mv_task!(
                                build_tasks,
                                mv_task_ids,
                                ctx,
                                frigg,
                                change,
                                mv_id,
                                si_frontend_types::newhotness::component::ComponentList,
                                dal_materialized_views::component::as_frontend_list_type(
                                    ctx.clone(),
                                ),
                            );
                        }
                        ReferenceKind::SchemaVariantCategories => {
                            let mv_id = change_set_id.to_string();

                            let trigger_entity = <si_frontend_types::schema_variant::SchemaVariantCategories as si_frontend_types::materialized_view::MaterializedView>::trigger_entity();
                            if change.entity_kind != trigger_entity {
                                continue;
                            }

                            spawn_build_mv_task!(
                                build_tasks,
                                mv_task_ids,
                                ctx,
                                frigg,
                                change,
                                mv_id,
                                si_frontend_types::schema_variant::SchemaVariantCategories,
                                dal::schema::variant::SchemaVariant::as_frontend_list_type_by_category(
                                    ctx.clone(),
                                )
                            );
                        }
                        ReferenceKind::View => {
                            let mv_id = change.entity_id.to_string();

                            let trigger_entity = <si_frontend_types::view::View as si_frontend_types::materialized_view::MaterializedView>::trigger_entity();
                            if change.entity_kind != trigger_entity {
                                continue;
                            }

                            spawn_build_mv_task!(
                                build_tasks,
                                mv_task_ids,
                                ctx,
                                frigg,
                                change,
                                mv_id,
                                si_frontend_types::view::View,
                                dal::diagram::view::View::as_frontend_type(
                                    ctx.clone(),
                                    si_events::ulid::Ulid::from(change.entity_id).into()
                                )
                            );
                        }
                        ReferenceKind::ViewList => {
                            let mv_id = change_set_id.to_string();

                            let trigger_entity = <si_frontend_types::view::ViewList as si_frontend_types::materialized_view::MaterializedView>::trigger_entity();
                            if change.entity_kind != trigger_entity {
                                continue;
                            }

                            spawn_build_mv_task!(
                                build_tasks,
                                mv_task_ids,
                                ctx,
                                frigg,
                                change,
                                mv_id,
                                si_frontend_types::view::ViewList,
                                dal::diagram::view::View::as_frontend_list_type(ctx.clone()),
                            );
                        }
                        ReferenceKind::ViewComponentList => {
                            let mv_id = change.entity_id.to_string();

                            let trigger_entity = <si_frontend_types::view::ViewComponentList as si_frontend_types::materialized_view::MaterializedView>::trigger_entity();
                            if change.entity_kind != trigger_entity {
                                continue;
                            }

                            spawn_build_mv_task!(
                                build_tasks,
                                mv_task_ids,
                                ctx,
                                frigg,
                                change,
                                mv_id,
                                si_frontend_types::view::ViewComponentList,
                                dal_materialized_views::view::components_as_frontend_list_type(
                                    ctx.clone(),
                                    si_events::ulid::Ulid::from(change.entity_id).into(),
                                ),
                            );
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

        // Now that these MV kinds have had their build tasks kicked off, we want to prevent kicking them off again
        // as MV build tasks finish, and we also don't want to free up the things that depend on them until _all_
        // of the started tasks for that MV kind have finished. Once the build tasks have finished, we'll remove the
        // MV kind from the graph, which will let the downstream MVs start.
        for &running_mv_kind in &ready_to_start_mv_kinds {
            mv_dependency_graph.cycle_on_self(running_mv_kind);
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
                info!(kind = %kind, "All MV build tasks finished for MV kind.");
                mv_dependency_graph.remove_id(kind);
            }

            match execution_result {
                Ok((maybe_patch, maybe_frontend_object)) => {
                    if let Some(patch) = maybe_patch {
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
    Result<
        (
            Option<si_frontend_types::object::patch::ObjectPatch>,
            Option<si_frontend_types::object::FrontendObject>,
        ),
        MaterializedViewError,
    >,
);

#[instrument(
    name = "materialized_view.build_mv_task",
    level = "info",
    skip_all,
    fields(
        si.mv.kind = %mv_kind,
        si.mv.id = %mv_id,
    )
)]
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
    T: serde::Serialize
        + TryInto<si_frontend_types::object::FrontendObject>
        + si_frontend_types::checksum::FrontendChecksum,
    E: Into<MaterializedViewError>,
    MaterializedViewError: From<E>,
    MaterializedViewError: From<<T as TryInto<FrontendObject>>::Error>,
{
    info!(kind = %mv_kind, id = %mv_id, "Building MV");
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

type MvBuilderResult = Result<
    (
        Option<si_frontend_types::object::patch::ObjectPatch>,
        Option<si_frontend_types::object::FrontendObject>,
    ),
    MaterializedViewError,
>;

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
    T: serde::Serialize
        + TryInto<si_frontend_types::object::FrontendObject>
        + si_frontend_types::checksum::FrontendChecksum,
    E: Into<MaterializedViewError>,
    MaterializedViewError: From<E>,
    MaterializedViewError: From<<T as TryInto<FrontendObject>>::Error>,
{
    let mv_kind = mv_kind.to_string();
    if !ctx
        .workspace_snapshot()?
        .node_exists(change.entity_id)
        .await
    {
        // Object was removed
        Ok((
            Some(si_frontend_types::object::patch::ObjectPatch {
                kind: mv_kind,
                id: mv_id,
                // TODO: we need to get the prior version of this
                from_checksum: Checksum::default().to_string(),
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
        let to_checksum = si_frontend_types::checksum::FrontendChecksum::checksum(&mv).to_string();
        let frontend_object: si_frontend_types::object::FrontendObject = mv.try_into()?;

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
            Ok((None, None))
        } else {
            Ok((
                Some(si_frontend_types::object::patch::ObjectPatch {
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

macro_rules! add_reference_dependencies_to_dependency_graph {
    ($dependency_graph:expr_2021, $mv:ident $(,)?) => {
        for reference_kind in <$mv as MaterializedView>::reference_dependencies() {
            $dependency_graph.id_depends_on(<$mv as MaterializedView>::kind(), *reference_kind);
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

    // TODO(nick): we should really look into making this automatic from "ReferenceKind::revision_sensitive()"
    // fields... too easy to shoot yourself in the foot.
    //
    // All `MaterializedView` types must be covered here for them to be built.
    add_reference_dependencies_to_dependency_graph!(dependency_graph, ViewMv);
    add_reference_dependencies_to_dependency_graph!(dependency_graph, ViewListMv);
    add_reference_dependencies_to_dependency_graph!(dependency_graph, ViewComponentListMv);
    add_reference_dependencies_to_dependency_graph!(dependency_graph, SchemaVariantCategoriesMv);
    add_reference_dependencies_to_dependency_graph!(dependency_graph, ActionViewListMv);
    add_reference_dependencies_to_dependency_graph!(dependency_graph, ActionPrototypeViewListMv);
    add_reference_dependencies_to_dependency_graph!(dependency_graph, ComponentListMv);
    add_reference_dependencies_to_dependency_graph!(dependency_graph, ComponentMv);
    add_reference_dependencies_to_dependency_graph!(dependency_graph, AttributeTreeMv);

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
