use crate::models::billing_account::{signup_new_billing_account, NewBillingAccount};
use crate::models::edit_session::create_edit_session;
use crate::models::entity::create_entity;
use crate::models::ops::{
    create_op_entity_action, create_op_entity_delete, create_op_entity_set, create_op_set_name,
};
use crate::models::system::create_system;
use crate::{one_time_setup, TestContext};

use names::{Generator, Name};

use si_sdf::data::{NatsTxn, PgTxn};
use si_sdf::models::{ChangeSet, ChangeSetParticipant, ChangeSetStatus, Entity};

pub async fn create_change_set(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    nba: &NewBillingAccount,
) -> ChangeSet {
    ChangeSet::new(&txn, &nats, None, nba.workspace.id.clone())
        .await
        .expect("cannot create change_set")
}

#[tokio::test]
async fn new() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, _veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let nba = signup_new_billing_account(&txn, &nats).await;
    txn.commit()
        .await
        .expect("failed to commit the new billing account");

    let txn = conn.transaction().await.expect("cannot create txn");

    let change_set = ChangeSet::new(
        &txn,
        &nats,
        Some("poopy mcpants".to_string()),
        nba.workspace.id.clone(),
    )
    .await
    .expect("cannot create change_set");

    assert_eq!(&change_set.name, "poopy mcpants");
    assert_eq!(&change_set.status, &ChangeSetStatus::Open);
}

#[tokio::test]
async fn get() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, _veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let nba = signup_new_billing_account(&txn, &nats).await;
    txn.commit()
        .await
        .expect("failed to commit the new billing account");

    let txn = conn.transaction().await.expect("cannot create txn");

    let og_change_set = ChangeSet::new(
        &txn,
        &nats,
        Some("poopy mcpants".to_string()),
        nba.workspace.id.clone(),
    )
    .await
    .expect("cannot create change_set");

    let change_set = ChangeSet::get(&txn, &og_change_set.id)
        .await
        .expect("cannot get change set");
    assert_eq!(&og_change_set, &change_set);
}

#[tokio::test]
async fn list() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, _veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let nba = signup_new_billing_account(&txn, &nats).await;
    txn.commit()
        .await
        .expect("failed to commit the new billing account");

    let txn = conn.transaction().await.expect("cannot create txn");

    let _primary_change_set = ChangeSet::new(
        &txn,
        &nats,
        Some("poopy mcpants".to_string()),
        nba.workspace.id.clone(),
    )
    .await
    .expect("cannot create change_set");

    let _secondary_change_set = ChangeSet::new(
        &txn,
        &nats,
        Some("poopy mcbain".to_string()),
        nba.workspace.id.clone(),
    )
    .await
    .expect("cannot create change_set");

    let _tertiary_change_set = ChangeSet::new(
        &txn,
        &nats,
        Some("poopy explicitmonkey".to_string()),
        nba.workspace.id.clone(),
    )
    .await
    .expect("cannot create change_set");

    let reply = ChangeSet::list(&txn, &nba.billing_account.id, None, None, None, None, None)
        .await
        .expect("cannot list change sets");
    assert_eq!(reply.items.len(), 3);
}

