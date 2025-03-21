use serde::Deserialize;
use serde::Serialize;
use si_id::EntityId;
use strum::Display;

use crate::create_xxhash_type;
use crate::merkle_tree_hash::MerkleTreeHash;

create_xxhash_type!(Checksum);

#[derive(Debug, Clone, Copy)]
pub struct Change {
    pub entity_id: EntityId,
    pub entity_kind: EntityKind,
    pub merkle_tree_hash: MerkleTreeHash,
}

#[remain::sorted]
#[derive(Debug, Copy, Clone, Deserialize, Serialize, PartialEq, Eq, Display)]
pub enum EntityKind {
    Action,
    ActionPrototype,
    ApprovalRequirementDefinition,
    AttributePrototype,
    AttributePrototypeArgument,
    AttributeValue,
    CategoryAction,
    CategoryComponent,
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
    OutputSocket,
    Prop,
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
