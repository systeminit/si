use std::collections::{
    HashMap,
    HashSet,
};

use axum::Json;
use dal::{
    AttributeValue,
    ChangeSet,
    Component,
    DalContext,
    WsEvent,
    attribute::{
        path::AttributePath,
        prototype::argument::AttributePrototypeArgument,
        value::subscription::ValueSubscription,
    },
    workspace_snapshot::graph::validator::connections::{
        ConnectionMigration,
        PropConnection,
        SocketConnection,
    },
};
use sdf_core::force_change_set_response::ForceChangeSetResponse;
use sdf_extract::change_set::ChangeSetDalContext;

use super::Result;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub migrations: Vec<ConnectionMigrationWithMessage>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionMigrationWithMessage {
    #[serde(flatten)]
    pub migration: ConnectionMigration,
    pub message: String,
    pub migrated: bool,
}

// Migrates all connections, and reports the unmigrateable ones as well.
pub async fn migrate_connections(
    ChangeSetDalContext(ref mut ctx): ChangeSetDalContext,
) -> Result<ForceChangeSetResponse<Response>> {
    let force_change_set_id = ChangeSet::force_new(ctx).await?;

    // Migrate
    let mut migrations = get_connection_migrations(ctx).await?;
    for migration in &mut migrations {
        migration.migrated = migrate_connection(ctx, &migration.migration).await?;
    }

    // Send WsEvents for components we migrated
    let mut components = HashSet::new();
    for migration in &migrations {
        if !migration.migrated {
            continue;
        }
        let Some(ref socket_connection) = migration.migration.socket_connection else {
            continue;
        };

        if components.insert(socket_connection.destination.0) {
            let component = Component::get_by_id(ctx, socket_connection.destination.0).await?;

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

    ctx.commit().await?;

    Ok(ForceChangeSetResponse::new(
        force_change_set_id,
        Response { migrations },
    ))
}

/// Does the migrations, but doesn't commit them--just lets you see what would have happened.
pub async fn dry_run(ChangeSetDalContext(ref ctx): ChangeSetDalContext) -> Result<Json<Response>> {
    let migrations = get_connection_migrations(ctx).await?;
    for migration in &migrations {
        migrate_connection(ctx, &migration.migration).await?;
    }
    Ok(Json(Response { migrations }))
}

async fn get_connection_migrations(
    ctx: &DalContext,
) -> Result<Vec<ConnectionMigrationWithMessage>> {
    let snapshot = ctx.workspace_snapshot()?.as_legacy_snapshot()?;

    let inferred_connections = snapshot
        .inferred_connection_graph(ctx)
        .await?
        .inferred_connections_for_all_components(ctx)
        .await?
        .into_iter()
        .map(|connection| SocketConnection {
            source: (connection.source_component_id, connection.output_socket_id),
            destination: (
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
async fn migrate_connection(ctx: &DalContext, migration: &ConnectionMigration) -> Result<bool> {
    // Make sure it's migrateable (no issues and has all the data we need)
    let &ConnectionMigration {
        issue: None,
        explicit_connection_id,
        prop_connection:
            Some(PropConnection {
                dest_av_id,
                source_root_av_id,
                ref source_path,
                func_id,
                func_arg_id: _, // TODO handle funcs with multiple args but only get passed one
            }),
        socket_connection:
            Some(SocketConnection {
                source: (from_component_id, from_socket_id),
                destination: (to_component_id, to_socket_id),
            }),
    } = migration
    else {
        return Ok(false);
    };

    // Add the prop connection
    AttributeValue::set_to_subscriptions(
        ctx,
        dest_av_id,
        vec![ValueSubscription {
            attribute_value_id: source_root_av_id,
            path: AttributePath::from_json_pointer(source_path.to_string()),
        }],
        Some(func_id),
    )
    .await?;

    // Remove the existing socket connection (unless it was inferred, in which case there isn't one)
    if let Some(explicit_connection_id) = explicit_connection_id {
        AttributePrototypeArgument::remove(ctx, explicit_connection_id).await?;
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
    };

    Ok(true)
}