#[tokio::test]
async fn execute() {
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
    let first_entity = create_entity(
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
    let second_entity = create_entity(
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
    let third_entity = create_entity(
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

    create_op_set_name(
        &txn,
        &nats,
        &nba,
        &change_set,
        &edit_session,
        &first_entity,
        "chvrches",
    )
    .await;

    create_op_entity_set(
        &txn,
        &nats,
        &nba,
        &change_set,
        &edit_session,
        &second_entity,
        vec![String::from("mother")],
        serde_json::json!["we share"],
        None,
    )
    .await;

    create_op_entity_action(
        &txn,
        &nats,
        &nba,
        &change_set,
        &edit_session,
        &first_entity,
        "deploy",
        &system,
    )
    .await;

    create_op_entity_delete(&txn, &nats, &nba, &change_set, &edit_session, &third_entity).await;

    let impacted_objects = change_set
        .execute(&pg, &txn, &nats_conn, &nats, &veritech, true, None)
        .await
        .expect("cannot execute change set");
    assert_eq!(impacted_objects.iter().any(|id| id == &system.id), true);
    assert_eq!(
        impacted_objects.iter().any(|id| id == &first_entity.id),
        true
    );
    assert_eq!(
        impacted_objects.iter().any(|id| id == &second_entity.id),
        true
    );
    assert_eq!(
        impacted_objects.iter().any(|id| id == &third_entity.id),
        true
    );

    let first_entity_changed = Entity::get_projection(&txn, &first_entity.id, &change_set.id)
        .await
        .expect("cannot get projected first entity");
    assert_eq!(&first_entity_changed.name, "chvrches");

    let second_entity_changed = Entity::get_projection(&txn, &second_entity.id, &change_set.id)
        .await
        .expect("cannot get projected second entity");
    assert_eq!(
        second_entity_changed
            .manual_properties
            .get_property("/mother", None)
            .expect("cannot get property of second entity")
            .expect("no value found for mother"),
        &serde_json::json!["we share"]
    );

    let third_entity_changed = Entity::get_projection(&txn, &third_entity.id, &change_set.id)
        .await
        .expect("cannot get projected third entity");
    assert_eq!(third_entity_changed.si_storable.deleted, true);

    // do the hypotehtical twice
    let impacted_objects = change_set
        .execute(&pg, &txn, &nats_conn, &nats, &veritech, true, None)
        .await
        .expect("cannot execute change set");
    assert_eq!(impacted_objects.iter().any(|id| id == &system.id), true);
    assert_eq!(
        impacted_objects.iter().any(|id| id == &first_entity.id),
        true
    );
    assert_eq!(
        impacted_objects.iter().any(|id| id == &second_entity.id),
        true
    );
    assert_eq!(
        impacted_objects.iter().any(|id| id == &third_entity.id),
        true
    );

    let first_entity_changed = Entity::get_projection(&txn, &first_entity.id, &change_set.id)
        .await
        .expect("cannot get projected first entity");
    assert_eq!(&first_entity_changed.name, "chvrches");

    let second_entity_changed = Entity::get_projection(&txn, &second_entity.id, &change_set.id)
        .await
        .expect("cannot get projected second entity");
    assert_eq!(
        second_entity_changed
            .manual_properties
            .get_property("/mother", None)
            .expect("cannot get property of second entity")
            .expect("no value found for mother"),
        &serde_json::json!["we share"]
    );

    let third_entity_changed = Entity::get_projection(&txn, &third_entity.id, &change_set.id)
        .await
        .expect("cannot get projected third entity");
    assert_eq!(third_entity_changed.si_storable.deleted, true);

    // Now save the non-hypothetical objects!
    let impacted_objects = change_set
        .execute(&pg, &txn, &nats_conn, &nats, &veritech, false, None)
        .await
        .expect("cannot execute change set");
    assert_eq!(impacted_objects.iter().any(|id| id == &system.id), true);
    assert_eq!(
        impacted_objects.iter().any(|id| id == &first_entity.id),
        true
    );
    assert_eq!(
        impacted_objects.iter().any(|id| id == &second_entity.id),
        true
    );
    assert_eq!(
        impacted_objects.iter().any(|id| id == &third_entity.id),
        true
    );

    let first_entity_changed = Entity::get_head(&txn, &first_entity.id)
        .await
        .expect("cannot get projected first entity");
    assert_eq!(&first_entity_changed.name, "chvrches");

    let second_entity_changed = Entity::get_head(&txn, &second_entity.id)
        .await
        .expect("cannot get projected second entity");
    assert_eq!(
        second_entity_changed
            .manual_properties
            .get_property("/mother", None)
            .expect("cannot get property of second entity")
            .expect("no value found for mother"),
        &serde_json::json!["we share"]
    );

    let third_entity_changed = Entity::get_head(&txn, &third_entity.id)
        .await
        .expect("cannot get projected third entity");
    assert_eq!(third_entity_changed.si_storable.deleted, true);
}

#[tokio::test]
async fn change_set_participant_new() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, _veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let nba = signup_new_billing_account(&txn, &nats).await;
    txn.commit()
        .await
        .expect("failed to commit the new billing account");

    let txn = conn.transaction().await.expect("cannot create txn");

    let change_set = create_change_set(&txn, &nats, &nba).await;
    let fake_object_id = format!(
        "entity:{}",
        Generator::with_naming(Name::Numbered).next().unwrap()
    );

    let csp = ChangeSetParticipant::new(
        &txn,
        &nats,
        &change_set.id,
        &fake_object_id,
        &nba.workspace.id,
    )
    .await
    .expect("cannot create new change set participant");

    assert_eq!(&csp.change_set_id, &change_set.id);
    assert_eq!(&csp.object_id, &fake_object_id);
}

#[tokio::test]
async fn change_set_participant_exists() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, _veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let nba = signup_new_billing_account(&txn, &nats).await;
    txn.commit()
        .await
        .expect("failed to commit the new billing account");

    let txn = conn.transaction().await.expect("cannot create txn");

    let change_set = create_change_set(&txn, &nats, &nba).await;
    let fake_object_id = format!(
        "entity:{}",
        Generator::with_naming(Name::Numbered).next().unwrap()
    );

    let _csp = ChangeSetParticipant::new(
        &txn,
        &nats,
        &change_set.id,
        &fake_object_id,
        &nba.workspace.id,
    )
    .await
    .expect("cannot create new change set participant");

    let exists = ChangeSetParticipant::exists(&txn, &change_set.id, &fake_object_id)
        .await
        .expect("cannot check if change set participant already exists");
    assert_eq!(exists, true, "change set participant should exist");

    let not_exists = ChangeSetParticipant::exists(&txn, &change_set.id, "poopy mcpants canoe")
        .await
        .expect("cannot check if change set participant already exists");
    assert_eq!(not_exists, false, "change set participant should not exist");
}

#[tokio::test]
async fn change_set_participant_new_if_not_exists() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, _veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let nba = signup_new_billing_account(&txn, &nats).await;
    txn.commit()
        .await
        .expect("failed to commit the new billing account");

    let txn = conn.transaction().await.expect("cannot create txn");

    let change_set = create_change_set(&txn, &nats, &nba).await;
    let fake_object_id = format!(
        "entity:{}",
        Generator::with_naming(Name::Numbered).next().unwrap()
    );

    let csp = ChangeSetParticipant::new_if_not_exists(
        &txn,
        &nats,
        &change_set.id,
        &fake_object_id,
        &nba.workspace.id,
    )
    .await
    .expect("cannot create new change set participant");
    assert_eq!(csp.is_some(), true, "created change set participant");

    let not_today_satan_csp = ChangeSetParticipant::new_if_not_exists(
        &txn,
        &nats,
        &change_set.id,
        &fake_object_id,
        &nba.workspace.id,
    )
    .await
    .expect("cannot create new change set participant");
    assert_eq!(
        not_today_satan_csp.is_none(),
        true,
        "created change set participant when we shouldn't have"
    );
}

#[tokio::test]
async fn change_set_participant_list() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, _veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let nba = signup_new_billing_account(&txn, &nats).await;
    txn.commit()
        .await
        .expect("failed to commit the new billing account");

    let txn = conn.transaction().await.expect("cannot create txn");

    let change_set = create_change_set(&txn, &nats, &nba).await;
    let fake_object_id = format!(
        "entity:{}",
        Generator::with_naming(Name::Numbered).next().unwrap()
    );

    let _primary_csp = ChangeSetParticipant::new(
        &txn,
        &nats,
        &change_set.id,
        &fake_object_id,
        &nba.workspace.id,
    )
    .await
    .expect("cannot create new change set participant");

    let fake_object_id = format!(
        "entity:{}",
        Generator::with_naming(Name::Numbered).next().unwrap()
    );

    let _secondary_csp = ChangeSetParticipant::new(
        &txn,
        &nats,
        &change_set.id,
        &fake_object_id,
        &nba.workspace.id,
    )
    .await
    .expect("cannot create new change set participant");

    let reply =
        ChangeSetParticipant::list(&txn, &nba.billing_account.id, None, None, None, None, None)
            .await
            .expect("cannot list change set participants");
    assert_eq!(reply.items.len(), 2);
}
