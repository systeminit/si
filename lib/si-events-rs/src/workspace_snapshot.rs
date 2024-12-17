use serde::Deserialize;
use serde::Serialize;

use crate::create_xxhash_type;

create_xxhash_type!(Checksum);

#[remain::sorted]
#[derive(Debug, Copy, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum EntityKind {
    Action,
    ActionPrototype,
    AttributePrototype,
    AttributePrototypeArgument,
    AttributeValue,
    Category,
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
