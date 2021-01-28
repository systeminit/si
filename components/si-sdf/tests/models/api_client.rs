use crate::models::billing_account::{signup_new_billing_account, NewBillingAccount};
use crate::{one_time_setup, TestContext, SETTINGS};

use si_sdf::data::{NatsTxn, PgTxn};
use si_sdf::models::{ApiClient, ApiClientKind, Group};

pub async fn create_api_client(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    nba: &NewBillingAccount,
) -> (ApiClient, String) {
    let mut name_generator = names::Generator::with_naming(names::Name::Numbered);
    let name = name_generator.next().unwrap();

    let (api_client, token) = ApiClient::new(
        &txn,
        &nats,
        &SETTINGS.jwt_encrypt.key,
        name,
        ApiClientKind::Cli,
        &nba.billing_account.id,
    )
    .await
    .expect("cannot create new api client");
    (api_client, token)
}

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

#[tokio::test]
async fn list() {
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

    let (second_api_client, _token) = ApiClient::new(
        &txn,
        &nats,
        &SETTINGS.jwt_encrypt.key,
        "bobo",
        ApiClientKind::Cli,
        &nba.billing_account.id,
    )
    .await
    .expect("cannot create new api client");

    let result = ApiClient::list(&txn, &nba.billing_account.id, None, None, None, None, None)
        .await
        .expect("cannot list api clients");
    assert_eq!(result.total_count, 2);
    result
        .items
        .iter()
        .find(|&i| i["name"] == serde_json::json![&api_client.name])
        .expect("cannot find api client alex");
    result
        .items
        .iter()
        .find(|&i| i["name"] == serde_json::json![&second_api_client.name])
        .expect("cannot find api client bobo");
}
