use crate::models::billing_account::new_billing_account;
use crate::{one_time_setup, TestContext};

use si_sdf::data::{NatsTxn, PgTxn};
use si_sdf::models::{BooleanTerm, Item, Organization, Query};

pub async fn create_test_organization(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    name: impl Into<String>,
    billing_account_id: impl Into<String>,
) -> Organization {
    let name = name.into();
    let billing_account_id = billing_account_id.into();
    let o = Organization::new(txn, nats, name, billing_account_id)
        .await
        .expect("failed to create organization");
    o
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

    let organization = Organization::new(&txn, &nats, "jesse leach", &ba.id)
        .await
        .expect("cannot create organization");
    assert_eq!(organization.name, "jesse leach");
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

    let organization = Organization::new(&txn, &nats, "adam d", &ba.id)
        .await
        .expect("cannot create organization");
    assert_eq!(organization.name, "adam d");
    let wg = Organization::get(&txn, &organization.id)
        .await
        .expect("cannot get organization");
    assert_eq!(wg.name, organization.name);
    assert_eq!(wg.id, organization.id);
}

#[tokio::test]
async fn list() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, _veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");
    let ba = new_billing_account(&txn, &nats).await;

    let organizations = ["at the gates", "killswitch engage", "parkway drive"];
    for ws_name in organizations.iter() {
        let _organization = Organization::new(&txn, &nats, *ws_name, &ba.id)
            .await
            .expect("cannot create organization");
    }
    // List all items
    let results = Organization::list(&txn, &ba.id, None, None, None, None, None)
        .await
        .expect("organization list failed");
    assert_eq!(results.items.len(), 3);

    // List with a single, un-nested query
    let query_results = Organization::list(
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
    .expect("organization list query failed");
    assert_eq!(query_results.items.len(), 1);
    assert_eq!(
        query_results.items[0]["name"].to_string(),
        "\"at the gates\""
    );

    // List with a single nested query
    let query_results = Organization::list(
        &txn,
        &ba.id,
        Some(Query::generate_for_string(
            "siStorable.typeName",
            si_sdf::models::Comparison::Equals,
            "organization",
        )),
        None,
        None,
        None,
        None,
    )
    .await
    .expect("organization list query failed");
    assert_eq!(query_results.items.len(), 3);
    assert_eq!(
        query_results.items[0]["name"].to_string(),
        "\"at the gates\""
    );

    // List with multiple queries
    let query_results = Organization::list(
        &txn,
        &ba.id,
        Some(Query::new(
            vec![
                Item {
                    query: Some(Query::generate_for_string(
                        "siStorable.typeName",
                        si_sdf::models::Comparison::Equals,
                        "organization",
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
    .expect("organization list query failed");
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
    let organization = Organization::new(&txn, &nats, "adam d", &ba.id)
        .await
        .expect("cannot create organization");
    assert_eq!(organization.name, "adam d");
    let mut wg = Organization::get(&txn, &organization.id)
        .await
        .expect("cannot get organization");
    assert_eq!(wg.name, organization.name);
    assert_eq!(wg.id, organization.id);
    wg.name = String::from("poopy pants");
    let updated_wg = wg
        .save(&txn, &nats)
        .await
        .expect("cannot save organization");
    assert_eq!(&updated_wg.name, "poopy pants");
    let wg_updated = Organization::get(&txn, &organization.id)
        .await
        .expect("cannot get organization");
    assert_eq!(&wg_updated.name, "poopy pants");
}
