use serde::{
    Deserialize,
    Serialize,
};
use si_id::PropId;
use strum::EnumDiscriminants;

use super::value::{
    AttributeValueError,
    AttributeValueResult,
};
use crate::{
    AttributeValue,
    AttributeValueId,
    DalContext,
    Prop,
    PropKind,
    prop::PropError,
};

/// A path to an attribute value, relative to its root value
/// This type is postcard serialized and new enum variants *MUST* be added to the end *ONLY*.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, EnumDiscriminants)]
#[strum_discriminants(derive(Hash, Serialize, Deserialize, strum::EnumIter, strum::Display))]
pub enum AttributePath {
    /// A JSON pointer (e.g. `/domain/PolicyDocument/Statements/0/Operation`)
    JsonPointer(String),
}

impl AttributePath {
    pub fn from_json_pointer(path: impl Into<String>) -> Self {
        AttributePath::JsonPointer(path.into())
    }

    pub fn as_bytes(&self) -> &[u8] {
        match self {
            AttributePath::JsonPointer(path) => path.as_bytes(),
        }
    }

    pub async fn resolve(
        &self,
        ctx: &DalContext,
        av_id: AttributeValueId,
    ) -> AttributeValueResult<Option<AttributeValueId>> {
        match self {
            AttributePath::JsonPointer(pointer) => {
                let pointer = jsonptr::Pointer::parse(pointer)?;
                resolve_json_pointer(ctx, av_id, pointer).await
            }
        }
    }

    pub async fn vivify(
        &self,
        ctx: &DalContext,
        av_id: AttributeValueId,
    ) -> AttributeValueResult<AttributeValueId> {
        match self {
            AttributePath::JsonPointer(pointer) => {
                let pointer = jsonptr::Pointer::parse(pointer)?;
                vivify_json_pointer(ctx, av_id, pointer).await
            }
        }
    }

    pub async fn validate(&self, ctx: &DalContext, prop_id: PropId) -> AttributeValueResult<()> {
        match self {
            AttributePath::JsonPointer(pointer) => {
                let pointer = jsonptr::Pointer::parse(pointer)?;
                validate_json_pointer(ctx, prop_id, pointer).await
            }
        }
    }

    /// Returns true if the `possible_parent_path` is included but doesn't completely match this path
    pub fn is_under(&self, possible_parent_path: &Self) -> bool {
        match (self, possible_parent_path) {
            (Self::JsonPointer(self_path), Self::JsonPointer(parent_path)) => {
                self_path.starts_with(parent_path)
                    && self_path.as_bytes().get(parent_path.len()) == Some(&b'/')
            }
        }
    }
}

// Gets the attribute value at the JSON pointer, or None if it cannot be found.
// Returns None if the AV points at non-existent props.
async fn resolve_json_pointer(
    ctx: &DalContext,
    mut av_id: AttributeValueId,
    pointer: &jsonptr::Pointer,
) -> AttributeValueResult<Option<AttributeValueId>> {
    // Go through each segment of the JSON pointer (e.g. /foo/bar/0 = foo, bar, 0)
    // and look for its child
    for token in pointer {
        let prop = AttributeValue::prop(ctx, av_id).await?;
        let child_av_id = match prop.kind {
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
            PropKind::Map => {
                AttributeValue::map_child_opt(ctx, av_id, token.decoded().as_ref()).await?
            }

            // The object's children AVs will already have been vivified. Just grab the right one.
            PropKind::Object => {
                AttributeValue::object_child_opt(ctx, av_id, token.decoded().as_ref()).await?
            }

            // These cannot have children
            PropKind::Boolean
            | PropKind::Integer
            | PropKind::Json
            | PropKind::Float
            | PropKind::String => {
                return Err(AttributeValueError::NoChildWithName(
                    av_id,
                    token.decoded().into_owned(),
                ))?;
            }
        };
        let Some(child_av_id) = child_av_id else {
            return Ok(None);
        };
        av_id = child_av_id;
    }
    Ok(Some(av_id))
}

