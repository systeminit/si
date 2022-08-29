use dal::DalContext;

use crate::dal::test;
use dal::schema::SchemaKind;
use dal::test_harness::{create_schema, create_schema_ui_menu};
use dal::{schema::UiMenu, DiagramKind, HistoryActor, StandardModel, Visibility, WriteTenancy};

#[test]
async fn new(ctx: &DalContext<'_, '_>) {
    let _write_tenancy = WriteTenancy::new_universal();
    let _visibility = Visibility::new_head(false);
    let _history_actor = HistoryActor::SystemInit;
    let schema_ui_menu = UiMenu::new(ctx, &DiagramKind::Configuration)
        .await
        .expect("cannot create schema ui menu");
    assert_eq!(schema_ui_menu.name(), None);
    assert_eq!(schema_ui_menu.category(), None);
    assert_eq!(schema_ui_menu.diagram_kind(), &DiagramKind::Configuration);
}

#[test]
async fn set_schema(ctx: &DalContext<'_, '_>) {
    let schema = create_schema(ctx, &SchemaKind::Configuration).await;
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
