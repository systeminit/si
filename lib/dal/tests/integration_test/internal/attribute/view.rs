use dal::{
    attribute::context::AttributeContextBuilder, AttributeReadContext, AttributeValue,
    AttributeView, DalContext, PropKind, SchemaVariant, StandardModel,
};
use dal_test::test_harness::create_prop_and_set_parent;
use dal_test::{
    test,
    test_harness::{create_schema, create_schema_variant_with_root},
};
use pretty_assertions_sorted::assert_eq;

#[test]
async fn schema_variant_specific(ctx: &DalContext) {
    // "name": String
    let mut schema = create_schema(ctx).await;
    let (schema_variant, root_prop) = create_schema_variant_with_root(ctx, *schema.id()).await;
    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await
        .expect("cannot set default schema variant");
    let name_prop =
        create_prop_and_set_parent(ctx, PropKind::String, "name", root_prop.domain_prop_id).await;

    SchemaVariant::create_default_prototypes_and_values(ctx, *schema_variant.id())
        .await
        .expect("cannot create default prototypes and values for SchemaVariant");

    let root_attribute_value = AttributeValue::find_with_parent_and_key_for_context(
        ctx,
        None,
        None,
        AttributeReadContext {
            prop_id: Some(root_prop.prop_id),
            ..AttributeReadContext::default()
        },
    )
    .await
    .expect("cannot find attribute value")
    .expect("attribute value not found");

    let view = AttributeView::new(
        ctx,
        AttributeReadContext {
            prop_id: None,
            ..AttributeReadContext::default()
        },
        Some(*root_attribute_value.id()),
    )
    .await
    .expect("could not create attribute view");

    assert_eq!(
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
            ..AttributeReadContext::default()
        },
    )
    .await
    .expect("cannot find attribute value")
    .expect("attribute value not found");

    let name_attribute_value = AttributeValue::find_for_context(
        ctx,
        AttributeReadContext {
            prop_id: Some(*name_prop.id()),
            ..AttributeReadContext::default()
        },
    )
    .await
    .expect("cannot find attribute value")
    .expect("attribute value not found");

    let update_context = AttributeContextBuilder::from(AttributeReadContext::default())
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
            ..AttributeReadContext::default()
        },
        Some(*root_attribute_value.id()),
    )
    .await
    .expect("could not create attribute view");

    assert_eq!(
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
            ..AttributeReadContext::default()
        },
        Some(updated_name_attribute_value_id),
    )
    .await
    .expect("could not create attribute view");
    assert_eq!(
        serde_json::json!["toddhoward"], // expected
        view.value().clone(),            // actual
    );
}
