use serde_json;

use crate::filters::edit_sessions::create_edit_session;
use crate::filters::nodes::{create_node, node_entity_set};
use crate::{test_cleanup, test_setup, TestAccount};
use crate::{DB, NATS, SETTINGS};

use si_sdf::filters::api;
use si_sdf::models::{change_set, Entity};

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

pub async fn execute_change_set_hypothetical(
    test_account: &TestAccount,
    change_set_id: impl AsRef<str>,
) {
    let filter = api(&DB, &NATS, &SETTINGS.jwt_encrypt.key);
    let change_set_id = change_set_id.as_ref();

    let request = change_set::PatchRequest {
        op: change_set::PatchOps::Execute(change_set::ExecuteRequest { hypothetical: true }),
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

#[test]
fn calculates_properties_of_successors() {
    tokio_test::block_on(Box::pin(async move {
        let test_account = test_setup().await.expect("failed to setup test");

        //let filter = api(&DB, &NATS, &SETTINGS.jwt_encrypt.key);
        let change_set_id = create_change_set(&test_account).await;
        let edit_session_id = create_edit_session(&test_account, &change_set_id).await;

        let service_node_reply =
            create_node(&test_account, &change_set_id, &edit_session_id, "service").await;
        let docker_image_node_reply = create_node(
            &test_account,
            &change_set_id,
            &edit_session_id,
            "dockerImage",
        )
        .await;
        let _docker_image_edge = docker_image_node_reply
            .item
            .configured_by(&DB, &NATS, &service_node_reply.item.id)
            .await
            .expect("failed to create configured by edge for docker image");

        let k8s_deployment_node_reply = create_node(
            &test_account,
            &change_set_id,
            &edit_session_id,
            "kubernetesDeployment",
        )
        .await;
        let _k8s_deployment_edge = k8s_deployment_node_reply
            .item
            .configured_by(&DB, &NATS, &docker_image_node_reply.item.id)
            .await
            .expect("failed to create configured by edge for k8s deployment");

        execute_change_set_hypothetical(&test_account, &change_set_id).await;

        let k8s_deployment_projection: Entity = k8s_deployment_node_reply
            .item
            .get_object_projection(&DB, &change_set_id)
            .await
            .expect("failed to get k8s deployment projection");

        let docker_image_projection: Entity = docker_image_node_reply
            .item
            .get_object_projection(&DB, &change_set_id)
            .await
            .expect("failed to get docker image projection");

        let container_json = k8s_deployment_projection
            .inferred_properties
            .get_property("/kubernetesObject/spec/template/spec/containers/0", None)
            .expect("could not find container for k8s deployment")
            .expect("no value for k8s deployment container image 0");
        let docker_image_projection_image_value = docker_image_projection
            .properties
            .get_property("/image", None)
            .expect("cannot get dockerImage image value")
            .expect("no value for dockerImage image value");
        assert_eq!(
            &container_json["image"],
            docker_image_projection_image_value
        );

        node_entity_set(
            &test_account,
            &change_set_id,
            &edit_session_id,
            &docker_image_node_reply.item.id,
            vec!["image".into()],
            "systeminit/whiskers".into(),
        )
        .await;

        execute_change_set_hypothetical(&test_account, &change_set_id).await;

        let k8s_deployment_projection: Entity = k8s_deployment_node_reply
            .item
            .get_object_projection(&DB, &change_set_id)
            .await
            .expect("failed to get k8s deployment projection");

        let docker_image_projection: Entity = docker_image_node_reply
            .item
            .get_object_projection(&DB, &change_set_id)
            .await
            .expect("failed to get docker image projection");

        let container_json = k8s_deployment_projection
            .inferred_properties
            .get_property("/kubernetesObject/spec/template/spec/containers/0", None)
            .expect("could not find container for k8s deployment")
            .expect("no value for k8s deployment container image 0");
        let docker_image_projection_image_value = docker_image_projection
            .properties
            .get_property("/image", None)
            .expect("cannot get dockerImage image value")
            .expect("no value for dockerImage image value");
        assert_eq!(
            &container_json["image"], docker_image_projection_image_value,
            "failed to compute the image value when it is set manually",
        );

        test_cleanup(test_account)
            .await
            .expect("failed to finish test");
    }));
}
