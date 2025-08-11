use serde::{
    Deserialize,
    Serialize,
};
use si_id::EntityId;
use strum::Display;

use crate::{
    create_xxhash_type,
    merkle_tree_hash::MerkleTreeHash,
};

create_xxhash_type!(Checksum);

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
pub struct Change {
    pub entity_id: EntityId,
    pub entity_kind: EntityKind,
    pub merkle_tree_hash: MerkleTreeHash,
}

#[remain::sorted]
#[derive(Debug, Copy, Clone, Deserialize, Serialize, PartialEq, Eq, Display, Hash)]
pub enum EntityKind {
    Action,
    ActionPrototype,
    ApprovalRequirementDefinition,
    AttributePrototype,
    AttributePrototypeArgument,
    AttributeValue,
    CategoryAction,
    CategoryComponent,
    CategoryDefaultSubscriptionSources,
    CategoryDependentValueRoots,
    CategoryDeprecatedActionBatch,
    CategoryDiagramObject,
    CategoryFunc,
    CategoryModule,
    CategorySchema,
    CategorySecret,
    CategoryView,
    Component,
    DependentValueRoot,
    DeprecatedAction,
    DeprecatedActionBatch,
    DeprecatedActionRunner,
    DiagramObject,
    ExternalTarget,
    FinishedDependentValueRoot,
    Func,
    FuncArgument,
    Geometry,
    InputSocket,
    JsonValue,
    ManagementPrototype,
    Module,
    Ordering,
    OutOfGraph, // This is so we can generate deployment MVs, we should not have nodes of this kind on the graph
    OutputSocket,
    Prop,
    Reason,
    Root,
    Schema,
    SchemaVariant,
    Secret,
    StaticArgumentValue,
    SubGraphRoot,
    ValidationOutput,
    ValidationPrototype,
    View,
}
