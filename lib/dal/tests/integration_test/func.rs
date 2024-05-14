use dal::func::argument::{FuncArgument, FuncArgumentId, FuncArgumentKind};
use dal::func::authoring::FuncAuthoringClient;
use dal::func::summary::FuncSummary;
use dal::func::view::FuncView;
use dal::func::{FuncArgumentBag, FuncAssociations, FuncKind};
use dal::{DalContext, Func, FuncError, Prop, Schema, SchemaVariant};
use dal_test::helpers::ChangeSetTestHelpers;
use dal_test::test;
use pretty_assertions_sorted::assert_eq;

mod argument;
mod associations;
mod authoring;

#[test]
async fn summary(ctx: &mut DalContext) {
    let schema = Schema::find_by_name(ctx, "starfield")
        .await
        .expect("could not find schema")
        .expect("schema not found");
    let schema_variant_id = SchemaVariant::get_default_id_for_schema(ctx, schema.id())
        .await
        .expect("no schema variant found");

    // Ensure that the same func can be found within its schema variant and for all funcs in the workspace.
    let funcs_for_schema_variant = FuncSummary::list_for_schema_variant_id(ctx, schema_variant_id)
        .await
        .expect("could not list func summaries");
    let all_funcs = FuncSummary::list(ctx)
        .await
        .expect("could not list func summaries");

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
    let func_name = "Paul's Test Func".to_string();
    let authoring_func = FuncAuthoringClient::create_func(
        ctx,
        FuncKind::Qualification,
        Some(func_name.clone()),
        None,
    )
    .await
    .expect("unable to create func");

    let func = Func::get_by_id_or_error(ctx, authoring_func.id)
        .await
        .expect("Unable to get the authored func");

    let duplicated_func_name = "Paul's Test Func Clone".to_string();
    let duplicated_func = func
        .duplicate(ctx, duplicated_func_name)
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
}