// Gets the attribute value at the JSON pointer, or creates it if it doesn't exist.
// Returns an error if an AV cannot be created.
async fn vivify_json_pointer(
    ctx: &DalContext,
    mut av_id: AttributeValueId,
    pointer: &jsonptr::Pointer,
) -> AttributeValueResult<AttributeValueId> {
    // Go through each segment of the JSON pointer (e.g. /foo/bar/0 = foo, bar, 0)
    // and look for its child. If it doesn't exist, create it.
    for token in pointer {
        let prop = AttributeValue::prop(ctx, av_id).await?;
        av_id =
            match prop.kind {
                PropKind::Array => {
                    // Make sure the user asked to append (can't insert at an index that doesn't exist)
                    let elements = AttributeValue::get_child_av_ids_in_order(ctx, av_id).await?;
                    match token.to_index()? {
                        // If it's - or the next index (len+1), we can append!
                        jsonptr::index::Index::Next => {
                            AttributeValue::insert(ctx, av_id, None, None).await?
                        }
                        jsonptr::index::Index::Num(index) if index == elements.len() => {
                            AttributeValue::insert(ctx, av_id, None, None).await?
                        }

                        // If it's not the last index + 1, we retrieve the existing one
                        jsonptr::index::Index::Num(index) => elements.get(index).copied().ok_or(
                            AttributeValueError::IndexOutOfRange(index, av_id, elements.len()),
                        )?,
                    }
                }

                // Create empty map entry with given name
                PropKind::Map => {
                    let name = token.decoded();
                    match AttributeValue::map_child_opt(ctx, av_id, name.as_ref()).await? {
                        Some(child_av_id) => child_av_id,
                        None => {
                            AttributeValue::insert(ctx, av_id, None, Some(name.into_owned()))
                                .await?
                        }
                    }
                }

                // Get the matching child AV (all possible child AVs must already exist)
                PropKind::Object => {
                    let name = token.decoded();
                    AttributeValue::object_child(ctx, av_id, name.as_ref()).await?
                }

                // These cannot have children
                PropKind::Boolean
                | PropKind::Integer
                | PropKind::Json
                | PropKind::Float
                | PropKind::String => {
                    return Err(AttributeValueError::NoChildWithName(
                        av_id,
                        token.decoded().into_owned(),
                    ))?;
                }
            }
    }
    Ok(av_id)
}

// Validate that a JSON pointer *could* be used to access a child of the given AV (that it
// has valid prop names and indices).
async fn validate_json_pointer(
    ctx: &DalContext,
    mut prop_id: PropId,
    pointer: &jsonptr::Pointer,
) -> AttributeValueResult<()> {
    // Go through each segment of the JSON pointer (e.g. /foo/bar/0 = foo, bar, 0)
    // and validate that the child prop exists or that it is a valid index and not -
    for token in pointer {
        let prop = Prop::get_by_id(ctx, prop_id).await?;
        prop_id = match prop.kind {
            // Look up array index in ordering node
            PropKind::Array => match token.to_index()? {
                jsonptr::index::Index::Num(_) => Prop::element_prop_id(ctx, prop_id).await?,

                jsonptr::index::Index::Next => {
                    return Err(PropError::CannotSubscribeToNextElement(
                        prop_id,
                        pointer.to_string(),
                    ))?;
                }
            },

            // All strings are valid map keys
            PropKind::Map => Prop::element_prop_id(ctx, prop_id).await?,

            // The key must be a valid child prop name
            PropKind::Object => {
                Prop::child_prop_id(ctx, prop_id.into(), token.decoded().as_ref()).await?
            }

            // These cannot have children
            PropKind::Boolean
            | PropKind::Integer
            | PropKind::Json
            | PropKind::Float
            | PropKind::String => {
                return Err(PropError::ChildPropNotFoundByName(
                    prop_id.into(),
                    token.decoded().into_owned(),
                ))?;
            }
        }
    }
    Ok(())
}
