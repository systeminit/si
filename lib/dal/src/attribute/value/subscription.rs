use serde::{
    Deserialize,
    Serialize,
};
use strum::EnumDiscriminants;

use super::{
    AttributeValue,
    AttributeValueResult,
};
use crate::{
    AttributeValueId,
    DalContext,
    PropKind,
};

/// A subscription to an attribute value: the root value and path relative to that value
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ValueSubscription {
    // The root attribute value
    pub attribute_value_id: AttributeValueId,
    // The path to the actual attribute value, relative to the root
    pub path: ValueSubscriptionPath,
}

/// A path to an attribute value, relative to its root value
/// This type is postcard serialized and new enum variants *MUST* be added to the end *ONLY*.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, EnumDiscriminants)]
#[strum_discriminants(derive(Hash, Serialize, Deserialize, strum::EnumIter, strum::Display))]
pub enum ValueSubscriptionPath {
    /// A JSON pointer (e.g. `/domain/PolicyDocument/Statements/0/Operation`)
    JsonPointer(String),
}

impl ValueSubscription {
    /// Find the attribute value a subscription points to
    /// Returns `None` if the path leads to an attribute value that does not exist
    pub async fn resolve(
        &self,
        ctx: &DalContext,
    ) -> AttributeValueResult<Option<AttributeValueId>> {
        let mut av_id = self.attribute_value_id;
        match self.path {
            ValueSubscriptionPath::JsonPointer(ref json_pointer) => {
                let ptr = jsonptr::Pointer::parse(json_pointer)?;
                for token in ptr {
                    let Some(child_av_id) = resolve_child(ctx, av_id, token).await? else {
                        return Ok(None);
                    };
                    av_id = child_av_id;
                }
            }
        }

        Ok(Some(av_id))
    }
}

async fn resolve_child(
    ctx: &DalContext,
    av_id: AttributeValueId,
    token: jsonptr::Token<'_>,
) -> AttributeValueResult<Option<AttributeValueId>> {
    let prop = AttributeValue::prop(ctx, av_id).await?;
    Ok(match prop.kind {
        // Look up array index in ordering node
        PropKind::Array => match token.to_index() {
            Ok(jsonptr::index::Index::Num(index)) => {
                AttributeValue::get_child_av_ids_in_order(ctx, av_id)
                    .await?
                    .get(index)
                    .copied()
            }
            Ok(jsonptr::index::Index::Next) | Err(_) => None,
        },

        // Look at child Contains edges to find the one with the right name
        PropKind::Map => AttributeValue::map_children(ctx, av_id)
            .await?
            .get(token.decoded().as_ref())
            .copied(),

        // Look at all child AVs and find the one that matches the index
        PropKind::Object => AttributeValue::object_children(ctx, av_id)
            .await?
            .get(token.decoded().as_ref())
            .copied(),

        // These cannot have children
        PropKind::Boolean
        | PropKind::Integer
        | PropKind::Json
        | PropKind::Float
        | PropKind::String => None,
    })
}

impl ValueSubscriptionPath {
    pub fn from_json_pointer(path: impl Into<String>) -> Self {
        ValueSubscriptionPath::JsonPointer(path.into())
    }
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            ValueSubscriptionPath::JsonPointer(path) => path.as_bytes(),
        }
    }
}
