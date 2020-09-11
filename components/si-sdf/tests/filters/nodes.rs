use crate::filters::change_sets::create_change_set;
use crate::filters::edit_sessions::create_edit_session;
use crate::DB;
use crate::{test_cleanup, test_setup, TestAccount};
use si_sdf::filters::api;
use si_sdf::models::{node, Node};

pub async fn create_node(
    test_account: &TestAccount,
    change_set_id: impl AsRef<str>,
    edit_session_id: impl AsRef<str>,
    object_type: impl Into<String>,
) -> node::CreateReply {
    let change_set_id = change_set_id.as_ref();
    let edit_session_id = edit_session_id.as_ref();
    let object_type = object_type.into();
    let filter = api(&DB);
    let res = warp::test::request()
        .method("POST")
        .header("userId", &test_account.user_id)
        .header("billingAccountId", &test_account.billing_account_id)
        .header("organizationId", &test_account.organization_id)
        .header("workspaceId", &test_account.workspace_id)
        .header("changeSetId", change_set_id)
        .header("editSessionId", edit_session_id)
        .json(&node::CreateRequest {
            name: None,
            kind: node::NodeKind::Entity,
            object_type,
        })
        .path("/nodes")
        .reply(&filter)
        .await;
    assert_eq!(res.status(), 200, "create failed");
    let node_reply: node::CreateReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize node reply");
    return node_reply;
}

#[tokio::test]
async fn create() {
    let test_account = test_setup().await.expect("failed to setup test");

    let filter = api(&DB);
    let change_set_id = create_change_set(&test_account).await;
    let edit_session_id = create_edit_session(&test_account, &change_set_id).await;

    let res = warp::test::request()
        .method("POST")
        .header("userId", &test_account.user_id)
        .header("billingAccountId", &test_account.billing_account_id)
        .header("organizationId", &test_account.organization_id)
        .header("workspaceId", &test_account.workspace_id)
        .header("changeSetId", &change_set_id)
        .header("editSessionId", &edit_session_id)
        .json(&node::CreateRequest {
            name: None,
            kind: node::NodeKind::Entity,
            object_type: "service".into(),
        })
        .path("/nodes")
        .reply(&filter)
        .await;
    assert_eq!(res.status(), 200, "create should succeed");
    let _node_reply: node::CreateReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize node reply");

    test_cleanup(test_account)
        .await
        .expect("failed to finish test");
}

#[tokio::test]
async fn get() {
    let test_account = test_setup().await.expect("failed to setup test");

    let filter = api(&DB);
    let change_set_id = create_change_set(&test_account).await;
    let edit_session_id = create_edit_session(&test_account, &change_set_id).await;
    let node_reply = create_node(&test_account, &change_set_id, &edit_session_id, "service").await;

    let res = warp::test::request()
        .method("GET")
        .header("userId", &test_account.user_id)
        .header("billingAccountId", &test_account.billing_account_id)
        .header("organizationId", &test_account.organization_id)
        .header("workspaceId", &test_account.workspace_id)
        .path(format!("/nodes/{}", &node_reply.item.id).as_ref())
        .reply(&filter)
        .await;
    let get_reply: si_sdf::models::GetReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize get reply");
    let node: Node =
        serde_json::from_value(get_reply.item).expect("cannot extract object from reply");

    assert_eq!(
        node, node_reply.item,
        "created and fetched nodes must match"
    );

    test_cleanup(test_account)
        .await
        .expect("failed to finish test");
}
