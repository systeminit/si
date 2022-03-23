use crate::dal::test;
use dal::node_menu::get_node_menu_items;
use dal::test_harness::{
    billing_account_signup, create_component_for_schema, create_schema_variant,
};
use dal::{node_menu::MenuFilter, Schema, SchematicKind};
use dal::{HistoryActor, StandardModel, Visibility, WriteTenancy};

use crate::test_setup;

#[test]
async fn get_node_menu() {
    test_setup!(ctx, secret_key, _pg, _conn, txn, _nats_conn, nats, veritech, encr_key);
    let (nba, _key) = billing_account_signup(&txn, &nats, secret_key).await;
    let visibility = Visibility::new_head(false);
    let write_tenancy = WriteTenancy::new_workspace(*nba.workspace.id());
    let read_tenancy = write_tenancy
        .clone_into_read_tenancy(&txn)
        .await
        .expect("unable to generate read tenancy");
    let history_actor = HistoryActor::SystemInit;

    let application_schema = Schema::find_by_attr(
        &txn,
        &(&read_tenancy).into(),
        &visibility,
        "name",
        &String::from("application"),
    )
    .await
    .expect("cannot query for built in application")
    .first()
    .expect("no built in application found")
    .clone();
    dbg!(&write_tenancy);
    let schema_variant = create_schema_variant(
        &txn,
        &nats,
        &(&write_tenancy).into(),
        &visibility,
        &history_actor,
        veritech.clone(),
        encr_key,
        *application_schema.id(),
    )
    .await;

    dbg!(&schema_variant);
    let application = create_component_for_schema(
        &txn,
        &nats,
        veritech,
        &encr_key,
        &(&write_tenancy).into(),
        &visibility,
        &history_actor,
        application_schema.id(),
    )
    .await;

    let items = get_node_menu_items(
        &txn,
        &read_tenancy,
        &visibility,
        &MenuFilter::new(SchematicKind::Deployment, *application.id()),
    )
    .await
    .expect("cannot get items");

    let service_item = items.iter().find(|(_path, item)| item.name == "service");
    assert!(service_item.is_some(), "menu must include the service item");
}
