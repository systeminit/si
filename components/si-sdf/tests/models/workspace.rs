use crate::{
    generate_fake_name,
    models::billing_account::{new_billing_account, NewBillingAccount},
    models::organization::create_test_organization,
    one_time_setup, TestContext,
};

use si_sdf::{
    data::{NatsTxn, PgTxn},
    models::{BooleanTerm, Item, Query, Workspace},
};

pub async fn create_workspace(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    nba: &NewBillingAccount,
) -> Workspace {
    Workspace::new(
        txn,
        nats,
        generate_fake_name(),
        &nba.billing_account.id,
        &nba.organization.id,
    )
    .await
    .expect("cannot create workspace")
}

#[tokio::test]
async fn new() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, _veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");
    let ba = new_billing_account(&txn, &nats).await;
    let org = create_test_organization(&txn, &nats, "dark tranquility", &ba.id).await;

    let workspace = Workspace::new(&txn, &nats, "jesse leach", &ba.id, &org.id)
        .await
        .expect("cannot create workspace");
    assert_eq!(workspace.name, "jesse leach");
}

#[tokio::test]
async fn get() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, _veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");
    let ba = new_billing_account(&txn, &nats).await;
    let org = create_test_organization(&txn, &nats, "dark tranquility", &ba.id).await;

    let workspace = Workspace::new(&txn, &nats, "adam d", &ba.id, &org.id)
        .await
        .expect("cannot create workspace");
    assert_eq!(workspace.name, "adam d");
    let wg = Workspace::get(&txn, &workspace.id)
        .await
        .expect("cannot get workspace");
    assert_eq!(wg.name, workspace.name);
    assert_eq!(wg.id, workspace.id);
}

#[tokio::test]
async fn list_model() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, _veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");
    let ba = new_billing_account(&txn, &nats).await;
    let org = create_test_organization(&txn, &nats, "dark tranquility", &ba.id).await;

    let workspaces = ["at the gates", "killswitch engage", "parkway drive"];
    for ws_name in workspaces.iter() {
        let _workspace = Workspace::new(&txn, &nats, *ws_name, &ba.id, &org.id)
            .await
            .expect("cannot create workspace");
    }
    // List all items
    let results = Workspace::list(&txn, &ba.id, None, None, None, None, None)
        .await
        .expect("workspace list failed");
    assert_eq!(results.items.len(), 3);

    // List with a single, un-nested query
    let query_results = Workspace::list(
        &txn,
        &ba.id,
        Some(Query::generate_for_string(
            "name",
            si_sdf::models::Comparison::Equals,
            "at the gates",
        )),
        None,
        None,
        None,
        None,
    )
    .await
    .expect("workspace list query failed");
    assert_eq!(query_results.items.len(), 1);
    assert_eq!(
        query_results.items[0]["name"].to_string(),
        "\"at the gates\""
    );

    // List with a single nested query
    let query_results = Workspace::list(
        &txn,
        &ba.id,
        Some(Query::generate_for_string(
            "siStorable.typeName",
            si_sdf::models::Comparison::Equals,
            "workspace",
        )),
        None,
        None,
        None,
        None,
    )
    .await
    .expect("workspace list query failed");
    assert_eq!(query_results.items.len(), 3);
    assert_eq!(
        query_results.items[0]["name"].to_string(),
        "\"at the gates\""
    );

    // List with multiple queries
    let query_results = Workspace::list(
        &txn,
        &ba.id,
        Some(Query::new(
            vec![
                Item {
                    query: Some(Query::generate_for_string(
                        "siStorable.typeName",
                        si_sdf::models::Comparison::Equals,
                        "workspace",
                    )),
                    expression: None,
                },
                Item {
                    query: Some(Query::generate_for_string(
                        "name",
                        si_sdf::models::Comparison::Equals,
                        "at the gates",
                    )),
                    expression: None,
                },
            ],
            Some(BooleanTerm::And),
            None,
        )),
        None,
        None,
        None,
        None,
    )
    .await
    .expect("workspace list query failed");
    assert_eq!(query_results.items.len(), 1);
    assert_eq!(
        query_results.items[0]["name"].to_string(),
        "\"at the gates\""
    );
}

#[tokio::test]
async fn save() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, _veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");
    let ba = new_billing_account(&txn, &nats).await;
    let org = create_test_organization(&txn, &nats, "dark tranquility", &ba.id).await;

    let workspace = Workspace::new(&txn, &nats, "adam d", &ba.id, &org.id)
        .await
        .expect("cannot create workspace");
    assert_eq!(workspace.name, "adam d");
    let mut wg = Workspace::get(&txn, &workspace.id)
        .await
        .expect("cannot get workspace");
    assert_eq!(wg.name, workspace.name);
    assert_eq!(wg.id, workspace.id);
    wg.name = String::from("poopy pants");
    let updated_wg = wg.save(&txn, &nats).await.expect("cannot save workspace");
    assert_eq!(&updated_wg.name, "poopy pants");
    let wg_updated = Workspace::get(&txn, &workspace.id)
        .await
        .expect("cannot get workspace");
    assert_eq!(&wg_updated.name, "poopy pants");
}
