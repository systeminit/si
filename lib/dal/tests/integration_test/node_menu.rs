use dal::node_menu::get_node_menu_items;
use dal::test_harness::{billing_account_signup, create_component_for_schema};
use dal::{node_menu::MenuFilter, Schema, SchematicKind};
use dal::{HistoryActor, StandardModel, Tenancy, Visibility};
use test_env_log::test;

use crate::test_setup;

#[test(tokio::test)]
async fn get_node_menu() {
    test_setup!(ctx, secret_key, _pg, _conn, txn, _nats_conn, nats, veritech, encr_key);
    let (nba, _key) = billing_account_signup(&txn, &nats, secret_key).await;
    let visibility = Visibility::new_head(false);
    let tenancy = Tenancy::new_workspace(vec![*nba.workspace.id()]);
    let history_actor = HistoryActor::SystemInit;

    let tenancy_uni = Tenancy::new_universal();

    let application_schema = Schema::find_by_attr(
        &txn,
        &tenancy_uni,
        &visibility,
        "name",
        &String::from("application"),
    )
    .await
    .expect("cannot query for built in application")
    .first()
    .expect("no built in application found")
    .clone();

    let application = create_component_for_schema(
        &txn,
        &nats,
        veritech,
        &encr_key,
        &tenancy,
        &visibility,
        &history_actor,
        application_schema.id(),
    )
    .await;

    let items = get_node_menu_items(
        &txn,
        &tenancy,
        &visibility,
        &MenuFilter::new(SchematicKind::Deployment, *application.id()),
    )
    .await
    .expect("cannot get items");

    let service_item = items.iter().find(|(_path, item)| item.name == "service");
    assert!(service_item.is_some(), "menu must include the service item");
}
