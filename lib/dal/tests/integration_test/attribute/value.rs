use dal::{
    attribute::context::AttributeContextBuilder,
    component::view::ComponentView,
    test_harness::{create_prop_of_kind_with_name, create_schema, create_schema_variant_with_root},
    AttributeContext, AttributeReadContext, AttributeValue, Component, DalContext, PropKind,
    SchemaKind, SchemaVariant, StandardModel, SystemId,
};
use pretty_assertions_sorted::assert_eq_sorted;

use crate::dal::test;

#[test]
async fn update_for_context_simple(ctx: &DalContext<'_, '_>) {
    // "name": String
    let mut schema = create_schema(ctx, &SchemaKind::Concrete).await;
    let (schema_variant, root) = create_schema_variant_with_root(ctx, *schema.id()).await;
    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await
        .expect("cannot set default schema variant");

    let name_prop = create_prop_of_kind_with_name(ctx, PropKind::String, "name_prop").await;
    name_prop
        .set_parent_prop(ctx, root.domain_prop_id)
        .await
        .expect("cannot set parent of name_prop");

    SchemaVariant::create_default_prototypes_and_values(ctx, *schema_variant.id())
        .await
        .expect("cannot create default prototypes and values for SchemaVariant");
    SchemaVariant::create_implicit_internal_providers(ctx, *schema.id(), *schema_variant.id())
        .await
        .expect("unable to create implicit internal providers for schema variant");

    let (component, _, task) =
        Component::new_for_schema_with_node(ctx, "Basic component", schema.id())
            .await
            .expect("Unable to create component");
    task.run_updates_in_ctx(ctx)
        .await
        .expect("dependent values update failed");

    let base_attribute_read_context = AttributeReadContext {
        prop_id: None,
        schema_id: Some(*schema.id()),
        schema_variant_id: Some(*schema_variant.id()),
        component_id: Some(*component.id()),
        ..AttributeReadContext::default()
    };

    assert_eq_sorted!(
        serde_json::json![
            {
                "si": {
                    "name": "Basic component",
                },
                "domain": {},
            }
        ],
        ComponentView::for_context(ctx, base_attribute_read_context)
            .await
            .expect("cannot get component view")
            .properties,
    );

    let domain_value_id = *AttributeValue::find_for_context(
        ctx,
        AttributeReadContext {
            prop_id: Some(root.domain_prop_id),
            ..base_attribute_read_context
        },
    )
    .await
    .expect("cannot get domain AttributeValue")
    .expect("domain AttributeValue not found")
    .id();
    let base_name_value = AttributeValue::find_for_context(
        ctx,
        AttributeReadContext {
            prop_id: Some(*name_prop.id()),
            ..base_attribute_read_context
        },
    )
    .await
    .expect("cannot get name AttributeValue")
    .expect("name AttributeValue not found");

    let update_context: AttributeContext =
        AttributeContextBuilder::from(base_attribute_read_context)
            .set_prop_id(*name_prop.id())
            .to_context()
            .expect("cannot build write AttributeContext");

    let (_, name_value_id, task) = AttributeValue::update_for_context(
        ctx,
        *base_name_value.id(),
        Some(domain_value_id),
        update_context,
        Some(serde_json::to_value("Miles".to_string()).expect("cannot create new Value")),
        None,
    )
    .await
    .expect("cannot set value for context");
    task.run_updates_in_ctx(ctx)
        .await
        .expect("dependent values update failed");

    assert_eq_sorted!(
        serde_json::json![
            {
                "si": {
                    "name": "Basic component",
                },
                "domain": {
                    "name_prop": "Miles",
                },
            }
        ],
        ComponentView::for_context(ctx, base_attribute_read_context)
            .await
            .expect("cannot get component view")
            .properties,
    );

    let (_, _, task) = AttributeValue::update_for_context(
        ctx,
        name_value_id,
        Some(domain_value_id),
        update_context,
        Some(serde_json::to_value("Iria".to_string()).expect("cannot create new value")),
        None,
    )
    .await
    .expect("cannot update value for context");
    task.run_updates_in_ctx(ctx)
        .await
        .expect("dependent values update failed");

    assert_eq_sorted!(
        serde_json::json![
            {
                "si": {
                    "name": "Basic component",
                },
                "domain": {
                    "name_prop": "Iria",
                },
            }
        ],
        ComponentView::for_context(ctx, base_attribute_read_context)
            .await
            .expect("cannot get component view")
            .properties,
    );
}

