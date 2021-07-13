use si_model_test::{one_time_setup, signup_new_billing_account, TestContext};
use si_sdf::handlers;

#[tokio::test]
async fn validate_tenancy() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.get().await.expect("cannot connect to pg");
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
