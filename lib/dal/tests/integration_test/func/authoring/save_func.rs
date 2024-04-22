use dal::func::argument::{FuncArgument, FuncArgumentId};
use dal::func::authoring::{FuncAuthoringClient, SavedFunc};
use dal::func::view::FuncView;
use dal::func::FuncAssociations;
use dal::{AttributePrototype, AttributePrototypeId, DalContext, Func, FuncId};
use dal_test::test;
use pretty_assertions_sorted::assert_eq;

#[test]
async fn save_action_func(ctx: &mut DalContext) {
    let (_func_id, _saved_func) = setup(ctx, "test:createActionStarfield").await;
}

#[test]
async fn save_authentication_func(ctx: &mut DalContext) {
    let (_func_id, _saved_func) = setup(ctx, "test:setDummySecretString").await;
}

#[test]
async fn save_attribute_func(ctx: &mut DalContext) {
    let (func_id, saved_func) = setup(ctx, "test:falloutEntriesToGalaxies").await;

    // Find the associations.
    let (prototypes, arguments) = saved_func
        .associations
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

    // Let's delete the func argument and update some metadata.
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
    let saved_func = FuncAuthoringClient::save_func(
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
    .expect("unable to create func");

    // Ensure the updated func looks as we expect.
    let func = Func::get_by_id_or_error(ctx, func_id)
        .await
        .expect("could not get func by id");
    let func_view = FuncView::assemble(ctx, &func)
        .await
        .expect("could not assemble func view");
    assert_eq!(
        saved_func.associations, // expected
        func_view.associations   // actual
    );
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

#[test]
async fn save_code_generation_func(ctx: &mut DalContext) {
    let (_func_id, _saved_func) = setup(ctx, "test:generateCode").await;
}

#[test]
async fn save_qualification_func(ctx: &mut DalContext) {
    let (_func_id, _saved_func) = setup(ctx, "test:qualificationDummySecretStringIsTodd").await;
}

// Sets up the tests within the module. Find the func to be saved by name and then save it
// immediately when found. This is the basic "does it work in place" check.
async fn setup(ctx: &mut DalContext, func_name: impl AsRef<str>) -> (FuncId, SavedFunc) {
    let func_id = Func::find_by_name(ctx, func_name)
        .await
        .expect("could not perform find func by name")
        .expect("no func found");

    // Save the func immediately when found.
    let func = Func::get_by_id_or_error(ctx, func_id)
        .await
        .expect("could not get func by id");
    let func_view = FuncView::assemble(ctx, &func)
        .await
        .expect("could not assemble func view");
    let saved_func = FuncAuthoringClient::save_func(
        ctx,
        func_view.id,
        func_view.display_name,
        func_view.name,
        func_view.description,
        func_view.code,
        func_view.associations.clone(),
    )
    .await
    .expect("unable to create func");

    // We know it is successful and revertible because it should work immediately and the test
    // runs in a new change set.
    assert!(saved_func.success);
    assert!(saved_func.is_revertible);

    // Perform all other assertions before getting started.
    assert_eq!(
        func_id,      // expected
        func_view.id  // actual
    );
    assert_eq!(
        func_view.types,  // expected
        saved_func.types, // actual
    );

    (func_id, saved_func)
}
