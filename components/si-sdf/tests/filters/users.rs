use si_sdf::filters::api;
use si_sdf::models::{LoginReply, LoginRequest};

use crate::{test_cleanup, test_setup};
use crate::{DB, NATS, SETTINGS};

pub async fn login_user(
    billing_account_name: impl Into<String>,
    email: impl Into<String>,
) -> String {
    let billing_account_name = billing_account_name.into();
    let email = email.into();

    let filter = api(&DB, &NATS, &SETTINGS.jwt_encrypt.key);

    let request = LoginRequest {
        billing_account_name,
        email,
        password: String::from("boboR0cks"),
    };
    let res = warp::test::request()
        .method("POST")
        .path("/users/login")
        .json(&request)
        .reply(&filter)
        .await;

    let reply: LoginReply = serde_json::from_slice(res.body()).expect("cannot deserialize reply");
    reply.jwt
}

#[tokio::test]
async fn login() {
    let test_account = test_setup().await.expect("failed to setup test");

    let filter = api(&DB, &NATS, &SETTINGS.jwt_encrypt.key);

    let request = LoginRequest {
        billing_account_name: test_account.billing_account.name.clone(),
        email: test_account.user.email.clone(),
        password: String::from("boboR0cks"),
    };

    let res = warp::test::request()
        .method("POST")
        .path("/users/login")
        .json(&request)
        .reply(&filter)
        .await;

    assert!(res.status().is_success());

    let _reply: LoginReply = serde_json::from_slice(res.body()).expect("cannot deserialize reply");

    test_cleanup(test_account)
        .await
        .expect("failed to finish test");
}
