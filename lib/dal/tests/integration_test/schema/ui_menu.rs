use dal::DalContext;

use crate::dal::test;
use dal::schema::SchemaKind;
use dal::test_harness::create_schema;
use dal::{schema::SchemaUiMenu, DiagramKind, StandardModel};

#[test]
async fn new_and_set_schema(ctx: &DalContext) {
    let schema = create_schema(ctx, &SchemaKind::Configuration).await;
    let schema_ui_menu =
        SchemaUiMenu::new(ctx, "riot", "awaken, my love!", &DiagramKind::Configuration)
            .await
            .expect("cannot create schema ui menu");
    assert_eq!(schema_ui_menu.name(), "riot");
    assert_eq!(schema_ui_menu.category(), "awaken, my love!");
    assert_eq!(schema_ui_menu.diagram_kind(), DiagramKind::Configuration);

    schema_ui_menu
        .set_schema(ctx, schema.id())
        .await
        .expect("cannot associate ui menu with schema");
    let attached_schema = schema_ui_menu
        .schema(ctx)
        .await
        .expect("cannot get schema")
        .expect("should have a schema");
    assert_eq!(schema, attached_schema);

    schema_ui_menu
        .unset_schema(ctx)
        .await
        .expect("cannot associate ui menu with schema");
    let attached_schema = schema_ui_menu.schema(ctx).await.expect("cannot get schema");
    assert_eq!(attached_schema, None);
}
