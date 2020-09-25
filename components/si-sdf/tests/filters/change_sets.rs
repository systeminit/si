use serde_json;

use crate::{test_cleanup, test_setup, TestAccount};
use crate::{DB, NATS, SETTINGS};

use si_sdf::filters::api;
use si_sdf::models::change_set;

pub async fn create_change_set(test_account: &TestAccount) -> String {
    let filter = api(&DB, &NATS, &SETTINGS.jwt_encrypt.key);
    let res = warp::test::request()
        .method("POST")
        .header("authorization", &test_account.authorization)
        .json(&change_set::CreateRequest {
            name: None,
            workspace_id: test_account.workspace_id.clone(),
            organization_id: test_account.organization_id.clone(),
        })
        .path("/changeSets")
        .reply(&filter)
        .await;
    assert_eq!(res.status(), 200, "change set is created");
    let result_json: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(res.body()).as_ref())
            .expect("cannot create a change set, results do not deserialize");
    if result_json["item"]["id"].is_string() {
        return result_json["item"]["id"].as_str().unwrap().to_string();
    } else {
        panic!("change set output is wrong!");
    }
}

pub async fn execute_change_set(test_account: &TestAccount, change_set_id: impl AsRef<str>) {
    let filter = api(&DB, &NATS, &SETTINGS.jwt_encrypt.key);
    let change_set_id = change_set_id.as_ref();

    let request = change_set::PatchRequest {
        op: change_set::PatchOps::Execute(change_set::ExecuteRequest {
            hypothetical: false,
        }),
        workspace_id: test_account.workspace_id.clone(),
        organization_id: test_account.organization_id.clone(),
    };

    let res = warp::test::request()
        .method("PATCH")
        .header("authorization", &test_account.authorization)
        .path(format!("/changeSets/{}", change_set_id).as_ref())
        .json(&request)
        .reply(&filter)
        .await;
    assert_eq!(res.status(), 200, "change set is executed");
}

#[tokio::test]
async fn create() {
    let test_account = test_setup().await.expect("failed to setup test");

    let filter = api(&DB, &NATS, &SETTINGS.jwt_encrypt.key);

    let res = warp::test::request()
        .method("POST")
        .header("authorization", &test_account.authorization)
        .json(&change_set::CreateRequest {
            name: None,
            workspace_id: test_account.workspace_id.clone(),
            organization_id: test_account.organization_id.clone(),
        })
        .path("/changeSets")
        .reply(&filter)
        .await;
    assert_eq!(res.status(), 200, "change set is created");

    test_cleanup(test_account)
        .await
        .expect("failed to finish test");
}

#[tokio::test]
async fn execute() {
    let test_account = test_setup().await.expect("failed to setup test");

    let filter = api(&DB, &NATS, &SETTINGS.jwt_encrypt.key);
    let change_set_id = create_change_set(&test_account).await;
    let request = change_set::PatchRequest {
        op: change_set::PatchOps::Execute(change_set::ExecuteRequest { hypothetical: true }),
        workspace_id: test_account.workspace_id.clone(),
        organization_id: test_account.organization_id.clone(),
    };

    let res = warp::test::request()
        .method("PATCH")
        .header("authorization", &test_account.authorization)
        .path(format!("/changeSets/{}", &change_set_id).as_ref())
        .json(&request)
        .reply(&filter)
        .await;
    assert_eq!(res.status(), 200, "change set is executed");

    test_cleanup(test_account)
        .await
        .expect("failed to finish test");
}
