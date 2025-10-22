use std::{
    collections::BinaryHeap,
    sync::Arc,
    time::Duration,
};

use dal::DalContext;
use frigg::FriggStore;
use si_frontend_mv_types::{
    index::deployment::DeploymentMvIndexV2,
    materialized_view::MaterializedView,
    object::{
        FrontendObject,
        patch::{
            DeploymentIndexUpdate,
            DeploymentPatchBatch,
            DeploymentUpdateMeta,
        },
    },
    reference::ReferenceKind,
};
use telemetry::prelude::*;
use tokio::{
    task::JoinSet,
    time::Instant,
};

use crate::{
    materialized_view::{
        BuildMvInnerReturn,
        BuildMvOp,
        DeploymentBuildMvTaskResult,
        MaterializedViewError,
        MvBuilderResult,
        build_mv,
    },
    updates::EddaUpdates,
};

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
    parallel_build_limit: usize,
    reason_message: &'static str,
) -> Result<(), MaterializedViewError> {
    let span = current_span_for_instrument_at!("info");
    info!("Started building all deployment MVs");

    // Discover all deployment MVs that need to be built
    let deployment_tasks = discover_deployment_mvs(ctx).await?;

    // Use the deployment MV parallel processing system
    let (
        frontend_objects,
        patches,
        build_count,
        build_total_elapsed,
        build_max_elapsed,
        build_slowest_mv_kind,
    ) = build_deployment_mv_inner(ctx, frigg, parallel_build_limit, &deployment_tasks).await?;
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

    // Build the deployment index from all the frontend objects
    let mut index_entries: Vec<_> = frontend_objects.into_iter().map(Into::into).collect();
    index_entries.sort();

    debug!("index_entries {:?}", index_entries);
    let mv_index = DeploymentMvIndexV2::new(index_entries);
    let mv_index_frontend_object = FrontendObject::try_from(mv_index)?;
    let from_index_checksum = mv_index_frontend_object.checksum.to_owned();
    let to_index_checksum = from_index_checksum.clone();
    let meta = DeploymentUpdateMeta {
        from_index_checksum,
        to_index_checksum,
    };
    let patch_batch = DeploymentPatchBatch::new(meta.clone(), patches);
    let index_update =
        DeploymentIndexUpdate::new(meta, mv_index_frontend_object.checksum.to_owned());

    // Store the index on frigg
    frigg
        .put_deployment_index(&mv_index_frontend_object)
        .await?;

    // publish updates
    edda_updates
        .publish_deployment_patch_batch(patch_batch)
        .await?;
    edda_updates
        .publish_deployment_index_update(index_update)
        .await?;
    info!("Started building all deployment MVs");
    Ok(())
}

