use warp::http::StatusCode;

use si_sdf::filters::api;
use si_sdf::handlers::signup_dal::{CreateReply, CreateRequest};

use si_model_test::{one_time_setup, signup_new_billing_account, TestContext};

#[tokio::test]
async fn create_billing_account() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, event_log_fs, secret_key) = ctx.entries();

    let filter = api(pg, nats_conn, veritech, event_log_fs, secret_key);

    let request = CreateRequest {
        billing_account_name: "leroy".into(),
        billing_account_description: "the rooster".into(),
        user_name: "layne".into(),
        user_email: "layne@tclown.com".into(),
        user_password: "layneRules".into(),
    };

    let res = warp::test::request()
        .method("POST")
        .path("/signupDal/createBillingAccount")
        .json(&request)
        .reply(&filter)
        .await;
    assert_eq!(res.status(), StatusCode::OK, "billing account is created");
    let reply: CreateReply =
        serde_json::from_slice(res.body()).expect("could not deserialize response");
    assert_eq!(&reply.billing_account.name, &request.billing_account_name);
    assert_eq!(
        &reply.billing_account.description,
        &request.billing_account_description
    );
}

#[tokio::test]
async fn create_billing_account_denied_if_existing() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, event_log_fs, secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot get connection");
    let txn = conn.transaction().await.expect("cannot get transaction");
    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
    txn.commit().await.expect("cannot commit txn");

    let filter = api(pg, nats_conn, veritech, event_log_fs, secret_key);

    let request = CreateRequest {
        billing_account_name: nba.billing_account.name,
        billing_account_description: "the rooster".into(),
        user_name: "layne".into(),
        user_email: "layne@tclown.com".into(),
        user_password: "layneRules".into(),
    };

    let res = warp::test::request()
        .method("POST")
        .path("/signupDal/createBillingAccount")
        .json(&request)
        .reply(&filter)
        .await;
    assert_eq!(
        res.status(),
        StatusCode::BAD_REQUEST,
        "billing account should not be created"
    );
}
