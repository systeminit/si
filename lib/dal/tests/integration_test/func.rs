use dal::func::view::FuncSummary;
use dal::{DalContext, Schema, SchemaVariant};
use dal_test::test;
use pretty_assertions_sorted::assert_eq;

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
    let funcs_for_schema_variant = FuncSummary::list(ctx, Some(schema_variant_id))
        .await
        .expect("could not list func summaries");
    let all_funcs = FuncSummary::list(ctx, None)
        .await
        .expect("could not list func summaries");

    let func_name = "test:createActionStarfield".to_string();
    let found_func_for_all = all_funcs
        .iter()
        .find(|f| f.name() == func_name)
        .expect("could not find func");
    let found_func_for_schema_variant = funcs_for_schema_variant
        .iter()
        .find(|f| f.name() == func_name)
        .expect("could not find func");

    assert_eq!(found_func_for_all, found_func_for_schema_variant);
}
