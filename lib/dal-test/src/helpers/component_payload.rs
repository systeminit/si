//! This module contains [`ComponentPayload`], which is useful for creating
//! [`Components`](dal::Component), caching relevant information for them, and providing
//! helper functions for them by leveraging the cached information.

use dal::{
    attribute::context::AttributeContextBuilder, node::NodeId, AttributeContext,
    AttributeReadContext, AttributeValue, AttributeValueId, Component, ComponentId, ComponentView,
    ComponentViewProperties, DalContext, Prop, PropId, Schema, SchemaId, SchemaVariantId,
    StandardModel,
};
use serde_json::Value;
use std::collections::HashMap;

/// An assembler for creating [`Components`](dal::Component) and assembling
/// [`ComponentPayloads`](ComponentPayload).
#[derive(Debug, Default)]
pub struct ComponentPayloadAssembler {
    /// A _private_ cache used for creating multiple [`Components`](dal::Component) of the
    /// same [`Schema`](dal::Schema) and default [`SchemaVariant`](dal::SchemaVariant).
    schema_cache: HashMap<String, (SchemaId, SchemaVariantId)>,
}

impl ComponentPayloadAssembler {
    /// Create a new [`assembler`](ComponentPayloadAssembler) and initialize its cache.
    pub fn new() -> Self {
        Self {
            schema_cache: Default::default(),
        }
    }

    /// Create a [`Component`](dal::Component) and assemble a [`ComponentPayload`] using its
    /// metadata.
    pub async fn create_component(
        &mut self,
        ctx: &DalContext,
        component_name: &str,
        schema_name: impl Into<String>,
    ) -> ComponentPayload {
        let schema_name = schema_name.into();
        let (schema_id, schema_variant_id) =
            self.schema_cache.entry(schema_name.clone()).or_insert({
                let schema = Schema::find_by_name(ctx, schema_name)
                    .await
                    .expect("could not find schema by name");
                let schema_variant_id = schema
                    .default_schema_variant_id()
                    .expect("no default variant for schema");
                (*schema.id(), *schema_variant_id)
            });

        let (component, node) = Component::new(ctx, component_name, *schema_variant_id)
            .await
            .expect("could not create component");

        ComponentPayload {
            schema_id: *schema_id,
            schema_variant_id: *schema_variant_id,
            component_id: *component.id(),
            node_id: *node.id(),
            prop_map: HashMap::new(),
            base_attribute_read_context: AttributeReadContext {
                prop_id: None,
                component_id: Some(*component.id()),
                ..AttributeReadContext::default()
            },
        }
    }
}

#[derive(Debug)]
/// Payload used for bundling a [`Component`](dal::Component) with all metadata needed for a test.
pub struct ComponentPayload {
    pub schema_id: SchemaId,
    pub schema_variant_id: SchemaVariantId,
    pub component_id: ComponentId,
    pub node_id: NodeId,
    /// A map that uses [`Prop`] "json pointer names" as keys and their ids as values.
    pub prop_map: HashMap<&'static str, PropId>,
    /// An [`AttributeReadContext`] that can be used for generating a [`ComponentView`].
    pub base_attribute_read_context: AttributeReadContext,
}

impl ComponentPayload {
    /// Get the [`PropId`] (value) corresponding to the "json pointer name" (key) in the "prop_map"
    /// (e.g. "/root/si/name" passed in as the name).
    pub fn get_prop_id(&self, prop_name: &str) -> PropId {
        *self
            .prop_map
            .get(prop_name)
            .expect("could not find PropId for key")
    }

    /// Merge the base [`AttributeReadContext`] with the [`PropId`] found.
    pub fn attribute_read_context_with_prop_id(&self, prop_name: &str) -> AttributeReadContext {
        AttributeReadContext {
            prop_id: Some(self.get_prop_id(prop_name)),
            ..self.base_attribute_read_context
        }
    }

    /// Merge the base [`AttributeReadContext`] with the [`PropId`] found and convert into an
    /// [`AttributeContext`].
    pub fn attribute_context_with_prop_id(&self, prop_name: &str) -> AttributeContext {
        AttributeContextBuilder::from(self.base_attribute_read_context)
            .set_prop_id(self.get_prop_id(prop_name))
            .to_context()
            .expect("could not convert builder to attribute context")
    }

    /// Generates a new [`ComponentView`] and returns [`ComponentViewProperties`].
    ///
    /// Use this over [`Self::component_view_properties_raw()`] if you'd like to drop certain
    /// subtrees.
    pub async fn component_view_properties(&self, ctx: &DalContext) -> ComponentViewProperties {
        let component_view = ComponentView::new(ctx, self.component_id)
            .await
            .expect("cannot get component view");
        ComponentViewProperties::try_from(component_view)
            .expect("cannot create component view properties from component view")
    }

    /// Generates a new [`ComponentView`] and returns the "properties" field as a raw [`Value`].
    ///
    /// Use this over [`Self::component_view_properties()`] if you'd like to use the entire
    /// [`ComponentView`] with all subtrees (potentially) present.
    pub async fn component_view_properties_raw(&self, ctx: &DalContext) -> Value {
        ComponentView::new(ctx, self.component_id)
            .await
            .expect("cannot get component view")
            .properties
    }

