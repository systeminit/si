use dal::{
    DalContext,
    Schema,
    SchemaVariant,
    schema::variant::SchemaVariantMetadataView,
};
use dal_test::test;
use pretty_assertions_sorted::assert_eq;

#[test]
async fn list_schema_variant_definition_views(ctx: &DalContext) {
    let schema_variant_ids = SchemaVariant::list_default_ids(ctx)
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
    let swifty_schema = Schema::get_by_name(ctx, "swifty")
        .await
        .expect("unable to get schema");

    let sv_id = Schema::default_variant_id(ctx, swifty_schema.id())
        .await
        .expect("unable to get schema variant");

    let sv_funcs = SchemaVariant::all_funcs(ctx, sv_id)
        .await
        .expect("Unable to get all schema variant funcs");

    assert_eq!(9, sv_funcs.len());
    let mut func_names: Vec<String> = sv_funcs.iter().map(|f| f.name.clone()).collect();
    func_names.sort();
    let expected: Vec<String> = vec![
        "si:identity".to_string(),
        "si:resourcePayloadToValue".to_string(),
        "si:unset".to_string(),
        "test:createActionSwifty".to_string(),
        "test:deleteActionSwifty".to_string(),
        "test:generateCode".to_string(),
        "test:refreshActionSwifty".to_string(),
        "test:swiftyQualification".to_string(),
        "test:updateActionSwifty".to_string(),
    ];
    assert_eq!(expected, func_names);
}
