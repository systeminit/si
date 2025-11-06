use dal::{
    DalContext,
    Func,
    Prop,
    Schema,
    SchemaVariant,
    func::authoring::FuncAuthoringClient,
};
use dal_test::{
    helpers::create_unlocked_variant_copy_for_schema_name,
    test,
};
use pretty_assertions_sorted::assert_eq;

mod argument;
mod authoring;
mod debug;

#[test]
async fn summary(ctx: &mut DalContext) {
    let schema = Schema::get_by_name(ctx, "starfield")
        .await
        .expect("schema not found");
    let schema_variant_id = SchemaVariant::default_id_for_schema(ctx, schema.id())
        .await
        .expect("no schema variant found");

    // Ensure that the same func can be found within its schema variant and for all funcs in the workspace.
    let funcs_for_schema_variant = SchemaVariant::all_funcs(ctx, schema_variant_id)
        .await
        .expect("could not list funcs");
    let all_funcs = Func::list_for_default_and_editing(ctx)
        .await
        .expect("could not list all funcs");

    let func_name = "test:createActionStarfield".to_string();
    let found_func_for_all = all_funcs
        .iter()
        .find(|f| f.name == func_name)
        .expect("could not find func");
    let found_func_for_schema_variant = funcs_for_schema_variant
        .iter()
        .find(|f| f.name == func_name)
        .expect("could not find func");

    assert_eq!(found_func_for_all, found_func_for_schema_variant);
}

#[test]
async fn duplicate(ctx: &mut DalContext) {
    let schema_variant_id = create_unlocked_variant_copy_for_schema_name(ctx, "starfield")
        .await
        .expect("could not create unlocked copy");
    let func_name = "Paul's Test Func".to_string();
    let authoring_func =
        FuncAuthoringClient::create_new_auth_func(ctx, Some(func_name.clone()), schema_variant_id)
            .await
            .expect("unable to create func");

    let func = Func::get_by_id(ctx, authoring_func.id)
        .await
        .expect("Unable to get the authored func");

    let duplicated_func_name = "Paul's Test Func Clone".to_string();
    let duplicated_func = func
        .clone_func_with_new_name(ctx, duplicated_func_name)
        .await
        .expect("Unable to duplicate the func");

    assert_eq!(duplicated_func.display_name, func.display_name);
    assert_eq!(duplicated_func.description, func.description);
    assert_eq!(duplicated_func.link, func.link);
    assert_eq!(duplicated_func.hidden, func.hidden);
    assert_eq!(duplicated_func.backend_kind, func.backend_kind);
    assert_eq!(
        duplicated_func.backend_response_type,
        func.backend_response_type
    );
    assert_eq!(duplicated_func.handler, func.handler);
    assert_eq!(duplicated_func.code_base64, func.code_base64);
    assert_eq!(false, duplicated_func.is_locked);
}

#[test]
async fn get_ts_type_from_root(ctx: &mut DalContext) {
    let schema = Schema::get_by_name(ctx, "starfield")
        .await
        .expect("schema not found");
    let schema_variant_id = Schema::default_variant_id(ctx, schema.id())
        .await
        .expect("could not perform get default schema variant");

    let root_prop_id = SchemaVariant::get_root_prop_id(ctx, schema_variant_id)
        .await
        .expect("could not get root prop id");
    // TODO(nick): check that the ts type is right!
    let _ts_type = Prop::ts_type(ctx, root_prop_id)
        .await
        .expect("could not get ts type");
}
