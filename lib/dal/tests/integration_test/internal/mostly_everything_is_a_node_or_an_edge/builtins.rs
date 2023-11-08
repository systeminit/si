use dal::{func::intrinsics::IntrinsicFunc, DalContext, Func, Schema, SchemaVariant};
use dal_test::test;
use strum::IntoEnumIterator;

#[test]
async fn builtin_funcs_and_schemas_are_not_empty(ctx: &DalContext) {
    let funcs: Vec<String> = Func::list(ctx)
        .await
        .expect("list funcs should work")
        .iter()
        .map(|f| f.name.to_owned())
        .collect();

    // Check that the funcs at least contain all intrinsics.
    let intrinsics: Vec<String> = IntrinsicFunc::iter()
        .map(|intrinsic| intrinsic.name().to_owned())
        .collect();
    for intrinsic in intrinsics {
        assert!(funcs.contains(&intrinsic));
    }

    // Ensure that we have at least one schema variant for every schema and that we have at least
    // one schema.
    let schemas: Vec<Schema> = Schema::list(ctx).await.expect("could not list schemas");
    assert!(!schemas.is_empty());
    for schema in schemas {
        let schema_variants: Vec<SchemaVariant> = SchemaVariant::list_for_schema(ctx, schema.id())
            .await
            .expect("could not list schema variants");
        assert!(!schema_variants.is_empty());
    }
}
