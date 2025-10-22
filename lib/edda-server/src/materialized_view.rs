use std::{
    collections::BinaryHeap,
    future::Future,
    sync::Arc,
    time::Duration,
};

use dal::{
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
    slow_rt::{
        self,
        SlowRuntimeError,
    },
};
use frigg::{
    FriggError,
    FriggStore,
};
use serde_json::Value;
use si_events::{
    materialized_view::BuildPriority,
    workspace_snapshot::{
        Change,
        EntityKind,
    },
};
use si_frontend_mv_types::{
    action::{
        ActionPrototypeViewList as ActionPrototypeViewListMv,
        ActionViewList as ActionViewListMv,
        action_diff_list::ActionDiffList as ActionDiffListMv,
    },
    checksum::FrontendChecksum,
    component::{
        Component as ComponentMv,
        ComponentInList as ComponentInListMv,
        ComponentList as ComponentListMv,
        SchemaMembers,
        attribute_tree::AttributeTree as AttributeTreeMv,
        component_diff::ComponentDiff as ComponentDiffMv,
        erased_components::ErasedComponents as ErasedComponentsMv,
    },
    dependent_values::DependentValueComponentList as DependentValueComponentListMv,
    incoming_connections::{
        IncomingConnections as IncomingConnectionsMv,
        IncomingConnectionsList as IncomingConnectionsListMv,
        ManagementConnections as ManagementConnectionsMv,
    },
    materialized_view::MaterializedViewInventoryItem,
    object::{
        FrontendObject,
        patch::{
            ObjectPatch,
            StreamingPatch,
        },
    },
    reference::ReferenceKind,
    schema_variant::SchemaVariant as SchemaVariantMv,
    view::{
        View as ViewMv,
        ViewComponentList as ViewComponentListMv,
        ViewList as ViewListMv,
    },
};
use si_id::{
    ChangeSetId,
    EntityId,
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

pub mod change_set;
pub mod deployment;

// Re-export public functions from submodules
pub use change_set::{
    build_all_mv_for_change_set,
    build_mv_for_changes_in_change_set,
    try_reuse_mv_index_for_new_change_set,
};
pub use deployment::{
    build_all_mvs_for_deployment,
    build_outdated_mvs_for_deployment,
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
    #[error("Schema error: {0}")]
    Schema(#[from] dal::SchemaError),
    #[error("SchemaVariant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("serde_json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("slow runtime error: {0}")]
    SlowRuntime(#[from] SlowRuntimeError),
    #[error("Transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("WorkspaceSnapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

#[macro_export]
macro_rules! spawn_build_mv_task {
    ($build_tasks:expr, $ctx:expr, $frigg:expr, $change:expr, $mv_id:expr, $mv:ty, $build_fn:expr, $maybe_mv_index:expr $(,)?) => {
        let kind = <$mv as ::si_frontend_mv_types::materialized_view::MaterializedView>::kind();
        $build_tasks.spawn($crate::materialized_view::build_mv_for_graph_task(
            $ctx.clone(),
            $frigg.clone(),
            $change,
            $mv_id,
            kind,
            $build_fn,
            $maybe_mv_index,
        ));
    };
}

pub type BuildMvInnerReturn = (
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
pub async fn build_mv_inner(
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

    let maybe_mv_index = Arc::new(
        frigg
            .get_change_set_index(workspace_pk, change_set_id)
            .await?
            .map(|r| r.0),
    );

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
        // If there aren't any queued builds waiting for the concurrency limit then
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
                change_set_id,
                maybe_mv_index.clone(),
            )
            .await?
        }

        if let Some(join_result) = build_tasks.join_next().await {
            let (kind, mv_id, build_duration, entity_id, entity_kind, execution_result) =
                join_result?;
            metric!(
                counter.edda.mv_build = -1,
                label = format!("{workspace_pk}:{change_set_id}:{kind}")
            );

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
                    warn!(name = "mv_build_error", si.error.message = err.to_string(), kind = %kind.to_string(), id = %mv_id, entity_id = %entity_id, entity_kind = %entity_kind, change_set_id = %change_set_id, workspace_id = %workspace_pk);
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

pub type BuildMvTaskResult = (
    ReferenceKind,
    String,
    Duration,
    EntityId,
    EntityKind,
    Result<(Option<ObjectPatch>, Option<FrontendObject>), MaterializedViewError>,
);

pub type DeploymentBuildMvTaskResult = (
    ReferenceKind,
    String,
    Duration,
    Result<(Option<ObjectPatch>, Option<FrontendObject>), MaterializedViewError>,
);

pub async fn build_mv_for_graph_task<F, T, E>(
    ctx: DalContext,
    frigg: FriggStore,
    change: Change,
    mv_id: String,
    mv_kind: ReferenceKind,
    build_mv_future: F,
    maybe_mv_index: Option<FrontendObject>,
) -> BuildMvTaskResult
where
    F: Future<Output = Result<T, E>> + Send + 'static,
    T: serde::Serialize + TryInto<FrontendObject> + FrontendChecksum + Send + 'static,
    E: Into<MaterializedViewError> + Send + 'static,
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
        maybe_mv_index,
    )
    .await;

    (
        mv_kind,
        mv_id,
        start.elapsed(),
        change.entity_id,
        change.entity_kind,
        result,
    )
}

pub type MvBuilderResult =
    Result<(Option<ObjectPatch>, Option<FrontendObject>), MaterializedViewError>;

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
    maybe_mv_index: Option<FrontendObject>,
) -> MvBuilderResult
where
    F: Future<Output = Result<T, E>> + Send + 'static,
    T: serde::Serialize + TryInto<FrontendObject> + FrontendChecksum + Send + 'static,
    E: Into<MaterializedViewError> + Send + 'static,
    MaterializedViewError: From<E>,
    MaterializedViewError: From<<T as TryInto<FrontendObject>>::Error>,
{
    let op = {
        let maybe_previous_version = frigg
            .get_current_workspace_object_with_index(
                ctx.workspace_pk()?,
                ctx.change_set_id(),
                &mv_kind.to_string(),
                &mv_id,
                maybe_mv_index,
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

#[instrument(
    name = "edda.spawn_build_mv_task_for_change_and_mv_kind",
    level = "debug",
    skip_all,
    fields(
        otel.name = Empty,
        otel.status_code = Empty,
        otel.status_message = Empty,
        si.workspace.id = %workspace_pk,
        si.change_set.id = %change_set_id_for_metrics_only,
        si.edda.kind = %mv_kind,
    )
)]
#[allow(clippy::too_many_arguments)]
pub async fn spawn_build_mv_task_for_change_and_mv_kind(
    build_tasks: &mut JoinSet<BuildMvTaskResult>,
    ctx: &DalContext,
    frigg: &FriggStore,
    change: Change,
    mv_kind: ReferenceKind,
    workspace_pk: si_id::WorkspacePk,
    change_set_id_for_metrics_only: ChangeSetId,
    maybe_mv_index: Arc<Option<FrontendObject>>,
) -> Result<(), MaterializedViewError> {
    match mv_kind {
        ReferenceKind::ActionDiffList => {
            let workspace_mv_id = workspace_pk.to_string();
            metric!(
                counter.edda.mv_build = 1,
                label = format!("{workspace_pk}:{change_set_id_for_metrics_only}:{mv_kind}")
            );
            spawn_build_mv_task!(
                build_tasks,
                ctx,
                frigg,
                change,
                workspace_mv_id,
                ActionDiffListMv,
                dal_materialized_views::action::action_diff_list::assemble(ctx.clone()),
                (*maybe_mv_index).clone(),
            );
        }
        ReferenceKind::ActionPrototypeViewList => {
            let entity_mv_id = change.entity_id.to_string();
            metric!(
                counter.edda.mv_build = 1,
                label = format!("{workspace_pk}:{change_set_id_for_metrics_only}:{mv_kind}")
            );
            spawn_build_mv_task!(
                build_tasks,
                ctx,
                frigg,
                change,
                entity_mv_id,
                ActionPrototypeViewListMv,
                dal_materialized_views::action::action_prototype_view_list::assemble(
                    ctx.clone(),
                    si_events::ulid::Ulid::from(change.entity_id).into()
                ),
                (*maybe_mv_index).clone(),
            );
        }
        ReferenceKind::ActionViewList => {
            let workspace_mv_id = workspace_pk.to_string();
            metric!(
                counter.edda.mv_build = 1,
                label = format!("{workspace_pk}:{change_set_id_for_metrics_only}:{mv_kind}")
            );
            spawn_build_mv_task!(
                build_tasks,
                ctx,
                frigg,
                change,
                workspace_mv_id,
                ActionViewListMv,
                dal_materialized_views::action::action_view_list::assemble(ctx.clone()),
                (*maybe_mv_index).clone(),
            );
        }
        ReferenceKind::AttributeTree => {
            let entity_mv_id = change.entity_id.to_string();
            metric!(
                counter.edda.mv_build = 1,
                label = format!("{workspace_pk}:{change_set_id_for_metrics_only}:{mv_kind}")
            );
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
                (*maybe_mv_index).clone(),
            );
        }
        ReferenceKind::ComponentInList => {
            let entity_mv_id = change.entity_id.to_string();
            metric!(
                counter.edda.mv_build = 1,
                label = format!("{workspace_pk}:{change_set_id_for_metrics_only}:{mv_kind}")
            );
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
                (*maybe_mv_index).clone(),
            );
        }
        ReferenceKind::Component => {
            let entity_mv_id = change.entity_id.to_string();
            metric!(
                counter.edda.mv_build = 1,
                label = format!("{workspace_pk}:{change_set_id_for_metrics_only}:{mv_kind}")
            );
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
                (*maybe_mv_index).clone(),
            );
        }
        ReferenceKind::ComponentDiff => {
            let entity_mv_id = change.entity_id.to_string();
            metric!(
                counter.edda.mv_build = 1,
                label = format!("{workspace_pk}:{change_set_id_for_metrics_only}:{mv_kind}")
            );
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
                (*maybe_mv_index).clone(),
            );
        }
        ReferenceKind::ComponentList => {
            let workspace_mv_id = workspace_pk.to_string();
            metric!(
                counter.edda.mv_build = 1,
                label = format!("{workspace_pk}:{change_set_id_for_metrics_only}:{mv_kind}")
            );
            spawn_build_mv_task!(
                build_tasks,
                ctx,
                frigg,
                change,
                workspace_mv_id,
                ComponentListMv,
                dal_materialized_views::component_list::assemble(ctx.clone()),
                (*maybe_mv_index).clone(),
            );
        }
        ReferenceKind::ErasedComponents => {
            let workspace_mv_id = workspace_pk.to_string();
            metric!(
                counter.edda.mv_build = 1,
                label = format!("{workspace_pk}:{change_set_id_for_metrics_only}:{mv_kind}")
            );
            spawn_build_mv_task!(
                build_tasks,
                ctx,
                frigg,
                change,
                workspace_mv_id,
                ErasedComponentsMv,
                dal_materialized_views::component::erased_components::assemble(ctx.clone()),
                (*maybe_mv_index).clone(),
            );
        }
        ReferenceKind::DependentValueComponentList => {
            let workspace_mv_id = workspace_pk.to_string();
            metric!(
                counter.edda.mv_build = 1,
                label = format!("{workspace_pk}:{change_set_id_for_metrics_only}:{mv_kind}")
            );
            spawn_build_mv_task!(
                build_tasks,
                ctx,
                frigg,
                change,
                workspace_mv_id,
                DependentValueComponentListMv,
                dal_materialized_views::dependent_value_component_list::assemble(ctx.clone()),
                (*maybe_mv_index).clone(),
            );
        }
        ReferenceKind::IncomingConnections => {
            let entity_mv_id = change.entity_id.to_string();
            metric!(
                counter.edda.mv_build = 1,
                label = format!("{workspace_pk}:{change_set_id_for_metrics_only}:{mv_kind}")
            );
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
                (*maybe_mv_index).clone(),
            );
        }
        ReferenceKind::ManagementConnections => {
            let entity_mv_id = change.entity_id.to_string();
            metric!(
                counter.edda.mv_build = 1,
                label = format!("{workspace_pk}:{change_set_id_for_metrics_only}:{mv_kind}")
            );
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
                (*maybe_mv_index).clone(),
            );
        }
        ReferenceKind::IncomingConnectionsList => {
            let workspace_mv_id = workspace_pk.to_string();
            metric!(
                counter.edda.mv_build = 1,
                label = format!("{workspace_pk}:{change_set_id_for_metrics_only}:{mv_kind}")
            );
            spawn_build_mv_task!(
                build_tasks,
                ctx,
                frigg,
                change,
                workspace_mv_id,
                IncomingConnectionsListMv,
                dal_materialized_views::incoming_connections_list::assemble(ctx.clone()),
                (*maybe_mv_index).clone(),
            );
        }
        ReferenceKind::LuminorkSchemaVariant => {
            let entity_mv_id = change.entity_id.to_string();
            metric!(
                counter.edda.mv_build = 1,
                label = format!("{workspace_pk}:{change_set_id_for_metrics_only}:{mv_kind}")
            );
            spawn_build_mv_task!(
                build_tasks,
                ctx,
                frigg,
                change,
                entity_mv_id,
                si_frontend_mv_types::luminork_schema_variant::LuminorkSchemaVariant,
                dal_materialized_views::luminork::schema::variant::assemble(
                    ctx.clone(),
                    si_events::ulid::Ulid::from(change.entity_id).into()
                ),
                (*maybe_mv_index).clone(),
            );
        }
        ReferenceKind::LuminorkDefaultVariant => {
            let schema_id = si_events::ulid::Ulid::from(change.entity_id);
            let entity_mv_id = change.entity_id.to_string();
            metric!(
                counter.edda.mv_build = 1,
                label = format!("{workspace_pk}:{change_set_id_for_metrics_only}:{mv_kind}")
            );
            spawn_build_mv_task!(
                build_tasks,
                ctx,
                frigg,
                change,
                entity_mv_id,
                si_frontend_mv_types::luminork_default_variant::LuminorkDefaultVariant,
                dal_materialized_views::luminork::schema::variant::default::assemble(
                    ctx.clone(),
                    schema_id.into()
                ),
                (*maybe_mv_index).clone(),
            );
        }
        ReferenceKind::SchemaMembers => {
            let entity_mv_id = change.entity_id.to_string();
            metric!(
                counter.edda.mv_build = 1,
                label = format!("{workspace_pk}:{change_set_id_for_metrics_only}:{mv_kind}")
            );
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
                (*maybe_mv_index).clone(),
            );
        }
        ReferenceKind::SchemaVariant => {
            let entity_mv_id = change.entity_id.to_string();
            metric!(
                counter.edda.mv_build = 1,
                label = format!("{workspace_pk}:{change_set_id_for_metrics_only}:{mv_kind}")
            );
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
                (*maybe_mv_index).clone(),
            );
        }
        ReferenceKind::View => {
            let entity_mv_id = change.entity_id.to_string();
            metric!(
                counter.edda.mv_build = 1,
                label = format!("{workspace_pk}:{change_set_id_for_metrics_only}:{mv_kind}")
            );
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
                ),
                (*maybe_mv_index).clone(),
            );
        }
        ReferenceKind::ViewList => {
            let workspace_mv_id = workspace_pk.to_string();
            metric!(
                counter.edda.mv_build = 1,
                label = format!("{workspace_pk}:{change_set_id_for_metrics_only}:{mv_kind}")
            );
            spawn_build_mv_task!(
                build_tasks,
                ctx,
                frigg,
                change,
                workspace_mv_id,
                ViewListMv,
                dal_materialized_views::view_list::assemble(ctx.clone()),
                (*maybe_mv_index).clone(),
            );
        }
        ReferenceKind::ViewComponentList => {
            let entity_mv_id = change.entity_id.to_string();
            metric!(
                counter.edda.mv_build = 1,
                label = format!("{workspace_pk}:{change_set_id_for_metrics_only}:{mv_kind}")
            );
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
                (*maybe_mv_index).clone(),
            );
        }
        ReferenceKind::ChangeSetMvIndex | ReferenceKind::DeploymentMvIndex => {}
        ReferenceKind::ChangeSetList | ReferenceKind::ChangeSetRecord => {}
        ReferenceKind::LuminorkSchemaVariantFunc => {}
        ReferenceKind::CachedSchemas
        | ReferenceKind::CachedSchema
        | ReferenceKind::CachedSchemaVariant
        | ReferenceKind::CachedDefaultVariant => {
            error!(
                "Trying to build deployment-level MV '{}' via the changeset graph task. Deployment MVs should be built via build_all_mvs_for_deployment().",
                mv_kind
            );
        }
    }

    Ok(())
}

pub async fn build_mv<F, T, E>(
    operation: BuildMvOp,
    mv_kind: ReferenceKind,
    build_mv_future: F,
) -> MvBuilderResult
where
    F: Future<Output = Result<T, E>> + Send + 'static,
    T: serde::Serialize + TryInto<FrontendObject> + FrontendChecksum + Send + 'static,
    E: Into<MaterializedViewError> + std::marker::Send + 'static,
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
        let mv = slow_rt::spawn(build_mv_future)?.await??;
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
pub struct QueuedBuildMvTask {
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
