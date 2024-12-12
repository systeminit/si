use std::collections::{HashMap, HashSet};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use si_events::{merkle_tree_hash::MerkleTreeHash, workspace_snapshot::EntityKind};
use si_id::{ApprovalRequirementDefinitionId, EntityId, UserPk, WorkspacePk};

use crate::workspace_snapshot::graph::detector::Change;

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprovalRequirementPermissionLookup {
    pub object_type: String,
    pub object_id: String,
    pub permission: String,
}

// NOTE(nick,jacob): this cannot remain alphabetical due to postcard (de)serialization.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApprovalRequirementApprover {
    PermissionLookup(ApprovalRequirementPermissionLookup),
    User(UserPk),
}

#[derive(Debug, Clone)]
pub struct ApprovalRequirementRule {
    pub entity_id: EntityId,
    pub entity_kind: EntityKind,
    pub minimum: usize,
    pub approvers: HashSet<ApprovalRequirementApprover>,
}

#[derive(Debug)]
pub struct ApprovalRequirementsBag {
    pub entity_id: EntityId,
    pub entity_kind: EntityKind,
    pub explicit_approval_requirement_definition_ids: Vec<ApprovalRequirementDefinitionId>,
    pub virtual_approval_requirement_rules: Vec<ApprovalRequirementRule>,
}

pub trait ApprovalRequirementExt {
    fn approval_requirements_for_changes(
        &self,
        workspace_id: WorkspacePk,
        changes: &[Change],
    ) -> Result<(
        Vec<ApprovalRequirementsBag>,
        HashMap<EntityId, MerkleTreeHash>,
    )>;

    fn approval_requirement_definitions_for_entity_id_opt(
        &self,
        entity_id: EntityId,
    ) -> Result<Option<Vec<ApprovalRequirementDefinitionId>>>;

    fn entity_id_for_approval_requirement(
        &self,
        approval_requirement_definition_id: ApprovalRequirementDefinitionId,
    ) -> Result<EntityId>;
}
