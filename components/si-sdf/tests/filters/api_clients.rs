use crate::{test_cleanup, test_setup};
use crate::{DB, NATS, SETTINGS};
use si_sdf::filters::api;
use si_sdf::models::api_client::{ApiClientKind, CreateReply, CreateRequest};

#[tokio::test]
async fn create() {
    let test_account = test_setup().await.expect("failed to setup test");

    let filter = api(&DB, &NATS, &SETTINGS.jwt_encrypt.key);

    let res = warp::test::request()
        .method("POST")
        .header("authorization", &test_account.authorization)
        .json(&CreateRequest {
            name: String::from("killer be killed"),
            kind: ApiClientKind::Cli,
        })
        .path(format!("/apiClients").as_ref())
        .reply(&filter)
        .await;

    let result: CreateReply = serde_json::from_str(String::from_utf8_lossy(res.body()).as_ref())
        .expect("cannot create an api client, results do not deserialize");
    assert_eq!(result.api_client.name, "killer be killed", "name matches");

    test_cleanup(test_account)
        .await
        .expect("failed to cleanup test");
}
