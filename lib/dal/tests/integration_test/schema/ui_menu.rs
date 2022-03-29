use dal::DalContext;

use crate::dal::test;
use dal::schema::SchemaKind;
use dal::test_harness::{create_schema, create_schema_ui_menu};
use dal::{schema::UiMenu, HistoryActor, SchematicKind, StandardModel, Visibility, WriteTenancy};

#[test]
async fn new(ctx: &DalContext<'_, '_>) {
    let _write_tenancy = WriteTenancy::new_universal();
    let _visibility = Visibility::new_head(false);
    let _history_actor = HistoryActor::SystemInit;
    let schema_ui_menu = UiMenu::new(ctx, &SchematicKind::Component)
        .await
        .expect("cannot create schema ui menu");
    assert_eq!(schema_ui_menu.name(), None);
    assert_eq!(schema_ui_menu.category(), None);
    assert_eq!(schema_ui_menu.schematic_kind(), &SchematicKind::Component);
}

#[test]
async fn set_schema(ctx: &DalContext<'_, '_>) {
    let schema = create_schema(ctx, &SchemaKind::Concrete).await;
    let schema_ui_menu = create_schema_ui_menu(ctx).await;

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

#[test]
async fn root_schematics(ctx: &DalContext<'_, '_>) {
    let root_schema = create_schema(ctx, &SchemaKind::Concrete).await;

    let schema_ui_menu = create_schema_ui_menu(ctx).await;

    schema_ui_menu
        .add_root_schematic(ctx, root_schema.id())
        .await
        .expect("cannot add root schematic");

    let root_schematics = schema_ui_menu
        .root_schematics(ctx)
        .await
        .expect("cannot list root schematics");
    assert_eq!(root_schematics, vec![root_schema.clone()]);

    schema_ui_menu
        .remove_root_schematic(ctx, root_schema.id())
        .await
        .expect("cannot add root schematic");
    let no_root_schematics = schema_ui_menu
        .root_schematics(ctx)
        .await
        .expect("cannot list root schematics");
    assert_eq!(no_root_schematics, vec![]);
}
