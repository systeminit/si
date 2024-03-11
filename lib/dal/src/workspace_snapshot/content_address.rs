use content_store::ContentHash;
use serde::{Deserialize, Serialize};
use strum::EnumDiscriminants;

#[remain::sorted]
#[derive(
    EnumDiscriminants, Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq, strum::Display,
)]
#[strum_discriminants(derive(strum::Display, Serialize, Deserialize))]
/// The type of the object, and the content-addressable-storage address (content hash)
/// of the object itself.
pub enum ContentAddress {
    Action(ContentHash),
    ActionBatch(ContentHash),
    ActionPrototype(ContentHash),
    ActionRunner(ContentHash),
    AttributePrototype(ContentHash),
    Component(ContentHash),
    Func(ContentHash),
    FuncArg(ContentHash),
    InputSocket(ContentHash),
    JsonValue(ContentHash),
    OutputSocket(ContentHash),
    Prop(ContentHash),
    Root,
    Schema(ContentHash),
    SchemaVariant(ContentHash),
    Secret(ContentHash),
    StaticArgumentValue(ContentHash),
    ValidationPrototype(ContentHash),
}

impl ContentAddress {
    pub fn content_hash(&self) -> ContentHash {
        match self {
            ContentAddress::Root => None,
            ContentAddress::ActionPrototype(id)
            | ContentAddress::ActionBatch(id)
            | ContentAddress::ActionRunner(id)
            | ContentAddress::Action(id)
            | ContentAddress::AttributePrototype(id)
            | ContentAddress::Component(id)
            | ContentAddress::OutputSocket(id)
            | ContentAddress::FuncArg(id)
            | ContentAddress::Func(id)
            | ContentAddress::InputSocket(id)
            | ContentAddress::JsonValue(id)
            | ContentAddress::Prop(id)
            | ContentAddress::Schema(id)
            | ContentAddress::SchemaVariant(id)
            | ContentAddress::Secret(id)
            | ContentAddress::StaticArgumentValue(id)
            | ContentAddress::ValidationPrototype(id) => Some(*id),
        }
        .unwrap_or_default()
    }
}
