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
    AttributePrototype,
    AttributeValue,
    AttributeValueId,
    DalContext,
    Func,
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

impl std::fmt::Display for AttributePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttributePath::JsonPointer(path) => write!(f, "{path}"),
        }
    }
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

    /// Resolves the attribute value at the JSON pointer, if one exists.
    ///
    /// Returns None if it cannot be found.
    pub async fn resolve(
        &self,
        ctx: &DalContext,
        av_id: AttributeValueId,
    ) -> AttributeValueResult<Option<AttributeValueId>> {
        match self {
            AttributePath::JsonPointer(pointer) => {
                let pointer = jsonptr::Pointer::parse(pointer)
                    .map_err(|err| AttributeValueError::JsonptrParseError(pointer.clone(), err))?;
                resolve_json_pointer(ctx, av_id, pointer).await
            }
        }
    }

    /// Gets the attribute value at the JSON pointer. After this is complete, the AV can be
    /// set to an explicit value.
    ///
    /// Any parents that do not exist will be created. Any parents that are currently set to
    /// default values will become explicit values.
    pub async fn vivify(
        &self,
        ctx: &DalContext,
        av_id: AttributeValueId,
    ) -> AttributeValueResult<AttributeValueId> {
        match self {
            AttributePath::JsonPointer(pointer) => {
                let pointer = jsonptr::Pointer::parse(pointer)
                    .map_err(|err| AttributeValueError::JsonptrParseError(pointer.clone(), err))?;
                vivify_json_pointer(ctx, av_id, pointer).await
            }
        }
    }

    /// Validate that the JSON pointer can refer to something real under the given prop.
    ///
    /// Errors if the JSON pointer refers to a missing field under an object.
    pub async fn validate(&self, ctx: &DalContext, prop_id: PropId) -> AttributeValueResult<()> {
        match self {
            AttributePath::JsonPointer(pointer) => {
                let pointer = jsonptr::Pointer::parse(pointer)
                    .map_err(|err| AttributeValueError::JsonptrParseError(pointer.clone(), err))?;
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
    mut parent_id: AttributeValueId,
    pointer: &jsonptr::Pointer,
) -> AttributeValueResult<Option<AttributeValueId>> {
    // Go through each segment of the JSON pointer (e.g. /foo/bar/0 = foo, bar, 0)
    // and look for its child
    for token in pointer {
        let kind = AttributeValue::prop_kind(ctx, parent_id).await?;
        let child_av_id = match kind {
            // Look up array index in ordering node
            PropKind::Array => match token.to_index() {
                Ok(jsonptr::index::Index::Num(index)) => {
                    AttributeValue::get_child_av_ids_in_order(ctx, parent_id)
                        .await?
                        .get(index)
                        .copied()
                }
                Ok(jsonptr::index::Index::Next) | Err(_) => None,
            },

            // Look at child Contains edges to find the one with the right name
            PropKind::Map => {
                AttributeValue::map_child_opt(ctx, parent_id, token.decoded().as_ref()).await?
            }

            // The object's children AVs will already have been vivified. Just grab the right one.
            PropKind::Object => {
                AttributeValue::object_child_opt(ctx, parent_id, token.decoded().as_ref()).await?
            }

            // These cannot have children
            PropKind::Boolean
            | PropKind::Integer
            | PropKind::Json
            | PropKind::Float
            | PropKind::String => {
                return Err(AttributeValueError::NoChildWithName(
                    parent_id,
                    token.decoded().into_owned(),
                ))?;
            }
        };
        let Some(child_av_id) = child_av_id else {
            return Ok(None);
        };
        parent_id = child_av_id;
    }
    Ok(Some(parent_id))
}

/// Gets the attribute value at the JSON pointer, or creates it if it doesn't exist.
/// Returns an error if an AV cannot be created.
async fn vivify_json_pointer(
    ctx: &DalContext,
    mut parent_id: AttributeValueId,
    pointer: &jsonptr::Pointer,
) -> AttributeValueResult<AttributeValueId> {
    // Go through each segment of the JSON pointer (e.g. /foo/bar/0 = foo, bar, 0)
    // and look for its child. If it doesn't exist, create it.
    for token in pointer {
        //
        // Ensure the parent is si:setObject/si:setArray/si:setMap, so it can have children.
        //
        let kind = AttributeValue::prop_kind(ctx, parent_id).await?;
        let prototype_id = AttributeValue::prototype_id(ctx, parent_id).await?;
        let func_id = AttributePrototype::func_id(ctx, prototype_id).await?;
        let intrinsic = Func::intrinsic_kind(ctx, func_id).await?;
        // If it's not si:setObject/si:setArray/si:setMap, fix it!
        if intrinsic != Some(kind.intrinsic_set_func()) {
            if AttributePrototype::is_dynamic(ctx, prototype_id).await? {
                // You can't modify children of dynamic values on the component itself--only
                // if they are defaults from the prototype. i.e. if /domain/Foo is a subscription,
                // you can't modify /domain/Foo/Bar. If you want to do this, you have to clear
                // the subscription first. We don't mind, however, if the dynamic value comes
                // from the default (from the prop); that's just "override the default".
                if AttributeValue::component_prototype_id(ctx, parent_id)
                    .await?
                    .is_some()
                {
                    return Err(AttributeValueError::CannotSetChildOfDynamicValue(parent_id));
                }
                // If it's a dynamic default (or socket connections), we have to clear the
                // existing value.
                AttributeValue::update(ctx, parent_id, kind.empty_value()).await?;
            } else {
                // If it's si:unset (value=None), we set to empty value instead.
                // NOTE: we use set_value() here because we want to preserve default values!
                // This matches what AttributeValue::vivify_value_and_parent_values() does.
                AttributeValue::set_value(ctx, parent_id, kind.empty_value()).await?;
            }
        }

        //
        // Find or create the child!
        //
        parent_id =
            match kind {
                PropKind::Array => {
                    // Make sure the user asked to append (can't insert at an index that doesn't exist)
                    let elements =
                        AttributeValue::get_child_av_ids_in_order(ctx, parent_id).await?;
                    match token.to_index().map_err(|err| {
                        AttributeValueError::JsonptrParseIndexError(pointer.to_string(), err)
                    })? {
                        // If it's - or the next index (len+1), we can append!
                        jsonptr::index::Index::Next => {
                            AttributeValue::insert(ctx, parent_id, None, None).await?
                        }
                        jsonptr::index::Index::Num(index) if index == elements.len() => {
                            AttributeValue::insert(ctx, parent_id, None, None).await?
                        }

                        // If it's not the last index + 1, we retrieve the existing one
                        jsonptr::index::Index::Num(index) => elements.get(index).copied().ok_or(
                            AttributeValueError::IndexOutOfRange(index, parent_id, elements.len()),
                        )?,
                    }
                }

                // Create empty map entry with given name
                PropKind::Map => {
                    let name = token.decoded();
                    match AttributeValue::map_child_opt(ctx, parent_id, name.as_ref()).await? {
                        Some(child_av_id) => child_av_id,
                        None => {
                            AttributeValue::insert(ctx, parent_id, None, Some(name.into_owned()))
                                .await?
                        }
                    }
                }

                // Get the matching child AV (all possible child AVs must already exist)
                PropKind::Object => {
                    let name = token.decoded();
                    AttributeValue::object_child(ctx, parent_id, name.as_ref()).await?
                }

                // These cannot have children
                PropKind::Boolean
                | PropKind::Integer
                | PropKind::Json
                | PropKind::Float
                | PropKind::String => {
                    return Err(AttributeValueError::NoChildWithName(
                        parent_id,
                        token.decoded().into_owned(),
                    ))?;
                }
            };
    }
    Ok(parent_id)
}

// Validate that a JSON pointer *could* be used to access a child of the given AV (that it
// has valid prop names and indices).
async fn validate_json_pointer(
    ctx: &DalContext,
    mut parent_id: PropId,
    pointer: &jsonptr::Pointer,
) -> AttributeValueResult<()> {
    // Go through each segment of the JSON pointer (e.g. /foo/bar/0 = foo, bar, 0)
    // and validate that the child prop exists or that it is a valid index and not -
    for token in pointer {
        let prop = Prop::get_by_id(ctx, parent_id).await?;
        parent_id = match prop.kind {
            // Look up array index in ordering node
            PropKind::Array => match token.to_index().map_err(|err| {
                AttributeValueError::JsonptrParseIndexError(pointer.to_string(), err)
            })? {
                jsonptr::index::Index::Num(_) => Prop::element_prop_id(ctx, parent_id).await?,

                jsonptr::index::Index::Next => {
                    return Err(PropError::CannotSubscribeToNextElement(
                        parent_id,
                        pointer.to_string(),
                    ))?;
                }
            },

            // All strings are valid map keys
            PropKind::Map => Prop::element_prop_id(ctx, parent_id).await?,

            // The key must be a valid child prop name
            PropKind::Object => {
                Prop::child_prop_id(ctx, parent_id.into(), token.decoded().as_ref()).await?
            }

            // These cannot have children
            PropKind::Boolean
            | PropKind::Integer
            | PropKind::Json
            | PropKind::Float
            | PropKind::String => {
                return Err(PropError::ChildPropNotFoundByName(
                    parent_id.into(),
                    token.decoded().into_owned(),
                ))?;
            }
        }
    }
    Ok(())
}
