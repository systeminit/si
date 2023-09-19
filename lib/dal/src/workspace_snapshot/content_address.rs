use serde::{Deserialize, Serialize};

use crate::content::hash::ContentHash;

#[remain::sorted]
#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq)]
/// The type of the object, and the content-addressable-storage address (content hash)
/// of the object itself.
pub enum ContentAddress {
    AttributePrototype(ContentHash),
    AttributeValue(ContentHash),
    Component(ContentHash),
    ExternalProvider(ContentHash),
    Func(ContentHash),
    FuncArg(ContentHash),
    InternalProvider(ContentHash),
    Prop(ContentHash),
    Root,
    Schema(ContentHash),
    SchemaVariant(ContentHash),
}

impl ContentAddress {
    pub fn content_hash(&self) -> ContentHash {
        match self {
            ContentAddress::AttributePrototype(id) => Some(*id),
            ContentAddress::AttributeValue(id) => Some(*id),
            ContentAddress::Component(id) => Some(*id),
            ContentAddress::ExternalProvider(id) => Some(*id),
            ContentAddress::FuncArg(id) => Some(*id),
            ContentAddress::Func(id) => Some(*id),
            ContentAddress::InternalProvider(id) => Some(*id),
            ContentAddress::Prop(id) => Some(*id),
            ContentAddress::Root => None,
            ContentAddress::Schema(id) => Some(*id),
            ContentAddress::SchemaVariant(id) => Some(*id),
        }
        .unwrap_or_default()
    }
}
