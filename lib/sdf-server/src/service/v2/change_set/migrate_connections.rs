use axum::Json;
use dal::workspace_snapshot::graph::validator::connections::{
    ConnectionMigration,
    SocketConnection,
};
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
}

pub async fn migrate_connections(
    ChangeSetDalContext(ref mut ctx): ChangeSetDalContext,
) -> Result<Json<Response>> {
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

    let migrations = snapshot
        .connection_migrations(inferred_connections)
        .await?
        .into_iter()
        .map(|(migration, message)| ConnectionMigrationWithMessage { migration, message })
        .collect();

    Ok(Json(Response { migrations }))
}