#[test]
async fn create_and_delete_attribute_func_with_arguments(ctx: &mut DalContext) {
    // Declare variables for use throughout the test.
    let func_name = "Chloe or Sam or Sophia or Marcus";
    let string_func_argument_name = "Chloe or Sam";
    let array_func_argument_name = "Sophia or Marcus";
    let string_func_argument_bag = FuncArgumentBag {
        id: FuncArgumentId::NONE,
        name: string_func_argument_name.to_string(),
        kind: FuncArgumentKind::String,
        element_kind: None,
    };
    let array_func_argument_bag = FuncArgumentBag {
        id: FuncArgumentId::NONE,
        name: array_func_argument_name.to_string(),
        kind: FuncArgumentKind::Array,
        element_kind: Some(FuncArgumentKind::String),
    };

    // Create an attribute func and commit. Cache the func id because it will be stable for the
    // entire life of the func.
    let func_id = {
        let func = FuncAuthoringClient::create_func(
            ctx,
            FuncKind::Attribute,
            Some(func_name.to_owned()),
            None,
        )
        .await
        .expect("unable to create func");
        ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
            .await
            .expect("could not commit and update snapshot to visibility");
        func.id
    };

    // Fetch the view and prepare for adding two func arguments. Then, add the two func arguments
    // and commit.
    {
        let func = Func::get_by_id_or_error(ctx, func_id)
            .await
            .expect("could not get func");
        let func_view = FuncView::assemble(ctx, &func)
            .await
            .expect("could not assemble func view");
        let (prototypes, arguments) = func_view
            .associations
            .expect("could not get associations")
            .get_attribute_internals()
            .expect("could not get internals");
        assert!(prototypes.is_empty());
        assert!(arguments.is_empty());

        FuncAuthoringClient::save_func(
            ctx,
            func_id,
            func_view.display_name,
            func_view.name,
            func_view.description,
            func_view.code,
            Some(FuncAssociations::Attribute {
                prototypes,
                arguments: vec![
                    string_func_argument_bag.clone(),
                    array_func_argument_bag.clone(),
                ],
            }),
        )
        .await
        .expect("unable to save func");
        ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
            .await
            .expect("could not commit and update snapshot to visibility");
    }

    // Fetch the view and check that we added two func arguments successfully.
    let (cached_string_func_argument_id, cached_array_func_argument_id) = {
        let func = Func::get_by_id_or_error(ctx, func_id)
            .await
            .expect("could not get func");
        let func_view = FuncView::assemble(ctx, &func)
            .await
            .expect("could not assemble func view");
        let (prototypes, arguments) = func_view
            .associations
            .expect("could not get associations")
            .get_attribute_internals()
            .expect("could not get internals");
        assert!(prototypes.is_empty());
        assert_eq!(
            2,               // expected
            arguments.len()  // actual
        );

        let found_string_func_argument = arguments
            .iter()
            .find(|bag| bag.name == string_func_argument_name)
            .expect("could not find bag");
        let found_array_func_argument = arguments
            .iter()
            .find(|bag| bag.name == array_func_argument_name)
            .expect("could not find bag");
        assert_eq!(
            string_func_argument_bag.kind,   // expected
            found_string_func_argument.kind  // actual
        );
        assert_eq!(
            string_func_argument_bag.element_kind,   // expected
            found_string_func_argument.element_kind  // actual
        );
        assert_eq!(
            array_func_argument_bag.kind,   // expected
            found_array_func_argument.kind  // actual
        );
        assert_eq!(
            array_func_argument_bag.element_kind,   // expected
            found_array_func_argument.element_kind  // actual
        );

        (found_string_func_argument.id, found_array_func_argument.id)
    };

    // Try to delete the func. The deletion will fail because it has associations, which are the
    // two new arguments. Then, check that the two func arguments still exist.
    {
        match Func::delete_by_id(ctx, func_id).await {
            Ok(_) => panic!("deletion should fail since func has associations"),
            Err(FuncError::FuncToBeDeletedHasAssociations(_)) => {}
            Err(err) => panic!("unexpected error: {err:?}"),
        }

        let found_string_func_argument =
            FuncArgument::find_by_name_for_func(ctx, string_func_argument_name, func_id)
                .await
                .expect("could not perform find by name for func")
                .expect("func argument not found");
        let found_array_func_argument =
            FuncArgument::find_by_name_for_func(ctx, array_func_argument_name, func_id)
                .await
                .expect("could not perform find by name for func")
                .expect("func argument not found");
        assert_eq!(
            cached_string_func_argument_id, // expected
            found_string_func_argument.id   // actual
        );
        assert_eq!(
            cached_array_func_argument_id, // expected
            found_array_func_argument.id   // actual
        );
    }

    // Fetch the view. Then, delete the two arguments via "save func" and commit.
    {
        let func = Func::get_by_id_or_error(ctx, func_id)
            .await
            .expect("could not get func");
        let func_view = FuncView::assemble(ctx, &func)
            .await
            .expect("could not assemble func view");
        let (prototypes, arguments) = func_view
            .associations
            .expect("could not get associations")
            .get_attribute_internals()
            .expect("could not get internals");
        assert!(prototypes.is_empty());
        assert_eq!(
            2,               // expected
            arguments.len()  // actual
        );
        FuncAuthoringClient::save_func(
            ctx,
            func_id,
            func_view.display_name,
            func_view.name,
            func_view.description,
            func_view.code,
            Some(FuncAssociations::Attribute {
                prototypes,
                arguments: Vec::new(),
            }),
        )
        .await
        .expect("unable to save func");
        ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
            .await
            .expect("could not commit and update snapshot to visibility");
    }

    // Now that the func arguments are deleted, perform the deletion and commit. Then, check that
    // the func and its arguments were actually deleted.
    {
        let deleted_func_name = Func::delete_by_id(ctx, func_id)
            .await
            .expect("could not delete func by id");
        assert_eq!(
            func_name,                  // expected
            deleted_func_name.as_str()  // actual
        );
        ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
            .await
            .expect("could not commit and update snapshot to visibility");

        // Check that the func and its arguments do not exist.
        let maybe_func = Func::get_by_id(ctx, func_id)
            .await
            .expect("could not perform find by name");
        assert!(maybe_func.is_none());
        let maybe_string_func_argument =
            FuncArgument::get_by_id(ctx, cached_string_func_argument_id)
                .await
                .expect("could not perform find by name for func");
        assert!(maybe_string_func_argument.is_none());
        let maybe_array_func_argument = FuncArgument::get_by_id(ctx, cached_array_func_argument_id)
            .await
            .expect("could not perform find by name for func");
        assert!(maybe_array_func_argument.is_none());
    }
}

#[test]
async fn get_ts_type_from_root(ctx: &mut DalContext) {
    let schema = Schema::find_by_name(ctx, "starfield")
        .await
        .expect("could not perform find by name")
        .expect("schema not found");
    let schema_variant_id = schema
        .get_default_schema_variant_id(ctx)
        .await
        .expect("could not perform get default schema variant")
        .expect("schema variant not found");

    let root_prop_id = SchemaVariant::get_root_prop_id(ctx, schema_variant_id)
        .await
        .expect("could not get root prop id");
    let root_prop = Prop::get_by_id_or_error(ctx, root_prop_id)
        .await
        .expect("could not get prop by id");

    // TODO(nick): check that the ts type is right!
    let _ts_type = root_prop.ts_type(ctx).await.expect("could not get ts type");
}
