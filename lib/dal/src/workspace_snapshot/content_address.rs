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
    ActionPrototype(ContentHash),
    AttributePrototype(ContentHash),
    Component(ContentHash),
    ExternalProvider(ContentHash),
    Func(ContentHash),
    FuncArg(ContentHash),
    InternalProvider(ContentHash),
    JsonValue(ContentHash),
    Prop(ContentHash),
    Root,
    Schema(ContentHash),
    SchemaVariant(ContentHash),
    StaticArgumentValue(ContentHash),
    ValidationPrototype(ContentHash),
}

impl ContentAddress {
    pub fn content_hash(&self) -> ContentHash {
        match self {
            ContentAddress::Root => None,
            ContentAddress::ActionPrototype(id)
            | ContentAddress::AttributePrototype(id)
            | ContentAddress::Component(id)
            | ContentAddress::ExternalProvider(id)
            | ContentAddress::FuncArg(id)
            | ContentAddress::Func(id)
            | ContentAddress::InternalProvider(id)
            | ContentAddress::JsonValue(id)
            | ContentAddress::Prop(id)
            | ContentAddress::Schema(id)
            | ContentAddress::SchemaVariant(id)
            | ContentAddress::StaticArgumentValue(id)
            | ContentAddress::ValidationPrototype(id) => Some(*id),
        }
        .unwrap_or_default()
    }
}
