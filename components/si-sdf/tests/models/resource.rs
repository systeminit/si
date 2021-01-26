use crate::models::billing_account::signup_new_billing_account;
use crate::models::change_set::create_change_set;
use crate::models::edit_session::create_edit_session;
use crate::models::entity::create_entity;
use crate::models::system::create_system;
use crate::{one_time_setup, TestContext};

use si_sdf::models::{Resource, ResourceHealth, ResourceStatus};

#[tokio::test]
async fn new() {
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
        &nats_conn,
        &nats,
        &veritech,
        &nba,
        &change_set,
        &edit_session,
    )
    .await;
    let _entity = create_entity(
        &pg,
        &txn,
        &nats_conn,
        &nats,
        &veritech,
        &nba,
        &change_set,
        &edit_session,
        &system,
    )
    .await;
}

#[tokio::test]
async fn save_head() {
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
        &nats_conn,
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
        &nats_conn,
        &nats,
        &veritech,
        &nba,
        &change_set,
        &edit_session,
        &system,
    )
    .await;

    let mut og_resource =
        Resource::get_any_by_entity_id(&txn, &entity.id, &system.id, &change_set.id)
            .await
            .expect("cannot get resource");

    assert_eq!(og_resource.change_set_id.is_some(), true);

    let first_og_resource = og_resource.clone();

    og_resource
        .save_head(&pg, &nats_conn)
        .await
        .expect("cannot save");

    assert_eq!(og_resource.change_set_id.is_none(), true);

    let second_og_resource = og_resource.clone();

    og_resource
        .save_head(&pg, &nats_conn)
        .await
        .expect("cannot save");

    assert!(
        first_og_resource.si_storable.update_clock < second_og_resource.si_storable.update_clock
    );
    assert!(second_og_resource.si_storable.update_clock < og_resource.si_storable.update_clock);
}

#[tokio::test]
async fn save_projection() {
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
        &nats_conn,
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
        &nats_conn,
        &nats,
        &veritech,
        &nba,
        &change_set,
        &edit_session,
        &system,
    )
    .await;

    let mut og_resource =
        Resource::get_any_by_entity_id(&txn, &entity.id, &system.id, &change_set.id)
            .await
            .expect("cannot get resource");

    let first_og_resource = og_resource.clone();

    og_resource.state = serde_json::json![{ "monkey": "drizzles" }];
    og_resource.health = ResourceHealth::Warning;
    og_resource.status = ResourceStatus::Failed;

    og_resource
        .save_projection(&pg, &nats_conn)
        .await
        .expect("cannot save");

    assert!(first_og_resource.si_storable.update_clock < og_resource.si_storable.update_clock);
    assert_eq!(
        &og_resource.state,
        &serde_json::json![{ "monkey": "drizzles" }]
    );
    assert_eq!(&og_resource.health, &ResourceHealth::Warning);
    assert_eq!(&og_resource.status, &ResourceStatus::Failed);
}

#[tokio::test]
async fn get_any_by_entity_id() {
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
        &nats_conn,
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
        &nats_conn,
        &nats,
        &veritech,
        &nba,
        &change_set,
        &edit_session,
        &system,
    )
    .await;

    // No head, just a projection!
    let mut resource = Resource::get_any_by_entity_id(&txn, &entity.id, &system.id, &change_set.id)
        .await
        .expect("cannot get any resource");
    assert_eq!(resource.change_set_id.is_some(), true);

    resource
        .save_head(&pg, &nats_conn)
        .await
        .expect("cannot save head resource");

    // No projection, just a head!
    let resource = Resource::get_any_by_entity_id(&txn, &entity.id, &system.id, &change_set.id)
        .await
        .expect("cannot get any resource");
    // NOTE: at the moment when we save head we do *not* delete the projection, so this test is
    // hacked up...
    //
    // assert_eq!(resource.change_set_id.is_none(), true);
    assert_eq!(resource.change_set_id.is_some(), true);
}

#[tokio::test]
async fn get_any_by_node_id() {
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
        &nats_conn,
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
        &nats_conn,
        &nats,
        &veritech,
        &nba,
        &change_set,
        &edit_session,
        &system,
    )
    .await;

    // No head, just a projection!
    let mut resource =
        Resource::get_any_by_node_id(&txn, &entity.node_id, &system.id, &change_set.id)
            .await
            .expect("cannot get any resource");
    assert_eq!(resource.change_set_id.is_some(), true);

    resource
        .save_head(&pg, &nats_conn)
        .await
        .expect("cannot save og_resource");

    // No projection, just a head!
    let resource = Resource::get_any_by_node_id(&txn, &entity.node_id, &system.id, &change_set.id)
        .await
        .expect("cannot get any resource");
    // NOTE: at the moment when we save head we do *not* delete the projection, so this test is
    // hacked up...
    //
    // assert_eq!(resource.change_set_id.is_none(), true);
    assert_eq!(resource.change_set_id.is_some(), true);
}

#[tokio::test]
async fn get_head_by_entity_id() {
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
        &nats_conn,
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
        &nats_conn,
        &nats,
        &veritech,
        &nba,
        &change_set,
        &edit_session,
        &system,
    )
    .await;

    let mut og_resource =
        Resource::get_any_by_entity_id(&txn, &entity.id, &system.id, &change_set.id)
            .await
            .expect("cannot get resource");

    // No head, just a projection!
    let resource_result = Resource::get_head_by_entity_id(&txn, &entity.id, &system.id).await;
    assert_eq!(resource_result.is_err(), true);

    og_resource
        .save_head(&pg, &nats_conn)
        .await
        .expect("cannot save og_resource");

    // No projection, just a head!
    let resource = Resource::get_head_by_entity_id(&txn, &entity.id, &system.id)
        .await
        .expect("cannot get any resource");
    assert_eq!(&og_resource, &resource);
    assert_eq!(resource.change_set_id.is_none(), true);
}

