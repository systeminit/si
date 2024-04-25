use dal::attribute::prototype::argument::AttributePrototypeArgument;
use dal::func::argument::{FuncArgument, FuncArgumentKind};
use dal::func::FuncArgumentBag;
use dal::func::{AttributePrototypeArgumentBag, AttributePrototypeBag, FuncAssociations};
use dal::prop::PropPath;
use dal::schema::variant::leaves::LeafInputLocation;
use dal::{AttributePrototype, DalContext, DeprecatedActionKind, Func, Prop, Schema};
use dal_test::test;
use pretty_assertions_sorted::assert_eq;

#[test]
async fn for_action(ctx: &mut DalContext) {
    let schema = Schema::find_by_name(ctx, "swifty")
        .await
        .expect("could not perform find by name")
        .expect("no schema found");
    let schema_variant_id = schema
        .get_default_schema_variant_id(ctx)
        .await
        .expect("could not perform get default schema variant")
        .expect("default schema variant not found");

    let func_id = Func::find_by_name(ctx, "test:createActionSwifty")
        .await
        .expect("could not perform find func by name")
        .expect("func not found");
    let func = Func::get_by_id_or_error(ctx, func_id)
        .await
        .expect("could not get func by id");
    let (associations, _input_type) = FuncAssociations::from_func(ctx, &func)
        .await
        .expect("could not get associations");

    assert_eq!(
        FuncAssociations::Action {
            kind: DeprecatedActionKind::Create,
            schema_variant_ids: vec![schema_variant_id],
        }, // expected
        associations.expect("no associations found") // actual
    );
}

#[test]
async fn for_attribute(ctx: &mut DalContext) {
    let schema = Schema::find_by_name(ctx, "starfield")
        .await
        .expect("could not perform find by name")
        .expect("no schema found");
    let schema_variant_id = schema
        .get_default_schema_variant_id(ctx)
        .await
        .expect("could not perform get default schema variant")
        .expect("default schema variant not found");

    let func_id = Func::find_by_name(ctx, "test:falloutEntriesToGalaxies")
        .await
        .expect("could not perform find func by name")
        .expect("func not found");
    let func = Func::get_by_id_or_error(ctx, func_id)
        .await
        .expect("could not get func by id");
    let (associations, _input_type) = FuncAssociations::from_func(ctx, &func)
        .await
        .expect("could not get associations");

    // Find the sole func argument id. Ensure there is only one.
    let mut func_argument_ids = FuncArgument::list_ids_for_func(ctx, func_id)
        .await
        .expect("could not list func argument ids");
    let func_argument_id = func_argument_ids
        .pop()
        .expect("func argument ids are empty");
    assert!(func_argument_ids.is_empty());

    // Find the sole attribute prototype  id. Ensure there is only one.
    let mut attribute_prototype_ids = AttributePrototype::list_ids_for_func_id(ctx, func_id)
        .await
        .expect("could not list attribute prototype ids");
    let attribute_prototype_id = attribute_prototype_ids
        .pop()
        .expect("attribute prototype ids are empty");
    assert!(attribute_prototype_ids.is_empty());

    // Find the sole attribute prototype argument id. Ensure there is only one.
    let mut attribute_prototype_argument_ids =
        AttributePrototypeArgument::list_ids_for_prototype(ctx, attribute_prototype_id)
            .await
            .expect("could not list attribute prototype argument ids");
    let attribute_prototype_argument_id = attribute_prototype_argument_ids
        .pop()
        .expect("attribute prototype argument ids are empty");
    assert!(attribute_prototype_argument_ids.is_empty());

    // Find the sole input socket id. Ensure there is only one.
    let mut input_socket_ids =
        AttributePrototype::list_input_socket_sources_for_id(ctx, attribute_prototype_id)
            .await
            .expect("could not list input socket ids");
    let input_socket_id = input_socket_ids.pop().expect("input socket ids are empty");
    assert!(input_socket_ids.is_empty());

    // Find the prop for the prototype.
    let prop = Prop::find_prop_by_path(
        ctx,
        schema_variant_id,
        &PropPath::new(["root", "domain", "universe", "galaxies"]),
    )
    .await
    .expect("could not find prop by path");

    assert_eq!(
        FuncAssociations::Attribute {
            prototypes: vec![AttributePrototypeBag {
                id: attribute_prototype_id,
                component_id: None,
                schema_variant_id: Some(schema_variant_id),
                prop_id: Some(prop.id),
                output_socket_id: None,
                prototype_arguments: vec![AttributePrototypeArgumentBag {
                    func_argument_id,
                    id: attribute_prototype_argument_id,
                    input_socket_id: Some(input_socket_id),
                }],
            }],
            arguments: vec![FuncArgumentBag {
                id: func_argument_id,
                name: "entries".to_string(),
                kind: FuncArgumentKind::Array,
                element_kind: Some(FuncArgumentKind::Object),
            }],
        }, // expected
        associations.expect("no associations found") // actual
    );
}

