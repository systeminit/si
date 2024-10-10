use dal::{
    management::{operate, prototype::ManagementPrototype, ManagementFuncReturn},
    AttributeValue, Component, DalContext,
};
use dal_test::{helpers::create_component_for_default_schema_name, test};

#[test]
async fn execute_management_func(ctx: &DalContext) {
    let small_odd_lego =
        create_component_for_default_schema_name(ctx, "small odd lego", "small odd lego")
            .await
            .expect("could not create component");
    let variant = small_odd_lego
        .schema_variant(ctx)
        .await
        .expect("get variant");

    let av_id = Component::attribute_value_for_prop_by_id(
        ctx,
        small_odd_lego.id(),
        &["root", "si", "resourceId"],
    )
    .await
    .expect("av should exist");

    AttributeValue::update(ctx, av_id, Some(serde_json::json!("import id")))
        .await
        .expect("able to update value");

    let management_prototype = ManagementPrototype::list_for_variant_id(ctx, variant.id())
        .await
        .expect("get prototypes")
        .into_iter()
        .find(|proto| proto.name == "Import small odd lego")
        .expect("could not find prototype");

    let (result_value, _) = management_prototype
        .execute(ctx, small_odd_lego.id())
        .await
        .expect("should execute management prototype func");

    let result: ManagementFuncReturn = result_value
        .expect("should have a result success")
        .try_into()
        .expect("should be a valid management func return");

    operate(
        ctx,
        small_odd_lego.id(),
        result.operations.expect("should have operations"),
    )
    .await
    .expect("should operate");

    let av_id = Component::attribute_value_for_prop_by_id(
        ctx,
        small_odd_lego.id(),
        &["root", "domain", "two"],
    )
    .await
    .expect("get four");

    let two_av = AttributeValue::get_by_id_or_error(ctx, av_id)
        .await
        .expect("a fleetwood to my mac");

    let two_value = two_av.value(ctx).await.expect("get value");

    assert_eq!(Some(serde_json::json!("step")), two_value);
}
