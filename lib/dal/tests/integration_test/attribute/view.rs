use crate::dal::test;
use dal::{
    attribute::context::AttributeContextBuilder,
    test_harness::{create_prop_of_kind_with_name, create_schema, create_schema_variant_with_root},
    AttributeReadContext, AttributeValue, AttributeView, DalContext, PropKind, SchemaKind,
    SchemaVariant, StandardModel,
};
use pretty_assertions_sorted::assert_eq_sorted;

// This simplifies making the test more data-driven
enum ValueType {
    Str(String),
    ArrayOfStr(Vec<String>),
}

#[test]
async fn schema_variant_specific(ctx: &DalContext<'_, '_, '_>) {
    // "name": String
    let mut schema = create_schema(ctx, &SchemaKind::Configuration).await;
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

    let mut props = vec![];

    for (name, kind, value) in [
        (
            "name".to_string(),
            PropKind::String,
            ValueType::Str("toddhoward".to_string()),
        ),
        (
            "geniusoflove".to_string(),
            PropKind::Array,
            ValueType::ArrayOfStr(vec![
                "fun".to_string(),
                "natural".to_string(),
                "fun".to_string(),
            ]),
        ),
    ] {
        match value {
            ValueType::Str(_) => {
                let prop = create_prop_of_kind_with_name(ctx, kind, &name).await;
                prop.set_parent_prop(ctx, root_prop.domain_prop_id)
                    .await
                    .expect("cannot set root prop");
                props.push(((prop, None), name, kind, value));
            }
            ValueType::ArrayOfStr(_) => {
                let array_prop = create_prop_of_kind_with_name(ctx, kind, &name).await;
                array_prop
                    .set_parent_prop(ctx, root_prop.domain_prop_id)
                    .await
                    .expect("cannot set root prop");

                let elem_prop =
                    create_prop_of_kind_with_name(ctx, PropKind::String, "elem_ignore").await;
                elem_prop
                    .set_parent_prop(ctx, *array_prop.id())
                    .await
                    .expect("cannot set root prop of elem prop");
                props.push((
                    (array_prop, Some(elem_prop)),
                    "elem_ignore".to_string(),
                    kind,
                    value,
                ));
            }
        }
    }

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
        false,
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

    let view = AttributeView::new(
        ctx,
        AttributeReadContext {
            prop_id: None,
            ..base_context
        },
        Some(*root_attribute_value.id()),
        true,
    )
    .await
    .expect("could not create attribute view");

    assert_eq_sorted!(
        serde_json::json![
            {
                "domain": { "geniusoflove": [], },
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

    let mut new_leaf_props = vec![];

    for ((prop, maybe_elem_prop), name, _, value) in &props {
        let av = AttributeValue::find_for_context(
            ctx,
            AttributeReadContext {
                prop_id: Some(*prop.id()),
                ..base_context
            },
        )
        .await
        .expect("db error fetching AttributeValue")
        .expect("AttributeValue not there");

        let update_context = AttributeContextBuilder::from(base_context)
            .set_prop_id(*prop.id())
            .to_context()
            .expect("could not convert builder to attribute context");

        match value {
            ValueType::Str(string) => {
                let json_value = serde_json::to_value(string).expect("json value creation");
                let (_, updated_av_id) = AttributeValue::update_for_context(
                    ctx,
                    *av.id(),
                    Some(*domain_attribute_value.id()),
                    update_context,
                    Some(json_value.clone()),
                    None,
                )
                .await
                .unwrap_or_else(|_| panic!("could not update attribute value \"{}\"", name));

                new_leaf_props.push((updated_av_id, json_value));
            }
            ValueType::ArrayOfStr(strings) => {
                // create empty array for array value
                let (_, updated_av_id) = AttributeValue::update_for_context(
                    ctx,
                    *av.id(),
                    Some(*domain_attribute_value.id()),
                    update_context,
                    Some(serde_json::json![[]]),
                    None,
                )
                .await
                .unwrap_or_else(|_| panic!("could not update attribute value \"{}\"", name));

                let elem_prop = maybe_elem_prop.clone().unwrap();
                let elem_update_context = AttributeContextBuilder::from(base_context)
                    .set_prop_id(*elem_prop.id())
                    .to_context()
                    .expect("create element update context");

                for elem_value in strings {
                    AttributeValue::insert_for_context(
                        ctx,
                        elem_update_context,
                        updated_av_id,
                        Some(serde_json::to_value(elem_value).expect("jsonification")),
                        None,
                    )
                    .await
                    .expect("could not insert array element");
                }

                new_leaf_props.push((
                    updated_av_id,
                    serde_json::to_value(strings).expect("jsonificatin'"),
                ));
            }
        }
    }

    let view = AttributeView::new(
        ctx,
        AttributeReadContext {
            prop_id: None,
            ..base_context
        },
        Some(*root_attribute_value.id()),
        false,
    )
    .await
    .expect("could not create attribute view");

    assert_eq_sorted!(
        serde_json::json![
            {
                "domain": {
                    "name": "toddhoward",
                    "geniusoflove": ["fun", "natural", "fun"],
                },
                "si": {}
            }
        ], // expected
        view.value().clone(), // actual
    );

    // Ensure that leaf generation works as well.
    for (av_id, value) in &new_leaf_props {
        let view = AttributeView::new(
            ctx,
            AttributeReadContext {
                prop_id: None,
                ..base_context
            },
            Some(*av_id),
            false,
        )
        .await
        .expect("could not create attribute view");
        assert_eq_sorted!(
            value.clone(),        // expected
            view.value().clone(), // actual
        );
    }

    // Can we generate a view of *just* the domain?
    let view = AttributeView::new(
        ctx,
        AttributeReadContext {
            prop_id: None,
            ..base_context
        },
        Some(*domain_attribute_value.id()),
        false,
    )
    .await
    .expect("could not create attribute view for domain");

    assert_eq_sorted!(
        serde_json::json![
            {
                "name": "toddhoward",
                "geniusoflove": ["fun", "natural", "fun"],
            }
        ], // expected
        view.value().clone(), // actual
    );
}