#[test]
async fn insert_for_context_simple(ctx: &DalContext<'_, '_>) {
    let mut schema = create_schema(ctx, &SchemaKind::Concrete).await;
    let (schema_variant, root) = create_schema_variant_with_root(ctx, *schema.id()).await;
    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await
        .expect("cannot set default schema variant");

    let array_prop = create_prop_of_kind_with_name(ctx, PropKind::Array, "array_prop").await;
    array_prop
        .set_parent_prop(ctx, root.domain_prop_id)
        .await
        .expect("cannot set parent of array_prop");

    let array_element = create_prop_of_kind_with_name(ctx, PropKind::String, "array_element").await;
    array_element
        .set_parent_prop(ctx, *array_prop.id())
        .await
        .expect("cannot set parent of array_element");

    SchemaVariant::create_default_prototypes_and_values(ctx, *schema_variant.id())
        .await
        .expect("cannot create default prototypes and values for SchemaVariant");
    SchemaVariant::create_implicit_internal_providers(ctx, *schema.id(), *schema_variant.id())
        .await
        .expect("unable to create implicit internal providers for schema variant");

    let (component, _, task) =
        Component::new_for_schema_with_node(ctx, "Array Component", schema.id())
            .await
            .expect("Unable to create component");
    task.run_updates_in_ctx(ctx)
        .await
        .expect("dependent values update failed");

    let base_attribute_read_context = AttributeReadContext {
        prop_id: None,
        schema_id: Some(*schema.id()),
        schema_variant_id: Some(*schema_variant.id()),
        component_id: Some(*component.id()),
        ..AttributeReadContext::default()
    };

    assert_eq_sorted!(
        serde_json::json![{
            "si": {
                "name": "Array Component",
            },
            "domain": {},
        }],
        ComponentView::for_context(ctx, base_attribute_read_context)
            .await
            .expect("cannot get component view")
            .properties,
    );

    let array_value = AttributeValue::find_for_context(
        ctx,
        AttributeReadContext {
            prop_id: Some(*array_prop.id()),
            ..base_attribute_read_context
        },
    )
    .await
    .expect("cannot get array AttributeValue")
    .expect("array AttributeValue not found");
    let update_context = AttributeContextBuilder::from(base_attribute_read_context)
        .set_prop_id(*array_element.id())
        .to_context()
        .expect("cannot build write AttributeContext");

    let (_new_array_element_value_id, task) =
        AttributeValue::insert_for_context(ctx, update_context, *array_value.id(), None, None)
            .await
            .expect("cannot insert new array element");
    task.run_updates_in_ctx(ctx)
        .await
        .expect("dependent values update failed");

    assert_eq_sorted!(
        serde_json::json![{
            "si": {
                "name": "Array Component",
            },
            "domain": {
                "array_prop": [],
            },
        }],
        ComponentView::for_context(ctx, base_attribute_read_context)
            .await
            .expect("cannot get component view")
            .properties,
    );
}