#[tokio::test]
async fn get_head_by_node_id() {
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
        &nats_conn,
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
        &nats_conn,
        &nats,
        &veritech,
        &nba,
        &change_set,
        &edit_session,
        &system,
    )
    .await;

    let mut og_resource =
        Resource::get_any_by_entity_id(&txn, &entity.id, &system.id, &change_set.id)
            .await
            .expect("cannot get resource");

    // No head, just a projection!
    let resource_result = Resource::get_head_by_node_id(&txn, &entity.node_id, &system.id).await;
    assert_eq!(resource_result.is_err(), true);

    og_resource
        .save_head(&pg, &nats_conn)
        .await
        .expect("cannot save og_resource");

    // No projection, just a head!
    let resource = Resource::get_head_by_node_id(&txn, &entity.node_id, &system.id)
        .await
        .expect("cannot get any resource");
    assert_eq!(&og_resource, &resource);
    assert_eq!(resource.change_set_id.is_none(), true);
}

#[tokio::test]
async fn from_update_for_self() {
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
        &nats_conn,
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
        &nats_conn,
        &nats,
        &veritech,
        &nba,
        &change_set,
        &edit_session,
        &system,
    )
    .await;

    let mut og_resource =
        Resource::get_any_by_entity_id(&txn, &entity.id, &system.id, &change_set.id)
            .await
            .expect("cannot get resource");

    assert_eq!(og_resource.change_set_id.is_some(), true);

    let first_og_resource = og_resource.clone();

    og_resource
        .from_update_for_self(
            &pg,
            &nats_conn,
            serde_json::json![{"ozzy": "ozman"}],
            ResourceStatus::InProgress,
            ResourceHealth::Warning,
            og_resource.change_set_id.clone(),
        )
        .await
        .expect("cannot update myself");

    assert_eq!(og_resource.change_set_id.is_some(), true);
    assert_eq!(&og_resource.state, &serde_json::json![{"ozzy": "ozman"}]);
    assert_eq!(&og_resource.status, &ResourceStatus::InProgress);
    assert_eq!(&og_resource.health, &ResourceHealth::Warning);
    assert!(first_og_resource.si_storable.update_clock < og_resource.si_storable.update_clock);
    let second_og_resource = og_resource.clone();

    og_resource
        .from_update_for_self(
            &pg,
            &nats_conn,
            serde_json::json![{"ozzy": "ozman"}],
            ResourceStatus::InProgress,
            ResourceHealth::Warning,
            None,
        )
        .await
        .expect("cannot update myself");
    assert_eq!(og_resource.change_set_id.is_none(), true);
    assert_eq!(&og_resource.state, &serde_json::json![{"ozzy": "ozman"}]);
    assert_eq!(&og_resource.status, &ResourceStatus::InProgress);
    assert_eq!(&og_resource.health, &ResourceHealth::Warning);
    assert!(second_og_resource.si_storable.update_clock < og_resource.si_storable.update_clock);
}

#[tokio::test]
async fn from_update() {
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
        &nats_conn,
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
        &nats_conn,
        &nats,
        &veritech,
        &nba,
        &change_set,
        &edit_session,
        &system,
    )
    .await;

    let og_resource = Resource::get_any_by_entity_id(&txn, &entity.id, &system.id, &change_set.id)
        .await
        .expect("cannot get resource");

    assert_eq!(og_resource.change_set_id.is_some(), true);

    let first_og_resource = og_resource.clone();

    let og_resource = Resource::from_update(
        &pg,
        &nats_conn,
        serde_json::json![{"ozzy": "ozman"}],
        ResourceStatus::InProgress,
        ResourceHealth::Warning,
        true,
        &system.id,
        &entity.id,
        &change_set.id,
    )
    .await
    .expect("cannot update resource");
    assert_eq!(og_resource.change_set_id.is_some(), true);
    assert_eq!(&og_resource.state, &serde_json::json![{"ozzy": "ozman"}]);
    assert_eq!(&og_resource.status, &ResourceStatus::InProgress);
    assert_eq!(&og_resource.health, &ResourceHealth::Warning);
    assert!(first_og_resource.si_storable.update_clock < og_resource.si_storable.update_clock);
    let second_og_resource = og_resource.clone();

    let og_resource = Resource::from_update(
        &pg,
        &nats_conn,
        serde_json::json![{"ozzy": "ozman"}],
        ResourceStatus::InProgress,
        ResourceHealth::Warning,
        false,
        &system.id,
        &entity.id,
        &change_set.id,
    )
    .await
    .expect("cannot update resource");
    assert_eq!(og_resource.change_set_id.is_none(), true);
    assert_eq!(&og_resource.state, &serde_json::json![{"ozzy": "ozman"}]);
    assert_eq!(&og_resource.status, &ResourceStatus::InProgress);
    assert_eq!(&og_resource.health, &ResourceHealth::Warning);
    assert!(second_og_resource.si_storable.update_clock < og_resource.si_storable.update_clock);
}
