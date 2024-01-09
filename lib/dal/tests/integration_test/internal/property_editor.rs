use dal::func::argument::FuncArgumentKind;
use dal::{
    generate_name,
    property_editor::{schema::PropertyEditorSchema, values::PropertyEditorValues},
    DalContext, Func, FuncArgument, FuncBackendKind, FuncBackendResponseType, LeafInput,
    LeafInputLocation, LeafKind, PropKind, SchemaVariant, StandardModel,
};
use dal_test::helpers::component_bag::ComponentBagger;
use dal_test::test;
use dal_test::test_harness::create_schema;

#[test]
async fn property_editor_schema(ctx: &DalContext) {
    let schema = create_schema(ctx).await;
    let (mut schema_variant, root_prop) = SchemaVariant::new(ctx, *schema.id(), "v0")
        .await
        .expect("could not create schema variant");
    let schema_variant_id = *schema_variant.id();

    // Create a docker-image-ish schema variant.
    dal_test::test_harness::create_prop_without_ui_optionals(
        ctx,
        "poop",
        PropKind::Boolean,
        schema_variant_id,
        Some(root_prop.domain_prop_id),
    )
    .await;
    let exposed_ports_prop = dal_test::test_harness::create_prop_without_ui_optionals(
        ctx,
        "ExposedPorts",
        PropKind::Array,
        schema_variant_id,
        Some(root_prop.domain_prop_id),
    )
    .await;
    dal_test::test_harness::create_prop_without_ui_optionals(
        ctx,
        "ExposedPort",
        PropKind::String,
        schema_variant_id,
        Some(*exposed_ports_prop.id()),
    )
    .await;
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
            result: (input.domain?.poop ?? false) ? 'success' : 'failure'
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
    SchemaVariant::add_leaf(
        ctx,
        qualification_func_id,
        *schema_variant.id(),
        None,
        LeafKind::Qualification,
        vec![LeafInput {
            location: LeafInputLocation::Domain,
            func_argument_id: *qualified_func_argument.id(),
        }],
    )
    .await
    .expect("could not add leaf");

    // Finalize the schema variant.
    schema_variant
        .finalize(ctx, None)
        .await
        .expect("could not finalize");

    // TODO(nick): do something interesting with this.
    let _property_editor_schema =
        PropertyEditorSchema::for_schema_variant(ctx, *schema_variant.id())
            .await
            .expect("cannot create property editor schema from schema variant");
}

#[test]
async fn property_editor_value(ctx: &DalContext) {
    let mut bagger = ComponentBagger::new();
    let name = generate_name();
    let component_bag = bagger.create_component(ctx, &name, "starfield").await;

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    let property_editor_values =
        PropertyEditorValues::for_component(ctx, component_bag.component_id)
            .await
            .expect("cannot create property editor values from context");

    let mut si_name_value = None;
    let mut domain_name_value = None;
    for (_id, value) in property_editor_values.values {
        let prop = value
            .prop(ctx)
            .await
            .expect("could not get prop from property editor value");
        if let Some(parent_prop) = prop
            .parent_prop(ctx)
            .await
            .expect("could not perform parent prop fetch")
        {
            if prop.name() == "name" && parent_prop.name() == "si" {
                if si_name_value.is_some() {
                    panic!("found more than one property editor value with prop \"name\" and parent \"si\"");
                }
                si_name_value = Some(value.value());
            } else if prop.name() == "name" && parent_prop.name() == "domain" {
                if domain_name_value.is_some() {
                    panic!("found more than one property editor value with prop \"name\" and parent \"domain\"");
                }
                domain_name_value = Some(value.value());
            }
        }
    }
    let si_name_value = si_name_value
        .expect("did not find property editor value with prop \"name\" and parent \"si\"");
    let domain_name_value = domain_name_value
        .expect("did not find property editor value with prop \"name\" and parent \"domain\"");
    let found_name = serde_json::to_string(&si_name_value).expect("could not deserialize value");
    assert_eq!(found_name.replace('"', ""), name);
    assert_eq!(si_name_value, domain_name_value);
}
