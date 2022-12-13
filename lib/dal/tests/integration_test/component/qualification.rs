use dal::attribute::context::AttributeContextBuilder;
use dal::func::argument::{FuncArgument, FuncArgumentKind};
use dal::schema::variant::leaves::LeafKind;
use dal::{
    AttributeReadContext, AttributeValue, Component, ComponentView, DalContext, Func,
    FuncBackendKind, FuncBackendResponseType, PropKind, SchemaKind, SchemaVariant, StandardModel,
};
use dal_test::test;
use dal_test::test_harness::{
    create_prop_and_set_parent, create_schema, create_schema_variant_with_root,
};
use pretty_assertions_sorted::assert_eq;

#[test]
async fn add_and_list_qualifications(ctx: &DalContext) {
    let mut schema = create_schema(ctx, &SchemaKind::Configuration).await;
    let (schema_variant, root_prop) = create_schema_variant_with_root(ctx, *schema.id()).await;
    let schema_variant_id = *schema_variant.id();
    schema
        .set_default_schema_variant_id(ctx, Some(schema_variant_id))
        .await
        .expect("cannot set default schema variant");
    let poop_prop =
        create_prop_and_set_parent(ctx, PropKind::Boolean, "poop", root_prop.domain_prop_id).await;

    // Create a qualification func and ensure the func argument is ready to take in an object. In
    // a qualification leaf's case, that will be "/root/domain".
    let mut qualification_func = Func::new(
        ctx,
        "test:qualification",
        FuncBackendKind::JsAttribute,
        FuncBackendResponseType::Qualification,
    )
    .await
    .expect("could not create func");
    let qualification_func_id = *qualification_func.id();
    let code = "function isQualified(input) {
        return {
            qualified: input.domain?.poop ?? false
        };
    }";
    qualification_func
        .set_code_plaintext(ctx, Some(code))
        .await
        .expect("set code");
    qualification_func
        .set_handler(ctx, Some("isQualified"))
        .await
        .expect("set handler");
    let qualified_func_argument = FuncArgument::new(
        ctx,
        "domain",
        FuncArgumentKind::Object,
        None,
        qualification_func_id,
    )
    .await
    .expect("could not create func argument");

    // Add the leaf for the qualification.
    SchemaVariant::add_leaf(
        ctx,
        qualification_func_id,
        *qualified_func_argument.id(),
        schema_variant_id,
        LeafKind::Qualification,
    )
    .await
    .expect("could not add qualification");

    // Finalize the schema variant and create the component.
    schema_variant
        .finalize(ctx)
        .await
        .expect("unable to finalize schema variant");

    let (component, _) =
        Component::new_for_schema_variant_with_node(ctx, "component", &schema_variant_id)
            .await
            .expect("cannot create component");

    // Set a value on the prop to check if our qualification works as intended.
    let read_context = AttributeReadContext {
        prop_id: Some(*poop_prop.id()),
        component_id: Some(*component.id()),
        ..AttributeReadContext::default()
    };
    let attribute_value = AttributeValue::find_for_context(ctx, read_context)
        .await
        .expect("could not perform find for context")
        .expect("attribute value not found");
    let parent_attribute_value = attribute_value
        .parent_attribute_value(ctx)
        .await
        .expect("could not perform find parent attribute value")
        .expect("no parent attribute value found");
    let context = AttributeContextBuilder::from(read_context)
        .to_context()
        .expect("could not convert builder to attribute context");
    AttributeValue::update_for_context(
        ctx,
        *attribute_value.id(),
        Some(*parent_attribute_value.id()),
        context,
        Some(serde_json::json![true]),
        None,
    )
    .await
    .expect("could not perform update for context");

    // Observe that the qualification worked.
    let component_view = ComponentView::new(ctx, *component.id())
        .await
        .expect("could not generate component view");
    assert_eq!(
        serde_json::json![
            {
                "si": {
                    "name": "component",
                },
                "domain": {
                    "poop": true,
                },
                "qualification": {
                    "test:qualification": {
                        "qualified": true
                    },
                }
        }], // expected
        component_view.properties // actual
    );

    // List qualifications, check that we only find two and then ensure they look as we expect.
    let found_qualifications = Component::list_qualifications(ctx, *component.id())
        .await
        .expect("cannot list qualifications");
    assert_eq!(found_qualifications.len(), 2);

    let mut all_fields_valid_qualification = None;
    let mut test_qualification = None;
    for found_qualification in found_qualifications {
        match found_qualification.title.as_str() {
            "All fields are valid" => {
                assert!(
                    all_fields_valid_qualification.is_none(),
                    "already found all fields valid"
                );
                all_fields_valid_qualification = Some(found_qualification);
            }
            "test:qualification" => {
                assert!(
                    test_qualification.is_none(),
                    "already found all fields valid"
                );
                test_qualification = Some(found_qualification);
            }
            _ => panic!("found unexpected qualification: {:?}", found_qualification),
        }
    }
    let all_fields_valid_qualification =
        all_fields_valid_qualification.expect("could not find all fields valid qualification");
    let test_qualification = test_qualification.expect("could not find test qualification");

    assert!(
        all_fields_valid_qualification
            .result
            .expect("could not get result")
            .success
    );
    assert!(
        test_qualification
            .result
            .expect("could not get result")
            .success
    );
}
