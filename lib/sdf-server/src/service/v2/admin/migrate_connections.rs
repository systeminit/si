use std::collections::{
    HashMap,
    HashSet,
};

use axum::extract::Path;
use dal::{
    AttributeValue,
    ChangeSetId,
    Component,
    DalContext,
    WorkspacePk,
    WsEvent,
    attribute::{
        path::AttributePath,
        prototype::argument::AttributePrototypeArgument,
        value::subscription::ValueSubscription,
    },
    slow_rt,
    workspace_snapshot::{
        graph::validator::connections::{
            ConnectionMigration,
            ConnectionMigrationSummary,
            ConnectionMigrationWithMessage,
            ConnectionUnmigrateableBecause,
            PropConnection,
            SocketConnection,
        },
        node_weight::reason_node_weight::Reason,
    },
};
use sdf_extract::PosthogEventTracker;
use serde_json::json;
use si_db::Tenancy;
use si_id::AttributePrototypeArgumentId;
use telemetry::prelude::*;

use crate::service::v2::admin::{
    AdminAPIResult,
    AdminUserContext,
};

pub async fn migrate_connections(
    AdminUserContext(mut ctx): AdminUserContext,
    tracker: PosthogEventTracker,
    Path((workspace_id, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
) -> AdminAPIResult<()> {
    ctx.update_tenancy(Tenancy::new(workspace_id));
    ctx.update_visibility_and_snapshot_to_visibility(change_set_id)
        .await?;
    let span = Span::current();
    slow_rt::spawn(async move { migrate_connections_async(&ctx, tracker, span, false).await })?;
    Ok(())
}
pub async fn migrate_connections_dry_run(
    AdminUserContext(mut ctx): AdminUserContext,
    tracker: PosthogEventTracker,
    Path((workspace_id, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
) -> AdminAPIResult<()> {
    ctx.update_tenancy(Tenancy::new(workspace_id));
    ctx.update_visibility_and_snapshot_to_visibility(change_set_id)
        .await?;
    let span = Span::current();
    slow_rt::spawn(async move { migrate_connections_async(&ctx, tracker, span, true).await })?;
    Ok(())
}

/// Internal: migrates and reports any errors wherever they need to be.
#[instrument(level = "info", parent = &parent_span, skip(ctx, tracker))]
pub async fn migrate_connections_async(
    ctx: &DalContext,
    tracker: PosthogEventTracker,
    parent_span: Span,
    dry_run: bool,
) -> () {
    let span = Span::current();
    span.record("si.change_set.id", ctx.change_set_id().to_string());
    span.record(
        "si.workspace.id",
        ctx.workspace_pk().unwrap_or_default().to_string(),
    );

    // Capture a summary even if we fail, so we can report what we *did* do.
    let mut summary = ConnectionMigrationSummary {
        connections: 0,
        migrated: 0,
        unmigrateable: 0,
    };
    let err = migrate_connections_async_fallible(ctx, dry_run, &mut summary)
        .await
        .err();

    // Report state in span, WsEvent and Posthog event.
    span.record("connections", summary.connections);
    span.record("migrated", summary.migrated);
    span.record("unmigrateable", summary.unmigrateable);
    span.record("error", err.is_some());
    tracker.track(
        ctx,
        if dry_run {
            "migrate_connections_dry_run"
        } else {
            "migrate_connections"
        },
        json!({
            "dry_run": dry_run,
            "connections": summary.connections,
            "migrated": summary.migrated,
            "unmigrateable": summary.unmigrateable,
            "error": err.as_ref().map(|e| e.to_string()),
        }),
    );

    match WsEvent::connection_migration_finished(ctx, dry_run, err.map(|e| e.to_string()), summary)
        .await
    {
        Ok(event) => {
            if let Err(err) = event.publish_immediately(ctx).await {
                error!("Failed to send connection migration finished event: {err}");
            }
        }
        Err(err) => {
            error!("Failed to send connection migration finished event: {err}");
        }
    }
}

// Internal, called by migrate_connections_async to catch errors in the migration process.
async fn migrate_connections_async_fallible(
    ctx: &DalContext,
    dry_run: bool,
    summary: &mut ConnectionMigrationSummary,
) -> AdminAPIResult<()> {
    WsEvent::connection_migration_started(ctx, dry_run)
        .await?
        .publish_immediately(ctx)
        .await?;

    // Get the connections we want to migrate.
    let connections = get_connection_migrations(ctx).await?;
    summary.connections = connections.len();

    // Migrate.
    let mut migrations = Vec::with_capacity(connections.len());
    for mut connection in connections {
        let migrated = match migrate_connection(ctx, &connection).await {
            // Don't include noop migrations (already-migrated) in the report
            Ok(Some(false)) => continue,
            Ok(Some(true)) => true,
            Ok(None) => false,
            // Mark it as unmigrateable if there was an error
            Err(err) => {
                if connection.issue.is_none() {
                    connection.issue = Some(ConnectionUnmigrateableBecause::InvalidGraph {
                        error: err.to_string(),
                    });
                }
                false
            }
        };
        if migrated {
            summary.migrated += 1;
        } else {
            summary.unmigrateable += 1;
        }

        // Report that it was migrated.
        let message = connection.fmt_title(ctx).await;
        info!(message, migrated, "migrated socket connection");
        let migration = ConnectionMigrationWithMessage {
            connection,
            message,
        };
        WsEvent::connection_migrated(ctx, migration.clone())
            .await?
            .publish_immediately(ctx)
            .await?;
        migrations.push(migration);
    }

    // Send WsEvents for components we migrated, and commit (unless it's a dry_run)
    if !dry_run {
        let mut components = HashSet::new();
        for migration in migrations {
            let Some(ref socket_connection) = migration.connection.socket_connection else {
                continue;
            };

            // Send WsEvent that we modified properties of the component
            if components.insert(socket_connection.to.0) {
                let component = Component::get_by_id(ctx, socket_connection.to.0).await?;

                let mut socket_map = HashMap::new();
                let payload = component
                    .into_frontend_type(
                        ctx,
                        None,
                        component.change_status(ctx).await?,
                        &mut socket_map,
                    )
                    .await?;
                WsEvent::component_updated(ctx, payload)
                    .await?
                    .publish_on_commit(ctx)
                    .await?;
            }
        }

        // Commits
        ctx.commit().await?;
    }

    Ok(())
}

#[instrument(level = "info", skip(ctx))]
async fn get_connection_migrations(ctx: &DalContext) -> AdminAPIResult<Vec<ConnectionMigration>> {
    let snapshot = ctx.workspace_snapshot()?.as_legacy_snapshot()?;

    let inferred_connections = snapshot
        .inferred_connection_graph(ctx)
        .await?
        .inferred_connections_for_all_components(ctx)
        .await?
        .into_iter()
        .map(|connection| SocketConnection {
            from: (connection.source_component_id, connection.output_socket_id),
            to: (
                connection.destination_component_id,
                connection.input_socket_id,
            ),
        });

    Ok(snapshot.connection_migrations(inferred_connections).await?)
}

/// Migrate a single connection.
///
/// Returns true if any migration work was done, false if it was a noop, and None if the connection
/// is entirely unmigrateable.
///
/// Check `issue` to see if there was an issue (e.g. partial migration).
async fn migrate_connection(
    ctx: &DalContext,
    migration: &ConnectionMigration,
) -> AdminAPIResult<Option<bool>> {
    match &migration.issue {
        // If there's no issue, we can migrate.
        None => {}
        // We can migrate schema mismatches by removing the socket connection.
        // (We check migration.prop_connections to make sure we won't accidentally create a new
        // connection.)
        Some(
            ConnectionUnmigrateableBecause::DestinationSocketSchemaMismatch
            | ConnectionUnmigrateableBecause::SourceSocketSchemaMismatch,
        ) if migration.prop_connections.is_empty() => {}
        // If there's an issue, we can't migrate.
        Some(_) => {
            // If it's an inferred connection, we still want to make it explicit! No more
            // inferred connections.
            if migration.explicit_connection_id.is_none() {
                info!("Making inferred socket connection explicit due to issues");
                make_socket_connection_explicit(ctx, &migration.socket_connection).await?;
            }
            return Ok(None);
        }
    }

    let mut did_something = false;
    for prop_connection in &migration.prop_connections {
        if add_prop_connection(ctx, prop_connection).await? {
            did_something = true;
        }
    }

    if remove_socket_connection(
        ctx,
        migration.explicit_connection_id,
        &migration.socket_connection,
    )
    .await?
    {
        did_something = true;
    }

    Ok(Some(did_something))
}

/// Add the given prop connections
async fn add_prop_connection(
    ctx: &DalContext,
    &PropConnection {
        from: (from_component_id, ref from_path),
        to: (to_component_id, ref to_path),
        func_id,
    }: &PropConnection,
) -> AdminAPIResult<bool> {
    // If the destination already has an explicit value, we keep it instead of replacing it!
    let to_root_av_id = Component::root_attribute_value_id(ctx, to_component_id).await?;
    let to_path = AttributePath::from_json_pointer(to_path.to_string());
    let to_av_id = to_path.vivify(ctx, to_root_av_id).await?;
    if AttributeValue::component_prototype_id(ctx, to_av_id)
        .await?
        .is_some()
    {
        return Ok(false);
    }

    // Create the subscription
    let from_root_av_id = Component::root_attribute_value_id(ctx, from_component_id).await?;
    let from_path = AttributePath::from_json_pointer(from_path.to_string());
    AttributeValue::set_to_subscriptions(
        ctx,
        to_av_id,
        vec![ValueSubscription {
            attribute_value_id: from_root_av_id,
            path: from_path,
        }],
        Some(func_id),
        Reason::new_user_added(ctx),
    )
    .await?;

    Ok(true)
}

/// Remove the existing socket connection (unless it was inferred, in which case there isn't one)
async fn remove_socket_connection(
    ctx: &DalContext,
    explicit_connection_id: Option<AttributePrototypeArgumentId>,
    socket_connection: &Option<SocketConnection>,
) -> AdminAPIResult<bool> {
    // We don't remove inferred connections
    let Some(explicit_connection_id) = explicit_connection_id else {
        return Ok(false);
    };

    // Remove the connection
    AttributePrototypeArgument::remove(ctx, explicit_connection_id).await?;

    // Send the WsEvent
    if let &Some(SocketConnection {
        from: (from_component_id, from_socket_id),
        to: (to_component_id, to_socket_id),
    }) = socket_connection
    {
        // Send the WsEvent
        WsEvent::connection_deleted(
            ctx,
            from_component_id,
            to_component_id,
            from_socket_id,
            to_socket_id,
        )
        .await?
        .publish_on_commit(ctx)
        .await?;
    }

    Ok(true)
}

/// Make the socket connection explicit, if it isn't already.
/// Returns false if there is nothing to make explicit.
async fn make_socket_connection_explicit(
    ctx: &DalContext,
    socket_connection: &Option<SocketConnection>,
) -> AdminAPIResult<bool> {
    if let &Some(SocketConnection {
        from: (from_component_id, from_socket_id),
        to: (to_component_id, to_socket_id),
    }) = socket_connection
    {
        // Reuse the code from the create_connection endpoint to send WsEvents and such
        sdf_v1_routes_diagram::create_connection::create_connection_inner(
            ctx,
            from_component_id,
            from_socket_id,
            to_component_id,
            to_socket_id,
        )
        .await?;
        Ok(true)
    } else {
        Ok(false)
    }
}