#[test]
async fn update_for_context_object(ctx: &DalContext<'_, '_>) {
    let mut schema = create_schema(ctx, &SchemaKind::Concrete).await;
    let (schema_variant, root) = create_schema_variant_with_root(ctx, *schema.id()).await;
    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await
        .expect("cannot set default schema variant");

    let address_prop = create_prop_of_kind_with_name(ctx, PropKind::Object, "address").await;
    address_prop
        .set_parent_prop(ctx, root.domain_prop_id)
        .await
        .expect("cannot set parent of address_prop");

    let streets_prop = create_prop_of_kind_with_name(ctx, PropKind::Array, "streets").await;
    streets_prop
        .set_parent_prop(ctx, *address_prop.id())
        .await
        .expect("cannot set parent of streets prop");
    let streets_child_prop = create_prop_of_kind_with_name(ctx, PropKind::String, "street").await;
    streets_child_prop
        .set_parent_prop(ctx, *streets_prop.id())
        .await
        .expect("cannot set parent of street prop");

    let city_prop = create_prop_of_kind_with_name(ctx, PropKind::String, "city").await;
    city_prop
        .set_parent_prop(ctx, *address_prop.id())
        .await
        .expect("cannot set parent of city prop");
    let country_prop = create_prop_of_kind_with_name(ctx, PropKind::String, "country").await;
    country_prop
        .set_parent_prop(ctx, *address_prop.id())
        .await
        .expect("cannot set parent of country prop");

    let tags_prop = create_prop_of_kind_with_name(ctx, PropKind::Map, "tags").await;
    tags_prop
        .set_parent_prop(ctx, *address_prop.id())
        .await
        .expect("cannot set parent of tags prop");
    let tags_child_prop = create_prop_of_kind_with_name(ctx, PropKind::String, "tag").await;
    tags_child_prop
        .set_parent_prop(ctx, *tags_prop.id())
        .await
        .expect("cannot set parent of tags child prop");

    // Now, we can setup providers.
    SchemaVariant::create_implicit_internal_providers(ctx, *schema.id(), *schema_variant.id())
        .await
        .expect("unable to create implicit internal providers");

    let (component, _, task) =
        Component::new_for_schema_with_node(ctx, "Basic component", schema.id())
            .await
            .expect("Unable to create component");
    task.run_updates_in_ctx(ctx)
        .await
        .expect("dependent values update failed");

    let read_context = AttributeReadContext {
        prop_id: None,
        schema_id: Some(*schema.id()),
        schema_variant_id: Some(*schema_variant.id()),
        component_id: Some(*component.id()),
        ..AttributeReadContext::default()
    };
    let component_view = ComponentView::for_context(ctx, read_context)
        .await
        .expect("cannot get component view");

    assert_eq_sorted!(
        serde_json::json![
            {
                "si": {
                    "name": "Basic component",
                },
                "domain": {},
            }
        ],
        component_view.properties,
    );

    let root_value_id = *AttributeValue::list_for_context(
        ctx,
        AttributeReadContext {
            prop_id: Some(root.prop_id),
            component_id: Some(*component.id()),
            schema_id: Some(*schema.id()),
            schema_variant_id: Some(*schema_variant.id()),
            ..AttributeReadContext::any()
        },
    )
    .await
    .expect("cannot get root AttributeValue")
    .into_iter()
    .next()
    .expect("root AttributeValue not found")
    .id();

    let domain_value_id = *AttributeValue::list_for_context(
        ctx,
        AttributeReadContext {
            prop_id: Some(root.domain_prop_id),
            component_id: Some(*component.id()),
            schema_id: Some(*schema.id()),
            schema_variant_id: Some(*schema_variant.id()),
            ..AttributeReadContext::any()
        },
    )
    .await
    .expect("cannot get domain AttributeValue")
    .into_iter()
    .next()
    .expect("domain AttributeValue not found")
    .id();

    let update_context = AttributeContext::builder()
        .set_prop_id(root.domain_prop_id)
        .set_schema_id(*schema.id())
        .set_schema_variant_id(*schema_variant.id())
        .set_component_id(*component.id())
        .to_context()
        .expect("cannot build write AttributeContext");

    let (_, domain_value_id, task) = AttributeValue::update_for_context(
        ctx,
        domain_value_id,
        Some(root_value_id),
        update_context,
        Some(serde_json::json!({
            "address": {
                "streets": [
                    "Suite 4",
                    "14 Main Street"
                ],
                "city": "Plainstown",
                "country": "Eurasia",
                "tags": {
                    "cool": "beans",
                    "alpha": "bet",
                },
            },
        })),
        None,
    )
    .await
    .expect("cannot update value");
    task.run_updates_in_ctx(ctx)
        .await
        .expect("dependent values update failed");

    let component_view = ComponentView::for_context(ctx, read_context)
        .await
        .expect("cannot get component view");

    assert_eq_sorted!(
        serde_json::json![
            {
                "si": {
                    "name": "Basic component",
                },
                "domain": {
                    "address": {
                        "streets": [
                            "Suite 4",
                            "14 Main Street"
                        ],
                        "city": "Plainstown",
                        "country": "Eurasia",
                        "tags": {
                            "cool": "beans",
                            "alpha": "bet",
                        },
                    },
                },
            }
        ],
        component_view.properties,
    );

    let (_, _domain_value_id, task) = AttributeValue::update_for_context(
        ctx,
        domain_value_id,
        Some(root_value_id),
        update_context,
        Some(serde_json::json!({
            "address": {
                "streets": [
                    "123 Ok",
                ],
                "city": "Nowheresville",
                "tags": {
                    "new": "one",
                },
            },
        })),
        None,
    )
    .await
    .expect("cannot update value");
    task.run_updates_in_ctx(ctx)
        .await
        .expect("dependent values update failed");

    let component_view = ComponentView::for_context(ctx, read_context)
        .await
        .expect("cannot get component view");

    assert_eq_sorted!(
        serde_json::json![
            {
                "si": {
                    "name": "Basic component",
                },
                "domain": {
                    "address": {
                        "streets": [
                            "123 Ok",
                        ],
                        "city": "Nowheresville",
                        "tags": {
                            "new": "one",
                        },
                    },
                },
            }
        ],
        component_view.properties,
    );
}

