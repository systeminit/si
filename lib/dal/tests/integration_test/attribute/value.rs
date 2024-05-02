use dal::prop::PropPath;
use dal::{AttributeValue, Component, DalContext, Prop, Schema};
use dal_test::helpers::ChangeSetTestHelpers;
use dal_test::test;
use pretty_assertions_sorted::assert_eq;

#[test]
async fn arguments_for_prototype_function_execution(ctx: &mut DalContext) {
    let schema = Schema::find_by_name(ctx, "swifty")
        .await
        .expect("could not perform find by name")
        .expect("schema not found");
    let schema_variant_id = schema
        .get_default_schema_variant_id(ctx)
        .await
        .expect("could not perform get default schema variant")
        .expect("schema variant not found");

    // Create a component and commit. For context, the test exclusive schema has the identity
    // function set on "/root/domain/name" with an input from "/root/si/name". We need to ensure
    // that the value of "/root/si/name" comes in, as expected. The name is set when creating a
    // component, so we do not need to do additional setup.
    let expected = "you should see this name in the arguments";
    let _component = Component::new(ctx, expected, schema_variant_id)
        .await
        .expect("could not create component");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Ensure that the arguments look as we expect.
    let prop_id = Prop::find_prop_id_by_path(
        ctx,
        schema_variant_id,
        &PropPath::new(["root", "domain", "name"]),
    )
    .await
    .expect("could not find prop id by path");
    let mut attribute_value_ids = Prop::attribute_values_for_prop_id(ctx, prop_id)
        .await
        .expect("could not list attribute value ids for prop id");
    let attribute_value_id = attribute_value_ids
        .pop()
        .expect("empty attribute value ids");
    assert!(attribute_value_ids.is_empty());
    let (_, arguments) =
        AttributeValue::prepare_arguments_for_prototype_function_execution(ctx, attribute_value_id)
            .await
            .expect("could not prepare arguments");
    assert_eq!(
        serde_json::json![{
            "identity": expected
        }], // expected
        arguments // actual
    );
}
