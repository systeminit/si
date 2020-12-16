use warp::http::StatusCode;

use si_sdf::filters::api;
use si_sdf::models::{
    billing_account::{CreateReply, CreateRequest},
    BillingAccount, GetReply, PublicKey,
};

use crate::filters::users::login_user;
use crate::models::billing_account::signup_new_billing_account;
use crate::one_time_setup;
use crate::TestContext;

#[tokio::test]
async fn create() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, event_log_fs, secret_key) = ctx.entries();

    let filter = api(pg, nats_conn, veritech, event_log_fs, secret_key);

    let request = CreateRequest {
        billing_account_name: "alice".into(),
        billing_account_description: "the rooster".into(),
        user_name: "layne".into(),
        user_email: "layne@tclown.com".into(),
        user_password: "layneRules".into(),
    };

    let res = warp::test::request()
        .method("POST")
        .path("/billingAccounts")
        .json(&request)
        .reply(&filter)
        .await;
    assert_eq!(res.status(), 200, "billing account is created");
    let reply: CreateReply =
        serde_json::from_slice(res.body()).expect("could not deserialize response");
    assert_eq!(&reply.billing_account.name, &request.billing_account_name);
    assert_eq!(
        &reply.billing_account.description,
        &request.billing_account_description
    );
    assert_eq!(&reply.user.name, &request.user_name);
    assert_eq!(&reply.user.email, &request.user_email);
}

#[tokio::test]
async fn get_public_key() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, event_log_fs, secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot get connection");
    let txn = conn.transaction().await.expect("cannot get transaction");
    let nba = signup_new_billing_account(&txn, &nats).await;
    txn.commit().await.expect("cannot commit txn");

    let token = login_user(&ctx, &nba).await;
    let filter = api(pg, nats_conn, veritech, event_log_fs, secret_key);

    let res = warp::test::request()
        .method("GET")
        .header("authorization", token)
        .path(format!("/billingAccounts/{}/publicKey", &nba.billing_account.id,).as_ref())
        .reply(&filter)
        .await;
    assert_eq!(res.status(), 200, "model should be found");
    let reply: GetReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize get model reply");
    let item: PublicKey =
        serde_json::from_value(reply.item).expect("cannot deserialize mode from get model reply");

    assert_eq!(&nba.public_key.id, &item.id);
}

#[tokio::test]
async fn get() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, event_log_fs, secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot get connection");
    let txn = conn.transaction().await.expect("cannot get transaction");
    let nba = signup_new_billing_account(&txn, &nats).await;
    let second_nba = signup_new_billing_account(&txn, &nats).await;
    txn.commit().await.expect("cannot commit txn");

    let token = login_user(&ctx, &nba).await;
    let filter = api(pg, nats_conn, veritech, event_log_fs, secret_key);

    let res = warp::test::request()
        .method("GET")
        .header("authorization", &token)
        .path(format!("/billingAccounts/{}", &nba.billing_account.id,).as_ref())
        .reply(&filter)
        .await;
    assert_eq!(res.status(), 200, "model should be found");
    let reply: GetReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize get model reply");
    let item: BillingAccount =
        serde_json::from_value(reply.item).expect("cannot deserialize mode from get model reply");

    assert_eq!(&nba.billing_account.id, &item.id);

    let res = warp::test::request()
        .method("GET")
        .header("authorization", &token)
        .path(format!("/billingAccounts/{}", &second_nba.billing_account.id,).as_ref())
        .reply(&filter)
        .await;
    dbg!(&res);
    assert_eq!(
        res.status(),
        StatusCode::UNAUTHORIZED,
        "model should return unauthorized"
    );
}
