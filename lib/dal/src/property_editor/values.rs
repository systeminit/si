//! This module contains the ability to construct values reflecting the latest state of a
//! [`Component`](crate::Component)'s properties.

use std::collections::{HashMap, HashSet, VecDeque};

use petgraph::prelude::NodeIndex;
use petgraph::Direction;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::attribute::value::AttributeValueError;
use crate::component::ControllingFuncData;
use crate::property_editor::PropertyEditorResult;
use crate::property_editor::{PropertyEditorPropId, PropertyEditorValueId};
use crate::validation::{ValidationOutput, ValidationOutputNode};
use crate::workspace_snapshot::edge_weight::EdgeWeightKind;
use crate::{
    AttributeValue, AttributeValueId, Component, ComponentId, DalContext, InputSocketId, Prop,
    PropId,
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
        let component = Component::get_by_id(ctx, component_id).await?;

        let connected_sockets_on_component: HashSet<InputSocketId> = component
            .incoming_connections(ctx)
            .await?
            .iter()
            .map(|c| c.to_input_socket_id)
            .collect();

        let controlling_ancestors_for_av_id =
            Component::list_av_controlling_func_ids_for_id(ctx, component_id).await?;

        let mut values = HashMap::new();
        let mut child_values = HashMap::new();

        // Get the root attribute value and load it into the work queue.
        let root_attribute_value_id = Component::root_attribute_value_id(ctx, component_id).await?;
        let root_property_editor_value_id = PropertyEditorValueId::from(root_attribute_value_id);
        let root_prop_id =
            AttributeValue::prop_id_for_id_or_error(ctx, root_attribute_value_id).await?;
        let root_attribute_value = AttributeValue::get_by_id(ctx, root_attribute_value_id).await?;

        let validation =
            ValidationOutputNode::find_for_attribute_value_id(ctx, root_attribute_value_id)
                .await?
                .map(|node| node.validation);

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
                validation,
                is_from_external_source: false,
                can_be_set_by_socket: false,
                is_controlled_by_dynamic_func: false,
                is_controlled_by_ancestor: false,
                overridden: false,
            },
        );

        let workspace_snapshot = ctx.workspace_snapshot()?;
        let mut work_queue =
            VecDeque::from([(root_attribute_value_id, root_property_editor_value_id)]);

        while let Some((attribute_value_id, property_editor_value_id)) = work_queue.pop_front() {
            // Collect all child attribute values.
            let mut cache: Vec<(AttributeValueId, Option<String>)> = Vec::new();
            {
                let mut child_attribute_values_with_keys_by_id: HashMap<
                    AttributeValueId,
                    (NodeIndex, Option<String>),
                > = HashMap::new();

                for (edge_weight, _, target_idx) in workspace_snapshot
                    .edges_directed(attribute_value_id, Direction::Outgoing)
                    .await?
                {
                    if let EdgeWeightKind::Contain(key) = edge_weight.kind() {
                        let child_id = workspace_snapshot
                            .get_node_weight(target_idx)
                            .await?
                            .id()
                            .into();

                        child_attribute_values_with_keys_by_id
                            .insert(child_id, (target_idx, key.to_owned()));
                    }
                }

                let maybe_ordering =
                    AttributeValue::get_child_av_ids_for_ordered_parent(ctx, attribute_value_id)
                        .await
                        .ok();

                // Ideally every attribute value with children is connected via an ordering node
                // We don't error out on ordering not existing here because we don't have that
                // guarantee. If that becomes a certainty we should fail on maybe_ordering==None.
                for av_id in maybe_ordering.unwrap_or_else(|| {
                    child_attribute_values_with_keys_by_id
                        .keys()
                        .cloned()
                        .collect()
                }) {
                    let (child_attribute_value_node_index, key) =
                        &child_attribute_values_with_keys_by_id[&av_id];
                    let child_attribute_value_node_weight = workspace_snapshot
                        .get_node_weight(*child_attribute_value_node_index)
                        .await?;
                    let content =
                        child_attribute_value_node_weight.get_attribute_value_node_weight()?;
                    cache.push((content.id().into(), key.clone()));
                }
            }

            // Now that we have the child props, prepare the property editor props and load the work queue.
            let mut child_property_editor_value_ids = Vec::new();
            for (child_av_id, key) in cache {
                // NOTE(nick): we already have the node weight, but I believe we still want to use "get_by_id" to
                // get the content from the store. Perhaps, there's a more efficient way that we can do this.
                let child_attribute_value = AttributeValue::get_by_id(ctx, child_av_id).await?;
                let prop_id_for_child_attribute_value =
                    AttributeValue::prop_id_for_id_or_error(ctx, child_av_id).await?;
                let child_property_editor_value_id = PropertyEditorValueId::from(child_av_id);

                let sockets_for_av =
                    AttributeValue::list_input_socket_sources_for_id(ctx, child_av_id).await?;

                let is_from_external_source = sockets_for_av
                    .iter()
                    .any(|s| connected_sockets_on_component.contains(s));

                let ControllingFuncData {
                    av_id: this_controlling_attribute_value_id,
                    is_dynamic_func: this_controlling_func_is_dynamic,
                    ..
                } = *controlling_ancestors_for_av_id
                    .get(&child_av_id)
                    .ok_or(AttributeValueError::MissingForId(child_av_id))?;

                // Note (victor): An attribute value is overridden if there is an attribute
                // prototype for this specific AV, which means it's set for the component,
                // not the schema variant. If the av is controlled, this check should be
                // made for its controlling AV.
                // This could be standalone func for AV, but we'd have to implement a
                // controlling_ancestors_for_av_id for av, instead of for the whole component.
                // Not a complicated task, but the PR that adds this has enough code as it is.
                let overridden = AttributeValue::component_prototype_id(
                    ctx,
                    this_controlling_attribute_value_id,
                )
                .await?
                .is_some();

                let validation =
                    ValidationOutputNode::find_for_attribute_value_id(ctx, child_av_id)
                        .await?
                        .map(|node| node.validation);

                let child_property_editor_value = PropertyEditorValue {
                    id: child_property_editor_value_id,
                    prop_id: prop_id_for_child_attribute_value.into(),
                    key,
                    value: child_attribute_value
                        .value(ctx)
                        .await?
                        .unwrap_or(Value::Null),
                    validation,
                    can_be_set_by_socket: !sockets_for_av.is_empty(),
                    is_from_external_source,
                    is_controlled_by_ancestor: this_controlling_attribute_value_id != child_av_id,
                    is_controlled_by_dynamic_func: this_controlling_func_is_dynamic,
                    overridden,
                };

                // Load the work queue with the child attribute value.
                work_queue.push_back((child_av_id, child_property_editor_value.id));

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
    pub validation: Option<ValidationOutput>,
    pub can_be_set_by_socket: bool, // true if this prop value is currently driven by a socket, even if the socket isn't in use
    pub is_from_external_source: bool, // true if this prop has a value provided by a socket
    pub is_controlled_by_ancestor: bool, // if ancestor of prop is set by dynamic func, ID of ancestor that sets it
    pub is_controlled_by_dynamic_func: bool, // props driven by non-dynamic funcs have a statically set value
    pub overridden: bool, // true if this prop has a different controlling func id than the default for this asset
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
