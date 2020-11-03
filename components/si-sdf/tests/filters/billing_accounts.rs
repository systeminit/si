use crate::{
    billing_account_cleanup, one_time_setup, test_cleanup, test_setup, TestAccount, DB, NATS,
    SETTINGS,
};
use si_sdf::{
    filters::api,
    models::{billing_account, BillingAccount, GetReply, PublicKey},
};

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

#[tokio::test]
async fn get_public_key() {
    let test_account = test_setup().await.expect("failed to setup test");

    let filter = api(&DB, &NATS, &SETTINGS.jwt_encrypt.key);

    let res = warp::test::request()
        .method("GET")
        .header("authorization", &test_account.authorization)
        .path(
            format!(
                "/billingAccounts/{}/publicKey",
                &test_account.billing_account_id
            )
            .as_ref(),
        )
        .reply(&filter)
        .await;
    assert_eq!(res.status(), 200, "model should be found");
    let reply: GetReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize get model reply");
    let item: PublicKey =
        serde_json::from_value(reply.item).expect("cannot deserialize mode from get model reply");

    assert_eq!(test_account.billing_account.current_key_pair_id, item.id);

    test_cleanup(test_account)
        .await
        .expect("failed to finish test");
}

#[tokio::test]
async fn rotate_public_key() {
    let test_account = test_setup().await.expect("failed to setup test");

    async fn get_public_key(test_account: &TestAccount) -> PublicKey {
        let filter = api(&DB, &NATS, &SETTINGS.jwt_encrypt.key);

        let res = warp::test::request()
            .method("GET")
            .header("authorization", &test_account.authorization)
            .path(
                format!(
                    "/billingAccounts/{}/publicKey",
                    &test_account.billing_account_id
                )
                .as_ref(),
            )
            .reply(&filter)
            .await;
        assert_eq!(res.status(), 200, "model should be found");
        let reply: GetReply =
            serde_json::from_slice(res.body()).expect("cannot deserialize get model reply");

        serde_json::from_value(reply.item).expect("cannot deserialize mode from get model reply")
    }

    let current = get_public_key(&test_account).await;
    assert_eq!(test_account.billing_account.current_key_pair_id, current.id);

    BillingAccount::rotate_key_pair(&DB, &NATS, &test_account.billing_account_id)
        .await
        .expect("failed to rotate key pair");

    let new = get_public_key(&test_account).await;
    assert_ne!(new, current);
    assert_ne!(test_account.billing_account.current_key_pair_id, new.id);

    test_cleanup(test_account)
        .await
        .expect("failed to finish test");
}
