use dal::attribute::context::AttributeContextBuilder;
use dal::{
    component::view::ComponentView,
    test_harness::{create_prop_of_kind_with_name, create_schema, create_schema_variant_with_root},
    AttributeContext, AttributeReadContext, AttributeValue, Component, DalContext, PropKind,
    SchemaKind, StandardModel,
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

    let (component, _, _) =
        Component::new_for_schema_with_node(ctx, "Basic component", schema.id())
            .await
            .expect("Unable to create component");

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

    let (_, name_value_id, _) = AttributeValue::update_for_context(
        ctx,
        *base_name_value.id(),
        Some(domain_value_id),
        update_context,
        Some(serde_json::to_value("Miles".to_string()).expect("cannot create new Value")),
        None,
    )
    .await
    .expect("cannot set value for context");

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

    AttributeValue::update_for_context(
        ctx,
        name_value_id,
        Some(domain_value_id),
        update_context,
        Some(serde_json::to_value("Iria".to_string()).expect("cannot create new value")),
        None,
    )
    .await
    .expect("cannot update value for context");

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

    let (component, _, _) =
        Component::new_for_schema_with_node(ctx, "Array Component", schema.id())
            .await
            .expect("Unable to create component");

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

    let _new_array_element_value_id =
        AttributeValue::insert_for_context(ctx, update_context, *array_value.id(), None, None)
            .await
            .expect("cannot insert new array element");

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
