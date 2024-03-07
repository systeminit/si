use dal::schema::view::SchemaView;
use dal::{DalContext, Schema, SchemaId, SchemaVariant, SchemaVariantId};
use dal_test::test;
use pretty_assertions_sorted::assert_eq;
use std::collections::HashSet;

mod variant;

#[test]
async fn new(ctx: &DalContext) {
    let _schema = Schema::new(ctx, "mastodon")
        .await
        .expect("cannot create schema");
}

#[test]
async fn list_views(ctx: &DalContext) {
    let schemas = Schema::list(ctx).await.expect("could not list schemas");

    let views = SchemaView::list(ctx)
        .await
        .expect("could not list schema views");

    // First, check that the schemas look as expected.
    let expected: HashSet<SchemaId> = HashSet::from_iter(schemas.iter().filter_map(|e| {
        if e.ui_hidden {
            None
        } else {
            Some(e.id())
        }
    }));
    let actual: HashSet<SchemaId> = HashSet::from_iter(views.iter().map(|a| a.id()));

    assert_eq!(
        expected, // expected
        actual,   // actual
    );

    // Second, check that the schema variants look as expected.
    for view in views {
        let schema_variants = SchemaVariant::list_for_schema(ctx, view.id())
            .await
            .expect("could not list schema variants for schema");

        let expected: HashSet<SchemaVariantId> =
            HashSet::from_iter(schema_variants.iter().filter_map(|e| {
                if e.ui_hidden() {
                    None
                } else {
                    Some(e.id())
                }
            }));
        let actual: HashSet<SchemaVariantId> =
            HashSet::from_iter(view.variants().iter().map(|a| a.id()));

        assert_eq!(
            expected, // expected
            actual,   // actual
        );
    }
}
