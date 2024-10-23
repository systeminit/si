use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumIter, EnumString};

use crate::Actor;

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub enum AuditLog {
    V1(AuditLogV1),
}

impl AuditLog {
    pub fn new(actor: Actor, kind: AuditLogKind, timestamp: DateTime<Utc>) -> Self {
        Self::V1(AuditLogV1 {
            actor,
            kind,
            timestamp: timestamp.to_rfc3339(),
        })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct AuditLogV1 {
    pub actor: Actor,
    pub kind: AuditLogKindV1,
    pub timestamp: String,
}

pub type AuditLogKind = AuditLogKindV1;

#[remain::sorted]
#[derive(
    Clone,
    Debug,
    Deserialize,
    Serialize,
    Eq,
    PartialEq,
    Hash,
    AsRefStr,
    Display,
    EnumIter,
    EnumString,
)]
pub enum AuditLogKindV1 {
    CreateComponent,
    DeleteComponent,
    PerformRebase,
    RunAction,
    RunComputeValidations,
    RunDependentValuesUpdate,
    UpdatePropertyEditorValue,
}
