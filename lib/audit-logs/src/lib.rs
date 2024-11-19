//! This crate provides a centralized location for working with the audit logs domain.

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

pub mod pg;
mod stream;

use pg::AuditDatabaseContext;
use serde::Deserialize;
use serde::Serialize;
use si_data_pg::PgError;
use si_data_pg::PgPoolError;
use si_events::Actor;
use si_events::ChangeSetId;
use si_events::ChangeSetStatus;
use si_events::UserPk;
use si_events::WorkspacePk;
use strum::Display;
use strum::EnumDiscriminants;
use telemetry::prelude::*;
use thiserror::Error;

pub use stream::AuditLogsStream;
pub use stream::AuditLogsStreamError;

#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum AuditLogError {
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("pg pool error: {0}")]
    PgPool(#[from] PgPoolError),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
}

type Result<T> = std::result::Result<T, AuditLogError>;

// FIXME(nick): delete this once accessor patterns are in place.
// impl TryFrom<PgRow> for AuditLogRow {
//     type Error = AuditLogError;
//
//     fn try_from(value: PgRow) -> std::result::Result<Self, Self::Error> {
//         Ok(Self {
//             actor: todo!(),
//             kind: AuditLogKind::CreateChangeSet,
//             entity_name: todo!(),
//             timestamp: todo!(),
//             change_set_id: todo!(),
//         })
//         let status_string: String = value.try_get("status")?;
//         let status = ChangeSetStatus::try_from(status_string.as_str())?;
//         Ok(Self {
//             id: value.try_get("id")?,
//             created_at: value.try_get("created_at")?,
//             updated_at: value.try_get("updated_at")?,
//             name: value.try_get("name")?,
//             status,
//             base_change_set_id: value.try_get("base_change_set_id")?,
//             workspace_snapshot_address: value.try_get("workspace_snapshot_address")?,
//             workspace_id: value.try_get("workspace_id")?,
//             merge_requested_by_user_id: value.try_get("merge_requested_by_user_id")?,
//             merge_requested_at: value.try_get("merge_requested_at")?,
//             reviewed_by_user_id: value.try_get("reviewed_by_user_id")?,
//             reviewed_at: value.try_get("reviewed_at")?,
//         })
//     }
// }

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Display, EnumDiscriminants)]
pub enum AuditLogKind {
    #[allow(missing_docs)]
    AbandonChangeSet { from_status: ChangeSetStatus },
    #[allow(missing_docs)]
    CreateChangeSet,
}

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Debug, Serialize, Deserialize, EnumDiscriminants)]
#[serde(untagged, rename_all = "camelCase")]
pub enum AuditLogMetadata {
    #[allow(missing_docs)]
    #[serde(rename_all = "camelCase")]
    AbandonChangeSet { from_status: ChangeSetStatus },
    #[allow(missing_docs)]
    #[serde(rename_all = "camelCase")]
    CreateChangeSet,
}

impl From<AuditLogKind> for AuditLogMetadata {
    fn from(value: AuditLogKind) -> Self {
        match value {
            AuditLogKind::AbandonChangeSet { from_status } => {
                Self::AbandonChangeSet { from_status }
            }
            AuditLogKind::CreateChangeSet => Self::CreateChangeSet,
        }
    }
}

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
    title: String,
    change_set_id: Option<ChangeSetId>,
    actor: Actor,
    entity_name: Option<String>,
    entity_type: Option<String>,
) -> Result<()> {
    let kind_as_string = kind.to_string();
    let user_id: Option<UserPk> = match actor {
        Actor::System => None,
        Actor::User(user_id) => Some(user_id),
    };
    let serialized_metadata = serde_json::to_value(AuditLogMetadata::from(kind))?;

    let _ = context
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
                )",
            &[
                &workspace_id,
                &kind_as_string,
                &timestamp,
                &title,
                &change_set_id,
                &user_id,
                &entity_name,
                &entity_type,
                &serialized_metadata,
            ],
        )
        .await?;
    Ok(())
}
