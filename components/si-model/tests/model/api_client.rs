use si_model_test::model::billing_account::signup_new_billing_account;
use si_model_test::{one_time_setup, TestContext, SETTINGS};

use si_model::{ApiClient, ApiClientKind, Group};

#[tokio::test]
async fn new() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;

    let (api_client, _token) = ApiClient::new(
        &txn,
        &nats,
        &SETTINGS.jwt_encrypt.key,
        "alex",
        ApiClientKind::Cli,
        &nba.billing_account.id,
    )
    .await
    .expect("cannot create new api client");
    assert_eq!(api_client.name, "alex");
    assert_eq!(api_client.kind, ApiClientKind::Cli);
    assert_eq!(
        &api_client.si_storable.billing_account_id,
        &nba.billing_account.id
    );

    // The api client should be in the administrators group
    let admins = Group::get_administrators_group(&txn, &nba.billing_account.id)
        .await
        .expect("cannot get administators group");
    assert_eq!(&admins.api_client_ids[0], &api_client.id);
}

#[tokio::test]
async fn get() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;

    let (api_client, _token) = ApiClient::new(
        &txn,
        &nats,
        &SETTINGS.jwt_encrypt.key,
        "alex",
        ApiClientKind::Cli,
        &nba.billing_account.id,
    )
    .await
    .expect("cannot create new api client");

    let same_api_client = ApiClient::get(&txn, &api_client.id)
        .await
        .expect("cannot get api client");
    assert_eq!(&api_client, &same_api_client);
}
