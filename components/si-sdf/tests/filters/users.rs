use si_sdf::filters::api;
use si_sdf::models::{LoginReply, LoginRequest};

use crate::models::billing_account::{signup_new_billing_account, NewBillingAccount};
use crate::one_time_setup;
use crate::TestContext;

pub async fn login_user(ctx: &TestContext, nba: &NewBillingAccount) -> String {
    let (pg, nats_conn, veritech, event_log_fs, secret_key) = ctx.entries();
    let filter = api(pg, nats_conn, veritech, event_log_fs, secret_key);

    let request = LoginRequest {
        billing_account_name: nba.billing_account.name.clone(),
        email: nba.user.email.clone(),
        password: nba.user_password.clone(),
    };
    let res = warp::test::request()
        .method("POST")
        .path("/users/login")
        .json(&request)
        .reply(&filter)
        .await;

    let reply: LoginReply = serde_json::from_slice(res.body()).expect("cannot deserialize reply");
    format!("Bearer {}", reply.jwt)
}

#[tokio::test]
async fn login() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, event_log_fs, secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
    txn.commit()
        .await
        .expect("failed to commit the new billing account");

    let filter = api(pg, nats_conn, veritech, event_log_fs, secret_key);

    let request = LoginRequest {
        billing_account_name: nba.billing_account.name.clone(),
        email: nba.user.email.clone(),
        password: nba.user_password.clone(),
    };

    let res = warp::test::request()
        .method("POST")
        .path("/users/login")
        .json(&request)
        .reply(&filter)
        .await;

    assert!(res.status().is_success());

    let reply: LoginReply = serde_json::from_slice(res.body()).expect("cannot deserialize reply");
    assert_eq!(&reply.user.id, &nba.user.id);
}
