use si_sdf::handlers;

use crate::filters::users::login_user;
use crate::models::billing_account::signup_new_billing_account;
use crate::one_time_setup;
use crate::TestContext;

#[tokio::test]
async fn authorize() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");
    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
    txn.commit()
        .await
        .expect("failed to commit the new billing account");
    let txn = conn.transaction().await.expect("cannot create txn");

    handlers::authorize(&txn, &nba.user.id, "changeSet", "create")
        .await
        .expect("authorization to succeed");
}

#[tokio::test]
async fn authenticate() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");
    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
    txn.commit()
        .await
        .expect("failed to commit the new billing account");

    let txn = conn.transaction().await.expect("cannot create txn");
    let token = login_user(&ctx, &nba).await;

    let claim = handlers::authenticate(&txn, token)
        .await
        .expect("authentication to succeed");
    assert_eq!(claim.user_id, nba.user.id);
    assert_eq!(claim.billing_account_id, nba.billing_account.id);
}

#[tokio::test]
async fn validate_tenancy() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");
    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
    txn.commit()
        .await
        .expect("failed to commit the new billing account");

    let txn = conn.transaction().await.expect("cannot create txn");

    handlers::validate_tenancy(
        &txn,
        "workspaces",
        &nba.workspace.id,
        &nba.billing_account.id,
    )
    .await
    .expect("validation to succeed");
}
