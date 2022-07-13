use crate::dal::test;
use dal::{
    attribute::context::AttributeContextBuilder,
    test_harness::{create_prop_of_kind_with_name, create_schema, create_schema_variant_with_root},
    AttributeReadContext, AttributeValue, AttributeView, DalContext, PropKind, SchemaKind,
    SchemaVariant, StandardModel,
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

    SchemaVariant::create_default_prototypes_and_values(ctx, *schema_variant.id())
        .await
        .expect("cannot create default prototypes and values for SchemaVariant");

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
                "domain": {},
                "si": {}
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
    .expect("attribute value not found");

    let update_context = AttributeContextBuilder::from(base_context)
        .set_prop_id(*name_prop.id())
        .to_context()
        .expect("could not convert builder to attribute context");
    let update_value = Some(serde_json::to_value("toddhoward").expect("could not create value"));
    let (_, updated_name_attribute_value_id) = AttributeValue::update_for_context(
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
                "domain": {
                    "name": "toddhoward"
                },
                "si": {}
            }
        ], // expected
        view.value().clone(), // actual
    );

    // Ensure that leaf generation works as well.
    let view = AttributeView::new(
        ctx,
        AttributeReadContext {
            prop_id: None,
            ..base_context
        },
        Some(updated_name_attribute_value_id),
    )
    .await
    .expect("could not create attribute view");
    assert_eq_sorted!(
        serde_json::json!["toddhoward"], // expected
        view.value().clone(),            // actual
    );
}
