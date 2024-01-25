//! This module contains the ability to construct values reflecting the latest state of a
//! [`Component`](crate::Component)'s properties.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use ulid::Ulid;

use crate::property_editor::{PropertyEditorError, PropertyEditorResult};
use crate::property_editor::{PropertyEditorPropId, PropertyEditorValueId};
use crate::{
    AttributeReadContext, AttributeValue, AttributeValueId, Component, ComponentId, DalContext,
    FuncId, Prop, PropId, StandardModel,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropertyEditorValues {
    pub root_value_id: PropertyEditorValueId,
    pub values: HashMap<PropertyEditorValueId, PropertyEditorValue>,
    pub child_values: HashMap<PropertyEditorValueId, Vec<PropertyEditorValueId>>,
}

impl PropertyEditorValues {
    pub async fn for_component(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> PropertyEditorResult<Self> {
        let mut root_value_id = None;
        let mut values = HashMap::new();
        let mut child_values: HashMap<PropertyEditorValueId, Vec<PropertyEditorValueId>> =
            HashMap::new();
        let mut work_queue = AttributeValue::list_payload_for_read_context(
            ctx,
            AttributeReadContext {
                prop_id: None,
                component_id: Some(component_id),
                ..AttributeReadContext::default()
            },
        )
        .await?;

        // We sort the work queue according to the order of every nested IndexMap. This ensures that
        // when we reconstruct the final properties data, we don't have to worry about the order things
        // appear in - they are certain to be the right order.
        let attribute_value_order: Vec<AttributeValueId> = work_queue
            .iter()
            .filter_map(|avp| avp.attribute_value.index_map())
            .flat_map(|index_map| index_map.order())
            .copied()
            .collect();
        work_queue.sort_by_cached_key(|avp| {
            attribute_value_order
                .iter()
                .position(|attribute_value_id| attribute_value_id == avp.attribute_value.id())
                .unwrap_or(0)
        });

        let overrides = AttributeValue::list_attributes_with_overridden(ctx, component_id).await?;

        for work in work_queue {
            let work_attribute_value_id = *work.attribute_value.id();

            let sockets = Component::list_input_sockets_for_attribute_value(
                ctx,
                work_attribute_value_id,
                component_id,
            )
            .await?;

            let can_be_set_by_socket: bool = !sockets.is_empty();

            let is_from_external_source = sockets.iter().any(|(_socket, has_edge)| *has_edge);

            let (controlling_func_id, controlling_attribute_value_id, controlling_func_name) =
                AttributeValue::get_controlling_func_id(ctx, work_attribute_value_id).await?;

            let is_controlled_by_intrinsic_func = controlling_func_name.starts_with("si:set")
                || controlling_func_name.starts_with("si:unset");

            let overridden = overrides
                .get(&work_attribute_value_id)
                .copied()
                .unwrap_or(false);

            values.insert(
                work_attribute_value_id.into(),
                PropertyEditorValue {
                    id: work_attribute_value_id.into(),
                    prop_id: (*work.prop.id()).into(),
                    key: work.attribute_value.key().map(Into::into),
                    value: work
                        .func_binding_return_value
                        .and_then(|f| f.value().cloned())
                        .unwrap_or(Value::Null),
                    is_from_external_source,
                    can_be_set_by_socket,
                    is_controlled_by_intrinsic_func,
                    controlling_func_id,
                    controlling_attribute_value_id,
                    overridden,
                },
            );
            if let Some(parent_id) = work.parent_attribute_value_id {
                child_values
                    .entry(parent_id.into())
                    .or_default()
                    .push(work_attribute_value_id.into());
            } else {
                root_value_id = Some(work_attribute_value_id.into());
            }
        }

        if let Some(root_value_id) = root_value_id {
            Ok(PropertyEditorValues {
                root_value_id,
                child_values,
                values,
            })
        } else {
            Err(PropertyEditorError::RootPropNotFound)
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropertyEditorValue {
    pub id: PropertyEditorValueId,
    prop_id: PropertyEditorPropId,
    pub key: Option<String>,
    value: Value,
    is_from_external_source: bool,
    can_be_set_by_socket: bool,
    is_controlled_by_intrinsic_func: bool,
    controlling_func_id: FuncId,
    controlling_attribute_value_id: AttributeValueId,
    overridden: bool,
}

impl PropertyEditorValue {
    pub fn attribute_value_id(&self) -> AttributeValueId {
        self.id.into()
    }

    pub fn value(&self) -> Value {
        self.value.clone()
    }

    pub fn prop_id(&self) -> PropId {
        self.prop_id.into()
    }

    /// Returns the [`Prop`](crate::Prop) corresponding to the "prop_id" field.
    pub async fn prop(&self, ctx: &DalContext) -> PropertyEditorResult<Prop> {
        let prop = Prop::get_by_id(ctx, &self.prop_id.into())
            .await?
            .ok_or_else(|| PropertyEditorError::PropNotFound(self.prop_id.into()))?;
        Ok(prop)
    }
}
