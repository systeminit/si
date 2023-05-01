//! This module contains [`ComponentBag`], which is useful for creating
//! [`Components`](dal::Component), caching relevant information for them, and providing
//! helper functions for them by leveraging the cached information.

use dal::{
    attribute::context::AttributeContextBuilder, node::NodeId, AttributeReadContext,
    AttributeValue, AttributeValueId, Component, ComponentId, ComponentView,
    ComponentViewProperties, DalContext, ExternalProviderId, InternalProviderId, Node, Prop,
    PropId, PropKind, Schema, SchemaId, SchemaVariant, SchemaVariantId, StandardModel,
};
use serde_json::Value;
use std::collections::HashMap;

/// A "bagger" for creating [`Components`](dal::Component) and assembling
/// [`ComponentBags`](ComponentBag).
#[derive(Debug, Default)]
pub struct ComponentBagger {
    /// A _private_ cache used for creating multiple [`Components`](dal::Component) of the
    /// same [`Schema`](dal::Schema) and default [`SchemaVariant`](dal::SchemaVariant).
    schema_cache: HashMap<String, (SchemaId, SchemaVariantId)>,
}

impl ComponentBagger {
    /// Create a new [`bagger`](ComponentBagger) and initialize its cache.
    pub fn new() -> Self {
        Self {
            schema_cache: Default::default(),
        }
    }

    /// Create a [`Component`](dal::Component) and assemble a [`ComponentBag`] using its
    /// metadata.
    pub async fn create_component(
        &mut self,
        ctx: &DalContext,
        component_name: &str,
        schema_name: impl Into<String>,
    ) -> ComponentBag {
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

        ComponentBag {
            schema_id: *schema_id,
            schema_variant_id: *schema_variant_id,
            component_id: *component.id(),
            node_id: *node.id(),
            base_attribute_read_context: AttributeReadContext {
                prop_id: None,
                component_id: Some(*component.id()),
                ..AttributeReadContext::default()
            },
        }
    }
}

#[derive(Debug)]
/// This struct is used for bundling a [`Component`](dal::Component) with all metadata needed for a
/// test.
pub struct ComponentBag {
    pub schema_id: SchemaId,
    pub schema_variant_id: SchemaVariantId,
    pub component_id: ComponentId,
    pub node_id: NodeId,
    /// An [`AttributeReadContext`] that can be used for generating a [`ComponentView`].
    pub base_attribute_read_context: AttributeReadContext,
}

impl ComponentBag {
    pub fn bagger() -> ComponentBagger {
        ComponentBagger::default()
    }

    /// Gets the [`Node`](dal::Node) for [`self`](ComponentBag).
    pub async fn node(&self, ctx: &DalContext) -> Node {
        Node::get_by_id(ctx, &self.node_id)
            .await
            .expect("could not perform get by id")
            .expect("not found")
    }

    /// Gets the [`Component`](dal::Component) for [`self`](ComponentBag).
    pub async fn component(&self, ctx: &DalContext) -> Component {
        Component::get_by_id(ctx, &self.component_id)
            .await
            .expect("could not perform get by id")
            .expect("not found")
    }

    /// Gets the [`Schema`](dal::Schema) for [`self`](ComponentBag).
    pub async fn schema(&self, ctx: &DalContext) -> Schema {
        Schema::get_by_id(ctx, &self.schema_id)
            .await
            .expect("could not perform get by id")
            .expect("not found")
    }

    /// Gets the [`SchemaVariant`](dal::SchemaVariant) for [`self`](ComponentBag).
    pub async fn schema_variant(&self, ctx: &DalContext) -> SchemaVariant {
        SchemaVariant::get_by_id(ctx, &self.schema_variant_id)
            .await
            .expect("could not perform get by id")
            .expect("not found")
    }

    /// Finds the [`Prop`](dal::Prop) corresponding to the "path" provided.
    pub async fn find_prop(&self, ctx: &DalContext, prop_path: &[&str]) -> Prop {
        SchemaVariant::find_prop_in_tree(ctx, self.schema_variant_id, prop_path)
            .await
            .expect("could not find prop")
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

    /// Returns an [`AttributeReadContext`](dal::AttributeReadContext) using a set
    /// [`PropId`](dal::Prop) and [`ComponentId`](dal::Component).
    pub fn attribute_read_context_with_prop(&self, prop_id: PropId) -> AttributeReadContext {
        AttributeReadContext {
            prop_id: Some(prop_id),
            ..self.base_attribute_read_context
        }
    }

    /// Returns an [`AttributeReadContext`](dal::AttributeReadContext) using a set
    /// [`InternalProviderId`](dal::InternalProvider) and [`ComponentId`](dal::Component).
    pub fn attribute_read_context_with_internal_provider(
        &self,
        internal_provider_id: InternalProviderId,
    ) -> AttributeReadContext {
        AttributeReadContext {
            internal_provider_id: Some(internal_provider_id),
            ..self.base_attribute_read_context
        }
    }

    /// Returns an [`AttributeReadContext`](dal::AttributeReadContext) using a set
    /// [`ExternalProviderId`](dal::ExternalProvider) and [`ComponentId`](dal::Component).
    pub fn attribute_read_context_with_external_provider(
        &self,
        external_provider_id: ExternalProviderId,
    ) -> AttributeReadContext {
        AttributeReadContext {
            external_provider_id: Some(external_provider_id),
            ..self.base_attribute_read_context
        }
    }

    /// Update a [`AttributeValue`]. This only works if the parent `AttributeValue` for the same
    /// context corresponds to an _"object"_ [`Prop`].
    pub async fn update_attribute_value_for_prop(
        &self,
        ctx: &DalContext,
        prop_id: PropId,
        value: Option<Value>,
    ) -> AttributeValueId {
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
        assert_eq!(&PropKind::Object, parent_prop.kind());
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
        array_prop_id: PropId,
        element_prop_id: PropId,
        value: Value,
    ) -> AttributeValueId {
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
        array_prop_id: PropId,
        element_prop_id: PropId,
    ) -> AttributeValueId {
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
    pub async fn update_attribute_value_for_prop_and_parent_element_attribute_value_id(
        &self,
        ctx: &DalContext,
        prop_id: PropId,
        value: Option<Value>,
        element_attribute_value_id: AttributeValueId,
    ) -> AttributeValueId {
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
}
