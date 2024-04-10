use dal::func::argument::{FuncArgument, FuncArgumentKind};
use dal::func::view::FuncArgumentView;
use dal::func::FuncAssociations;
use dal::schema::variant::leaves::LeafInputLocation;
use dal::{DalContext, Func, Schema};
use dal_test::test;
use pretty_assertions_sorted::assert_eq;

#[test]
async fn for_action(ctx: &mut DalContext) {
    let schema = Schema::find_by_name(ctx, "swifty")
        .await
        .expect("could not perform find by name")
        .expect("no schema found");
    let schema_variant_id = schema
        .get_default_schema_variant(ctx)
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
            schema_variant_ids: vec![schema_variant_id],
        }, // expected
        associations.expect("no associations found") // actual
    );
}

#[test]
async fn for_attribute(ctx: &mut DalContext) {
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

    assert_eq!(
        FuncAssociations::Attribute {
            prototypes: vec![],
            arguments: vec![FuncArgumentView {
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
        .get_default_schema_variant(ctx)
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
        .get_default_schema_variant(ctx)
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
        .get_default_schema_variant(ctx)
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
