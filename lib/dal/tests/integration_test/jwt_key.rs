use dal::jwt_key;
use dal::test_harness::{one_time_setup, TestContext};
use jwt_simple::algorithms::RSAKeyPairLike;

// {get_jwt_signing_key, get_jwt_validation_key};

#[tokio::test]
async fn get_jwt_signing_key() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, _nats_conn, secret_key) = ctx.entries();
    let mut conn = pg.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let _signing_key = jwt_key::get_jwt_signing_key(&txn, &secret_key)
        .await
        .expect("cannot get jwt signing key");
}

#[tokio::test]
async fn get_jwt_validation_key() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, _nats_conn, secret_key) = ctx.entries();
    let mut conn = pg.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let signing_key = jwt_key::get_jwt_signing_key(&txn, &secret_key)
        .await
        .expect("cannot get jwt signing key");

    let _validation_key = jwt_key::get_jwt_validation_key(
        &txn,
        signing_key
            .key_id()
            .as_ref()
            .expect("this key should have an id, that it doesn't is a problem"),
    )
    .await
    .expect("cannot get jwt validation key");
}
