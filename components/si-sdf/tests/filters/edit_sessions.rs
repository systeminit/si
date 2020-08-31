use crate::filters::change_sets::create_change_set;
use crate::{test_cleanup, test_setup, TestAccount};
use crate::{DB, NATS, SETTINGS};
use si_sdf::filters::api;
use si_sdf::models::edit_session::CreateRequest;

pub async fn create_edit_session(
    test_account: &TestAccount,
    change_set_id: impl AsRef<str>,
) -> String {
    let change_set_id = change_set_id.as_ref();
    let filter = api(&DB, &NATS, &SETTINGS.jwt_encrypt.key);

    let res = warp::test::request()
        .method("POST")
        .header("authorization", &test_account.authorization)
        .json(&CreateRequest {
            name: None,
            workspace_id: test_account.workspace_id.clone(),
            organization_id: test_account.organization_id.clone(),
        })
        .path(format!("/changeSets/{}/editSessions", change_set_id).as_ref())
        .reply(&filter)
        .await;

    let result_json: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(res.body()).as_ref())
            .expect("cannot create an edit session, results do not deserialize");
    if result_json["item"]["id"].is_string() {
        return result_json["item"]["id"].as_str().unwrap().to_string();
    } else {
        panic!("editSession output is wrong!");
    }
}

#[tokio::test]
async fn create() {
    let test_account = test_setup().await.expect("failed to setup test");

    let filter = api(&DB, &NATS, &SETTINGS.jwt_encrypt.key);

    let change_set_id = create_change_set(&test_account).await;

    let res = warp::test::request()
        .method("POST")
        .header("authorization", &test_account.authorization)
        .json(&CreateRequest {
            name: None,
            workspace_id: test_account.workspace_id.clone(),
            organization_id: test_account.organization_id.clone(),
        })
        .path(format!("/changeSets/{}/editSessions", change_set_id).as_ref())
        .reply(&filter)
        .await;
    assert_eq!(res.status(), 200, "edit session is created");

    test_cleanup(test_account)
        .await
        .expect("failed to finish test");
}
