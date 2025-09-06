//! This module contains the ability to construct values reflecting the latest state of a
//! [`Component`](crate::Component)'s properties.

use std::collections::{
    HashMap,
    HashSet,
    VecDeque,
};

use petgraph::prelude::*;
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::Value;
use telemetry::prelude::*;

use crate::{
    AttributePrototype,
    AttributeValue,
    AttributeValueId,
    Component,
    ComponentId,
    DalContext,
    EdgeWeightKind,
    InputSocketId,
    Prop,
    PropId,
    Secret,
    attribute::{
        path::AttributePath,
        value::AttributeValueError,
    },
    property_editor::{
        PropertyEditorError,
        PropertyEditorPropId,
        PropertyEditorResult,
        PropertyEditorValueId,
    },
    validation::{
        ValidationOutput,
        ValidationOutputNode,
    },
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

        let sockets_on_component: HashSet<InputSocketId> =
            Component::inferred_incoming_connections(ctx, component_id)
                .await?
                .iter()
                .map(|c| c.to_input_socket_id)
                .collect();

        let controlling_ancestors_for_av_id =
            Component::list_av_controlling_func_ids_for_id(ctx, component_id).await?;

        let mut values = HashMap::new();
        let mut child_values = HashMap::new();

        // Get the root attribute value and load it into the work queue.
        let root_av_id = Component::root_attribute_value_id(ctx, component_id).await?;
        let root_value_id = PropertyEditorValueId::from(root_av_id);
        let root_prop_id = AttributeValue::prop_id(ctx, root_av_id).await?;
        let root_av = AttributeValue::get_by_id(ctx, root_av_id).await?;

        let validation = ValidationOutputNode::find_for_attribute_value_id(ctx, root_av_id)
            .await?
            .map(|node| node.validation);

        // Collect a map of all secret ids by key in the graph. In the future, we may want to cache
        // this or search while iterating. For now, the "list_ids_by_key_bench" test ensures that we
        // meet a baseline performance target.
        let secret_ids_by_key = {
            let start = tokio::time::Instant::now();
            let secret_ids_by_key = Secret::list_ids_by_key(ctx).await?;
            debug!(%component_id, "listing secret ids by key took {:?}", start.elapsed());
            secret_ids_by_key
        };

        let secrets_av_id =
            Component::attribute_value_for_prop(ctx, component.id(), &["root", "secrets"]).await?;
        values.insert(
            root_value_id,
            PropertyEditorValue {
                id: root_value_id,
                prop_id: root_prop_id.into(),
                key: None,
                value: root_av.value_or_default_or_null(ctx, root_prop_id).await?,
                validation,
                is_from_external_source: false,
                can_be_set_by_socket: false,
                is_controlled_by_dynamic_func: false,
                is_controlled_by_ancestor: false,
                overridden: false,
                source: None,
            },
        );

        let mut work_queue = VecDeque::from([(root_av_id, root_value_id)]);

        while let Some((parent_av_id, parent_value_id)) = work_queue.pop_front() {
            // Now that we have the child props, prepare the property editor props and load the work queue.
            let mut child_value_ids = Vec::new();
            for av_id in AttributeValue::get_child_av_ids_in_order(ctx, parent_av_id).await? {
                let key = AttributeValue::key_for_id(ctx, av_id).await?;

                // NOTE(nick): we already have the node weight, but I believe we still want to use "get_by_id" to
                // get the content from the store. Perhaps, there's a more efficient way that we can do this.
                let prop_id = AttributeValue::prop_id(ctx, av_id).await?;
                let value_id = PropertyEditorValueId::from(av_id);

                let sockets_for_av =
                    AttributeValue::list_input_socket_sources_for_id(ctx, av_id).await?;
                let can_be_set_by_socket = !sockets_for_av.is_empty();
                let is_from_external_source = sockets_for_av
                    .into_iter()
                    .any(|s| sockets_on_component.contains(&s));

                let controlling_func = *controlling_ancestors_for_av_id
                    .get(&av_id)
                    .ok_or(AttributeValueError::MissingForId(av_id))?;

                let controlling_prototype_id =
                    AttributeValue::component_prototype_id(ctx, controlling_func.av_id).await?;
                let mut source = None;
                // Check if the component value is explicitly connected to another component.
                if let Some(controlling_prototype_id) = controlling_prototype_id {
                    for apa_id in
                        AttributePrototype::list_arguments(ctx, controlling_prototype_id).await?
                    {
                        for (edge, _, target) in ctx
                            .workspace_snapshot()?
                            .edges_directed(apa_id, Direction::Outgoing)
                            .await?
                        {
                            if let EdgeWeightKind::ValueSubscription(AttributePath::JsonPointer(
                                path,
                            )) = edge.kind
                            {
                                let component =
                                    AttributeValue::component_id(ctx, target.into()).await?;
                                source = Some(PropertyEditorValueSource::Subscription {
                                    component,
                                    path,
                                });
                            }
                        }
                    }
                }

                // Note (victor): An attribute value is overridden if there is an attribute
                // prototype for this specific AV, which means it's set for the component,
                // not the schema variant. If the av is controlled, this check should be
                // made for its controlling AV.
                // This could be standalone func for AV, but we'd have to implement a
                // controlling_ancestors_for_av_id for av, instead of for the whole component.
                // Not a complicated task, but the PR that adds this has enough code as it is.
                let overridden = controlling_prototype_id.is_some();

                let validation = ValidationOutputNode::find_for_attribute_value_id(ctx, av_id)
                    .await?
                    .map(|node| node.validation);

                // Get the value
                let mut value = AttributeValue::get_by_id(ctx, av_id)
                    .await?
                    .value_or_default_or_null(ctx, prop_id)
                    .await?;

                // If this is a secret, the JSON value has the secret key, not the secret id.
                // The editor needs the secret id, so we look in our mapto find which Secret in
                // the current graph has that key.
                if parent_av_id == secrets_av_id && value != Value::Null {
                    let secret_key = Secret::key_from_value_in_attribute_value(value)?;
                    value = match secret_ids_by_key.get(&secret_key) {
                        Some(secret_id) => serde_json::to_value(secret_id)?,
                        None => {
                            // If none of the secrets in the workspace have this key, we assume
                            // that dependent values haven't updated yet and will be fixed
                            // shortly. Thus we treat the property as missing for now and
                            // return null.
                            //
                            // This is an expected issue, so we don't warn--but it could trigger
                            // if something more serious is going on that is making the lookup
                            // fail more persistently, so we may want to measure how often it
                            // happens and figure out how to alert in that case.
                            warn!(
                                name: "Secret key does not match",
                                av_id = %av_id,
                                "Secret key in dependent value does not match any secret key; assuming that dependent values are not up to date and treating the property temporarily as missing",
                            );
                            Value::Null
                        }
                    }
                }

                let value = PropertyEditorValue {
                    id: value_id,
                    prop_id: prop_id.into(),
                    key,
                    value,
                    validation,
                    can_be_set_by_socket,
                    is_from_external_source,
                    is_controlled_by_ancestor: controlling_func.av_id != av_id,
                    is_controlled_by_dynamic_func: controlling_func.is_dynamic_func,
                    overridden,
                    source,
                };

                // Load the work queue with the child attribute value.
                work_queue.push_back((av_id, value.id));

                // Cache the child property editor values to eventually insert into the child property editor values map.
                child_value_ids.push(value.id);

                // Insert the child property editor value into the values map.
                values.insert(value.id, value);
            }
            child_values.insert(parent_value_id, child_value_ids);
        }

        Ok(Self {
            root_value_id,
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

    /// Finds the [`AttributeValueId`](AttributeValue) for a given [`PropId`](Prop).
    ///
    /// This is useful for non-maps and non-array [`Props`](Prop).
    pub fn find_by_prop_id_or_err(
        &self,
        prop_id: PropId,
    ) -> PropertyEditorResult<AttributeValueId> {
        self.values
            .iter()
            .find(|(_, property_editor_value)| property_editor_value.prop_id() == prop_id)
            .map(|(_, found_property_editor_value)| {
                found_property_editor_value.attribute_value_id()
            })
            .ok_or_else(|| PropertyEditorError::PropertyEditorValueNotFoundByPropId(prop_id))
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<PropertyEditorValueSource>, // The source of this value (set to None if it's a static value)
}

// The source for a value (unless it's a static value).
// Try to keep this in sync with v2::component::attributes::Source
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub enum PropertyEditorValueSource {
    // We don't send this
    // // { value: <value> } - set value (null is a valid value to set it to)
    // Value(serde_json::Value),

    // { component: "ComponentNameOrId", path: "/domain/Foo/Bar/0/Baz" } - subscribe this value to a path from a component
    #[serde(untagged)]
    Subscription {
        component: ComponentId,
        path: String,
    },
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