    /// Update a [`AttributeValue`]. This only works if the parent `AttributeValue` for the same
    /// context corresponds to an _"object"_ [`Prop`].
    pub async fn update_attribute_value_for_prop_name(
        &self,
        ctx: &DalContext,
        prop_name: impl AsRef<str>,
        value: Option<Value>,
    ) -> AttributeValueId {
        let prop_id = self.get_prop_id(prop_name.as_ref());

        let attribute_value = AttributeValue::find_for_context(
            ctx,
            AttributeReadContext {
                prop_id: Some(prop_id),
                ..self.base_attribute_read_context
            },
        )
        .await
        .expect("cannot get attribute value")
        .expect("attribute value not found");

        let parent_prop = Prop::get_by_id(ctx, &prop_id)
            .await
            .expect("could not get prop by id")
            .expect("prop not found by id")
            .parent_prop(ctx)
            .await
            .expect("could not find parent prop")
            .expect("parent prop not found or prop does not have parent");
        let parent_attribute_value = AttributeValue::find_for_context(
            ctx,
            AttributeReadContext {
                prop_id: Some(*parent_prop.id()),
                ..self.base_attribute_read_context
            },
        )
        .await
        .expect("cannot get attribute value")
        .expect("attribute value not found");

        let update_attribute_context =
            AttributeContextBuilder::from(self.base_attribute_read_context)
                .set_prop_id(prop_id)
                .to_context()
                .expect("could not convert builder to attribute context");

        let (_, updated_attribute_value_id) = AttributeValue::update_for_context(
            ctx,
            *attribute_value.id(),
            Some(*parent_attribute_value.id()),
            update_attribute_context,
            value,
            None,
        )
        .await
        .expect("cannot update value for context");

        // Return the updated attribute value id.
        updated_attribute_value_id
    }

    /// Inserts an [`AttributeValue`] corresponding to a _primitive_ [`Prop`] (string, boolean or
    /// integer) in an _array_ `Prop`.
    pub async fn insert_array_primitive_element(
        &self,
        ctx: &DalContext,
        array_prop_name: impl AsRef<str>,
        element_prop_name: impl AsRef<str>,
        value: Value,
    ) -> AttributeValueId {
        let array_prop_id = self.get_prop_id(array_prop_name.as_ref());
        let element_prop_id = self.get_prop_id(element_prop_name.as_ref());

        let array_attribute_value = AttributeValue::find_for_context(
            ctx,
            AttributeReadContext {
                prop_id: Some(array_prop_id),
                ..self.base_attribute_read_context
            },
        )
        .await
        .expect("cannot get attribute value")
        .expect("attribute value not found");

        let insert_attribute_context =
            AttributeContextBuilder::from(self.base_attribute_read_context)
                .set_prop_id(element_prop_id)
                .to_context()
                .expect("could not create insert context");

        // Return the element attribute value id.
        AttributeValue::insert_for_context(
            ctx,
            insert_attribute_context,
            *array_attribute_value.id(),
            Some(value),
            None,
        )
        .await
        .expect("could not insert object into array")
    }

    /// Inserts an [`AttributeValue`] corresponding to an "empty" _object_ [`Prop`] in an _array_
    /// `Prop`.
    pub async fn insert_array_object_element(
        &self,
        ctx: &DalContext,
        array_prop_name: impl AsRef<str>,
        element_prop_name: impl AsRef<str>,
    ) -> AttributeValueId {
        let array_prop_id = self.get_prop_id(array_prop_name.as_ref());
        let element_prop_id = self.get_prop_id(element_prop_name.as_ref());

        let array_attribute_value = AttributeValue::find_for_context(
            ctx,
            AttributeReadContext {
                prop_id: Some(array_prop_id),
                ..self.base_attribute_read_context
            },
        )
        .await
        .expect("cannot get attribute value")
        .expect("attribute value not found");

        let insert_attribute_context =
            AttributeContextBuilder::from(self.base_attribute_read_context)
                .set_prop_id(element_prop_id)
                .to_context()
                .expect("could not create insert context");

        // Return the element attribute value id.
        AttributeValue::insert_for_context(
            ctx,
            insert_attribute_context,
            *array_attribute_value.id(),
            Some(serde_json::json![{}]),
            None,
        )
        .await
        .expect("could not insert object into array")
    }

    /// Using the element [`AttributeValueId`] from [`Self::insert_array_object_element()`], update
    /// an [`AttributeValue`] corresponding to a "field" within the _object_ element.
    pub async fn update_attribute_value_for_prop_name_and_parent_element_attribute_value_id(
        &self,
        ctx: &DalContext,
        prop_name: impl AsRef<str>,
        value: Option<Value>,
        element_attribute_value_id: AttributeValueId,
    ) -> AttributeValueId {
        let prop_id = self.get_prop_id(prop_name.as_ref());
        let attribute_value = AttributeValue::find_with_parent_and_key_for_context(
            ctx,
            Some(element_attribute_value_id),
            None,
            AttributeReadContext {
                prop_id: Some(prop_id),
                ..self.base_attribute_read_context
            },
        )
        .await
        .expect("cannot get attribute value")
        .expect("attribute value not found");

        let update_attribute_context =
            AttributeContextBuilder::from(self.base_attribute_read_context)
                .set_prop_id(prop_id)
                .to_context()
                .expect("could not convert builder to attribute context");

        let (_, updated_attribute_value_id) = AttributeValue::update_for_context(
            ctx,
            *attribute_value.id(),
            Some(element_attribute_value_id),
            update_attribute_context,
            value,
            None,
        )
        .await
        .expect("cannot update value for context");

        // Return the updated attribute value id.
        updated_attribute_value_id
    }

    /// Get the full [`Component`](dal::Component) using the [`ComponentId`](dal::Component)
    /// from [`self`](Self).
    pub async fn component(&self, ctx: &DalContext) -> Component {
        Component::get_by_id(ctx, &self.component_id)
            .await
            .expect("could not get component by id")
            .expect("no component found")
    }
}
