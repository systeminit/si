use crate::models::billing_account::signup_new_billing_account;
use crate::models::change_set::create_change_set;
use crate::models::edit_session::create_edit_session;
use crate::models::entity::create_entity;
use crate::models::ops::create_op_set_name;
use crate::models::secret::create_secret;
use crate::models::system::create_system;
use crate::{one_time_setup, TestContext};

use si_sdf::models::{update, Resource, UpdateClock};

#[tokio::test]
async fn load_data_model() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");
    let nba = signup_new_billing_account(&txn, &nats).await;
    txn.commit()
        .await
        .expect("failed to commit the new billing account");

    let txn = conn.transaction().await.expect("cannot create txn");
    let change_set = create_change_set(&txn, &nats, &nba).await;
    let edit_session = create_edit_session(&txn, &nats, &nba, &change_set).await;
    txn.commit()
        .await
        .expect("failed to commit the new change set");
    let txn = conn.transaction().await.expect("cannot create txn");
    let system = create_system(
        &pg,
        &txn,
        &nats,
        &veritech,
        &nba,
        &change_set,
        &edit_session,
    )
    .await;
    let entity = create_entity(
        &pg,
        &txn,
        &nats,
        &veritech,
        &nba,
        &change_set,
        &edit_session,
        &system,
    )
    .await;
    let resource = Resource::get_any_by_entity_id(&txn, &entity.id, &system.id, &change_set.id)
        .await
        .expect("failed to get the resource");
    let secret = create_secret(&txn, &nats, &nba).await;
    let op = create_op_set_name(
        &txn,
        &nats,
        &nba,
        &change_set,
        &edit_session,
        &entity,
        "spirit crusher",
    )
    .await;
    txn.commit()
        .await
        .expect("failed to commit the new change set");

    let rows = update::load_data_model(
        &pg,
        &nba.workspace.id,
        &nba.billing_account.id,
        &UpdateClock {
            epoch: 0,
            update_count: 0,
        },
    )
    .await
    .expect("cannot load data model");
    assert_eq!(
        rows.iter()
            .any(|j| j["id"].as_str().expect("cannot get json id") == &change_set.id),
        true,
    );
    assert_eq!(
        rows.iter()
            .any(|j| j["id"].as_str().expect("cannot get json id") == &edit_session.id),
        true,
    );
    assert_eq!(
        rows.iter()
            .any(|j| j["id"].as_str().expect("cannot get json id") == &system.id),
        true,
    );
    assert_eq!(
        rows.iter()
            .any(|j| j["id"].as_str().expect("cannot get json id") == &system.node_id),
        true,
    );
    assert_eq!(
        rows.iter()
            .any(|j| j["id"].as_str().expect("cannot get json id") == &entity.id),
        true,
    );
    assert_eq!(
        rows.iter()
            .any(|j| j["id"].as_str().expect("cannot get json id") == &entity.node_id),
        true,
    );
    assert_eq!(
        rows.iter()
            .any(|j| j["id"].as_str().expect("cannot get json id") == &secret.id),
        true,
    );
    assert_eq!(
        rows.iter()
            .any(|j| j["id"].as_str().expect("cannot get json id") == &op.id),
        true,
    );
    assert_eq!(
        rows.iter()
            .any(|j| j["id"].as_str().expect("cannot get json id") == &resource.id),
        true,
    );
    assert_eq!(
        rows.iter()
            .any(|j| j["id"].as_str().expect("cannot get json id") == &nba.billing_account.id),
        true,
    );
    assert_eq!(
        rows.iter()
            .any(|j| j["id"].as_str().expect("cannot get json id") == &nba.public_key.id),
        true,
    );
}
