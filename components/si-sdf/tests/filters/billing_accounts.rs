use serde_json;

use crate::{billing_account_cleanup, one_time_setup, test_cleanup, TestAccount};
use crate::{DB, NATS, SETTINGS};

use si_sdf::filters::api;
use si_sdf::models::billing_account;

pub async fn signup() -> billing_account::CreateReply {
    let fake_name = si_sdf::models::generate_id("clown");
    let filter = api(&DB, &NATS, &SETTINGS.jwt_encrypt.key);
    let request = billing_account::CreateRequest {
        billing_account_name: fake_name.clone(),
        billing_account_description: "The Clown Company".into(),
        user_name: fake_name.clone(),
        user_email: format!("{}@tclown.com", fake_name),
        user_password: "boboR0cks".into(),
    };

    let res = warp::test::request()
        .method("POST")
        .path("/billingAccounts")
        .json(&request)
        .reply(&filter)
        .await;
    println!("{:?}", res);
    assert_eq!(res.status(), 200, "billing account is created");
    let reply: billing_account::CreateReply =
        serde_json::from_slice(res.body()).expect("could not deserialize response");
    reply
}

#[tokio::test]
async fn create() {
    one_time_setup().await.expect("failed setup");
    billing_account_cleanup()
        .await
        .expect("failed to delete billing account");
    let filter = api(&DB, &NATS, &SETTINGS.jwt_encrypt.key);
    let request = billing_account::CreateRequest {
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
    println!("{:?}", res);
    assert_eq!(res.status(), 200, "billing account is created");
    let reply: billing_account::CreateReply =
        serde_json::from_slice(res.body()).expect("could not deserialize response");

    let test_account = TestAccount {
        user_id: reply.user.id.clone(),
        billing_account_id: reply.billing_account.id.clone(),
        workspace_id: reply.workspace.id,
        organization_id: reply.organization.id,
        user: reply.user,
        billing_account: reply.billing_account,
        authorization: String::from("poop"),
        system_ids: None,
    };

    test_cleanup(test_account)
        .await
        .expect("failed to finish test");
}
