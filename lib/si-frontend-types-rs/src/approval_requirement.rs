use std::collections::HashMap;

use serde::{
    Deserialize,
    Serialize,
};
use si_events::workspace_snapshot::EntityKind;
use si_id::{
    ApprovalRequirementDefinitionId,
    EntityId,
    UserPk,
};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalRequirementDefinition {
    // What is the ID of this ApprovalRequirementDefinition?
    pub id: ApprovalRequirementDefinitionId,
    // What is the ID of the entity that is requiring approvals?
    pub entity_id: EntityId,
    // What is the kind of the entity corresponding to the ID?
    pub entity_kind: EntityKind,
    // What is the minimum number needed?
    pub required_count: usize,
    // What groups can approve this?
    pub approver_groups: HashMap<String, Vec<UserPk>>,
    // What individuals can approve this?
    pub approver_individuals: Vec<UserPk>,
}