/// This function builds only deployment MVs whose definition checksum is outdated
/// by comparing current definition checksums with those stored in the deployment index
#[instrument(
    name = "materialized_view.build_outdated_mvs_for_deployment",
    level = "info",
    skip_all,
    fields(
        si.materialized_view.reason = reason_message,
        si.edda.mv.count = Empty,
        si.edda.mv.avg_build_elapsed_ms = Empty,
        si.edda.mv.max_build_elapsed_ms = Empty,
        si.edda.mv.slowest_kind = Empty,
        si.edda.mv.outdated_count = Empty,
        si.edda.mv.total_discovered = Empty,
    ),
)]
pub async fn build_outdated_mvs_for_deployment(
    ctx: &DalContext,
    frigg: &FriggStore,
    edda_updates: &EddaUpdates,
    parallel_build_limit: usize,
    reason_message: &'static str,
) -> Result<(), MaterializedViewError> {
    let span = current_span_for_instrument_at!("info");

    // Get current definition checksums from the registry
    let current_definition_checksums =
        si_frontend_mv_types::definition_checksum::materialized_view_definition_checksums();

    // Get existing deployment index to compare checksums
    let existing_deployment_index_frontend_object = frigg.get_deployment_index().await?;
    let existing_deployment_index = match existing_deployment_index_frontend_object {
        Some((deployment_index_obj, _)) => {
            // Parse the deployment index to get the stored definition checksums
            match serde_json::from_value::<
                si_frontend_mv_types::index::deployment::DeploymentMvIndexVersion,
            >(deployment_index_obj.data)
            {
                Ok(si_frontend_mv_types::index::deployment::DeploymentMvIndexVersion::V1(_)) => {
                    // We don't have enough information to determine which MV kinds have had definition changes.
                    return build_all_mvs_for_deployment(
                        ctx,
                        frigg,
                        edda_updates,
                        parallel_build_limit,
                        "fallback because of index upgrade",
                    )
                    .await;
                }
                Err(_) => {
                    return build_all_mvs_for_deployment(
                        ctx,
                        frigg,
                        edda_updates,
                        parallel_build_limit,
                        "fallback because of index parsing error",
                    )
                    .await;
                }
                Ok(si_frontend_mv_types::index::deployment::DeploymentMvIndexVersion::V2(
                    deployment_index,
                )) => deployment_index,
            }
        }
        None => {
            // No existing index, so rebuild everything
            debug!("No existing deployment index found, rebuilding all MVs");
            return build_all_mvs_for_deployment(
                ctx,
                frigg,
                edda_updates,
                parallel_build_limit,
                "fallback because of missing index",
            )
            .await;
        }
    };

    // Discover all deployment MVs that could be built
    let all_deployment_tasks = discover_deployment_mvs(ctx).await?;
    span.record("si.edda.mv.total_discovered", all_deployment_tasks.len());

    // Filter to only tasks where checksums differ or are missing
    let outdated_tasks: Vec<_> = all_deployment_tasks
        .into_iter()
        .filter(|task| {
            let mv_kind_str = task.mv_kind.to_string();
            let current_checksum = current_definition_checksums.get(&mv_kind_str);
            let existing_checksum = existing_deployment_index
                .definition_checksums
                .get(&mv_kind_str);

            match (current_checksum, existing_checksum) {
                (Some(current), Some(existing)) => current != existing,
                (Some(_), None) => true,  // New MV type
                (None, Some(_)) => false, // MV type no longer exists (shouldn't happen but handle gracefully)
                (None, None) => false,    // Neither exists (shouldn't happen)
            }
        })
        .collect();

    span.record("si.edda.mv.outdated_count", outdated_tasks.len());

    if outdated_tasks.is_empty() {
        info!("No outdated deployment MVs found, skipping rebuild");
        return Ok(());
    }

    debug!(
        "Found {} outdated deployment MVs out of {} total: {:?}",
        outdated_tasks.len(),
        outdated_tasks.len() + existing_deployment_index.definition_checksums.len(),
        outdated_tasks
            .iter()
            .map(|t| t.mv_kind.to_string())
            .collect::<Vec<_>>()
    );

    // Use the deployment MV parallel processing system with filtered tasks
    let (
        frontend_objects,
        patches,
        build_count,
        build_total_elapsed,
        build_max_elapsed,
        build_slowest_mv_kind,
    ) = build_deployment_mv_inner(ctx, frigg, parallel_build_limit, &outdated_tasks).await?;
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

    // Build the deployment index from all the frontend objects (both updated and existing)
    let mut index_entries: Vec<_> = frontend_objects.into_iter().map(Into::into).collect();

    // For MVs that weren't rebuilt, keep their existing entries from the current index
    let rebuilt_kinds: std::collections::HashSet<_> = outdated_tasks
        .iter()
        .map(|task| task.mv_kind.to_string())
        .collect();

    // Add entries for MVs that weren't rebuilt
    for entry in existing_deployment_index.mv_list {
        if !rebuilt_kinds.contains(&entry.kind) {
            index_entries.push(entry);
        }
    }

    index_entries.sort();

    debug!("index_entries {:?}", index_entries);
    let mv_index = si_frontend_mv_types::index::deployment::DeploymentMvIndexV2::new(index_entries);
    let mv_index_frontend_object =
        si_frontend_mv_types::object::FrontendObject::try_from(mv_index)?;
    let from_index_checksum = mv_index_frontend_object.checksum.to_owned();
    let to_index_checksum = from_index_checksum.clone();
    let meta = si_frontend_mv_types::object::patch::DeploymentUpdateMeta {
        from_index_checksum,
        to_index_checksum,
    };
    let patch_batch =
        si_frontend_mv_types::object::patch::DeploymentPatchBatch::new(meta.clone(), patches);
    let index_update = si_frontend_mv_types::object::patch::DeploymentIndexUpdate::new(
        meta,
        mv_index_frontend_object.checksum.to_owned(),
    );

    // Store the index on frigg
    frigg
        .put_deployment_index(&mv_index_frontend_object)
        .await?;

    // publish updates
    edda_updates
        .publish_deployment_patch_batch(patch_batch)
        .await?;
    edda_updates
        .publish_deployment_index_update(index_update)
        .await?;

    Ok(())
}

