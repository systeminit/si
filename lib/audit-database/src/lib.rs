//! This crate contains functionality for setting up and communicating with the audit database.

#![warn(
    bad_style,
    clippy::missing_panics_doc,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    dead_code,
    improper_ctypes,
    missing_debug_implementations,
    missing_docs,
    no_mangle_generic_items,
    non_shorthand_field_patterns,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    unconditional_recursion,
    unreachable_pub,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true
)]

use std::str::FromStr;

use chrono::{
    DateTime,
    Utc,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_data_pg::{
    PgError,
    PgPoolError,
    PgRow,
};
use si_events::{
    Actor,
    AuthenticationMethod,
    ChangeSetId,
    UserPk,
    WorkspacePk,
    audit_log::{
        AuditLogKind,
        AuditLogMetadata,
    },
    ulid,
};
use telemetry::prelude::*;
use thiserror::Error;

mod config;
mod context;
mod migrate;

pub use config::{
    AuditDatabaseConfig,
    DBNAME,
    default_pg_pool_config,
};
pub use context::{
    AuditDatabaseContext,
    AuditDatabaseContextError,
};
pub use migrate::{
    AuditDatabaseMigrationError,
    migrate,
};

#[allow(missing_docs)]
#[remain::sorted]
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
    #[error("ulid decode error: {0}")]
    UlidDecode(#[from] ulid::DecodeError),
}

type Result<T> = std::result::Result<T, AuditDatabaseError>;

/// A row in the audit logs table of the audit database.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditLogRow {
    /// Indicates the workspace that the row belongs to.
    pub workspace_id: WorkspacePk,
    /// The [kind](AuditLogKind) of the [`AuditLog`] (converted into a string because enum discriminants are not
    /// serializable).
    pub kind: String,
    /// The timestamp that can be used in ISO RFC 3339 format.
    pub timestamp: DateTime<Utc>,
    /// The title of the [`AuditLog`]. It will likely be combined with the `entity_type` to make a full display name.
    pub title: String,
    /// The identifier of the change set, which will only be empty for actions taken outside of the workspace.
    pub change_set_id: Option<ChangeSetId>,
    /// The identifier of the user. If this is empty, it is the system user.
    pub user_id: Option<UserPk>,
    /// The entity name.
    pub entity_name: Option<String>,
    /// The entity type.
    pub entity_type: Option<String>,
    /// Serialized version of [`AuditLogMetadata`](si_events::audit_log::AuditLogMetadata), which is an
    /// untagged version of the specific [`AuditLogKind`](si_events::audit_log::AuditLogKind).
    pub metadata: Option<serde_json::Value>,
    /// Serialized version of Actor Metadata as string
    pub authentication_method: AuthenticationMethod,
}

