use crate::models::billing_account::{signup_new_billing_account, NewBillingAccount};
use crate::models::change_set::create_change_set;
use crate::models::edit_session::create_edit_session;
use crate::models::system::create_system;

use crate::{one_time_setup, TestContext};

use si_sdf::data::{NatsConn, NatsTxn, PgPool, PgTxn};
use si_sdf::models::{
    ChangeSet, Edge, EdgeKind, EditSession, Entity, Node, NodeKind, Position, Resource, System,
};
use si_sdf::veritech::Veritech;

#[allow(dead_code)]
pub async fn create_custom_entity_node(
    pg: &PgPool,
    txn: &PgTxn<'_>,
    nats_conn: &NatsConn,
    nats: &NatsTxn,
    veritech: &Veritech,
    nba: &NewBillingAccount,
    system: &System,
    change_set: &ChangeSet,
    edit_session: &EditSession,
    object_type: impl AsRef<str>,
) -> Node {
    let object_type = object_type.as_ref();
    let entity_node = Node::new(
        &pg,
        &txn,
        &nats_conn,
        &nats,
        &veritech,
        None,
        NodeKind::Entity,
        object_type,
        &nba.workspace.id,
        &change_set.id,
        &edit_session.id,
        Some(vec![system.id.clone()]),
    )
    .await
    .expect("cannot create new node");

    entity_node
}

pub async fn create_entity_node(
    pool: &PgPool,
    txn: &PgTxn<'_>,
    nats_conn: &NatsConn,
    nats: &NatsTxn,
    veritech: &Veritech,
    nba: &NewBillingAccount,
    system: &System,
    change_set: &ChangeSet,
    edit_session: &EditSession,
) -> Node {
    let entity_node = Node::new(
        &pool,
        &txn,
        &nats_conn,
        &nats,
        &veritech,
        None,
        NodeKind::Entity,
        "service",
        &nba.workspace.id,
        &change_set.id,
        &edit_session.id,
        Some(vec![system.id.clone()]),
    )
    .await
    .expect("cannot create new node");

    entity_node
}

pub async fn create_system_node(
    pool: &PgPool,
    txn: &PgTxn<'_>,
    nats_conn: &NatsConn,
    nats: &NatsTxn,
    veritech: &Veritech,
    nba: &NewBillingAccount,
    change_set: &ChangeSet,
    edit_session: &EditSession,
) -> Node {
    let system_node = Node::new(
        &pool,
        &txn,
        &nats_conn,
        &nats,
        &veritech,
        None,
        NodeKind::System,
        "system",
        &nba.workspace.id,
        &change_set.id,
        &edit_session.id,
        None,
    )
    .await
    .expect("cannot create new node");

    system_node
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

    let system_node = Node::new(
        &pg,
        &txn,
        &nats_conn,
        &nats,
        &veritech,
        None,
        NodeKind::System,
        "system",
        &nba.workspace.id,
        &change_set.id,
        &edit_session.id,
        None,
    )
    .await
    .expect("cannot create new node");

    assert_eq!(&system_node.object_id.starts_with("system:"), &true);
    assert_eq!(&system_node.positions, &std::collections::HashMap::new());
    assert_eq!(&system_node.kind, &NodeKind::System);
    assert_eq!(&system_node.object_type, "system");

    let entity_node = Node::new(
        &pg,
        &txn,
        &nats_conn,
        &nats,
        &veritech,
        None,
        NodeKind::Entity,
        "service",
        &nba.workspace.id,
        &change_set.id,
        &edit_session.id,
        Some(vec![system_node.object_id.clone()]),
    )
    .await
    .expect("cannot create new node");

    assert_eq!(&entity_node.object_id.starts_with("entity:"), &true);
    assert_eq!(&entity_node.positions, &std::collections::HashMap::new());
    assert_eq!(&entity_node.kind, &NodeKind::Entity);
    assert_eq!(&entity_node.object_type, "service");
}

