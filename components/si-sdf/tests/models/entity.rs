use crate::models::billing_account::{signup_new_billing_account, NewBillingAccount};
use crate::models::change_set::create_change_set;
use crate::models::edit_session::create_edit_session;
use crate::models::node::{create_custom_entity_node, create_entity_node};
use crate::models::secret::create_secret_with_message;
use crate::models::system::create_system;
use crate::{one_time_setup, TestContext};

use si_sdf::data::{NatsConn, NatsTxn, PgPool, PgTxn};
use si_sdf::models::{
    ChangeSet, ChangeSetParticipant, Edge, EdgeKind, EditSession, Entity, System,
};
use si_sdf::veritech::Veritech;

pub async fn create_custom_entity(
    pg: &PgPool,
    txn: &PgTxn<'_>,
    nats_conn: &NatsConn,
    nats: &NatsTxn,
    veritech: &Veritech,
    nba: &NewBillingAccount,
    change_set: &ChangeSet,
    edit_session: &EditSession,
    system: &System,
    object_type: impl AsRef<str>,
) -> Entity {
    let object_type = object_type.as_ref();
    let node = create_custom_entity_node(
        &pg,
        &txn,
        &nats_conn,
        &nats,
        &veritech,
        &nba,
        &system,
        &change_set,
        &edit_session,
        object_type,
    )
    .await;
    let entity = node
        .get_projection_object_entity(&txn, &change_set.id)
        .await
        .expect("cannot get object projection");
    entity
}

pub async fn create_entity(
    pool: &PgPool,
    txn: &PgTxn<'_>,
    nats_conn: &NatsConn,
    nats: &NatsTxn,
    veritech: &Veritech,
    nba: &NewBillingAccount,
    change_set: &ChangeSet,
    edit_session: &EditSession,
    system: &System,
) -> Entity {
    let node = create_entity_node(
        &pool,
        &txn,
        &nats_conn,
        &nats,
        &veritech,
        &nba,
        &system,
        &change_set,
        &edit_session,
    )
    .await;
    let entity = node
        .get_projection_object_entity(&txn, &change_set.id)
        .await
        .expect("cannot get object projection");
    entity
}

#[tokio::test]
async fn new() {
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
    assert_eq!(&entity.object_type, "service");
    assert_eq!(
        &entity.expression_properties.get("__baseline"),
        &Some(&serde_json::json![{}])
    );
    assert_eq!(
        &entity.manual_properties.get("__baseline"),
        &Some(&serde_json::json![{}])
    );
    assert_eq!(
        &entity.inferred_properties.get("__baseline"),
        &Some(&serde_json::json![{}])
    );
    assert_eq!(
        &entity.properties.get("__baseline"),
        &Some(&serde_json::json![{}])
    );
    assert_eq!(&entity.head, &false, "entity head should be false");
    assert_eq!(&entity.base, &false, "entity base should be true");

    let csp_exists = ChangeSetParticipant::exists(&txn, &change_set.id, &entity.id)
        .await
        .expect("cannot check if entity is in change set participants");
    assert_eq!(&csp_exists, &true);

    let edges = Edge::direct_predecessor_edges_by_object_id(&txn, &EdgeKind::Includes, &entity.id)
        .await
        .expect("cannot get includes edges for checking system inclusion");

    let has_system_edge = edges.iter().any(|e| &e.tail_vertex.object_id == &system.id);
    assert_eq!(has_system_edge, true);
}

