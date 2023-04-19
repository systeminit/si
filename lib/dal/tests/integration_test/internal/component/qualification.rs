use dal::func::argument::{FuncArgument, FuncArgumentKind};
use dal::schema::variant::leaves::LeafKind;
use dal::{
    attribute::context::AttributeContextBuilder,
    qualification::QualificationSubCheckStatus,
    schema::variant::leaves::{LeafInput, LeafInputLocation},
    AttributeReadContext, AttributeValue, Component, ComponentView, DalContext, Func,
    FuncBackendKind, FuncBackendResponseType, Prop, PropKind, SchemaVariant, StandardModel,
};
use dal_test::test;
use dal_test::test_harness::{create_schema, create_schema_variant_with_root};
use pretty_assertions_sorted::assert_eq;

#[test]
async fn add_and_list_qualifications(ctx: &DalContext) {
    let mut schema = create_schema(ctx).await;
    let (mut schema_variant, root_prop) = create_schema_variant_with_root(ctx, *schema.id()).await;
    let schema_variant_id = *schema_variant.id();
    schema
        .set_default_schema_variant_id(ctx, Some(schema_variant_id))
        .await
        .expect("cannot set default schema variant");
    let schema_variant_id = *schema_variant.id();

    let poop_prop = Prop::new(
        ctx,
        "poop",
        PropKind::Boolean,
        None,
        schema_variant_id,
        Some(root_prop.domain_prop_id),
    )
    .await
    .expect("could not create prop");

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
    let code = r##"function isQualified(input) {
        return {
            result: (input.domain?.poop ?? false) ? 'success' : 'failure',
            message: "must be present when result is not 'success'",
        };
    }"##;
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
        schema_variant_id,
        None,
        LeafKind::Qualification,
        vec![LeafInput {
            location: LeafInputLocation::Domain,
            func_argument_id: *qualified_func_argument.id(),
        }],
    )
    .await
    .expect("could not add qualification");

    // Finalize the schema variant and create the component.
    schema_variant
        .finalize(ctx, None)
        .await
        .expect("unable to finalize schema variant");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    let (component, _) = Component::new(ctx, "component", schema_variant_id)
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

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    // Observe that the qualification worked.
    let component_view = ComponentView::new(ctx, *component.id())
        .await
        .expect("could not generate component view");
    assert_eq!(
        serde_json::json![
            {
                "si": {
                    "name": "component",
                    "type": "component",
                    "protected": false
                },
                "domain": {
                    "poop": true,
                },
                "qualification": {
                    "test:qualification": {
                        "result": "success",
                        "message": "must be present when result is not 'success'",
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
            _ => panic!("found unexpected qualification: {found_qualification:?}"),
        }
    }
    let all_fields_valid_qualification =
        all_fields_valid_qualification.expect("could not find all fields valid qualification");
    let test_qualification = test_qualification.expect("could not find test qualification");

    assert_eq!(
        all_fields_valid_qualification
            .result
            .expect("could not get result")
            .status,
        QualificationSubCheckStatus::Success,
    );
    assert_eq!(
        test_qualification
            .result
            .expect("could not get result")
            .status,
        QualificationSubCheckStatus::Success,
    );
}
