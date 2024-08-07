use std::collections::{HashMap, HashSet};

use petgraph::prelude::*;
use serde::{Deserialize, Serialize};
use si_events::{merkle_tree_hash::MerkleTreeHash, ulid::Ulid, ContentHash, EncryptedSecretKey};
use strum::EnumDiscriminants;

use crate::{
    action::{prototype::ActionKind, ActionState},
    func::FuncKind,
    workspace_snapshot::{
        content_address::ContentAddress,
        graph::LineageId,
        node_weight::{category_node_weight::CategoryNodeKind, ArgumentTargets},
        vector_clock::VectorClock,
    },
    ChangeSetId, EdgeWeightKind, PropKind, Timestamp,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeprecatedActionNodeWeightV1 {
    pub id: Ulid,
    pub state: ActionState,
    pub originating_change_set_id: ChangeSetId,
    pub lineage_id: LineageId,
    pub merkle_tree_hash: MerkleTreeHash,
    pub vector_clock_first_seen: VectorClock,
    pub vector_clock_recently_seen: VectorClock,
    pub vector_clock_write: VectorClock,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeprecatedActionPrototypeNodeWeightV1 {
    pub id: Ulid,
    pub kind: ActionKind,
    pub name: String,
    pub description: Option<String>,
    pub lineage_id: LineageId,
    pub merkle_tree_hash: MerkleTreeHash,
    pub vector_clock_first_seen: VectorClock,
    pub vector_clock_recently_seen: VectorClock,
    pub vector_clock_write: VectorClock,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct DeprecatedAttributeValueNodeWeightV1 {
    pub id: Ulid,
    pub lineage_id: LineageId,
    pub merkle_tree_hash: MerkleTreeHash,
    pub vector_clock_first_seen: VectorClock,
    pub vector_clock_recently_seen: VectorClock,
    pub vector_clock_write: VectorClock,
    pub unprocessed_value: Option<ContentAddress>,
    pub value: Option<ContentAddress>,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct DeprecatedAttributePrototypeArgumentNodeWeightV1 {
    pub id: Ulid,
    pub lineage_id: LineageId,
    pub merkle_tree_hash: MerkleTreeHash,
    pub vector_clock_first_seen: VectorClock,
    pub vector_clock_recently_seen: VectorClock,
    pub vector_clock_write: VectorClock,
    pub targets: Option<ArgumentTargets>,
    pub timestamp: Timestamp,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct DeprecatedCategoryNodeWeightV1 {
    pub id: Ulid,
    pub lineage_id: LineageId,
    pub kind: CategoryNodeKind,
    pub merkle_tree_hash: MerkleTreeHash,
    pub vector_clock_first_seen: VectorClock,
    pub vector_clock_recently_seen: VectorClock,
    pub vector_clock_write: VectorClock,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeprecatedComponentNodeWeightV1 {
    pub id: Ulid,
    pub lineage_id: LineageId,
    pub content_address: ContentAddress,
    pub merkle_tree_hash: MerkleTreeHash,
    pub vector_clock_first_seen: VectorClock,
    pub vector_clock_recently_seen: VectorClock,
    pub vector_clock_write: VectorClock,
    pub to_delete: bool,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct DeprecatedContentNodeWeightV1 {
    pub id: Ulid,
    pub lineage_id: LineageId,
    pub content_address: ContentAddress,
    pub merkle_tree_hash: MerkleTreeHash,
    pub vector_clock_first_seen: VectorClock,
    pub vector_clock_recently_seen: VectorClock,
    pub vector_clock_write: VectorClock,
    pub to_delete: bool,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct DeprecatedDependentValueRootNodeWeightV1 {
    pub id: Ulid,
    pub lineage_id: Ulid,
    pub value_id: Ulid,
    pub merkle_tree_hash: MerkleTreeHash,
    pub vector_clock_first_seen: VectorClock,
    pub vector_clock_recently_seen: VectorClock,
    pub vector_clock_write: VectorClock,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct DeprecatedFuncArgumentNodeWeightV1 {
    pub id: Ulid,
    pub lineage_id: LineageId,
    pub content_address: ContentAddress,
    pub merkle_tree_hash: MerkleTreeHash,
    pub vector_clock_first_seen: VectorClock,
    pub vector_clock_recently_seen: VectorClock,
    pub vector_clock_write: VectorClock,
    pub name: String,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct DeprecatedFuncNodeWeightV1 {
    pub id: Ulid,
    pub lineage_id: LineageId,
    pub content_address: ContentAddress,
    pub merkle_tree_hash: MerkleTreeHash,
    pub vector_clock_first_seen: VectorClock,
    pub vector_clock_recently_seen: VectorClock,
    pub vector_clock_write: VectorClock,
    pub name: String,
    pub func_kind: FuncKind,
}

#[derive(Clone, Serialize, Deserialize, Default, PartialEq, Eq, Debug)]
pub struct DeprecatedOrderingNodeWeightV1 {
    pub id: Ulid,
    pub lineage_id: Ulid,
    pub order: Vec<Ulid>,
    pub content_hash: ContentHash,
    pub merkle_tree_hash: MerkleTreeHash,
    pub vector_clock_first_seen: VectorClock,
    pub vector_clock_recently_seen: VectorClock,
    pub vector_clock_write: VectorClock,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct DeprecatedPropNodeWeightV1 {
    pub id: Ulid,
    pub lineage_id: LineageId,
    pub content_address: ContentAddress,
    pub merkle_tree_hash: MerkleTreeHash,
    pub kind: PropKind,
    pub name: String,
    pub can_be_used_as_prototype_arg: bool,
    pub vector_clock_first_seen: VectorClock,
    pub vector_clock_recently_seen: VectorClock,
    pub vector_clock_write: VectorClock,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct DeprecatedSecretNodeWeightV1 {
    pub id: Ulid,
    pub lineage_id: LineageId,
    pub content_address: ContentAddress,
    pub merkle_tree_hash: MerkleTreeHash,
    pub vector_clock_first_seen: VectorClock,
    pub vector_clock_recently_seen: VectorClock,
    pub vector_clock_write: VectorClock,
    pub encrypted_secret_key: EncryptedSecretKey,
}

#[derive(Debug, Serialize, Deserialize, Clone, EnumDiscriminants, PartialEq, Eq)]
#[strum_discriminants(derive(strum::Display, Hash, Serialize, Deserialize))]
pub enum DeprecatedNodeWeightV1 {
    Action(DeprecatedActionNodeWeightV1),
    ActionPrototype(DeprecatedActionPrototypeNodeWeightV1),
    AttributePrototypeArgument(DeprecatedAttributePrototypeArgumentNodeWeightV1),
    AttributeValue(DeprecatedAttributeValueNodeWeightV1),
    Category(DeprecatedCategoryNodeWeightV1),
    Component(DeprecatedComponentNodeWeightV1),
    Content(DeprecatedContentNodeWeightV1),
    DependentValueRoot(DeprecatedDependentValueRootNodeWeightV1),
    Func(DeprecatedFuncNodeWeightV1),
    FuncArgument(DeprecatedFuncArgumentNodeWeightV1),
    Ordering(DeprecatedOrderingNodeWeightV1),
    Prop(DeprecatedPropNodeWeightV1),
    Secret(DeprecatedSecretNodeWeightV1),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct DeprecatedEdgeWeightV1 {
    pub kind: EdgeWeightKind,
    pub vector_clock_first_seen: VectorClock,
    pub vector_clock_recently_seen: VectorClock,
    pub vector_clock_write: VectorClock,
}

#[derive(Default, Deserialize, Serialize, Clone, Debug)]
pub struct DeprecatedWorkspaceSnapshotGraphV1 {
    pub graph: StableDiGraph<DeprecatedNodeWeightV1, DeprecatedEdgeWeightV1>,
    pub node_index_by_id: HashMap<Ulid, NodeIndex>,
    pub node_indices_by_lineage_id: HashMap<LineageId, HashSet<NodeIndex>>,
    pub root_index: NodeIndex,
}
