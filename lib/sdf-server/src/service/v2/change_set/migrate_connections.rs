use axum::Json;
use dal::workspace_snapshot::graph::validator::connections::ConnectionMigration;
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
    let migrations = snapshot
        .connection_migrations()
        .await?
        .into_iter()
        .map(|(migration, message)| ConnectionMigrationWithMessage { migration, message })
        .collect();
    Ok(Json(Response { migrations }))
}
