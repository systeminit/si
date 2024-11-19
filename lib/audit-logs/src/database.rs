//! Contains functionality for setting up and communicating with the audit database.

use chrono::DateTime;
use chrono::Utc;
use si_data_pg::PgError;
use si_data_pg::PgPoolError;
use si_events::audit_log::AuditLogKind;
use si_events::audit_log::AuditLogMetadata;
use si_events::Actor;
use si_events::ChangeSetId;
use si_events::WorkspacePk;
use telemetry::prelude::*;
use thiserror::Error;

mod config;
mod context;
mod migrate;

pub use config::AuditDatabaseConfig;
pub use config::DBNAME;
pub use context::AuditDatabaseContext;
pub use context::AuditDatabaseContextError;
pub use migrate::{migrate, AuditDatabaseMigrationError};

#[allow(missing_docs)]
#[derive(Error, Debug)]
pub enum AuditDatabaseError {
    #[error("chrono parse error: {0}")]
    ChronoParse(#[from] chrono::ParseError),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("pg pool error: {0}")]
    PgPool(#[from] PgPoolError),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
}

type Result<T> = std::result::Result<T, AuditDatabaseError>;

#[allow(clippy::too_many_arguments, missing_docs)]
#[instrument(
    name = "audit_log.insert",
    level = "debug",
    skip_all,
    fields(
        si.workspace.id = %workspace_id,
    ),
)]
pub async fn insert(
    context: &AuditDatabaseContext,
    workspace_id: WorkspacePk,
    kind: AuditLogKind,
    timestamp: String,
    change_set_id: Option<ChangeSetId>,
    actor: Actor,
    entity_name: Option<String>,
) -> Result<()> {
    let kind_as_string = kind.to_string();
    let user_id = match actor {
        Actor::System => None,
        Actor::User(user_id) => Some(user_id),
    };

    let metadata = AuditLogMetadata::from(kind);
    let (title, entity_type) = metadata.title_and_entity_type();
    let serialized_metadata = serde_json::to_value(metadata)?;
    let timestamp: DateTime<Utc> = timestamp.parse()?;

    context
        .pg_pool()
        .get()
        .await?
        .query_one(
            "INSERT INTO audit_logs (
                workspace_id,
                kind,
                timestamp,
                title,
                change_set_id,
                user_id,
                entity_name,
                entity_type,
                metadata
            ) VALUES (
                $1,
                $2,
                $3,
                $4,
                $5,
                $6,
                $7,
                $8,
                $9
            ) RETURNING *",
            &[
                &workspace_id,
                &kind_as_string,
                &timestamp,
                &title,
                &change_set_id.map(|id| id.to_string()),
                &user_id.map(|id| id.to_string()),
                &entity_name,
                &entity_type,
                &serialized_metadata,
            ],
        )
        .await?;
    Ok(())
}