#[test]
async fn for_authentication(ctx: &mut DalContext) {
    let schema = Schema::find_by_name(ctx, "dummy-secret")
        .await
        .expect("could not perform find by name")
        .expect("no schema found");
    let schema_variant_id = schema
        .get_default_schema_variant_id(ctx)
        .await
        .expect("could not perform get default schema variant")
        .expect("default schema variant not found");

    let func_id = Func::find_by_name(ctx, "test:setDummySecretString")
        .await
        .expect("could not perform find func by name")
        .expect("func not found");
    let func = Func::get_by_id_or_error(ctx, func_id)
        .await
        .expect("could not get func by id");
    let (associations, _input_type) = FuncAssociations::from_func(ctx, &func)
        .await
        .expect("could not get associations");

    assert_eq!(
        FuncAssociations::Authentication {
            schema_variant_ids: vec![schema_variant_id],
        }, // expected
        associations.expect("no associations found") // actual
    );
}

#[test]
async fn for_code_generation(ctx: &mut DalContext) {
    let schema = Schema::find_by_name(ctx, "katy perry")
        .await
        .expect("could not perform find by name")
        .expect("no schema found");
    let schema_variant_id = schema
        .get_default_schema_variant_id(ctx)
        .await
        .expect("could not perform get default schema variant")
        .expect("default schema variant not found");

    let func_id = Func::find_by_name(ctx, "test:generateStringCode")
        .await
        .expect("could not perform find func by name")
        .expect("func not found");
    let func = Func::get_by_id_or_error(ctx, func_id)
        .await
        .expect("could not get func by id");
    let (associations, _input_type) = FuncAssociations::from_func(ctx, &func)
        .await
        .expect("could not get associations");

    assert_eq!(
        FuncAssociations::CodeGeneration {
            schema_variant_ids: vec![schema_variant_id],
            component_ids: vec![],
            inputs: vec![LeafInputLocation::Domain]
        }, // expected
        associations.expect("no associations found") // actual
    );
}

#[test]
async fn for_qualification(ctx: &mut DalContext) {
    let schema = Schema::find_by_name(ctx, "dummy-secret")
        .await
        .expect("could not perform find by name")
        .expect("no schema found");
    let schema_variant_id = schema
        .get_default_schema_variant_id(ctx)
        .await
        .expect("could not perform get default schema variant")
        .expect("default schema variant not found");

    let func_id = Func::find_by_name(ctx, "test:qualificationDummySecretStringIsTodd")
        .await
        .expect("could not perform find func by name")
        .expect("func not found");
    let func = Func::get_by_id_or_error(ctx, func_id)
        .await
        .expect("could not get func by id");
    let (associations, _input_type) = FuncAssociations::from_func(ctx, &func)
        .await
        .expect("could not get associations");

    assert_eq!(
        FuncAssociations::Qualification {
            schema_variant_ids: vec![schema_variant_id],
            component_ids: vec![],
            inputs: vec![LeafInputLocation::Secrets]
        }, // expected
        associations.expect("no associations found") // actual
    );
}
