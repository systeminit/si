use dal::schema::variant::view::SchemaVariantView;
use dal::{DalContext, SchemaVariant};
use dal_test::test;

use pretty_assertions_sorted::assert_eq;

#[test]
async fn list_schema_variant_views(ctx: &DalContext) {
    let schema_variant_ids = SchemaVariant::list_ids(ctx)
        .await
        .expect("could not list schema variants");

    // TODO(nick): do something more useful with this test. For now, just make sure that it works.
    let views = SchemaVariantView::list(ctx)
        .await
        .expect("could not list schema variant views");

    assert_eq!(
        schema_variant_ids.len(), // expected
        views.len()               // actual
    )
}
