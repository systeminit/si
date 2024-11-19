use chrono::Utc;
use serde::{Deserialize, Serialize};

use v1::{AuditLogKindV1, AuditLogMetadataV1, AuditLogV1};

use crate::{Actor, ChangeSetId};

mod v1;

pub type AuditLogKind = AuditLogKindV1;
pub type AuditLogMetadata = AuditLogMetadataV1;

// TODO(nick): switch to something like "naxum-api-types" crate to avoid sizing issues.
#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub enum AuditLog {
    V1(Box<AuditLogV1>),
}

impl AuditLog {
    /// Creates a new [`AuditLog`] with a mandatory [`ChangeSetId`].
    ///
    /// _Note:_ [`ChangeSetId`] is required for almost all kinds of audit logging except for
    /// workspace management and authentication.
    pub fn new(
        actor: Actor,
        kind: AuditLogKind,
        entity_name: String,
        change_set_id: ChangeSetId,
    ) -> Self {
        Self::V1(Box::new(AuditLogV1 {
            actor,
            kind,
            entity_name,
            timestamp: Utc::now().to_rfc3339(),
            change_set_id: Some(change_set_id),
        }))
    }
}
