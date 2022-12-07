//! This module contains the ability to construct values reflecting the latest state of a
//! [`Component`](crate::Component)'s properties.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{cmp::Ordering, collections::HashMap};

use crate::attribute::value::FuncWithPrototypeContext;
use crate::property_editor::{PropertyEditorError, PropertyEditorResult};
use crate::property_editor::{PropertyEditorPropId, PropertyEditorValueId};
use crate::{
    AttributeReadContext, AttributeValue, AttributeValueId, Component, ComponentId, DalContext,
    Prop, PropId, StandardModel,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropertyEditorValues {
    root_value_id: PropertyEditorValueId,
    pub values: HashMap<PropertyEditorValueId, PropertyEditorValue>,
    child_values: HashMap<PropertyEditorValueId, Vec<PropertyEditorValueId>>,
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

        for work in work_queue {
            let work_attribute_value_id = *work.attribute_value.id();

            let sockets = Component::list_connected_input_sockets_for_attribute_value(
                ctx,
                work_attribute_value_id,
                component_id,
            )
            .await?;
            let is_from_external_source = !sockets.is_empty();

            values.insert(
                i64::from(work_attribute_value_id).into(),
                PropertyEditorValue {
                    id: i64::from(work_attribute_value_id).into(),
                    prop_id: i64::from(*work.prop.id()).into(),
                    key: work.attribute_value.key().map(Into::into),
                    value: work
                        .func_binding_return_value
                        .and_then(|f| f.value().cloned())
                        .unwrap_or(Value::Null),
                    is_from_external_source,
                    func: work.func_with_prototype_context,
                },
            );
            if let Some(parent_id) = work.parent_attribute_value_id {
                child_values
                    .entry(i64::from(parent_id).into())
                    .or_default()
                    .push(i64::from(work_attribute_value_id).into());
            } else {
                root_value_id = Some(i64::from(work_attribute_value_id).into());
            }
        }

        // Note: hackish ordering to ensure consistency in the frontend
        for value in child_values.values_mut() {
            value.sort_by(|a, b| {
                let a = &values[a];
                let b = &values[b];
                match a.prop_id.cmp(&b.prop_id) {
                    Ordering::Equal => a.id.cmp(&b.id),
                    ordering => ordering,
                }
            });
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
    id: PropertyEditorValueId,
    prop_id: PropertyEditorPropId,
    key: Option<String>,
    value: Value,
    is_from_external_source: bool,
    func: FuncWithPrototypeContext,
}

impl PropertyEditorValue {
    pub fn value(&self) -> Value {
        self.value.clone()
    }

    /// Returns the [`Prop`](crate::Prop) corresponding to the "prop_id" field.
    pub async fn prop(&self, ctx: &DalContext) -> PropertyEditorResult<Prop> {
        // FIXME(nick): implement from.
        let unchecked_id: i64 = self.prop_id.into();
        let prop_id: PropId = unchecked_id.into();
        let prop = Prop::get_by_id(ctx, &prop_id)
            .await?
            .ok_or(PropertyEditorError::PropNotFound(prop_id))?;
        Ok(prop)
    }
}
