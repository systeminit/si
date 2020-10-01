use crate::filters::change_sets::{create_change_set, execute_change_set};
use crate::filters::edit_sessions::create_edit_session;
use crate::{test_cleanup, test_setup, TestAccount};
use crate::{DB, NATS, SETTINGS};
use si_sdf::filters::api;
use si_sdf::models::{entity, node, ops, Entity, Node};

pub async fn create_system(test_account: &TestAccount) -> Vec<String> {
    let change_set_id = create_change_set(test_account).await;
    let edit_session_id = create_edit_session(test_account, &change_set_id).await;
    let response = create_node(test_account, &change_set_id, &edit_session_id, "system").await;

    execute_change_set(test_account, &change_set_id).await;

    let node = response.item;
    let system_id = node
        .get_object_id(&DB)
        .await
        .expect("cannot get newly created system id");
    vec![system_id]
}

pub async fn create_node(
    test_account: &TestAccount,
    change_set_id: impl AsRef<str>,
    edit_session_id: impl AsRef<str>,
    object_type: impl Into<String>,
) -> node::CreateReply {
    let change_set_id = change_set_id.as_ref();
    let edit_session_id = edit_session_id.as_ref();
    let object_type = object_type.into();
    let filter = api(&DB, &NATS, &SETTINGS.jwt_encrypt.key);
    let kind = if object_type == "system" {
        node::NodeKind::System
    } else {
        node::NodeKind::Entity
    };
    let res = warp::test::request()
        .method("POST")
        .header("authorization", &test_account.authorization)
        .json(&node::CreateRequest {
            name: None,
            kind,
            object_type,
            organization_id: test_account.organization_id.clone(),
            workspace_id: test_account.workspace_id.clone(),
            change_set_id: change_set_id.into(),
            edit_session_id: edit_session_id.into(),
            system_ids: test_account.system_ids.clone(),
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

    let filter = api(&DB, &NATS, &SETTINGS.jwt_encrypt.key);
    let change_set_id = create_change_set(&test_account).await;
    let edit_session_id = create_edit_session(&test_account, &change_set_id).await;

    let res = warp::test::request()
        .method("POST")
        .header("authorization", &test_account.authorization)
        .json(&node::CreateRequest {
            name: None,
            kind: node::NodeKind::Entity,
            object_type: "service".into(),
            organization_id: test_account.organization_id.clone(),
            workspace_id: test_account.workspace_id.clone(),
            change_set_id,
            edit_session_id,
            system_ids: test_account.system_ids.clone(),
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

    let filter = api(&DB, &NATS, &SETTINGS.jwt_encrypt.key);
    let change_set_id = create_change_set(&test_account).await;
    let edit_session_id = create_edit_session(&test_account, &change_set_id).await;
    let node_reply = create_node(&test_account, &change_set_id, &edit_session_id, "service").await;

    let res = warp::test::request()
        .method("GET")
        .header("authorization", &test_account.authorization)
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

#[tokio::test]
async fn get_object() {
    let test_account = test_setup().await.expect("failed to setup test");

    let filter = api(&DB, &NATS, &SETTINGS.jwt_encrypt.key);
    let change_set_id = create_change_set(&test_account).await;
    let edit_session_id = create_edit_session(&test_account, &change_set_id).await;

    let node_reply = create_node(&test_account, &change_set_id, &edit_session_id, "service").await;
    execute_change_set(&test_account, &change_set_id).await;

    let res = warp::test::request()
        .method("GET")
        .header("authorization", &test_account.authorization)
        .path(format!("/nodes/{}/object", &node_reply.item.id).as_ref())
        .reply(&filter)
        .await;

    let get_reply: si_sdf::models::GetReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize get reply");

    let entity: Entity =
        serde_json::from_value(get_reply.item).expect("cannot extract object from reply");

    assert_eq!(
        entity.object_type, "service",
        "fetched object must be the same object type"
    );

    test_cleanup(test_account)
        .await
        .expect("failed to finish test");
}

#[tokio::test]
async fn patch_object() {
    let test_account = test_setup().await.expect("failed to setup test");

    let filter = api(&DB, &NATS, &SETTINGS.jwt_encrypt.key);
    let change_set_id = create_change_set(&test_account).await;
    let edit_session_id = create_edit_session(&test_account, &change_set_id).await;
    let node_reply = create_node(&test_account, &change_set_id, &edit_session_id, "service").await;
    let node = node_reply.item;
    let entity: entity::Entity = node
        .get_object_projection(&DB, &change_set_id)
        .await
        .expect("cannot get head object for node");

    let request = node::ObjectPatchRequest {
        op: ops::OpRequest::EntitySet(ops::OpEntitySetRequest {
            path: vec!["strahd".into()],
            value: "von zarovich".into(),
            override_system: None,
        }),
        organization_id: test_account.organization_id.clone(),
        workspace_id: test_account.workspace_id.clone(),
        change_set_id: change_set_id.clone(),
        edit_session_id: edit_session_id.clone(),
    };

    let res = warp::test::request()
        .method("PATCH")
        .header("authorization", &test_account.authorization)
        .json(&request)
        .path(format!("/nodes/{}/object", &node.id).as_ref())
        .reply(&filter)
        .await;
    let wtf = String::from_utf8(res.body().to_vec()).expect("cannot amke string fromb ody");
    tracing::error!(?wtf, "wtf");
    let reply: node::ObjectPatchReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize reply");

    let op_reply = match reply {
        node::ObjectPatchReply::Op(op_reply) => op_reply,
    };
    assert_eq!(
        op_reply.item_ids,
        vec![entity.id],
        "expect the ids of all impacted objects back",
    );

    let updated_entity: entity::Entity = node
        .get_object_projection(&DB, &change_set_id)
        .await
        .expect("cannot get updated head object for node");

    let entity_strahd = entity
        .manual_properties
        .get_property("/strahd", None)
        .expect("invalid override system");
    assert_eq!(entity_strahd, None, "old entity has no value");

    let updated_entity_strahd = updated_entity
        .manual_properties
        .get_property("/strahd", None)
        .expect("invalid override system");

    assert_eq!(
        updated_entity_strahd,
        Some(&serde_json::json!["von zarovich"]),
        "new entity has correct value"
    );

    test_cleanup(test_account)
        .await
        .expect("failed to finish test");
}
