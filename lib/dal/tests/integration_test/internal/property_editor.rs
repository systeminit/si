use dal::func::argument::FuncArgumentKind;
use dal::{
    generate_name,
    property_editor::{schema::PropertyEditorSchema, values::PropertyEditorValues},
    Component, DalContext, Func, FuncArgument, FuncBackendKind, FuncBackendResponseType, LeafInput,
    LeafInputLocation, LeafKind, Prop, PropKind, Schema, SchemaVariant, StandardModel,
};
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
    let _poop_prop = Prop::new(
        ctx,
        "poop",
        PropKind::Boolean,
        None,
        schema_variant_id,
        Some(root_prop.domain_prop_id),
    )
    .await
    .expect("could not create prop");
    let exposed_ports_prop = Prop::new(
        ctx,
        "ExposedPorts",
        PropKind::Array,
        None,
        schema_variant_id,
        Some(root_prop.domain_prop_id),
    )
    .await
    .expect("could not create prop");
    let _exposed_port_prop = Prop::new(
        ctx,
        "ExposedPort",
        PropKind::String,
        None,
        schema_variant_id,
        Some(*exposed_ports_prop.id()),
    )
    .await
    .expect("could not create prop");
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
    let schema = Schema::find_by_attr(ctx, "name", &"Docker Image".to_string())
        .await
        .expect("cannot find docker image schema")
        .pop()
        .expect("no docker image schema found");
    let schema_variant_id = schema
        .default_schema_variant_id()
        .expect("missing default schema variant id");
    let name = generate_name();
    let (component, _node) = Component::new(ctx, &name, *schema_variant_id)
        .await
        .expect("could not create component");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    let property_editor_values = PropertyEditorValues::for_component(ctx, *component.id())
        .await
        .expect("cannot create property editor values from context");

    let mut name_value = None;
    let mut image_value = None;
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
                if name_value.is_some() {
                    panic!("found more than one property editor value with prop \"name\" and parent \"si\"");
                }
                name_value = Some(value.value());
            } else if prop.name() == "image" && parent_prop.name() == "domain" {
                if image_value.is_some() {
                    panic!("found more than one property editor value with prop \"image\" and parent \"domain\"");
                }
                image_value = Some(value.value());
            }
        }
    }
    let name_value = name_value
        .expect("did not find property editor value with prop \"name\" and parent \"si\"");
    let image_value = image_value
        .expect("did not find property editor value with prop \"image\" and parent \"domain\"");
    let found_name = serde_json::to_string(&name_value).expect("could not deserialize value");
    assert_eq!(found_name.replace('"', ""), name);
    assert_eq!(name_value, image_value);
}
