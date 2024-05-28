use dal::func::argument::{FuncArgument, FuncArgumentKind};
use dal::func::authoring::FuncAuthoringClient;
use dal::func::view::FuncView;
use dal::func::FuncKind;
use dal::{DalContext, Func};
use dal_test::helpers::ChangeSetTestHelpers;
use dal_test::test;
use pretty_assertions_sorted::assert_eq;

#[test]
async fn create_and_delete_attribute_func_with_arguments(ctx: &mut DalContext) {
    // Declare variables for use throughout the test.
    let func_name = "Chloe or Sam or Sophia or Marcus";

    let string_func_argument_name = "Chloe or Sam";
    let string_func_argument_kind = FuncArgumentKind::String;
    let string_func_argument_element_kind = None;

    let array_func_argument_name = "Sophia or Marcus";
    let array_func_argument_kind = FuncArgumentKind::Array;
    let array_func_argument_element_kind = Some(FuncArgumentKind::String);

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
        let prototypes = func_view
            .associations
            .expect("could not get associations")
            .get_attribute_internals()
            .expect("could not get internals");
        let arguments = FuncArgument::list_for_func(ctx, func_id)
            .await
            .expect("could not list func arguments");
        assert!(prototypes.is_empty());
        assert!(arguments.is_empty());

        FuncAuthoringClient::create_func_argument(
            ctx,
            func_id,
            string_func_argument_name,
            string_func_argument_kind,
            string_func_argument_element_kind,
        )
        .await
        .expect("unable to create func argument");
        ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
            .await
            .expect("could not commit and update snapshot to visibility");

        FuncAuthoringClient::create_func_argument(
            ctx,
            func_id,
            array_func_argument_name,
            array_func_argument_kind,
            array_func_argument_element_kind,
        )
        .await
        .expect("unable to create func argument");
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
        let prototypes = func_view
            .associations
            .expect("could not get associations")
            .get_attribute_internals()
            .expect("could not get internals");
        assert!(prototypes.is_empty());
        let arguments = FuncArgument::list_for_func(ctx, func_id)
            .await
            .expect("could not list func arguments");
        assert_eq!(
            2,               // expected
            arguments.len()  // actual
        );

        let found_string_func_argument = arguments
            .iter()
            .find(|arg| arg.name == string_func_argument_name)
            .expect("could not find bag");
        let found_array_func_argument = arguments
            .iter()
            .find(|arg| arg.name == array_func_argument_name)
            .expect("could not find bag");
        assert_eq!(
            string_func_argument_kind,       // expected
            found_string_func_argument.kind  // actual
        );
        assert_eq!(
            string_func_argument_element_kind,       // expected
            found_string_func_argument.element_kind  // actual
        );
        assert_eq!(
            array_func_argument_kind,       // expected
            found_array_func_argument.kind  // actual
        );
        assert_eq!(
            array_func_argument_element_kind,       // expected
            found_array_func_argument.element_kind  // actual
        );

        (found_string_func_argument.id, found_array_func_argument.id)
    };
    // note from brit: unclear if this test was ever passing, but because this new
    // func was never actually associated with any schema variants/props/sockets, this deletion did
    // not fail.

    // // Try to delete the func. The deletion will fail because it has associations, which are the
    // // two new arguments. Then, check that the two func arguments still exist.
    // {
    //     match Func::delete_by_id(ctx, func_id).await {
    //         Ok(_) => panic!("deletion should fail since func has associations"),
    //         Err(FuncError::FuncToBeDeletedHasAssociations(_)) => {}
    //         Err(err) => panic!("unexpected error: {err:?}"),
    //     }

    //     let found_string_func_argument =
    //         FuncArgument::find_by_name_for_func(ctx, string_func_argument_name, func_id)
    //             .await
    //             .expect("could not perform find by name for func")
    //             .expect("func argument not found");
    //     let found_array_func_argument =
    //         FuncArgument::find_by_name_for_func(ctx, array_func_argument_name, func_id)
    //             .await
    //             .expect("could not perform find by name for func")
    //             .expect("func argument not found");
    //     assert_eq!(
    //         cached_string_func_argument_id, // expected
    //         found_string_func_argument.id   // actual
    //     );
    //     assert_eq!(
    //         cached_array_func_argument_id, // expected
    //         found_array_func_argument.id   // actual
    //     );
    // }

    // Fetch the view. Then, delete the two arguments and commit.
    {
        let func = Func::get_by_id_or_error(ctx, func_id)
            .await
            .expect("could not get func");
        let func_view = FuncView::assemble(ctx, &func)
            .await
            .expect("could not assemble func view");
        let prototypes = func_view
            .associations
            .expect("could not get associations")
            .get_attribute_internals()
            .expect("could not get internals");
        assert!(prototypes.is_empty());
        let arguments = FuncArgument::list_for_func(ctx, func_id)
            .await
            .expect("could not list func arguments");
        assert_eq!(
            2,               // expected
            arguments.len()  // actual
        );

        FuncAuthoringClient::delete_func_argument(ctx, cached_string_func_argument_id)
            .await
            .expect("unable to delete func argument");
        ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
            .await
            .expect("could not commit and update snapshot to visibility");

        FuncAuthoringClient::delete_func_argument(ctx, cached_array_func_argument_id)
            .await
            .expect("unable to delete func argument");
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
