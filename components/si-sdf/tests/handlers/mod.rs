use si_sdf::handlers;

use crate::DB;
use crate::{test_cleanup, test_setup};

#[tokio::test]
async fn authorize() {
    let test_account_one = test_setup().await.expect("failed to setup test1");
    let test_account_two = test_setup().await.expect("failed to setup test2");

    handlers::authorize(
        &DB,
        &test_account_one.user_id,
        &test_account_one.billing_account_id,
        "changeSet",
        "create",
    )
    .await
    .expect("authorization to succeed");

    handlers::authorize(
        &DB,
        &test_account_two.user_id,
        &test_account_two.billing_account_id,
        "changeSet",
        "create",
    )
    .await
    .expect("authorization to succeed");

    let error = handlers::authorize(
        &DB,
        &test_account_one.user_id,
        &test_account_two.billing_account_id,
        "changeSet",
        "create",
    )
    .await;
    match error {
        Ok(_) => panic!("succeeded in authorization when it should fail"),
        Err(handlers::HandlerError::Unauthorized) => {}
        Err(err) => panic!("cannot check auth, unknown error: {}", err),
    }

    let error = handlers::authorize(
        &DB,
        &test_account_two.user_id,
        &test_account_one.billing_account_id,
        "changeSet",
        "create",
    )
    .await;
    match error {
        Ok(_) => panic!("succeeded in authorization when it should fail"),
        Err(handlers::HandlerError::Unauthorized) => {}
        Err(err) => panic!("cannot check auth, unknown error: {}", err),
    }
}
