use serde::{Deserialize, Serialize};
use si_events::ContentHash;
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
    // TODO(fnichol): remove the `Deprecated*` variants
    DeprecatedAction(ContentHash),
    DeprecatedActionBatch(ContentHash),
    DeprecatedActionRunner(ContentHash),
    Func(ContentHash),
    FuncArg(ContentHash),
    InputSocket(ContentHash),
    JsonValue(ContentHash),
    Module(ContentHash),
    OutputSocket(ContentHash),
    Prop(ContentHash),
    Root,
    Schema(ContentHash),
    SchemaVariant(ContentHash),
    Secret(ContentHash),
    StaticArgumentValue(ContentHash),
    ValidationOutput(ContentHash),
    ValidationPrototype(ContentHash), // TODO(victor): Remove this after module index gets new data
}

impl ContentAddress {
    pub fn content_hash(&self) -> ContentHash {
        match self {
            ContentAddress::Root => None,
            ContentAddress::ActionPrototype(id)
            | ContentAddress::AttributePrototype(id)
            | ContentAddress::Component(id)
            | ContentAddress::DeprecatedAction(id)
            | ContentAddress::DeprecatedActionBatch(id)
            | ContentAddress::DeprecatedActionRunner(id)
            | ContentAddress::OutputSocket(id)
            | ContentAddress::FuncArg(id)
            | ContentAddress::Func(id)
            | ContentAddress::InputSocket(id)
            | ContentAddress::JsonValue(id)
            | ContentAddress::Module(id)
            | ContentAddress::Prop(id)
            | ContentAddress::Schema(id)
            | ContentAddress::SchemaVariant(id)
            | ContentAddress::Secret(id)
            | ContentAddress::StaticArgumentValue(id)
            | ContentAddress::ValidationPrototype(id)
            | ContentAddress::ValidationOutput(id) => Some(*id),
        }
        .unwrap_or_default()
    }
}
