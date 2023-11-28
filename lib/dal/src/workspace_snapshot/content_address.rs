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
    AttributePrototypeArgument(ContentHash),
    AttributeValue(ContentHash),
    Component(ContentHash),
    ExternalProvider(ContentHash),
    Func(ContentHash),
    FuncArg(ContentHash),
    InternalProvider(ContentHash),
    Node(ContentHash),
    Prop(ContentHash),
    Root,
    Schema(ContentHash),
    SchemaVariant(ContentHash),
    Socket(ContentHash),
    StaticArgumentValue(ContentHash),
    ValidationPrototype(ContentHash),
}

impl ContentAddress {
    pub fn content_hash(&self) -> ContentHash {
        match self {
            ContentAddress::Root => None,
            ContentAddress::ActionPrototype(id)
            | ContentAddress::AttributePrototype(id)
            | ContentAddress::AttributePrototypeArgument(id)
            | ContentAddress::AttributeValue(id)
            | ContentAddress::Component(id)
            | ContentAddress::ExternalProvider(id)
            | ContentAddress::FuncArg(id)
            | ContentAddress::Func(id)
            | ContentAddress::InternalProvider(id)
            | ContentAddress::Node(id)
            | ContentAddress::Prop(id)
            | ContentAddress::Schema(id)
            | ContentAddress::SchemaVariant(id)
            | ContentAddress::Socket(id)
            | ContentAddress::StaticArgumentValue(id)
            | ContentAddress::ValidationPrototype(id) => Some(*id),
        }
        .unwrap_or_default()
    }
}
