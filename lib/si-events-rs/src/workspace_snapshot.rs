use serde::Deserialize;
use serde::Serialize;
use strum::Display;

use crate::create_xxhash_type;

create_xxhash_type!(Checksum);

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
    ValidationOutput,
    ValidationPrototype,
    View,
}