#[test]
async fn insert_for_context_creates_array_in_final_context(ctx: &DalContext<'_, '_>) {
    let mut schema = create_schema(ctx, &SchemaKind::Concrete).await;
    let (schema_variant, root) = create_schema_variant_with_root(ctx, *schema.id()).await;
    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await
        .expect("cannot set default schema variant");

    let array_prop = create_prop_of_kind_with_name(ctx, PropKind::Array, "array_prop").await;
    array_prop
        .set_parent_prop(ctx, root.domain_prop_id)
        .await
        .expect("cannot set parent of array_prop");

    let array_element = create_prop_of_kind_with_name(ctx, PropKind::String, "array_element").await;
    array_element
        .set_parent_prop(ctx, *array_prop.id())
        .await
        .expect("cannot set parent of array_element");

    SchemaVariant::create_default_prototypes_and_values(ctx, *schema_variant.id())
        .await
        .expect("cannot create default prop AttributePrototypes");
    SchemaVariant::create_implicit_internal_providers(ctx, *schema.id(), *schema_variant.id())
        .await
        .expect("unable to create implicit internal providers for schema variant");

    let (component, _, task) =
        Component::new_for_schema_with_node(ctx, "Array Component", schema.id())
            .await
            .expect("Unable to create component");
    task.run_updates_in_ctx(ctx)
        .await
        .expect("unable to run async updates");

    let base_attribute_read_context = AttributeReadContext {
        prop_id: None,
        schema_id: Some(*schema.id()),
        schema_variant_id: Some(*schema_variant.id()),
        component_id: Some(*component.id()),
        ..AttributeReadContext::default()
    };

    assert_eq_sorted!(
        serde_json::json![{
            "si": {
                "name": "Array Component",
            },
            "domain": {},
        }],
        ComponentView::for_context(ctx, base_attribute_read_context)
            .await
            .expect("cannot get component view")
            .properties,
    );

    let array_value = AttributeValue::find_for_context(
        ctx,
        AttributeReadContext {
            prop_id: Some(*array_prop.id()),
            ..base_attribute_read_context
        },
    )
    .await
    .expect("cannot get array AttributeValue")
    .expect("array AttributeValue not found");
    let update_context = AttributeContextBuilder::from(base_attribute_read_context)
        .set_prop_id(*array_element.id())
        .to_context()
        .expect("cannot build write AttributeContext");

    let (_new_array_element_value_id, task) = AttributeValue::insert_for_context(
        ctx,
        update_context,
        *array_value.id(),
        Some(serde_json::json!("Component Element")),
        None,
    )
    .await
    .expect("cannot insert new array element");
    task.run_updates_in_ctx(ctx)
        .await
        .expect("unable to run async updates");

    assert_eq_sorted!(
        serde_json::json![{
            "si": {
                "name": "Array Component",
            },
            "domain": {
                "array_prop": [
                    "Component Element",
                ],
            },
        }],
        ComponentView::for_context(ctx, base_attribute_read_context)
            .await
            .expect("cannot get component view")
            .properties,
    );

    let component_array_value = AttributeValue::find_for_context(
        ctx,
        AttributeReadContext {
            prop_id: Some(*array_prop.id()),
            ..base_attribute_read_context
        },
    )
    .await
    .expect("cannot get component array AttributeValue")
    .expect("component array AttributeValue not found");

    let system_id: SystemId = 12_000.into();

    let system_update_context = AttributeContextBuilder::from(update_context)
        .set_system_id(system_id)
        .to_context()
        .expect("cannot build system write AttributeContext");

    let (_new_system_array_element_value_id, task) = AttributeValue::insert_for_context(
        ctx,
        system_update_context,
        *component_array_value.id(),
        Some(serde_json::json!("System Element")),
        None,
    )
    .await
    .expect("cannot insert new system array element");
    task.run_updates_in_ctx(ctx)
        .await
        .expect("unable to run async updates");

    let system_attribute_read_context = AttributeReadContext {
        system_id: Some(system_id),
        ..base_attribute_read_context
    };

    assert_eq_sorted!(
        serde_json::json![{
            "si": {
                "name": "Array Component",
            },
            "domain": {
                "array_prop": [
                    "System Element",
                    "Component Element",
                ],
            },
        }],
        ComponentView::for_context(ctx, system_attribute_read_context)
            .await
            .expect("cannot get component view")
            .properties,
    );

    let system_array_read_context = AttributeReadContext {
        prop_id: Some(*array_prop.id()),
        ..system_update_context.into()
    };
    let mut system_array_values = AttributeValue::list_for_context(ctx, system_array_read_context)
        .await
        .expect("cannot retrieve array AttributeValue");
    assert_eq!(
        system_array_values.len(),
        1,
        "Should only find a single AttributeValue for the array"
    );
    let system_array_value = system_array_values
        .pop()
        .expect("cannot get system array AttributeValue");
    let system_array_context = AttributeContextBuilder::from(system_update_context)
        .set_prop_id(*array_prop.id())
        .to_context()
        .expect("cannot create system array AttributeContext");
    assert_eq!(system_array_context, system_array_value.context);
}
