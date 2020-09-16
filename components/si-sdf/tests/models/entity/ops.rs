use crate::{
    filters::change_sets::create_change_set, filters::edit_sessions::create_edit_session,
    filters::nodes::create_node, test_cleanup, test_setup, DB,
};

use si_sdf::models::entity::ops::OpSetString;
use si_sdf::models::Entity;

#[tokio::test]
async fn op_set_string() {
    let test_account = test_setup().await.expect("failed to setup test");

    let change_set_id = create_change_set(&test_account).await;
    let edit_session_id = create_edit_session(&test_account, &change_set_id).await;
    let node_reply = create_node(&test_account, &change_set_id, &edit_session_id, "service").await;
    let node = node_reply.item;
    let mut head_entity: Entity = node
        .get_head_object(&DB)
        .await
        .expect("cannot get head object for node");

    // Test the creation of a new top level key
    let op_set_string = OpSetString::new(
        &DB,
        &head_entity.id,
        "/slipknot",
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

    op_set_string
        .apply(&mut head_entity)
        .expect("cannot apply op set string");

    tracing::debug!(?op_set_string, ?head_entity, "slipknot?");

    let baseline = head_entity
        .manual_properties
        .get("__baseline")
        .expect("cannot find system or baseline to set value");
    let value = baseline
        .get("slipknot")
        .expect("cannot find slipknot, should be there");
    assert_eq!(value, "heretic", "cannot find value for op");

    // Test that we can change an existing key
    let op_set_string = OpSetString::new(
        &DB,
        &head_entity.id,
        "/slipknot",
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

    op_set_string
        .apply(&mut head_entity)
        .expect("cannot apply second op set string");

    let baseline = head_entity
        .manual_properties
        .get("__baseline")
        .expect("cannot find system or baseline to set value");
    let value = baseline
        .get("slipknot")
        .expect("cannot find slipknot, should be there");
    assert_eq!(value, "sic", "cannot find value for op");

    test_cleanup(test_account)
        .await
        .expect("failed to finish test");
}
