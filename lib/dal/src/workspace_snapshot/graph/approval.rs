//! This module contains graph-specific functionality related to approvals.

use serde::{Deserialize, Serialize};
use si_events::workspace_snapshot::EntityKind;
use si_id::{EntityId, UserPk};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequirement {
    pub entity_id: EntityId,
    pub entity_kind: EntityKind,
    pub number: usize,
    pub lookup_groups: Vec<ApprovalRequirementApprover>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApprovalRequirementApprover {
    User(UserPk),
    PermissionLookup(ApprovalRequirementPermissionLookup),
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprovalRequirementPermissionLookup {
    pub object_type: String,
    pub object_id: String,
    pub permission: String,
}
