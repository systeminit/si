use crate::models::billing_account::signup_new_billing_account;
use crate::models::change_set::create_change_set;
use crate::models::edit_session::create_edit_session;
use crate::models::node::create_custom_entity_node;
use crate::models::ops::create_op_entity_set;
use crate::models::system::create_system;

use crate::{one_time_setup, TestContext};

#[tokio::test]
async fn property_set_fails_with_docker_image() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
    txn.commit()
        .await
        .expect("failed to commit the new billing account");

    let txn = conn.transaction().await.expect("cannot create txn");
    let mut change_set = create_change_set(&txn, &nats, &nba).await;
    let edit_session = create_edit_session(&txn, &nats, &nba, &change_set).await;
    txn.commit()
        .await
        .expect("failed to commit the new change set");

    let txn = conn.transaction().await.expect("cannot create txn");
    let system = create_system(
        &pg,
        &txn,
        &nats_conn,
        &nats,
        &veritech,
        &nba,
        &change_set,
        &edit_session,
    )
    .await;
    let docker_image_node = create_custom_entity_node(
        &pg,
        &txn,
        &nats_conn,
        &nats,
        &veritech,
        &nba,
        &system,
        &change_set,
        &edit_session,
        "dockerImage",
    )
    .await;
    let entity = docker_image_node
        .get_projection_object_entity(&txn, &change_set.id)
        .await
        .expect("cannot get entity");
    let _op = create_op_entity_set(
        &txn,
        &nats,
        &nba,
        &change_set,
        &edit_session,
        &entity,
        vec![String::from("image")],
        serde_json::json!["nginx"],
        None,
    )
    .await;
    change_set
        .execute(&pg, &txn, &nats_conn, &nats, &veritech, true, None)
        .await
        .expect("execute should succeed");
    tokio::time::delay_for(tokio::time::Duration::from_secs(30)).await;
}
