//! This module contains graph-specific functionality related to approvals.

use si_events::workspace_snapshot::EntityKind;
use si_id::EntityId;

#[derive(Debug)]
pub struct ApprovalRequirement {
    pub entity_id: EntityId,
    pub entity_kind: EntityKind,
    pub number: usize,
    pub lookup_groups: Vec<ApprovalRequirementLookupGroup>,
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct ApprovalRequirementLookupGroup {
    pub object_type: String,
    pub object_id: String,
    pub permission: String,
}
