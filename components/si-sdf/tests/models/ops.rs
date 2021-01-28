use crate::models::billing_account::{signup_new_billing_account, NewBillingAccount};
use crate::models::change_set::create_change_set;
use crate::models::edit_session::create_edit_session;
use crate::models::entity::create_entity;
use crate::models::system::create_system;

use crate::{one_time_setup, TestContext};

use si_sdf::data::{NatsTxn, PgTxn};
use si_sdf::models::{
    ChangeSet, EditSession, Entity, OpEntityAction, OpEntityDelete, OpEntitySet, OpSetName,
    Resource, ResourceHealth, ResourceStatus, System,
};

pub async fn create_op_set_name(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    nba: &NewBillingAccount,
    change_set: &ChangeSet,
    edit_session: &EditSession,
    entity: &Entity,
    value: impl Into<String>,
) -> OpSetName {
    let value = value.into();
    let op = OpSetName::new(
        &txn,
        &nats,
        &entity.id,
        value,
        &nba.workspace.id,
        &change_set.id,
        &edit_session.id,
    )
    .await
    .expect("cannot create new op set name");
    op
}

pub async fn create_op_entity_set(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    nba: &NewBillingAccount,
    change_set: &ChangeSet,
    edit_session: &EditSession,
    entity: &Entity,
    path: Vec<String>,
    value: impl Into<serde_json::Value>,
    override_system: Option<String>,
) -> OpEntitySet {
    let value = value.into();
    let op = OpEntitySet::new(
        &txn,
        &nats,
        &entity.id,
        path,
        value,
        override_system,
        &nba.workspace.id,
        &change_set.id,
        &edit_session.id,
    )
    .await
    .expect("cannot create new op entity set");
    op
}

pub async fn create_op_entity_action(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    nba: &NewBillingAccount,
    change_set: &ChangeSet,
    edit_session: &EditSession,
    entity: &Entity,
    action: impl Into<String>,
    system: &System,
) -> OpEntityAction {
    let action = action.into();
    let op = OpEntityAction::new(
        &txn,
        &nats,
        &entity.id,
        action,
        &system.id,
        &nba.workspace.id,
        &change_set.id,
        &edit_session.id,
    )
    .await
    .expect("cannot create new op entity action");
    op
}

pub async fn create_op_entity_delete(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    nba: &NewBillingAccount,
    change_set: &ChangeSet,
    edit_session: &EditSession,
    entity: &Entity,
) -> OpEntityDelete {
    let op = OpEntityDelete::new(
        &txn,
        &nats,
        &entity.id,
        &nba.workspace.id,
        &change_set.id,
        &edit_session.id,
    )
    .await
    .expect("cannot create new op entity delete");
    op
}

#[tokio::test]
async fn op_entity_set() {
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

    let op = OpEntitySet::new(
        &txn,
        &nats,
        &entity.id,
        vec![String::from("poop")],
        serde_json::json!["canoe"],
        Some(String::from("__baseline")),
        &nba.workspace.id,
        &change_set.id,
        &edit_session.id,
    )
    .await
    .expect("cannot create op");
    assert_eq!(&op.path, &vec![String::from("poop")]);
    assert_eq!(&op.value, &serde_json::json!["canoe"]);
    assert_eq!(&op.si_op.skip, &false);
    assert_eq!(&op.si_op.override_system, &Some(String::from("__baseline")));
}

#[tokio::test]
async fn op_entity_set_apply() {
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

    let op = OpEntitySet::new(
        &txn,
        &nats,
        &entity.id,
        vec![String::from("poop")],
        serde_json::json!["canoe"],
        Some(String::from("__baseline")),
        &nba.workspace.id,
        &change_set.id,
        &edit_session.id,
    )
    .await
    .expect("cannot create op");

    let mut entity_json = serde_json::to_value(entity.clone()).expect("cannot serialize entity");
    op.apply(&mut entity_json).await.expect("cannot apply op");
    let changed_entity: Entity =
        serde_json::from_value(entity_json).expect("cannot deserizalize entity");
    let changed_prop = changed_entity
        .manual_properties
        .get_property("/poop", Some("__baseline"))
        .expect("cannot get changed property")
        .expect("no value for changed property");
    assert_eq!(changed_prop, &serde_json::json!["canoe"]);
}

