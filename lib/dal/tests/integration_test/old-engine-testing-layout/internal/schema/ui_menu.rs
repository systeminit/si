use dal::{schema::SchemaUiMenu, DalContext, StandardModel};
use dal_test::{test, test_harness::create_schema};

#[test]
async fn new_and_set_schema(ctx: &DalContext) {
    let schema = create_schema(ctx).await;
    let schema_ui_menu = SchemaUiMenu::new(ctx, "riot", "awaken, my love!")
        .await
        .expect("cannot create schema ui menu");
    assert_eq!(schema_ui_menu.name(), "riot");
    assert_eq!(schema_ui_menu.category(), "awaken, my love!");

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
