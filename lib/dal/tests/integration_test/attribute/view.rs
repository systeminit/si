use crate::dal::test;
use dal::attribute::context::AttributeContextBuilder;
use dal::DalContext;
use dal::{
    test_harness::{create_prop_of_kind_with_name, create_schema, create_schema_variant_with_root},
    AttributeReadContext, AttributeValue, AttributeView, PropKind, SchemaKind, StandardModel,
};
use pretty_assertions_sorted::assert_eq_sorted;

#[test]
async fn schema_variant_specific(ctx: &DalContext<'_, '_>) {
    // "name": String
    let mut schema = create_schema(ctx, &SchemaKind::Concrete).await;
    let (schema_variant, root_prop) = create_schema_variant_with_root(ctx, *schema.id()).await;
    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await
        .expect("cannot set default schema variant");

    let base_context = AttributeReadContext {
        schema_id: Some(*schema.id()),
        schema_variant_id: Some(*schema_variant.id()),
        ..AttributeReadContext::default()
    };

    let name_prop = create_prop_of_kind_with_name(ctx, PropKind::String, "name").await;
    name_prop
        .set_parent_prop(ctx, root_prop.domain_prop_id)
        .await
        .expect("cannot set parent of name_prop");

    let root_attribute_value = AttributeValue::find_with_parent_and_key_for_context(
        ctx,
        None,
        None,
        AttributeReadContext {
            prop_id: Some(root_prop.prop_id),
            ..base_context
        },
    )
    .await
    .expect("cannot find attribute value")
    .expect("attribute value not found");

    let view = AttributeView::new(
        ctx,
        AttributeReadContext {
            prop_id: None,
            ..base_context
        },
        Some(*root_attribute_value.id()),
    )
    .await
    .expect("could not create attribute view");

    assert_eq_sorted!(
        serde_json::json![
            {
                "root": {
                    "domain": {},
                    "si": {}
                }
            }
        ], // expected
        view.value().clone(), // actual
    );

    let domain_attribute_value = AttributeValue::find_for_context(
        ctx,
        AttributeReadContext {
            prop_id: Some(root_prop.domain_prop_id),
            ..base_context
        },
    )
    .await
    .expect("cannot find attribute value")
    .pop()
    .expect("attribute value not found");

    let name_attribute_value = AttributeValue::find_for_context(
        ctx,
        AttributeReadContext {
            prop_id: Some(*name_prop.id()),
            ..base_context
        },
    )
    .await
    .expect("cannot find attribute value")
    .pop()
    .expect("attribute value not found");

    let update_context = AttributeContextBuilder::from(base_context)
        .set_prop_id(*name_prop.id())
        .to_context()
        .expect("could not convert builder to attribute context");
    let update_value = Some(serde_json::to_value("toddhoward").expect("could not create value"));
    let (_, _, _) = AttributeValue::update_for_context(
        ctx,
        *name_attribute_value.id(),
        Some(*domain_attribute_value.id()),
        update_context,
        update_value,
        None,
    )
    .await
    .expect("could not update attribute value");

    let view = AttributeView::new(
        ctx,
        AttributeReadContext {
            prop_id: None,
            ..base_context
        },
        Some(*root_attribute_value.id()),
    )
    .await
    .expect("could not create attribute view");

    assert_eq_sorted!(
        serde_json::json![
            {
                "root": {
                    "domain": {
                        "name": "toddhoward"
                    },
                    "si": {}
                },
            }
        ], // expected
        view.value().clone(), // actual
    );
}

#[test]
async fn free_floating_props(ctx: &DalContext<'_, '_>) {
    // {
    //   "object": {
    //     "array": []
    //   }
    // }
    let base_context = AttributeReadContext {
        ..AttributeReadContext::default()
    };
    let view_context = AttributeReadContext {
        prop_id: None,
        ..base_context
    };

    let object_prop = create_prop_of_kind_with_name(ctx, PropKind::Object, "object").await;
    let array_prop = create_prop_of_kind_with_name(ctx, PropKind::Array, "array").await;
    array_prop
        .set_parent_prop(ctx, *object_prop.id())
        .await
        .expect("cannot set parent of array");
    let array_element = create_prop_of_kind_with_name(ctx, PropKind::String, "element").await;
    array_element
        .set_parent_prop(ctx, *array_prop.id())
        .await
        .expect("cannot set parent of element");

    let object_attribute_value = AttributeValue::find_with_parent_and_key_for_context(
        ctx,
        None,
        None,
        AttributeReadContext {
            prop_id: Some(*object_prop.id()),
            ..base_context
        },
    )
    .await
    .expect("cannot find attribute value")
    .expect("attribute value not found");

    let view = AttributeView::new(ctx, view_context, Some(*object_attribute_value.id()))
        .await
        .expect("could not create attribute view");
    assert_eq_sorted!(
        serde_json::json![{}], // expected
        view.value().clone(),  // actual
    );

    let array_attribute_value = AttributeValue::find_for_context(
        ctx,
        AttributeReadContext {
            prop_id: Some(*array_prop.id()),
            ..base_context
        },
    )
    .await
    .expect("cannot find attribute value")
    .pop()
    .expect("attribute value not found");

    let insert_context = AttributeContextBuilder::from(base_context)
        .set_prop_id(*array_prop.id())
        .to_context()
        .expect("could not convert builder to attribute context");
    let (element_attribute_value_id, _) = AttributeValue::insert_for_context(
        ctx,
        insert_context,
        *array_attribute_value.id(),
        None,
        None,
    )
    .await
    .expect("cannot insert new array element");

    let update_context = AttributeContextBuilder::from(base_context)
        .set_prop_id(*array_element.id())
        .to_context()
        .expect("could not convert builder to attribute context");
    let update_value = Some(serde_json::to_value("toddhoward").expect("could not create value"));
    let (_, _, _) = AttributeValue::update_for_context(
        ctx,
        element_attribute_value_id,
        Some(*array_attribute_value.id()),
        update_context,
        update_value,
        array_attribute_value.key,
    )
    .await
    .expect("could not update attribute value");

    let view = AttributeView::new(ctx, view_context, Some(*object_attribute_value.id()))
        .await
        .expect("could not create attribute view");
    assert_eq_sorted!(
        serde_json::json![{
            "object": {
                "array": ["toddhoward"],
            },
        }], // expected
        view.value().clone(), // actual
    );
}
