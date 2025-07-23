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
    workspace_snapshot::graph::validator::connections::{
        ConnectionMigration,
        ConnectionMigrationSummary,
        ConnectionMigrationWithMessage,
        PropConnection,
        SocketConnection,
    },
};
use sdf_extract::PosthogEventTracker;
use serde_json::json;
use si_db::Tenancy;
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
    slow_rt::spawn(async move { migrate_connections_async(&ctx, tracker, false).await })?;
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
    slow_rt::spawn(async move { migrate_connections_async(&ctx, tracker, true).await })?;
    Ok(())
}

/// Internal: migrates and reports any errors wherever they need to be.
#[instrument(level = "info", skip(ctx, tracker))]
pub async fn migrate_connections_async(
    ctx: &DalContext,
    tracker: PosthogEventTracker,
    dry_run: bool,
) -> () {
    let span = Span::current();

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

    // Migrate
    let migrations = get_connection_migrations(ctx).await?;
    summary.connections = migrations.len();
    for migration in &migrations {
        migrate_connection(ctx, &migration.migration).await?;
        if migration.migrated {
            summary.migrated += 1;
        } else {
            summary.unmigrateable += 1;
        }
        WsEvent::connection_migrated(ctx, migration.clone())
            .await?
            .publish_immediately(ctx)
            .await?;
    }

    // Send WsEvents for components we migrated, and commit (unless it's a dry_run)
    if !dry_run {
        let mut components = HashSet::new();
        for migration in migrations {
            if !migration.migrated {
                continue;
            }
            let Some(ref socket_connection) = migration.migration.socket_connection else {
                continue;
            };

            // Send WsEvent
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
async fn get_connection_migrations(
    ctx: &DalContext,
) -> AdminAPIResult<Vec<ConnectionMigrationWithMessage>> {
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

    Ok(snapshot
        .connection_migrations(inferred_connections)
        .await?
        .into_iter()
        .map(|(migration, message)| ConnectionMigrationWithMessage {
            migration,
            message,
            migrated: false,
        })
        .collect())
}

// Returns true if migrated, false if we didn't migrate
async fn migrate_connection(
    ctx: &DalContext,
    migration: &ConnectionMigration,
) -> AdminAPIResult<bool> {
    // Make sure it's migrateable (no issues and has all the data we need)
    if migration.issue.is_some() {
        return Ok(false);
    }
    // Add the prop connections
    for &PropConnection {
        from: (from_component_id, ref from_path),
        to: (to_component_id, ref to_path),
        func_id,
    } in &migration.prop_connections
    {
        let from_root_av_id = Component::root_attribute_value_id(ctx, from_component_id).await?;
        let to_root_av_id = Component::root_attribute_value_id(ctx, to_component_id).await?;
        let to_av_id = AttributePath::from_json_pointer(to_path.to_string())
            .vivify(ctx, to_root_av_id)
            .await?;
        AttributeValue::set_to_subscriptions(
            ctx,
            to_av_id,
            vec![ValueSubscription {
                attribute_value_id: from_root_av_id,
                path: AttributePath::from_json_pointer(from_path.to_string()),
            }],
            Some(func_id),
        )
        .await?;
    }

    // Remove the existing socket connection (unless it was inferred, in which case there isn't one)
    if let Some(explicit_connection_id) = migration.explicit_connection_id {
        AttributePrototypeArgument::remove(ctx, explicit_connection_id).await?;
        if let Some(SocketConnection {
            from: (from_component_id, from_socket_id),
            to: (to_component_id, to_socket_id),
        }) = migration.socket_connection
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
    }

    Ok(true)
}
