use dal::func::argument::{FuncArgument, FuncArgumentId, FuncArgumentKind};
use dal::func::authoring::FuncAuthoringClient;
use dal::func::view::FuncView;
use dal::func::{FuncArgumentBag, FuncAssociations};
use dal::{AttributePrototype, AttributePrototypeId, DalContext, Func, FuncId};
use dal_test::helpers::ChangeSetTestHelpers;
use dal_test::test;
use pretty_assertions_sorted::assert_eq;

use crate::integration_test::func::authoring::save_func::save_func_setup;

#[test]
async fn modify_add_delete(ctx: &mut DalContext) {
    let (func_id, saved_func) = save_func_setup(ctx, "test:falloutEntriesToGalaxies").await;
    base_assertions_for_attribute_funcs(ctx, func_id, &saved_func).await;

    // Scenario 1: modify the type of the existing func argument.
    {
        let func = Func::get_by_id_or_error(ctx, func_id)
            .await
            .expect("could not get func by id");
        let func_view = FuncView::assemble(ctx, &func)
            .await
            .expect("could not assemble func view");
        let (prototypes, mut arguments) = func_view
            .associations
            .clone()
            .expect("could not get associations")
            .get_attribute_internals()
            .expect("could not get internals");
        let argument = arguments.pop().expect("empty arguments");
        assert!(arguments.is_empty());
        let cached_func_argument_id = argument.id;

        // Save the func and commit.
        FuncAuthoringClient::save_func(
            ctx,
            func_view.id,
            func_view.display_name,
            func_view.name,
            func_view.description,
            func_view.code,
            Some(FuncAssociations::Attribute {
                prototypes,
                arguments: vec![FuncArgumentBag {
                    id: cached_func_argument_id,
                    name: argument.name,
                    kind: FuncArgumentKind::Boolean,
                    element_kind: None,
                }],
            }),
        )
        .await
        .expect("unable to save func");
        ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
            .await
            .expect("could not commit and update snapshot to visibility");

        // Ensure that it updated.
        let func = Func::get_by_id_or_error(ctx, func_id)
            .await
            .expect("could not get func by id");
        let func_view = FuncView::assemble(ctx, &func)
            .await
            .expect("could not assemble func view");
        assert_eq!(
            func_id,      // expected
            func_view.id  // actual
        );
        let (_, mut arguments) = func_view
            .associations
            .expect("could not get associations")
            .get_attribute_internals()
            .expect("could not get internals");
        let argument = arguments.pop().expect("empty arguments");
        assert!(arguments.is_empty());
        assert_eq!(
            FuncArgumentKind::Boolean, // expected
            argument.kind              // actual
        );
        assert_eq!(
            cached_func_argument_id, // expected
            argument.id              // actual
        );

        // Check the actual func argument and that exactly one still exists.
        let mut func_argument_ids = FuncArgument::list_ids_for_func(ctx, func_id)
            .await
            .expect("could not list ids for func");
        let func_argument_id = func_argument_ids.pop().expect("empty arguments");
        assert!(func_argument_ids.is_empty());
        assert_eq!(
            cached_func_argument_id, // expected
            func_argument_id         // actual
        );
        let func_argument = FuncArgument::get_by_id_or_error(ctx, func_argument_id)
            .await
            .expect("could not get func argument");
        assert_eq!(
            FuncArgumentKind::Boolean, // expected
            func_argument.kind         // actual
        );
    }

    // Scenario 2: add a func argument.
    {
        let func = Func::get_by_id_or_error(ctx, func_id)
            .await
            .expect("could not get func by id");
        let func_view = FuncView::assemble(ctx, &func)
            .await
            .expect("could not assemble func view");
        let (prototypes, mut arguments) = func_view
            .associations
            .clone()
            .expect("could not get associations")
            .get_attribute_internals()
            .expect("could not get internals");
        arguments.push(FuncArgumentBag {
            id: FuncArgumentId::NONE,
            name: "armillary".to_string(),
            kind: FuncArgumentKind::String,
            element_kind: None,
        });

        // Save the func and commit.
        FuncAuthoringClient::save_func(
            ctx,
            func_view.id,
            func_view.display_name,
            func_view.name,
            func_view.description,
            func_view.code,
            Some(FuncAssociations::Attribute {
                prototypes,
                arguments,
            }),
        )
        .await
        .expect("unable to save func");
        ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
            .await
            .expect("could not commit and update snapshot to visibility");

        // Ensure that we have the two arguments.
        let func = Func::get_by_id_or_error(ctx, func_id)
            .await
            .expect("could not get func by id");
        let func_view = FuncView::assemble(ctx, &func)
            .await
            .expect("could not assemble func view");
        assert_eq!(
            func_id,      // expected
            func_view.id  // actual
        );
        let (mut prototypes, arguments) = func_view
            .associations
            .expect("could not get associations")
            .get_attribute_internals()
            .expect("could not get internals");
        let prototype = prototypes.pop().expect("empty prototypes");
        assert!(prototypes.is_empty());
        assert_eq!(1, prototype.prototype_arguments.len());
        assert_eq!(2, arguments.len());
    }

    // Scenario 3: delete all func arguments and update some metadata.
    {
        let func = Func::get_by_id_or_error(ctx, func_id)
            .await
            .expect("could not get func by id");
        let func_view = FuncView::assemble(ctx, &func)
            .await
            .expect("could not assemble func view");
        let (prototypes, _arguments) = func_view
            .associations
            .expect("could not get associations")
            .get_attribute_internals()
            .expect("could not get internals");

        // Save the func and commit.
        FuncAuthoringClient::save_func(
            ctx,
            func_view.id,
            Some("updated-display-name".to_string()),
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

        // Ensure the updated func looks as we expect.
        let func = Func::get_by_id_or_error(ctx, func_id)
            .await
            .expect("could not get func by id");
        let func_view = FuncView::assemble(ctx, &func)
            .await
            .expect("could not assemble func view");
        assert_eq!(
            func_id,      // expected
            func_view.id  // actual
        );

        let (mut prototypes, arguments) = func_view
            .associations
            .expect("could not get associations")
            .get_attribute_internals()
            .expect("could not get internals");
        assert!(arguments.is_empty());
        let prototype = prototypes.pop().expect("empty prototypes");
        assert!(prototypes.is_empty());
        assert!(prototype.prototype_arguments.is_empty());
    }
}

#[test]
async fn delete_add_modify(ctx: &mut DalContext) {
    let (func_id, saved_func) = save_func_setup(ctx, "test:falloutEntriesToGalaxies").await;
    base_assertions_for_attribute_funcs(ctx, func_id, &saved_func).await;

    // Scenario 1: delete all func arguments.
    {
        let func = Func::get_by_id_or_error(ctx, func_id)
            .await
            .expect("could not get func by id");
        let func_view = FuncView::assemble(ctx, &func)
            .await
            .expect("could not assemble func view");
        let (prototypes, _arguments) = func_view
            .associations
            .expect("could not get associations")
            .get_attribute_internals()
            .expect("could not get internals");

        // Save the func and commit.
        FuncAuthoringClient::save_func(
            ctx,
            func_view.id,
            Some("updated-display-name".to_string()),
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

        // Ensure the updated func looks as we expect.
        let func = Func::get_by_id_or_error(ctx, func_id)
            .await
            .expect("could not get func by id");
        let func_view = FuncView::assemble(ctx, &func)
            .await
            .expect("could not assemble func view");
        assert_eq!(
            func_id,      // expected
            func_view.id  // actual
        );

        let (mut prototypes, arguments) = func_view
            .associations
            .expect("could not get associations")
            .get_attribute_internals()
            .expect("could not get internals");
        assert!(arguments.is_empty());
        let prototype = prototypes.pop().expect("empty prototypes");
        assert!(prototypes.is_empty());
        assert!(prototype.prototype_arguments.is_empty());
    }

    // Scenario 2: add a func argument.
    let new_func_argument_id = {
        let func = Func::get_by_id_or_error(ctx, func_id)
            .await
            .expect("could not get func by id");
        let func_view = FuncView::assemble(ctx, &func)
            .await
            .expect("could not assemble func view");
        let (prototypes, arguments) = func_view
            .associations
            .clone()
            .expect("could not get associations")
            .get_attribute_internals()
            .expect("could not get internals");
        assert!(arguments.is_empty());

        // Save the func and commit.
        FuncAuthoringClient::save_func(
            ctx,
            func_view.id,
            func_view.display_name,
            func_view.name,
            func_view.description,
            func_view.code,
            Some(FuncAssociations::Attribute {
                prototypes,
                arguments: vec![FuncArgumentBag {
                    id: FuncArgumentId::NONE,
                    name: "unity".to_string(),
                    kind: FuncArgumentKind::String,
                    element_kind: None,
                }],
            }),
        )
        .await
        .expect("unable to save func");
        ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
            .await
            .expect("could not commit and update snapshot to visibility");

        // Ensure that we have the one argument.
        let func = Func::get_by_id_or_error(ctx, func_id)
            .await
            .expect("could not get func by id");
        let func_view = FuncView::assemble(ctx, &func)
            .await
            .expect("could not assemble func view");
        assert_eq!(
            func_id,      // expected
            func_view.id  // actual
        );
        let (mut prototypes, mut arguments) = func_view
            .associations
            .expect("could not get associations")
            .get_attribute_internals()
            .expect("could not get internals");
        let prototype = prototypes.pop().expect("empty prototypes");
        assert!(prototypes.is_empty());
        let argument = arguments.pop().expect("empty arguments");
        assert!(arguments.is_empty());

        // Ensure that there are no prototype arguments and return the func argument id.
        assert!(prototype.prototype_arguments.is_empty());
        argument.id
    };

    // Scenario 3: modify the type of the existing func argument.
    {
        let func = Func::get_by_id_or_error(ctx, func_id)
            .await
            .expect("could not get func by id");
        let func_view = FuncView::assemble(ctx, &func)
            .await
            .expect("could not assemble func view");
        let (prototypes, mut arguments) = func_view
            .associations
            .clone()
            .expect("could not get associations")
            .get_attribute_internals()
            .expect("could not get internals");
        let argument = arguments.pop().expect("empty arguments");
        assert!(arguments.is_empty());

        // Ensure that the collected func has the same argument.
        let cached_func_argument_id = argument.id;
        assert_eq!(
            new_func_argument_id,    // expected
            cached_func_argument_id, // actual
        );

        // Save the func and commit.
        FuncAuthoringClient::save_func(
            ctx,
            func_view.id,
            func_view.display_name,
            func_view.name,
            func_view.description,
            func_view.code,
            Some(FuncAssociations::Attribute {
                prototypes,
                arguments: vec![FuncArgumentBag {
                    id: cached_func_argument_id,
                    name: argument.name,
                    kind: FuncArgumentKind::Boolean,
                    element_kind: None,
                }],
            }),
        )
        .await
        .expect("unable to save func");
        ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
            .await
            .expect("could not commit and update snapshot to visibility");

        // Ensure that it updated.
        let func = Func::get_by_id_or_error(ctx, func_id)
            .await
            .expect("could not get func by id");
        let func_view = FuncView::assemble(ctx, &func)
            .await
            .expect("could not assemble func view");
        assert_eq!(
            func_id,      // expected
            func_view.id  // actual
        );
        let (mut prototypes, mut arguments) = func_view
            .associations
            .expect("could not get associations")
            .get_attribute_internals()
            .expect("could not get internals");
        let argument = arguments.pop().expect("empty arguments");
        assert!(arguments.is_empty());
        assert_eq!(
            FuncArgumentKind::Boolean, // expected
            argument.kind              // actual
        );
        assert_eq!(
            cached_func_argument_id, // expected
            argument.id              // actual
        );

        // Ensure that there are no prototype arguments and return the func argument id.
        let prototype = prototypes.pop().expect("empty prototypes");
        assert!(prototypes.is_empty());
        assert!(prototype.prototype_arguments.is_empty());

        // Check the actual func argument and that exactly one still exists.
        let mut func_argument_ids = FuncArgument::list_ids_for_func(ctx, func_id)
            .await
            .expect("could not list ids for func");
        let func_argument_id = func_argument_ids.pop().expect("empty arguments");
        assert!(func_argument_ids.is_empty());
        assert_eq!(
            cached_func_argument_id, // expected
            func_argument_id         // actual
        );
        let func_argument = FuncArgument::get_by_id_or_error(ctx, func_argument_id)
            .await
            .expect("could not get func argument");
        assert_eq!(
            FuncArgumentKind::Boolean, // expected
            func_argument.kind         // actual
        );
    }
}

// Ensure everything looks as we expect before starting tests.
async fn base_assertions_for_attribute_funcs(
    ctx: &DalContext,
    func_id: FuncId,
    func_view: &FuncView,
) {
    let (prototypes, arguments) = func_view
        .associations
        .to_owned()
        .expect("could not get associations")
        .get_attribute_internals()
        .expect("could not get internals");

    // Ensure the prototypes look as we expect.
    let attribute_prototype_ids = AttributePrototype::list_ids_for_func_id(ctx, func_id)
        .await
        .expect("could not list ids for func id");
    assert_eq!(
        attribute_prototype_ids, // expected
        prototypes
            .iter()
            .map(|v| v.id)
            .collect::<Vec<AttributePrototypeId>>()  // actual
    );

    // Ensure the arguments look as we expect.
    let func_argument_ids = FuncArgument::list_ids_for_func(ctx, func_id)
        .await
        .expect("could not list ids for func");
    assert_eq!(
        func_argument_ids, // expected
        arguments
            .iter()
            .map(|v| v.id)
            .collect::<Vec<FuncArgumentId>>()  // actual
    );
}
