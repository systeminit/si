use crate::{test_cleanup, test_setup, TestAccount, DB, NATS, SETTINGS};
use si_sdf::{
    filters::api,
    models::{
        secret, GetReply, ListReply, Secret, SecretAlgorithm, SecretKind, SecretObjectType,
        SecretVersion,
    },
};

pub async fn create_secret(
    test_account: &TestAccount,
    name: impl Into<String>,
    object_type: SecretObjectType,
    kind: SecretKind,
    crypted: impl Into<Vec<u8>>,
    key_pair_id: impl Into<String>,
) -> secret::CreateReply {
    let name = name.into();
    let crypted = crypted.into();
    let key_pair_id = key_pair_id.into();
    let filter = api(&DB, &NATS, &SETTINGS.jwt_encrypt.key);
    let res = warp::test::request()
        .method("POST")
        .header("authorization", &test_account.authorization)
        .json(&secret::CreateRequest {
            name,
            object_type,
            kind,
            crypted,
            key_pair_id,
            version: Default::default(),
            algorithm: Default::default(),
            organization_id: test_account.organization_id.clone(),
            workspace_id: test_account.workspace_id.clone(),
        })
        .path("/secrets")
        .reply(&filter)
        .await;
    assert_eq!(res.status(), 200, "model should be created");

    serde_json::from_slice(res.body()).expect("cannot deserialize node reply")
}

#[tokio::test]
async fn create() {
    let test_account = test_setup().await.expect("failed to setup test");

    let filter = api(&DB, &NATS, &SETTINGS.jwt_encrypt.key);

    let res = warp::test::request()
        .method("POST")
        .header("authorization", &test_account.authorization)
        .json(&secret::CreateRequest {
            name: "my-fear".to_string(),
            object_type: SecretObjectType::Credential,
            kind: SecretKind::DockerHub,
            crypted: "clown orgs".as_bytes().to_owned(),
            key_pair_id: "keyPair:huh...".to_string(),
            version: SecretVersion::V1,
            algorithm: SecretAlgorithm::Sealedbox,
            organization_id: test_account.organization_id.clone(),
            workspace_id: test_account.workspace_id.clone(),
        })
        .path("/secrets")
        .reply(&filter)
        .await;
    assert_eq!(res.status(), 200, "model should be created");
    let reply: secret::CreateReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize create secret reply");

    assert!(reply.item.id.starts_with("secret:"));
    assert_eq!("my-fear", reply.item.name);
    assert_eq!("secret", reply.item.si_storable.type_name);
    assert_eq!(
        test_account.billing_account_id,
        reply.item.si_storable.billing_account_id
    );
    assert_eq!(
        test_account.organization_id,
        reply.item.si_storable.organization_id
    );
    assert_eq!(
        test_account.workspace_id,
        reply.item.si_storable.workspace_id
    );
    assert_eq!(
        Some(&test_account.user_id),
        reply.item.si_storable.created_by_user_id.as_ref()
    );

    test_cleanup(test_account)
        .await
        .expect("failed to finish test");
}

#[tokio::test]
async fn get() {
    let test_account = test_setup().await.expect("failed to setup test");
    let created = create_secret(
        &test_account,
        "tom-petty",
        SecretObjectType::Credential,
        SecretKind::DockerHub,
        "big-weekend",
        "keyPair:huh...",
    )
    .await;

    let filter = api(&DB, &NATS, &SETTINGS.jwt_encrypt.key);

    let res = warp::test::request()
        .method("GET")
        .header("authorization", &test_account.authorization)
        .path(format!("/secrets/{}", &created.item.id).as_ref())
        .reply(&filter)
        .await;
    assert_eq!(res.status(), 200, "model should be found");
    let reply: GetReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize get model reply");
    let fetched: Secret =
        serde_json::from_value(reply.item).expect("cannot deserialize model from get model reply");

    assert_eq!(created.item, fetched);

    test_cleanup(test_account)
        .await
        .expect("failed to finish test");
}

#[tokio::test]
async fn get_missing() {
    let test_account = test_setup().await.expect("failed to setup test");

    let filter = api(&DB, &NATS, &SETTINGS.jwt_encrypt.key);

    let res = warp::test::request()
        .method("GET")
        .header("authorization", &test_account.authorization)
        .path("/secrets/secret:this-surely-wont-be-an-id")
        .reply(&filter)
        .await;
    assert_eq!(res.status(), 404, "model should not be found");

    test_cleanup(test_account)
        .await
        .expect("failed to finish test");
}

#[tokio::test]
async fn list() {
    async fn secret(test_account: &TestAccount, name: &str, crypted: &str) -> secret::CreateReply {
        create_secret(
            test_account,
            name,
            SecretObjectType::Credential,
            SecretKind::DockerHub,
            crypted,
            "keyPair:huh...",
        )
        .await
    }

    let test_account = test_setup().await.expect("failed to setup test");

    let alpha = secret(&test_account, "alpha", "bleep").await;
    let charlie = secret(&test_account, "charlie", "blorp").await;
    let bravo = secret(&test_account, "bravo", "blurp").await;

    let filter = api(&DB, &NATS, &SETTINGS.jwt_encrypt.key);

    let res = warp::test::request()
        .method("GET")
        .header("authorization", &test_account.authorization)
        .path("/secrets?pageSize=1000")
        .reply(&filter)
        .await;
    assert_eq!(res.status(), 200, "model list should be found");
    let reply: ListReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize get model reply");
    let mut items: Vec<secret::Secret> = Vec::new();
    for item in reply.items.into_iter() {
        items.push(
            serde_json::from_value(item)
                .expect("cannot deserialze model from get list model reply"),
        );
    }

    assert_eq!(3, reply.total_count);
    assert!(reply.page_token.is_none());

    let items_alpha = items.iter().find(|m| m.name == "alpha");
    assert_eq!(Some(&alpha.item), items_alpha);
    let items_bravo = items.iter().find(|m| m.name == "bravo");
    assert_eq!(Some(&bravo.item), items_bravo);
    let items_charlie = items.iter().find(|m| m.name == "charlie");
    assert_eq!(Some(&charlie.item), items_charlie);

    test_cleanup(test_account)
        .await
        .expect("failed to finish test");
}

#[tokio::test]
async fn list_none() {
    let test_account = test_setup().await.expect("failed to setup test");

    let filter = api(&DB, &NATS, &SETTINGS.jwt_encrypt.key);

    let res = warp::test::request()
        .method("GET")
        .header("authorization", &test_account.authorization)
        .path("/secrets?pageSize=1000")
        .reply(&filter)
        .await;
    assert_eq!(res.status(), 200, "model list should be found");
    let reply: ListReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize get model reply");

    assert_eq!(0, reply.total_count);
    assert!(reply.items.is_empty());
    assert!(reply.page_token.is_none());

    test_cleanup(test_account)
        .await
        .expect("failed to finish test");
}