#[tokio::test]
async fn save_projection() {
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
    let mut entity = create_entity(
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

    let og_entity = entity.clone();

    entity.name = String::from("flank steak");
    let manual_prop_baseline = entity
        .manual_properties
        .get_or_create_mut("__baseline")
        .as_object_mut()
        .expect("__baseline is not an object");
    manual_prop_baseline.insert(String::from("black dahlia"), serde_json::json!("murder"));
    entity
        .save_projection(&txn, &nats)
        .await
        .expect("cannot save projection");

    assert_eq!(&entity.name, "flank steak");
    assert_eq!(
        entity.manual_properties.get("__baseline").unwrap(),
        &serde_json::json![{ "black dahlia": "murder" }]
    );
    assert_eq!(&entity.base, &false, "base is false");
    assert_eq!(&entity.head, &false, "head is false");
    assert!(
        og_entity.si_storable.update_clock < entity.si_storable.update_clock,
        "update clock was updated"
    );
}

#[tokio::test]
async fn save_base() {
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

    let mut entity = create_entity(
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
    let og_entity = entity.clone();

    entity.name = String::from("flank steak");
    let manual_prop_baseline = entity
        .manual_properties
        .get_or_create_mut("__baseline")
        .as_object_mut()
        .expect("__baseline is not an object");
    manual_prop_baseline.insert(String::from("black dahlia"), serde_json::json!("murder"));
    entity
        .save_base(&txn, &nats)
        .await
        .expect("cannot save projection");

    assert_eq!(&entity.name, "flank steak");
    assert_eq!(
        entity.manual_properties.get("__baseline").unwrap(),
        &serde_json::json![{ "black dahlia": "murder" }]
    );
    assert_eq!(&entity.base, &true, "base is false");
    assert_eq!(&entity.head, &false, "head is false");
    assert!(
        og_entity.si_storable.update_clock < entity.si_storable.update_clock,
        "update clock was updated"
    );
}

#[tokio::test]
async fn save_head() {
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

    let mut entity = create_entity(
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
    let og_entity = entity.clone();

    entity.name = String::from("flank steak");
    let manual_prop_baseline = entity
        .manual_properties
        .get_or_create_mut("__baseline")
        .as_object_mut()
        .expect("__baseline is not an object");
    manual_prop_baseline.insert(String::from("black dahlia"), serde_json::json!("murder"));
    entity
        .save_head(&txn, &nats)
        .await
        .expect("cannot save projection");

    assert_eq!(&entity.name, "flank steak");
    assert_eq!(
        entity.manual_properties.get("__baseline").unwrap(),
        &serde_json::json![{ "black dahlia": "murder" }]
    );
    assert_eq!(&entity.base, &false, "base is false");
    assert_eq!(&entity.head, &true, "head is true");
    assert_eq!(&entity.si_change_set.is_none(), &true, "no change set");
    assert!(
        og_entity.si_storable.update_clock < entity.si_storable.update_clock,
        "update clock was updated"
    );
}

#[tokio::test]
async fn calculate_properties() {
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

    let mut entity = create_entity(
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
    entity
        .calculate_properties(&txn)
        .await
        .expect("cannot calculate properties");
}

#[tokio::test]
async fn update_properties_if_secret() {
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

    let mut entity = create_entity(
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

    let secret = create_secret_with_message(
        &txn,
        &nats,
        &nba,
        serde_json::json![{ "nightbringers": "kings of the underworld" }],
    )
    .await;

    entity
        .properties
        .get_or_create_mut("__baseline")
        .as_object_mut()
        .expect("baseline is not an object")
        .insert(
            String::from("secretId"),
            serde_json::json![secret.id.clone()],
        );

    entity
        .update_properties_if_secret(&txn)
        .await
        .expect("cannot update properties for decrypted secret");
    let decrypted_property = entity
        .properties
        .get_property("/decrypted", None)
        .expect("cannot get decrypted property")
        .expect("no value in decrypted property")
        .as_object()
        .expect("value in decrypted is not an object");
    assert_eq!(
        decrypted_property
            .get("nightbringers")
            .expect("cannot get key"),
        &serde_json::json!["kings of the underworld"]
    );
}

#[tokio::test]
async fn get_any() {
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

    let og_entity = create_entity(
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

    let entity = Entity::get_any(&txn, &og_entity.id)
        .await
        .expect("cannot get entity");
    assert_eq!(og_entity.id, entity.id);
}

#[tokio::test]
async fn get_head() {
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

    let mut og_entity = create_entity(
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

    let first_head_response = Entity::get_head(&txn, &og_entity.id).await;
    assert_eq!(first_head_response.is_err(), true);

    og_entity
        .save_head(&txn, &nats)
        .await
        .expect("cannot save entity head");

    let entity = Entity::get_head(&txn, &og_entity.id)
        .await
        .expect("cannot get head entity");
    assert_eq!(og_entity, entity);
}

#[tokio::test]
async fn get_projection() {
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

    let mut og_entity = create_entity(
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

    og_entity
        .save_projection(&txn, &nats)
        .await
        .expect("failed to save a projection");

    let entity = Entity::get_projection(&txn, &og_entity.id, &change_set.id)
        .await
        .expect("cannot get head entity");
    assert_eq!(og_entity, entity);
}

#[tokio::test]
async fn get_projection_or_head() {
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

    let mut og_entity = create_entity(
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

    og_entity
        .save_projection(&txn, &nats)
        .await
        .expect("failed to save a projection");

    let entity = Entity::get_projection_or_head(&txn, &og_entity.id, &change_set.id)
        .await
        .expect("cannot get projection entity");
    assert_eq!(og_entity, entity);

    let change_set_two = create_change_set(&txn, &nats, &nba).await;

    og_entity
        .save_head(&txn, &nats)
        .await
        .expect("failed to save a projection");
    let entity = Entity::get_projection_or_head(&txn, &og_entity.id, &change_set_two.id)
        .await
        .expect("cannot get head entity");
    assert_eq!(og_entity, entity);
}

#[tokio::test]
async fn get_all() {
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

    let mut og_entity = create_entity(
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

    og_entity
        .save_projection(&txn, &nats)
        .await
        .expect("failed to save a projection");

    let entities = Entity::get_all(&txn, &og_entity.id)
        .await
        .expect("cannot get all entities");
    assert_eq!(entities.len(), 1);
    assert_eq!(
        entities.iter().all(|o| o.id == og_entity.id),
        true,
        "all entities are the same id"
    );
    assert_eq!(
        entities.iter().any(|o| o.base == false && o.head == false),
        true,
        "the projection entity is returned"
    );

    og_entity
        .save_head(&txn, &nats)
        .await
        .expect("failed to save a projection");
    let entities = Entity::get_all(&txn, &og_entity.id)
        .await
        .expect("cannot get all entities");

    assert_eq!(entities.len(), 2);
    assert_eq!(
        entities.iter().all(|o| o.id == og_entity.id),
        true,
        "all entities are the same id"
    );
    assert_eq!(
        entities.iter().any(|o| o.base == false && o.head == true),
        true,
        "the head entity is returned"
    );
    assert_eq!(
        entities.iter().any(|o| o.base == false && o.head == false),
        true,
        "the projection entity is returned"
    );
}

#[tokio::test]
async fn list() {
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

    let mut first_entity = create_entity(
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
    first_entity
        .save_head(&txn, &nats)
        .await
        .expect("cannot save head");
    let mut second_entity = create_entity(
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
    second_entity
        .save_head(&txn, &nats)
        .await
        .expect("cannot save head");

    let reply = Entity::list(&txn, &nba.billing_account.id, None, None, None, None, None)
        .await
        .expect("cannot list entities");
    assert_eq!(reply.items.len(), 2);
}
