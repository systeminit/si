//! This module contains the [`AttributeView`] struct and its methods. This object does not exist
//! in the database.

use serde_json::Value;
use std::collections::{HashMap, VecDeque};
use telemetry::prelude::*;

use crate::{
    AttributeReadContext, AttributeValue, AttributeValueError, AttributeValueId,
    AttributeValuePayload, AttributeValueResult, DalContext, Prop, PropError, PropKind,
    StandardModel,
};

/// A generated view for an [`AttributeReadContext`] and an optional root [`AttributeValueId`](AttributeValue). The
/// requirements for the context are laid out in [`Self::new()`].
#[derive(Debug)]
pub struct AttributeView {
    /// The value that was generated from [`Self::new()`]. This can also be referred to as the
    /// "properties" or "tree" of the view.
    value: Value,
    json_pointer_for_attribute_value_id: HashMap<AttributeValueId, String>,
}

impl AttributeView {
    /// Generates an [`AttributeView`] with an [`AttributeReadContext`]and an optional root
    /// [`AttributeValueId`](AttributeValue). The context's requirements are specified in the following locations:
    ///
    /// - If the root is _not_ provided: [`AttributeValue::list_payload_for_read_context()`]
    /// - If the root is provided: [`AttributeValue::list_payload_for_read_context_and_root()`]
    ///
    /// The view is generated based on the [`AttributeValuePayloads`](AttributeValuePayload) found, including their
    /// corresponding [`Props`](Prop). Usually, the root should be provided if a view is desired for any given context
    /// and "location" in the object value. If the [`SchemaVariant`](crate::SchemaVariant) is known and you only desire
    /// to generate a view for the entire value, you do not need to provide the root.
    pub async fn new(
        ctx: &DalContext,
        attribute_read_context: AttributeReadContext,
        root_attribute_value_id: Option<AttributeValueId>,
    ) -> AttributeValueResult<Self> {
        let mut initial_work = match root_attribute_value_id {
            Some(root_attribute_value_id) => {
                AttributeValue::list_payload_for_read_context_and_root(
                    ctx,
                    root_attribute_value_id,
                    attribute_read_context,
                )
                .await?
            }
            None => {
                AttributeValue::list_payload_for_read_context(ctx, attribute_read_context).await?
            }
        };

        // When we have a parent AttributeValueId (K: AttributeValueId), we need to know where in
        // the structure we need to insert the value we are working with (V: String).
        let mut json_pointer_for_attribute_value_id: HashMap<AttributeValueId, String> =
            HashMap::new();

        // Handle scenarios where we are generating views starting anywhere other than the root
        // of a prop tree.
        let maybe_parent_attribute_value_id =
            if let Some(root_attribute_value_id) = root_attribute_value_id {
                let root_attribute_value = AttributeValue::get_by_id(ctx, &root_attribute_value_id)
                    .await?
                    .ok_or(AttributeValueError::Missing)?;
                root_attribute_value
                    .parent_attribute_value(ctx)
                    .await?
                    .map(|av| *av.id())
            } else {
                None
            };
        if let Some(parent_attribute_value_id) = maybe_parent_attribute_value_id {
            json_pointer_for_attribute_value_id.insert(parent_attribute_value_id, "".to_string());
        }

        // We sort the work queue according to the order of every nested IndexMap. This ensures that
        // when we reconstruct the final shape, we don't have to worry about the order that things
        // appear in.
        let attribute_value_order: Vec<AttributeValueId> = initial_work
            .iter()
            .filter_map(|avp| avp.attribute_value.index_map())
            .flat_map(|index_map| index_map.order())
            .copied()
            .collect();
        initial_work.sort_by_cached_key(|avp| {
            attribute_value_order
                .iter()
                .position(|attribute_value_id| attribute_value_id == avp.attribute_value.id())
                .unwrap_or(0)
        });

        // We need the work queue to be a VecDeque so we can pop elements off of the front
        // as it's supposed to be a queue, not a stack.
        let mut work_queue: VecDeque<AttributeValuePayload> = VecDeque::from(initial_work);

        let mut properties = serde_json::json![{}];
        let mut root_stack: Vec<(Option<AttributeValueId>, String)> =
            vec![(maybe_parent_attribute_value_id, "".to_string())];

        while !work_queue.is_empty() {
            let mut unprocessed: Vec<AttributeValuePayload> = vec![];
            if root_stack.is_empty() {
                warn!(
                    "Unexpected empty root stack with work_queue: {:?}",
                    &work_queue
                );
                break;
            }
            let (root_id, json_pointer) = root_stack.pop().ok_or_else(|| {
                error!(
                    "unexpected empty root stack, current work queue state: {:?}",
                    work_queue
                );
                AttributeValueError::UnexpectedEmptyRootStack
            })?;

            while let Some(AttributeValuePayload {
                prop,
                func_binding_return_value,
                attribute_value,
                parent_attribute_value_id,
            }) = work_queue.pop_front()
            {
                if let Some(func_binding_return_value) = func_binding_return_value {
                    if let Some(found_value) = func_binding_return_value.value() {
                        if root_id == parent_attribute_value_id {
                            let insertion_pointer =
                                if let Some(parent_avi) = parent_attribute_value_id {
                                    match json_pointer_for_attribute_value_id.get(&parent_avi) {
                                        Some(ptr) => ptr.clone(),
                                        // A `None` here would mean that we're trying to process a child before we've handled its parent,
                                        // and that shouldn't be possible given how we're going through the work_queue.
                                        None => unreachable!(),
                                    }
                                } else {
                                    // After we've processed the "root" property, we shouldn't hit this case any more.
                                    json_pointer.clone()
                                };

                            let write_location = match properties.pointer_mut(&insertion_pointer) {
                                Some(write_location) => write_location,
                                None => {
                                    return Err(AttributeValueError::BadJsonPointer(
                                        insertion_pointer.clone(),
                                        properties.to_string(),
                                    ));
                                }
                            };
                            let next_json_pointer =
                                if let Some(object) = write_location.as_object_mut() {
                                    if let Some(key) = attribute_value.key() {
                                        object.insert(key.to_string(), found_value.clone());
                                        format!("{insertion_pointer}/{key}")
                                    } else {
                                        object.insert(prop.name().to_string(), found_value.clone());
                                        format!("{}/{}", insertion_pointer, prop.name())
                                    }
                                } else if let Some(array) = write_location.as_array_mut() {
                                    // This code can just push, because we ordered the work queue above.
                                    // Magic!
                                    array.push(found_value.clone());
                                    format!("{}/{}", insertion_pointer, array.len() - 1)
                                } else {
                                    // Note: this shouldn't ever actually get used.
                                    insertion_pointer.to_string()
                                };
                            // Record the json pointer path to this specific attribute value's location.
                            json_pointer_for_attribute_value_id
                                .insert(*attribute_value.id(), next_json_pointer.clone());

                            match prop.kind() {
                                &PropKind::Object | &PropKind::Array | &PropKind::Map => {
                                    root_stack
                                        .push((Some(*attribute_value.id()), next_json_pointer));
                                }
                                _ => {}
                            }
                        } else {
                            unprocessed.push(AttributeValuePayload::new(
                                prop,
                                Some(func_binding_return_value),
                                attribute_value,
                                parent_attribute_value_id,
                            ));
                        }
                    }
                }
            }
            work_queue = VecDeque::from(unprocessed);
        }

        if let Some(root_attribute_value_id) = root_attribute_value_id {
            let root_json_pointer = match json_pointer_for_attribute_value_id
                .get(&root_attribute_value_id)
            {
                Some(pointer) => pointer,
                None => {
                    let root_attribute_value =
                        AttributeValue::get_by_id(ctx, &root_attribute_value_id)
                            .await?
                            .ok_or_else(|| {
                                AttributeValueError::NotFound(
                                    root_attribute_value_id,
                                    *ctx.visibility(),
                                )
                            })?;
                    let root_prop = Prop::get_by_id(ctx, &root_attribute_value.context.prop_id())
                        .await?
                        .ok_or_else(|| {
                            PropError::NotFound(
                                root_attribute_value.context.prop_id(),
                                *ctx.visibility(),
                            )
                        })
                        .map_err(Box::new)?;
                    debug!("\
                        likely tried to build an attribute view for an attribute value that is unset, \
                        so the \"properties\" object is empty ({:?}), and does not contain a key matching \
                        our prop's name (root attribute value ({:?}) and root prop ({:?}))", properties, root_attribute_value, root_prop
                    );
                    return Ok(Self {
                        value: Value::Null,
                        json_pointer_for_attribute_value_id,
                    });
                }
            };

            let properties = properties
                .pointer(root_json_pointer)
                .ok_or(AttributeValueError::NoValueForJsonPointer)?;
            return Ok(Self {
                value: properties.to_owned(),
                json_pointer_for_attribute_value_id,
            });
        }

        Ok(Self {
            value: properties.to_owned(),
            json_pointer_for_attribute_value_id,
        })
    }

    pub fn value(&self) -> &serde_json::Value {
        &self.value
    }

    pub fn json_pointers_for_attribute_value_id(&self) -> &HashMap<AttributeValueId, String> {
        &self.json_pointer_for_attribute_value_id
    }
}
