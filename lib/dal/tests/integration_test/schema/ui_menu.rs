use crate::test_setup;

use dal::schema::SchemaKind;
use dal::test_harness::{create_schema, create_schema_ui_menu};
use dal::{schema::UiMenu, HistoryActor, SchematicKind, StandardModel, Tenancy, Visibility};

#[tokio::test]
async fn new() {
    test_setup!(
        ctx,
        _secret_key,
        _pg,
        _conn,
        txn,
        _nats_conn,
        nats,
        _veritech,
    );
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let schema_ui_menu = UiMenu::new(&txn, &nats, &tenancy, &visibility, &history_actor)
        .await
        .expect("cannot create schema ui menu");
    assert_eq!(schema_ui_menu.name(), None);
    assert_eq!(schema_ui_menu.category(), None);
    assert_eq!(schema_ui_menu.schematic_kind(), &SchematicKind::Component);
}

#[tokio::test]
async fn set_schema() {
    test_setup!(
        ctx,
        _secret_key,
        _pg,
        _conn,
        txn,
        _nats_conn,
        nats,
        _veritech,
    );
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let schema = create_schema(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        &SchemaKind::Concrete,
    )
    .await;
    let schema_ui_menu =
        create_schema_ui_menu(&txn, &nats, &tenancy, &visibility, &history_actor).await;

    schema_ui_menu
        .set_schema(&txn, &nats, &visibility, &history_actor, schema.id())
        .await
        .expect("cannot associate ui menu with schema");
    let attached_schema = schema_ui_menu
        .schema(&txn, &visibility)
        .await
        .expect("cannot get schema")
        .expect("should have a schema");
    assert_eq!(schema, attached_schema);

    schema_ui_menu
        .unset_schema(&txn, &nats, &visibility, &history_actor)
        .await
        .expect("cannot associate ui menu with schema");
    let attached_schema = schema_ui_menu
        .schema(&txn, &visibility)
        .await
        .expect("cannot get schema");
    assert_eq!(attached_schema, None);
}

#[tokio::test]
async fn root_schematics() {
    test_setup!(
        ctx,
        _secret_key,
        _pg,
        _conn,
        txn,
        _nats_conn,
        nats,
        _veritech,
    );
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let root_schema = create_schema(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        &SchemaKind::Concrete,
    )
    .await;

    let schema_ui_menu =
        create_schema_ui_menu(&txn, &nats, &tenancy, &visibility, &history_actor).await;

    schema_ui_menu
        .add_root_schematic(&txn, &nats, &visibility, &history_actor, root_schema.id())
        .await
        .expect("cannot add root schematic");

    let root_schematics = schema_ui_menu
        .root_schematics(&txn, &visibility)
        .await
        .expect("cannot list root schematics");
    assert_eq!(root_schematics, vec![root_schema.clone()]);

    schema_ui_menu
        .remove_root_schematic(&txn, &nats, &visibility, &history_actor, root_schema.id())
        .await
        .expect("cannot add root schematic");
    let no_root_schematics = schema_ui_menu
        .root_schematics(&txn, &visibility)
        .await
        .expect("cannot list root schematics");
    assert_eq!(no_root_schematics, vec![]);
}
