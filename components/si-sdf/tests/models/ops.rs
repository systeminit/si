use crate::{
    filters::change_sets::create_change_set, filters::edit_sessions::create_edit_session,
    filters::nodes::create_node, test_cleanup, test_setup, DB, NATS,
};

use si_sdf::models::ops::OpEntitySetString;
use si_sdf::models::Entity;

#[tokio::test]
async fn op_set_string() {
    let test_account = test_setup().await.expect("failed to setup test");

    let change_set_id = create_change_set(&test_account).await;
    let edit_session_id = create_edit_session(&test_account, &change_set_id).await;
    let node_reply = create_node(&test_account, &change_set_id, &edit_session_id, "service").await;
    let node = node_reply.item;
    let head_entity: Entity = node
        .get_object_projection(&DB, &change_set_id)
        .await
        .expect("cannot get object projection for node");

    // Test the creation of a new top level key
    let op_set_string = OpEntitySetString::new(
        &DB,
        &NATS,
        &head_entity.id,
        "slipknot",
        "heretic",
        None,
        test_account.billing_account_id.clone(),
        test_account.organization_id.clone(),
        test_account.workspace_id.clone(),
        change_set_id.clone(),
        edit_session_id.clone(),
        test_account.user_id.clone(),
    )
    .await
    .expect("cannot create op set string");

    let mut entity_json = serde_json::json![head_entity];

    op_set_string
        .apply(&mut entity_json)
        .await
        .expect("cannot apply op set string");

    tracing::debug!(?op_set_string, ?entity_json, "slipknot?");

    let baseline = entity_json["manualProperties"]["__baseline"]
        .as_object()
        .expect("baseline is not an object");
    let value = baseline
        .get("slipknot")
        .expect("cannot find slipknot, should be there");
    assert_eq!(
        value,
        &serde_json::json!["heretic"],
        "cannot find value for op"
    );

    // Test that we can change an existing key
    let op_set_string = OpEntitySetString::new(
        &DB,
        &NATS,
        &head_entity.id,
        "slipknot",
        "sic",
        None,
        test_account.billing_account_id.clone(),
        test_account.organization_id.clone(),
        test_account.workspace_id.clone(),
        change_set_id.clone(),
        edit_session_id.clone(),
        test_account.user_id.clone(),
    )
    .await
    .expect("cannot create second op set string");

    let mut entity_json = serde_json::json![head_entity];

    op_set_string
        .apply(&mut entity_json)
        .await
        .expect("cannot apply second op set string");

    let baseline = entity_json["manualProperties"]["__baseline"]
        .as_object()
        .expect("baseline is not an object");
    let value = baseline
        .get("slipknot")
        .expect("cannot find slipknot, should be there");
    assert_eq!(value, &serde_json::json!["sic"], "cannot find value for op");

    test_cleanup(test_account)
        .await
        .expect("failed to finish test");
}
