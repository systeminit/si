use warp::http::StatusCode;

use si_sdf::filters::api;
use si_sdf::handlers::session_dal::{LoginReply, LoginRequest, RestoreAuthenticationReply};
use si_sdf::handlers::HandlerErrorReply;

use crate::models::billing_account::{signup_new_billing_account, NewBillingAccount};
use crate::one_time_setup;
use crate::TestContext;

pub async fn login_user(ctx: &TestContext, nba: &NewBillingAccount) -> String {
    let (pg, nats_conn, veritech, event_log_fs, secret_key) = ctx.entries();

    let filter = api(pg, nats_conn, veritech, event_log_fs, secret_key);

    let request = LoginRequest {
        billing_account_name: nba.billing_account.name.clone(),
        user_email: nba.user.email.clone(),
        user_password: nba.user_password.clone(),
    };
    let res = warp::test::request()
        .method("POST")
        .path("/sessionDal/login")
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
        user_email: nba.user.email.clone(),
        user_password: nba.user_password.clone(),
    };

    let res = warp::test::request()
        .method("POST")
        .path("/sessionDal/login")
        .json(&request)
        .reply(&filter)
        .await;

    assert_eq!(res.status(), StatusCode::OK);

    let reply: LoginReply = serde_json::from_slice(res.body()).expect("cannot deserialize reply");
    assert_eq!(&reply.user.id, &nba.user.id);
    assert_eq!(&reply.billing_account.id, &nba.billing_account.id);
}

#[tokio::test]
async fn login_bad_password() {
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
        user_email: nba.user.email.clone(),
        user_password: "i-dont-think-so".into(),
    };

    let res = warp::test::request()
        .method("POST")
        .path("/sessionDal/login")
        .json(&request)
        .reply(&filter)
        .await;

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    let reply: HandlerErrorReply =
        serde_json::from_slice(res.body()).expect("could not deserialize response");
    let cause = reply.into_cause();
    assert_eq!(cause.code, StatusCode::UNAUTHORIZED);
    assert_eq!(cause.message, "request is unauthorized");
}

#[tokio::test]
async fn restore_authentication() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, event_log_fs, secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot get connection");
    let txn = conn.transaction().await.expect("cannot get transaction");
    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
    txn.commit().await.expect("cannot commit txn");

    let token = login_user(&ctx, &nba).await;
    let filter = api(pg, nats_conn, veritech, event_log_fs, secret_key);

    let res = warp::test::request()
        .method("GET")
        .header("authorization", &token)
        .path("/sessionDal/restoreAuthentication")
        .reply(&filter)
        .await;

    assert_eq!(res.status(), StatusCode::OK, "create should succeed");
    let reply: RestoreAuthenticationReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize node reply");
    assert_eq!(reply.user, nba.user);
    assert_eq!(reply.billing_account, nba.billing_account);
}