/// A deployment MV task that needs to be built for the deployment environment.
/// Unlike changeset MVs, these are not triggered by graph changes but by deployment-scoped
/// data that exists outside the workspace graph (e.g., cached modules, global configuration, etc.).
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DeploymentMvTask {
    pub mv_kind: ReferenceKind,
    pub mv_id: String,
    pub priority: si_events::materialized_view::BuildPriority,
}

impl Ord for DeploymentMvTask {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.priority.cmp(&other.priority) {
            std::cmp::Ordering::Equal => {
                // Within the same priority, order by MV kind and then by ID
                match self.mv_kind.cmp(&other.mv_kind) {
                    std::cmp::Ordering::Equal => self.mv_id.cmp(&other.mv_id),
                    ord => ord,
                }
            }
            ord => ord,
        }
    }
}

impl PartialOrd for DeploymentMvTask {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Discovers all deployment-scoped MVs that need to be built.
/// This function examines deployment-scoped data sources and creates tasks
/// for all individual objects and collections that should be built.
async fn discover_deployment_mvs(
    ctx: &DalContext,
) -> Result<Vec<DeploymentMvTask>, MaterializedViewError> {
    use dal::cached_module::CachedModule;

    let mut tasks = Vec::new();

    // Add the existing CachedSchemas collection MV
    tasks.push(DeploymentMvTask {
        mv_kind: ReferenceKind::CachedSchemas,
        mv_id: "cached_schemas".to_string(), // Collection has a fixed ID
        priority: <si_frontend_mv_types::cached_schemas::CachedSchemas as MaterializedView>::build_priority(),
    });

    // Discover individual CachedSchema and CachedDefaultVariant objects from cached modules
    let modules = CachedModule::latest_modules(ctx).await?;
    for module in modules {
        // Add individual CachedSchema MV task
        tasks.push(DeploymentMvTask {
                mv_kind: ReferenceKind::CachedSchema,
                mv_id: module.schema_id.to_string(),
                priority: <si_frontend_mv_types::cached_schema::CachedSchema as MaterializedView>::build_priority(),
            });

        // Add CachedDefaultVariant MV task
        tasks.push(DeploymentMvTask {
                mv_kind: ReferenceKind::CachedDefaultVariant,
                mv_id: module.schema_id.to_string(),
                priority: <si_frontend_mv_types::cached_default_variant::CachedDefaultVariant as MaterializedView>::build_priority(),
            });
    }

    Ok(tasks)
}

/// Builds deployment MVs using the same parallel processing pattern as changeset MVs.
/// Uses a priority queue and JoinSet for concurrent execution with error handling.
#[instrument(
    name = "materialized_view.build_deployment_mv_inner",
    level = "debug",
    skip_all
)]
async fn build_deployment_mv_inner(
    ctx: &DalContext,
    frigg: &FriggStore,
    parallel_build_limit: usize,
    deployment_tasks: &[DeploymentMvTask],
) -> Result<BuildMvInnerReturn, MaterializedViewError> {
    let mut frontend_objects = Vec::new();
    let mut patches = Vec::new();
    let mut build_tasks = JoinSet::new();
    let mut queued_mv_builds = BinaryHeap::new();

    let maybe_deployment_mv_index =
        Arc::new(frigg.get_deployment_index().await?.map(|result| result.0));

    // Queue all deployment tasks
    for task in deployment_tasks {
        queued_mv_builds.push(task.clone());
    }

    let mut build_total_elapsed = Duration::from_nanos(0);
    let mut build_count: u128 = 0;
    let mut build_max_elapsed = Duration::from_nanos(0);
    let mut build_slowest_mv_kind: &str = "N/A";

    loop {
        // If there aren't any queued builds waiting for the concurrency limit then
        // we've finished everything and are able to send off the collected
        // FrontendObjects and ObjectPatches to update the index.
        if queued_mv_builds.is_empty() && build_tasks.is_empty() {
            break;
        }

        // Spawn as many of the queued build tasks as we can, up to the concurrency limit.
        while !queued_mv_builds.is_empty() && build_tasks.len() < parallel_build_limit {
            let Some(DeploymentMvTask { mv_kind, mv_id, .. }) = queued_mv_builds.pop() else {
                // This _really_ shouldn't ever return `None` as we just checked that
                // `queued_mv_builds` is not empty.
                break;
            };
            spawn_deployment_mv_task(
                &mut build_tasks,
                ctx,
                frigg,
                mv_kind,
                mv_id,
                maybe_deployment_mv_index.clone(),
            )
            .await?;
        }

        if let Some(join_result) = build_tasks.join_next().await {
            let (kind, mv_id, build_duration, execution_result) = join_result?;
            telemetry_utils::metric!(counter.edda.mv_build = -1, label = kind.to_string());

            match execution_result {
                Ok((maybe_patch, maybe_frontend_object)) => {
                    // Store deployment objects using deployment storage
                    if let Some(frontend_object) = maybe_frontend_object {
                        frigg.insert_deployment_object(&frontend_object).await?;
                        frontend_objects.push(frontend_object);
                    }
                    if let Some(patch) = maybe_patch {
                        // Deployment MVs don't need streaming patches since they're not changeset-scoped
                        debug!("Deployment Patch: {:?}", patch);
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
                    error!(name = "deployment_mv_build_error", si.error.message = err.to_string(), kind = %kind, id = %mv_id);
                    return Err(err);
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

/// Spawns a build task for a specific deployment MV.
async fn spawn_deployment_mv_task(
    build_tasks: &mut JoinSet<DeploymentBuildMvTaskResult>,
    ctx: &DalContext,
    frigg: &FriggStore,
    mv_kind: ReferenceKind,
    mv_id: String,
    maybe_deployment_mv_index: Arc<Option<FrontendObject>>,
) -> Result<(), MaterializedViewError> {
    // Record the currently running number of MV build tasks
    telemetry_utils::metric!(counter.edda.mv_build = 1, label = mv_kind.to_string());
    // Info span for the spawned task
    let span = info_span!(
        "si.edda.deployment.mv.build",
        si.edda.mv.kind = %mv_kind,
        si.edda.mv.id = %mv_id,
    )
    .or_current();
    match mv_kind {
        ReferenceKind::CachedSchemas => {
            build_tasks.spawn(
                build_mv_for_deployment_task(
                    ctx.clone(),
                    frigg.clone(),
                    mv_id,
                    mv_kind,
                    dal_materialized_views::cached::schemas::assemble(ctx.clone()),
                    maybe_deployment_mv_index,
                )
                .instrument(span),
            );
        }
        ReferenceKind::CachedSchema => {
            let schema_id: dal::SchemaId = mv_id.parse().map_err(|_| {
                dal::SchemaError::UninstalledSchemaNotFound(dal::SchemaId::from(ulid::Ulid::nil()))
            })?;
            build_tasks.spawn(
                build_mv_for_deployment_task(
                    ctx.clone(),
                    frigg.clone(),
                    mv_id,
                    mv_kind,
                    dal_materialized_views::cached::schema::assemble(ctx.clone(), schema_id),
                    maybe_deployment_mv_index,
                )
                .instrument(span),
            );
        }
        ReferenceKind::CachedSchemaVariant => {
            // Note(Brit): Currently, we only want to expose the latest variant (which is the default) for a schema
            // which means it's sufficient to only build the DefaultVariant.
            // We'll stop building them first, then we can clean up later
            return Ok(());
        }
        ReferenceKind::CachedDefaultVariant => {
            let schema_id: dal::SchemaId = mv_id.parse().map_err(|_| {
                dal::SchemaError::UninstalledSchemaNotFound(dal::SchemaId::from(ulid::Ulid::nil()))
            })?;

            build_tasks.spawn(
                build_mv_for_deployment_task(
                    ctx.clone(),
                    frigg.clone(),
                    mv_id,
                    mv_kind,
                    dal_materialized_views::cached::schema::variant::default::assemble(
                        ctx.clone(),
                        schema_id,
                    ),
                    maybe_deployment_mv_index,
                )
                .instrument(span),
            );
        }
        _ => {
            // This shouldn't happen for deployment MVs, but we'll handle it gracefully
            warn!("Unexpected MV kind for deployment: {:?}", mv_kind);
            return Ok(());
        }
    }

    Ok(())
}

/// Builds an MV task specifically for deployment (no Change context needed).
async fn build_mv_for_deployment_task<F, T, E>(
    _ctx: DalContext,
    frigg: FriggStore,
    mv_id: String,
    mv_kind: ReferenceKind,
    build_mv_future: F,
    maybe_deployment_mv_index: Arc<Option<FrontendObject>>,
) -> DeploymentBuildMvTaskResult
where
    F: std::future::Future<Output = Result<T, E>> + Send + 'static,
    T: serde::Serialize
        + TryInto<FrontendObject>
        + si_frontend_mv_types::checksum::FrontendChecksum
        + Send
        + 'static,
    E: Into<MaterializedViewError> + Send + 'static,
    MaterializedViewError: From<E>,
    MaterializedViewError: From<<T as TryInto<FrontendObject>>::Error>,
{
    debug!(kind = %mv_kind, id = %mv_id, "Building Deployment MV");
    let start = Instant::now();

    let result = build_mv_for_deployment_task_inner(
        &frigg,
        mv_id.clone(),
        mv_kind,
        build_mv_future,
        maybe_deployment_mv_index,
    )
    .await;

    (mv_kind, mv_id, start.elapsed(), result)
}

/// Inner function for building deployment MV tasks.
async fn build_mv_for_deployment_task_inner<F, T, E>(
    frigg: &FriggStore,
    mv_id: String,
    mv_kind: ReferenceKind,
    build_mv_future: F,
    maybe_deployment_mv_index: Arc<Option<FrontendObject>>,
) -> MvBuilderResult
where
    F: std::future::Future<Output = Result<T, E>> + Send + 'static,
    T: serde::Serialize
        + TryInto<FrontendObject>
        + si_frontend_mv_types::checksum::FrontendChecksum
        + Send
        + 'static,
    E: Into<MaterializedViewError> + Send + 'static,
    MaterializedViewError: From<E>,
    MaterializedViewError: From<<T as TryInto<FrontendObject>>::Error>,
{
    // For deployment MVs, check if object exists in deployment storage
    let op = {
        let maybe_previous_version = match frigg
            .get_current_deployment_object_with_index(
                mv_kind.into(),
                &mv_id,
                (*maybe_deployment_mv_index).clone(),
            )
            .await
        {
            Ok(maybe_previous) => maybe_previous,
            Err(err) => {
                warn!(
                    "Unable to retreive previous deployment MV version; proceeding without: {err}"
                );
                None
            }
        };

        if let Some(previous_version) = maybe_previous_version {
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