#[tokio::test]
async fn op_set_name() {
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

    let op = OpSetName::new(
        &txn,
        &nats,
        &entity.id,
        "anxious",
        &nba.workspace.id,
        &change_set.id,
        &edit_session.id,
    )
    .await
    .expect("cannot create op");
    assert_eq!(&op.value, "anxious");
    assert_eq!(&op.si_op.skip, &false);
}

#[tokio::test]
async fn op_set_name_apply() {
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
    let op = OpSetName::new(
        &txn,
        &nats,
        &entity.id,
        "anxious",
        &nba.workspace.id,
        &change_set.id,
        &edit_session.id,
    )
    .await
    .expect("cannot create op");

    let mut entity_json = serde_json::to_value(entity.clone()).expect("cannot serialize entity");
    op.apply(&mut entity_json).await.expect("cannot apply op");
    let changed_entity: Entity =
        serde_json::from_value(entity_json).expect("cannot deserizalize entity");
    assert_eq!(&changed_entity.name, "anxious");
}

#[tokio::test]
async fn op_entity_action() {
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

    let op = OpEntityAction::new(
        &txn,
        &nats,
        &entity.id,
        "deploy",
        &system.id,
        &nba.workspace.id,
        &change_set.id,
        &edit_session.id,
    )
    .await
    .expect("cannot create op");
    assert_eq!(&op.action, "deploy");
    assert_eq!(&op.system_id, &system.id);
    assert_eq!(&op.si_op.skip, &false);
}

#[tokio::test]
async fn op_entity_action_apply() {
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
    let resource = Resource::get_any_by_entity_id(&txn, &entity.id, &system.id, &change_set.id)
        .await
        .expect("cannot get resource");

    let op = OpEntityAction::new(
        &txn,
        &nats,
        &entity.id,
        "deploy",
        &system.id,
        &nba.workspace.id,
        &change_set.id,
        &edit_session.id,
    )
    .await
    .expect("cannot create op");

    let mut entity_json = serde_json::to_value(entity.clone()).expect("cannot serialize entity");
    op.apply(
        &pg,
        &txn,
        &nats_conn,
        &veritech,
        true,
        &mut entity_json,
        None,
    )
    .await
    .expect("cannot apply op");
    let second_resource =
        Resource::get_any_by_entity_id(&txn, &entity.id, &system.id, &change_set.id)
            .await
            .expect("cannot get resource");
    assert_ne!(resource, second_resource);
    assert_eq!(&second_resource.health, &ResourceHealth::Ok);
    assert_eq!(&second_resource.status, &ResourceStatus::Created);
}

#[tokio::test]
async fn op_entity_delete() {
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

    let op = OpEntityDelete::new(
        &txn,
        &nats,
        &entity.id,
        &nba.workspace.id,
        &change_set.id,
        &edit_session.id,
    )
    .await
    .expect("cannot create op");
    assert_eq!(&op.to_id, &entity.id);
    assert_eq!(&op.si_op.skip, &false);
}

#[tokio::test]
async fn op_entity_delete_apply() {
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

    let op = OpEntityDelete::new(
        &txn,
        &nats,
        &entity.id,
        &nba.workspace.id,
        &change_set.id,
        &edit_session.id,
    )
    .await
    .expect("cannot create op");

    let mut entity_json = serde_json::to_value(entity.clone()).expect("cannot serialize entity");
    op.apply(&mut entity_json).await.expect("cannot apply op");
    let changed_entity: Entity =
        serde_json::from_value(entity_json).expect("cannot create entity from json");
    assert_eq!(&changed_entity.si_storable.deleted, &true);
}