impl AuditLogRow {
    /// Inserts a new row into the audit logs table of the audit database.
    #[allow(clippy::too_many_arguments)]
    #[instrument(
        name = "audit_log.database.insert",
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
        authentication_method: AuthenticationMethod,
    ) -> Result<()> {
        let kind_as_string = kind.to_string();
        let user_id = match actor {
            Actor::System => None,
            Actor::User(user_id) => Some(user_id),
        };
        let metadata = AuditLogMetadata::from(kind);
        let (title, entity_type) = metadata.title_and_entity_type();
        let serialized_metadata = serde_json::to_value(metadata)?;
        let serialized_authentication_method = serde_json::to_value(authentication_method)?;
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
                    metadata,
                    authentication_method
                ) VALUES (
                    $1,
                    $2,
                    $3,
                    $4,
                    $5,
                    $6,
                    $7,
                    $8,
                    $9,
                    $10
                ) RETURNING *",
                &[
                    &workspace_id.to_string(),
                    &kind_as_string,
                    &timestamp,
                    &title,
                    &change_set_id.map(|id| id.to_string()),
                    &user_id.map(|id| id.to_string()),
                    &entity_name,
                    &entity_type,
                    &serialized_metadata,
                    &serialized_authentication_method,
                ],
            )
            .await?;
        Ok(())
    }

    /// Lists rows of the audit logs table in the audit database.
    #[instrument(
        name = "audit_log.database.list",
        level = "debug",
        skip_all,
        fields(
            si.workspace.id = %workspace_id,
        ),
    )]
    pub async fn list(
        context: &AuditDatabaseContext,
        workspace_id: WorkspacePk,
        change_set_ids: Vec<ChangeSetId>,
        size: usize,
        sort_ascending: bool,
    ) -> Result<(Vec<Self>, bool)> {
        let size = size as i64;
        let change_set_ids: Vec<String> = change_set_ids.iter().map(|id| id.to_string()).collect();

        let client = context.pg_pool().get().await?;
        let row = client
            .query_one(
                "SELECT COUNT(*) from audit_logs WHERE workspace_id = $1 AND change_set_id = ANY($2)",
                &[&workspace_id, &change_set_ids],
            )
            .await?;
        let count: i64 = row.try_get("count")?;
        let can_load_more = count > size;

        let query = if sort_ascending {
            "SELECT * from audit_logs WHERE workspace_id = $1 AND change_set_id = ANY($2) ORDER BY timestamp ASC LIMIT $3"
        } else {
            "SELECT * from audit_logs WHERE workspace_id = $1 AND change_set_id = ANY($2) ORDER BY timestamp DESC LIMIT $3"
        };

        let mut result = Vec::new();
        let rows = client
            .query(query, &[&workspace_id, &change_set_ids, &size])
            .await?;
        for row in rows {
            result.push(Self::try_from(row)?);
        }

        Ok((result, can_load_more))
    }

    /// Lists rows of the audit logs table filtered by component ID in the audit database.
    #[instrument(
        name = "audit_log.database.list_for_component",
        level = "debug",
        skip_all,
        fields(
            si.workspace.id = %workspace_id,
            si.component.id = %component_id
        ),
    )]
    pub async fn list_for_component(
        context: &AuditDatabaseContext,
        workspace_id: WorkspacePk,
        change_set_ids: Vec<ChangeSetId>,
        component_id: si_events::ComponentId,
        size: usize,
        sort_ascending: bool,
    ) -> Result<(Vec<Self>, bool)> {
        let size = size as i64;
        let change_set_ids: Vec<String> = change_set_ids.iter().map(|id| id.to_string()).collect();
        let component_id_str = component_id.to_string();

        let client = context.pg_pool().get().await?;
        
        // Count total matching records
        let row = client
            .query_one(
                "SELECT COUNT(*) from audit_logs WHERE workspace_id = $1 AND change_set_id = ANY($2) AND (metadata::text LIKE $3 OR entity_name = $4)",
                &[&workspace_id, &change_set_ids, &format!("%\"componentId\":\"{}\"%", component_id_str), &component_id_str],
            )
            .await?;
        let count: i64 = row.try_get("count")?;
        let can_load_more = count > size;

        let query = if sort_ascending {
            "SELECT * from audit_logs WHERE workspace_id = $1 AND change_set_id = ANY($2) AND (metadata::text LIKE $3 OR entity_name = $4) ORDER BY timestamp ASC LIMIT $5"
        } else {
            "SELECT * from audit_logs WHERE workspace_id = $1 AND change_set_id = ANY($2) AND (metadata::text LIKE $3 OR entity_name = $4) ORDER BY timestamp DESC LIMIT $5"
        };

        let mut result = Vec::new();
        let rows = client
            .query(query, &[&workspace_id, &change_set_ids, &format!("%\"componentId\":\"{}\"%", component_id_str), &component_id_str, &size])
            .await?;
        for row in rows {
            result.push(Self::try_from(row)?);
        }

        Ok((result, can_load_more))
    }
}

impl TryFrom<PgRow> for AuditLogRow {
    type Error = AuditDatabaseError;

    fn try_from(value: PgRow) -> std::result::Result<Self, Self::Error> {
        let workspace_id = {
            let inner: String = value.try_get("workspace_id")?;
            WorkspacePk::from_str(&inner)?
        };
        let change_set_id = {
            let maybe_inner: Option<String> = value.try_get("change_set_id")?;
            match maybe_inner {
                Some(inner) => Some(ChangeSetId::from_str(&inner)?),
                None => None,
            }
        };
        let user_id = {
            let maybe_inner: Option<String> = value.try_get("user_id")?;
            match maybe_inner {
                Some(inner) => Some(UserPk::from_str(&inner)?),
                None => None,
            }
        };
        let authentication_method = {
            let inner: serde_json::Value = value.try_get("authentication_method")?;
            serde_json::from_value::<AuthenticationMethod>(inner)?
        };

        Ok(Self {
            workspace_id,
            kind: value.try_get("kind")?,
            timestamp: value.try_get("timestamp")?,
            title: value.try_get("title")?,
            change_set_id,
            user_id,
            entity_name: value.try_get("entity_name")?,
            entity_type: value.try_get("entity_type")?,
            metadata: value.try_get("metadata")?,
            authentication_method,
        })
    }
}
