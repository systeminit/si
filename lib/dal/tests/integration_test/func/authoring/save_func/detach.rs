use dal::func::authoring::FuncAuthoringClient;
use dal::func::summary::FuncSummary;
use dal::func::view::FuncView;
use dal::func::FuncAssociations;
use dal::{DalContext, Func, Schema, SchemaVariant};
use dal_test::helpers::ChangeSetTestHelpers;
use dal_test::test;

#[test]
async fn detach_attribute_func(ctx: &mut DalContext) {
    let schema = Schema::find_by_name(ctx, "starfield")
        .await
        .expect("unable to find by name")
        .expect("no schema found");
    let schema_variant_id = SchemaVariant::get_default_id_for_schema(ctx, schema.id())
        .await
        .expect("unable to get default schema variant");

    // Cache the total number of funcs before continuing.
    let funcs = FuncSummary::list_for_schema_variant_id(ctx, schema_variant_id)
        .await
        .expect("unable to get the funcs for a schema variant");
    let total_funcs = funcs.len();

    // Detach one action func to the schema variant and commit.
    let func_id = Func::find_by_name(ctx, "test:falloutEntriesToGalaxies")
        .await
        .expect("unable to find the func")
        .expect("no func found");
    let func = Func::get_by_id_or_error(ctx, func_id)
        .await
        .expect("unable to get func by id");
    let func_view = FuncView::assemble(ctx, &func)
        .await
        .expect("unable to assemble a func view");
    let (prototypes, arguments) = func_view
        .associations
        .expect("empty associations")
        .get_attribute_internals()
        .expect("could not get internals");
    let prototypes = prototypes
        .into_iter()
        .filter(|p| p.schema_variant_id != Some(schema_variant_id))
        .collect();
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
    .expect("unable to save the func");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Now, let's list all funcs and see what's left.
    let funcs = FuncSummary::list_for_schema_variant_id(ctx, schema_variant_id)
        .await
        .expect("unable to get the funcs for a schema variant");
    assert_eq!(
        total_funcs - 1, // expected
        funcs.len()      // actual
    );
    assert!(!funcs.iter().any(|summary| summary.id == func_id));
}
