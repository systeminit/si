//! This module contains the ability to construct values reflecting the latest state of a
//! [`Component`](crate::Component)'s properties.

use petgraph::prelude::{EdgeRef, NodeIndex};
use petgraph::Direction;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, VecDeque};

use crate::property_editor::PropertyEditorResult;
use crate::property_editor::{PropertyEditorPropId, PropertyEditorValueId};
use crate::workspace_snapshot::edge_weight::EdgeWeightKind;

use crate::{
    AttributeValue, AttributeValueId, Component, ComponentId, DalContext, FuncId, Prop, PropId,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropertyEditorValues {
    pub root_value_id: PropertyEditorValueId,
    pub values: HashMap<PropertyEditorValueId, PropertyEditorValue>,
    pub child_values: HashMap<PropertyEditorValueId, Vec<PropertyEditorValueId>>,
}

impl PropertyEditorValues {
    pub async fn assemble(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> PropertyEditorResult<Self> {
        let mut values = HashMap::new();
        let mut child_values = HashMap::new();

        // Get the root attribute value and load it into the work queue.
        let root_attribute_value_id = Component::root_attribute_value_id(ctx, component_id).await?;
        let root_property_editor_value_id = PropertyEditorValueId::from(root_attribute_value_id);
        let root_prop_id = AttributeValue::prop(ctx, root_attribute_value_id).await?;
        let root_attribute_value = AttributeValue::get_by_id(ctx, root_attribute_value_id).await?;

        let controlling_func_id = FuncId::NONE;

        values.insert(
            root_property_editor_value_id,
            PropertyEditorValue {
                id: root_property_editor_value_id,
                prop_id: root_prop_id.into(),
                key: None,
                value: root_attribute_value
                    .value(ctx)
                    .await?
                    .unwrap_or(Value::Null),
                // TODO(nick): restore all these fields below.
                is_from_external_source: false,
                can_be_set_by_socket: false,
                is_controlled_by_intrinsic_func: true,
                controlling_func_id,
                controlling_attribute_value_id: root_property_editor_value_id.into(),
                overridden: false,
            },
        );

        let mut work_queue =
            VecDeque::from([(root_attribute_value_id, root_property_editor_value_id)]);
        while let Some((attribute_value_id, property_editor_value_id)) = work_queue.pop_front() {
            // Collect all child attribute values.
            let mut cache: Vec<(AttributeValueId, Option<String>)> = Vec::new();
            {
                let workspace_snapshot = ctx.workspace_snapshot()?.read().await;

                let child_attribute_values_with_keys: Vec<(NodeIndex, Option<String>)> =
                    workspace_snapshot
                        .edges_directed(attribute_value_id, Direction::Outgoing)?
                        .filter_map(|edge_ref| {
                            if let EdgeWeightKind::Contain(key) = edge_ref.weight().kind() {
                                Some((edge_ref.target(), key.to_owned()))
                            } else {
                                None
                            }
                        })
                        .collect();

                // NOTE(nick): this entire function is likely wasteful. Zack and Jacob, have mercy on me.
                for (child_attribute_value_node_index, key) in child_attribute_values_with_keys {
                    let child_attribute_value_node_weight =
                        workspace_snapshot.get_node_weight(child_attribute_value_node_index)?;
                    let content =
                        child_attribute_value_node_weight.get_attribute_value_node_weight()?;
                    cache.push((content.id().into(), key));
                }
            }

            // Now that we have the child props, prepare the property editor props and load the work queue.
            let mut child_property_editor_value_ids = Vec::new();
            for (child_attribute_value_id, key) in cache {
                // NOTE(nick): we already have the node weight, but I believe we still want to use "get_by_id" to
                // get the content from the store. Perhaps, there's a more efficient way that we can do this.
                let child_attribute_value =
                    AttributeValue::get_by_id(ctx, child_attribute_value_id).await?;
                let prop_id_for_child_attribute_value =
                    AttributeValue::prop(ctx, child_attribute_value_id).await?;
                let child_property_editor_value_id =
                    PropertyEditorValueId::from(child_attribute_value_id);

                let child_property_editor_value = PropertyEditorValue {
                    id: child_property_editor_value_id,
                    prop_id: prop_id_for_child_attribute_value.into(),
                    key,
                    value: child_attribute_value
                        .value(ctx)
                        .await?
                        .unwrap_or(Value::Null),
                    // TODO(nick): restore all the fields below.
                    is_from_external_source: false,
                    can_be_set_by_socket: false,
                    is_controlled_by_intrinsic_func: true,
                    controlling_func_id,
                    controlling_attribute_value_id: child_property_editor_value_id.into(),
                    overridden: false,
                };

                // Load the work queue with the child attribute value.
                work_queue.push_back((child_attribute_value_id, child_property_editor_value.id));

                // Cache the child property editor values to eventually insert into the child property editor values map.
                child_property_editor_value_ids.push(child_property_editor_value.id);

                // Insert the child property editor value into the values map.
                values.insert(child_property_editor_value.id, child_property_editor_value);
            }
            child_values.insert(property_editor_value_id, child_property_editor_value_ids);
        }

        Ok(PropertyEditorValues {
            root_value_id: root_property_editor_value_id,
            child_values,
            values,
        })
    }

    /// Finds the [`AttributeValueId`](AttributeValue) for a given [`PropId`](Prop).
    ///
    /// This is useful for non-maps and non-array [`Props`](Prop).
    pub fn find_by_prop_id(&self, prop_id: PropId) -> Option<AttributeValueId> {
        self.values
            .iter()
            .find(|(_, property_editor_value)| property_editor_value.prop_id() == prop_id)
            .map(|(_, found_property_editor_value)| {
                found_property_editor_value.attribute_value_id()
            })
    }

    /// Finds the [`AttributeValueId`](AttributeValue) and the [`Value`] corresponding to it for a
    /// given [`PropId`](Prop).
    ///
    /// This is useful for non-maps and non-array [`Props`](Prop).
    pub fn find_with_value_by_prop_id(&self, prop_id: PropId) -> Option<(Value, AttributeValueId)> {
        self.values
            .iter()
            .find(|(_, property_editor_value)| property_editor_value.prop_id() == prop_id)
            .map(|(_, found_property_editor_value)| {
                (
                    found_property_editor_value.value.to_owned(),
                    found_property_editor_value.attribute_value_id(),
                )
            })
    }

    /// Lists the [`AttributeValueIds`](AttributeValue) for a given [`PropId`](Prop).
    ///
    /// This is useful for map and array [`Props`](Prop).
    pub fn list_by_prop_id(&self, prop_id: PropId) -> Vec<AttributeValueId> {
        self.values
            .iter()
            .filter_map(|(_, property_editor_value)| {
                if property_editor_value.prop_id() == prop_id {
                    Some(property_editor_value.attribute_value_id())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Lists the [`AttributeValueIds`](AttributeValue) and the [`Values`] corresponding to them for
    /// a given [`PropId`](Prop).
    ///
    /// This is useful for map and array [`Props`](Prop).
    pub fn list_with_values_by_prop_id(&self, prop_id: PropId) -> Vec<(Value, AttributeValueId)> {
        self.values
            .iter()
            .filter_map(|(_, property_editor_value)| {
                if property_editor_value.prop_id() == prop_id {
                    Some((
                        property_editor_value.value(),
                        property_editor_value.attribute_value_id(),
                    ))
                } else {
                    None
                }
            })
            .collect()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropertyEditorValue {
    pub id: PropertyEditorValueId,
    pub prop_id: PropertyEditorPropId,
    pub key: Option<String>,
    pub value: Value,
    pub is_from_external_source: bool,
    pub can_be_set_by_socket: bool,
    pub is_controlled_by_intrinsic_func: bool,
    pub controlling_func_id: FuncId,
    pub controlling_attribute_value_id: AttributeValueId,
    pub overridden: bool,
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
        let prop = Prop::get_by_id(ctx, self.prop_id.into()).await?;
        Ok(prop)
    }
}

impl postgres_types::ToSql for PropertyEditorValues {
    fn to_sql(
        &self,
        ty: &postgres_types::Type,
        out: &mut postgres_types::private::BytesMut,
    ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>>
    where
        Self: Sized,
    {
        let json = serde_json::to_value(self)?;
        postgres_types::ToSql::to_sql(&json, ty, out)
    }

    fn accepts(ty: &postgres_types::Type) -> bool
    where
        Self: Sized,
    {
        ty == &postgres_types::Type::JSONB
    }

    fn to_sql_checked(
        &self,
        ty: &postgres_types::Type,
        out: &mut postgres_types::private::BytesMut,
    ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
        let json = serde_json::to_value(self)?;
        postgres_types::ToSql::to_sql(&json, ty, out)
    }
}
