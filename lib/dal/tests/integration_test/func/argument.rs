use dal::attribute::prototype::argument::AttributePrototypeArgument;
use dal::func::argument::FuncArgument;
use dal::{AttributePrototype, DalContext, Func};
use dal_test::helpers::ChangeSetTestHelpers;
use dal_test::test;
use pretty_assertions_sorted::assert_eq;

#[test]
async fn modify(ctx: &mut DalContext) {
    let func_id = Func::find_by_name(ctx, "test:falloutEntriesToGalaxies")
        .await
        .expect("could not perform find by name")
        .expect("no func found");
    let func_argument = FuncArgument::find_by_name_for_func(ctx, "entries", func_id)
        .await
        .expect("could not perform find by name")
        .expect("no func argument found");
    let cached_func_argument_id = func_argument.id;

    // Update the name.
    let new_name = "the unity";
    FuncArgument::modify_by_id(ctx, cached_func_argument_id, |f| {
        f.name = new_name.to_string();
        Ok(())
    })
    .await
    .expect("could not modify func argument");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Ensure that the update worked.
    let updated_func_argument = FuncArgument::find_by_name_for_func(ctx, new_name, func_id)
        .await
        .expect("could not perform find by name")
        .expect("no func argument found");
    assert_eq!(
        cached_func_argument_id,  // expected
        updated_func_argument.id, // actual
    );
}

#[test]
async fn list_attribute_prototype_argument_ids(ctx: &DalContext) {
    let func_id = Func::find_by_name(ctx, "test:falloutEntriesToGalaxies")
        .await
        .expect("could not perform find by name")
        .expect("no func found");
    let func_argument = FuncArgument::find_by_name_for_func(ctx, "entries", func_id)
        .await
        .expect("could not perform find by name")
        .expect("no func argument found");
    let mut attribute_prototype_argument_ids =
        FuncArgument::list_attribute_prototype_argument_ids(ctx, func_argument.id)
            .await
            .expect("could not list attribute prototype argument ids");

    // Ensure that the attribute prototype argument is what we expect and that it leads back to the
    // original func.
    let attribute_prototype_argument_id = attribute_prototype_argument_ids
        .pop()
        .expect("empty attribute prototype argument ids");
    assert!(attribute_prototype_argument_ids.is_empty());
    let attribute_prototype_id = AttributePrototypeArgument::prototype_id_for_argument_id(
        ctx,
        attribute_prototype_argument_id,
    )
    .await
    .expect("could not get attribute prototype id");
    let found_func_id = AttributePrototype::func_id(ctx, attribute_prototype_id)
        .await
        .expect("could not get func id");
    assert_eq!(
        func_id,       // expected
        found_func_id  // actual
    );
}
