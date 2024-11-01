use serde::{Deserialize, Serialize};

use crate::{Actor, ChangeSetId};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct AuditLogV3 {
    pub actor: Actor,
    pub kind: AuditLogKindV3,
    pub entity_name: Option<String>,
    pub timestamp: String,
    pub change_set_id: Option<ChangeSetId>,
}

pub type AuditLogKindV3 = super::v2::AuditLogKindV2;
