use dal::{component::ComponentKind, schema::SchemaUiMenu, DalContext, Schema, StandardModel};

use dal_test::{test, test_harness::create_schema};

pub mod ui_menu;
pub mod variant;
pub mod variant_definition;

#[test]
async fn new(ctx: &DalContext) {
    let _schema = Schema::new(ctx, "mastodon", &ComponentKind::Standard)
        .await
        .expect("cannot create schema");
}

#[test]
async fn ui_menus(ctx: &DalContext) {
    let schema = create_schema(ctx).await;
    let schema_ui_menu = SchemaUiMenu::new(ctx, "visa", "m.i.a.")
        .await
        .expect("cannot create schema ui menu");
    schema_ui_menu
        .set_schema(ctx, schema.id())
        .await
        .expect("cannot set schema");
    let ui_menus = schema.ui_menus(ctx).await.expect("cannot get ui menus");
    assert_eq!(ui_menus, vec![schema_ui_menu.clone()]);
}
