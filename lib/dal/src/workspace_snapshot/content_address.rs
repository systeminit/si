use serde::{
    Deserialize,
    Serialize,
};
use si_events::ContentHash;
use strum::EnumDiscriminants;

#[derive(
    EnumDiscriminants,
    Debug,
    Serialize,
    Deserialize,
    Copy,
    Clone,
    PartialEq,
    Eq,
    Hash,
    strum::Display,
)]
#[strum_discriminants(derive(strum::Display, Serialize, Deserialize))]
/// The type of the object, and the content-addressable-storage address (content hash)
/// of the object itself.
/// NOTE: This type is postcard serialized, so cannot be
/// #[remain::sorted]. New enum variants must come at the end of the enum!
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
    // With validations moving to the props and not having prototypes anymore, this is unused
    // TODO(victor): remove this when we migrate the graph next
    ValidationPrototype(ContentHash),
    ManagementPrototype(ContentHash),
    Geometry(ContentHash),
    View(ContentHash),
    ApprovalRequirementDefinition(ContentHash),
    AttributePaths(ContentHash),
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
            | ContentAddress::FuncArg(id)
            | ContentAddress::Func(id)
            | ContentAddress::Geometry(id)
            | ContentAddress::JsonValue(id)
            | ContentAddress::InputSocket(id)
            | ContentAddress::Module(id)
            | ContentAddress::OutputSocket(id)
            | ContentAddress::Prop(id)
            | ContentAddress::Schema(id)
            | ContentAddress::SchemaVariant(id)
            | ContentAddress::Secret(id)
            | ContentAddress::StaticArgumentValue(id)
            | ContentAddress::ValidationPrototype(id)
            | ContentAddress::ValidationOutput(id)
            | ContentAddress::View(id)
            | ContentAddress::ManagementPrototype(id)
            | ContentAddress::ApprovalRequirementDefinition(id)
            | ContentAddress::AttributePaths(id) => Some(*id),
        }
        .unwrap_or_default()
    }
}
