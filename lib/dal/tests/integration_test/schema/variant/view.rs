use dal::{DalContext, Schema, SchemaVariant};
use dal_test::test;

use dal::schema::variant::SchemaVariantMetadataView;
use pretty_assertions_sorted::assert_eq;

#[test]
async fn list_schema_variant_definition_views(ctx: &DalContext) {
    let schema_variant_ids = SchemaVariant::list_ids(ctx)
        .await
        .expect("could not list schema variants");

    // We are going to ensure that we get a default schema variant
    // for each schema variant
    let views = SchemaVariantMetadataView::list(ctx)
        .await
        .expect("could not list schema variant views");

    assert_eq!(
        schema_variant_ids.len(), // expected
        views.len()               // actual
    )
}

#[test]
async fn get_schema_variant(ctx: &DalContext) {
    let maybe_swifty_schema = Schema::find_by_name(ctx, "swifty")
        .await
        .expect("unable to get schema");

    assert!(maybe_swifty_schema.is_some());

    let swifty_schema = maybe_swifty_schema.unwrap();
    let maybe_sv_id = swifty_schema
        .get_default_schema_variant(ctx)
        .await
        .expect("unable to get schema variant");

    assert!(maybe_sv_id.is_some());

    let sv_id = maybe_sv_id.unwrap();
    let sv_funcs = SchemaVariant::all_funcs(ctx, sv_id)
        .await
        .expect("Unable to get all schema variant funcs");

    assert_eq!(4, sv_funcs.len());

    let mut func_names: Vec<String> = sv_funcs.iter().map(|f| f.name.clone()).collect();
    func_names.sort();
    let expected: Vec<String> = vec![
        "si:resourcePayloadToValue".to_string(),
        "test:createActionSwifty".to_string(),
        "test:generateCode".to_string(),
        "test:refreshActionSwifty".to_string(),
    ];
    assert_eq!(expected, func_names);
}
