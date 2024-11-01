use chrono::Utc;
use serde::{Deserialize, Serialize};

use v1::AuditLogV1;
use v2::AuditLogV2;
use v3::{AuditLogKindV3, AuditLogV3};

use crate::{Actor, ChangeSetId};

mod v1;
mod v2;
mod v3;

pub type AuditLogKind = AuditLogKindV3;

// TODO(nick): switch to something like "naxum-api-types" crate to avoid sizing issues.
#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub enum AuditLog {
    V3(Box<AuditLogV3>),
    V2(Box<AuditLogV2>),
    V1(AuditLogV1),
}

impl AuditLog {
    /// Creates a new [`AuditLog`] with a mandatory [`ChangeSetId`].
    ///
    /// _Note:_ [`ChangeSetId`] is required for almost all kinds of audit logging except for
    /// workspace management and authentication.
    pub fn new(
        actor: Actor,
        kind: AuditLogKind,
        entity_name: Option<String>,
        change_set_id: ChangeSetId,
    ) -> Self {
        Self::V3(Box::new(AuditLogV3 {
            actor,
            kind,
            entity_name,
            timestamp: Utc::now().to_rfc3339(),
            change_set_id: Some(change_set_id),
        }))
    }
}
