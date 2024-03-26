use dal::{DalContext, SchemaVariant};
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