#[tokio::test]
async fn set_position() {
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
    let mut system_node = create_system_node(
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
    system_node.set_position("appview", Position::new(0, 0));
    assert_eq!(
        system_node
            .positions
            .get("appview")
            .expect("cannot find appview position context"),
        &Position::new(0, 0)
    );
}

#[tokio::test]
async fn save() {
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
    let mut system_node = create_system_node(
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
    system_node.set_position("appview", Position::new(0, 0));
    assert_eq!(
        system_node
            .positions
            .get("appview")
            .expect("cannot find appview position context"),
        &Position::new(0, 0)
    );
    let pre_save_node = system_node.clone();
    system_node
        .save(&txn, &nats)
        .await
        .expect("cannot save node");
    assert_eq!(
        &pre_save_node.si_storable.update_clock < &system_node.si_storable.update_clock,
        true
    );
}

#[tokio::test]
async fn sync_resource() {
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
    let entity_node = create_entity_node(
        &pg,
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
    entity_node
        .sync_resource(
            &pg,
            &txn,
            &nats_conn,
            &veritech,
            &system.id,
            Some(change_set.id.clone()),
        )
        .await
        .expect("cannot sync resource for changeset");

    let mut entity = Entity::get_projection(&txn, &entity_node.object_id, &change_set.id)
        .await
        .expect("cannot get entity");
    entity
        .save_head(&txn, &nats)
        .await
        .expect("saving entity head");
    let mut resource =
        Resource::get_any_by_node_id(&txn, &entity_node.id, &system.id, &change_set.id)
            .await
            .expect("cannot get resource");
    resource
        .save_head(&pg, &nats_conn)
        .await
        .expect("cannot save resource head");

    txn.commit().await.expect("cannot commit txn");
    let txn = conn.transaction().await.expect("cannot create txn");
    entity_node
        .sync_resource(&pg, &txn, &nats_conn, &veritech, &system.id, None)
        .await
        .expect("cannot sync head resource");
}

#[tokio::test]
async fn configured_by() {
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
    let first_entity_node = create_entity_node(
        &pg,
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
    let second_entity_node = create_entity_node(
        &pg,
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
    second_entity_node
        .configured_by(&txn, &nats, &first_entity_node.id)
        .await
        .expect("error configuring_by node");
    let edges = Edge::direct_predecessor_edges_by_node_id(
        &txn,
        &EdgeKind::Configures,
        &second_entity_node.id,
    )
    .await
    .expect("cannot get predecessor edges");
    assert_eq!(
        edges
            .iter()
            .any(|edge| edge.tail_vertex.node_id == first_entity_node.id),
        true,
        "first entity configures the second"
    );
}

#[tokio::test]
async fn include_in_system() {
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
    let second_system = create_system(
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
    let entity_node = create_entity_node(
        &pg,
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

    entity_node
        .include_in_system(&txn, &nats, &second_system.id)
        .await
        .expect("cannot include node to system");

    let edges =
        Edge::direct_predecessor_edges_by_node_id(&txn, &EdgeKind::Includes, &entity_node.id)
            .await
            .expect("cannot get predecessor edges");
    assert_eq!(edges.len(), 2);
    assert_eq!(
        edges
            .iter()
            .any(|edge| edge.tail_vertex.object_id == second_system.id),
        true,
        "enttity is included by the system"
    );
}

#[tokio::test]
async fn get_object_id() {
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
    let entity_node = create_entity_node(
        &pg,
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
    assert_eq!(entity_node.get_object_id(), entity_node.object_id);
}

#[tokio::test]
async fn get_head_object_entity() {
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
    let entity_node = create_entity_node(
        &pg,
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
    let mut entity = Entity::get_projection(&txn, &entity_node.object_id, &change_set.id)
        .await
        .expect("cannot get entity for node");
    entity
        .save_head(&txn, &nats)
        .await
        .expect("cannot save entity head");
    let fetched_entity = entity_node
        .get_head_object_entity(&txn)
        .await
        .expect("cannot get head object entity from node");
    assert_eq!(entity, fetched_entity);
}

#[tokio::test]
async fn get_projection_object_entity() {
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
    let entity_node = create_entity_node(
        &pg,
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
    let entity = Entity::get_projection(&txn, &entity_node.object_id, &change_set.id)
        .await
        .expect("cannot get entity for node");
    let fetched_entity = entity_node
        .get_projection_object_entity(&txn, &change_set.id)
        .await
        .expect("cannot get head object entity from node");
    assert_eq!(entity, fetched_entity);
}

#[tokio::test]
async fn get_head_object_system() {
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

    let system_node = create_system_node(
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
    let mut system = System::get_projection(&txn, &system_node.object_id, &change_set.id)
        .await
        .expect("cannot get system for node");
    system
        .save_head(&txn, &nats)
        .await
        .expect("cannot save entity head");
    let fetched_system = system_node
        .get_head_object_system(&txn)
        .await
        .expect("cannot get head object entity from node");
    assert_eq!(system, fetched_system);
}

#[tokio::test]
async fn get_projection_object_system() {
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

    let system_node = create_system_node(
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
    let system = System::get_projection(&txn, &system_node.object_id, &change_set.id)
        .await
        .expect("cannot get system for node");
    let fetched_system = system_node
        .get_projection_object_system(&txn, &change_set.id)
        .await
        .expect("cannot get head object entity from node");
    assert_eq!(system, fetched_system);
}

#[tokio::test]
async fn get() {
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
    let entity_node = create_entity_node(
        &pg,
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

    let node = Node::get(&txn, &entity_node.id)
        .await
        .expect("cannot get node");
    assert_eq!(entity_node, node);
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
    let _first_entity_node = create_entity_node(
        &pg,
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
    let _second_entity_node = create_entity_node(
        &pg,
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

    let reply = Node::list(&txn, &nba.workspace.id, None, None, None, None, None)
        .await
        .expect("cannot list node");
    // System, two Entities
    assert_eq!(reply.items.len(), 4);
}
