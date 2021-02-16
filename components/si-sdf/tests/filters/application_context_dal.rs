use crate::{
    filters::session_dal::login_user,
    generate_fake_name,
    models::{
        billing_account::{signup_new_billing_account, NewBillingAccount},
        change_set::create_change_set,
        edit_session::create_edit_session,
        entity::create_custom_entity,
    },
    one_time_setup, TestContext,
};
use si_sdf::{
    filters::api,
    handlers::{
        application_context_dal::{
            CancelEditSessionReply, CancelEditSessionRequest, CreateChangeSetAndEditSessionReply,
            CreateChangeSetAndEditSessionRequest, CreateEditSessionAndGetChangeSetReply,
            CreateEditSessionAndGetChangeSetRequest, CreateEditSessionReply,
            CreateEditSessionRequest, GetApplicationContextReply, GetApplicationContextRequest,
            GetChangeSetAndEditSessionReply, GetChangeSetAndEditSessionRequest,
        },
        LabelListItem,
    },
    models::{ChangeSet, EditSession, Entity},
};
use warp::http::StatusCode;

async fn create_application(ctx: &TestContext, nba: &NewBillingAccount) -> Entity {
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let mut conn = pg.pool.get().await.expect("cannot get connection");

    let txn = conn.transaction().await.expect("cannot get transaction");
    let nats = nats_conn.transaction();

    let mut change_set = create_change_set(&txn, &nats, &nba).await;
    let edit_session = create_edit_session(&txn, &nats, &nba, &change_set).await;

    txn.commit().await.expect("cannot commit txn");
    nats.commit().await.expect("cannot commit nats txn");

    let txn = conn.transaction().await.expect("cannot get transaction");
    let nats = nats_conn.transaction();

    let application = create_custom_entity(
        &pg,
        &txn,
        &nats_conn,
        &nats,
        &veritech,
        &nba,
        &change_set,
        &edit_session,
        &nba.system,
        "application",
    )
    .await;
    change_set
        .execute(&pg, &txn, &nats_conn, &nats, &veritech, false, None)
        .await
        .expect("cannot execute changeset");

    txn.commit().await.expect("cannot commit txn");
    nats.commit().await.expect("cannot commit nats txn");

    application
}

#[tokio::test]
async fn get_application_context() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, event_log_fs, secret_key) = ctx.entries();
    let mut conn = pg.pool.get().await.expect("cannot get connection");
    let txn = conn.transaction().await.expect("cannot get transaction");
    let nats = nats_conn.transaction();

    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;

    txn.commit().await.expect("cannot commit txn");
    nats.commit().await.expect("cannot commit nats txn");

    let txn = conn.transaction().await.expect("cannot get transaction");
    let nats = nats_conn.transaction();

    let application = create_application(&ctx, &nba).await;
    let change_set = create_change_set(&txn, &nats, &nba).await;

    txn.commit().await.expect("cannot commit txn");
    nats.commit().await.expect("cannot commit nats txn");

    let token = login_user(&ctx, &nba).await;
    let filter = api(pg, nats_conn, veritech, event_log_fs, secret_key);

    let request = GetApplicationContextRequest {
        application_id: application.id,
        workspace_id: nba.workspace.id,
    };

    let res = warp::test::request()
        .method("GET")
        .header("authorization", &token)
        .path(&format!(
            "/applicationContextDal/getApplicationContext?{}",
            serde_urlencoded::to_string(&request).expect("cannot serialize to params")
        ))
        .reply(&filter)
        .await;

    assert_eq!(res.status(), StatusCode::OK, "create should succeed");
    let reply: GetApplicationContextReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize node reply");
    assert_eq!(reply.application_name, application.name);
    assert_eq!(
        reply.systems_list,
        vec![nba.system]
            .into_iter()
            .map(|s| LabelListItem {
                label: s.name.to_string(),
                value: s.id.to_string()
            })
            .collect::<Vec<_>>()
    );
    assert_eq!(
        reply.open_change_sets_list,
        vec![change_set]
            .into_iter()
            .map(|cs| LabelListItem {
                label: cs.name.to_string(),
                value: cs.id.to_string(),
            })
            .collect::<Vec<_>>()
    );
}

#[tokio::test]
async fn create_change_set_and_edit_session() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, event_log_fs, secret_key) = ctx.entries();
    let mut conn = pg.pool.get().await.expect("cannot get connection");
    let txn = conn.transaction().await.expect("cannot get transaction");
    let nats = nats_conn.transaction();

    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;

    txn.commit().await.expect("cannot commit txn");
    nats.commit().await.expect("cannot commit nats txn");

    let token = login_user(&ctx, &nba).await;
    let filter = api(pg, nats_conn, veritech, event_log_fs, secret_key);

    let name = generate_fake_name();

    let res = warp::test::request()
        .method("POST")
        .header("authorization", &token)
        .path("/applicationContextDal/createChangeSetAndEditSession")
        .json(&CreateChangeSetAndEditSessionRequest {
            change_set_name: name.clone(),
            workspace_id: nba.workspace.id.clone(),
        })
        .reply(&filter)
        .await;

    assert_eq!(res.status(), StatusCode::OK, "create should succeed");
    let reply: CreateChangeSetAndEditSessionReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize node reply");

    let txn = conn.transaction().await.expect("cannot get transaction");

    let expected_change_set = ChangeSet::get(&txn, &reply.change_set.id)
        .await
        .expect("cannot get change set");
    let expected_edit_session = EditSession::get(&txn, &reply.edit_session.id)
        .await
        .expect("cannot get edit session");

    assert_eq!(expected_change_set, reply.change_set);
    assert_eq!(expected_edit_session, reply.edit_session);
}

#[tokio::test]
async fn get_change_set_and_edit_session() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, event_log_fs, secret_key) = ctx.entries();
    let mut conn = pg.pool.get().await.expect("cannot get connection");
    let txn = conn.transaction().await.expect("cannot get transaction");
    let nats = nats_conn.transaction();

    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;

    txn.commit().await.expect("cannot commit txn");
    nats.commit().await.expect("cannot commit nats txn");

    let txn = conn.transaction().await.expect("cannot get transaction");
    let nats = nats_conn.transaction();

    let change_set = create_change_set(&txn, &nats, &nba).await;
    let edit_session = create_edit_session(&txn, &nats, &nba, &change_set).await;

    txn.commit().await.expect("cannot commit txn");
    nats.commit().await.expect("cannot commit nats txn");

    let token = login_user(&ctx, &nba).await;
    let filter = api(pg, nats_conn, veritech, event_log_fs, secret_key);

    let request = GetChangeSetAndEditSessionRequest {
        change_set_id: change_set.id.clone(),
        edit_session_id: edit_session.id.clone(),
    };

    let res = warp::test::request()
        .method("GET")
        .header("authorization", &token)
        .path(&format!(
            "/applicationContextDal/getChangeSetAndEditSession?{}",
            serde_urlencoded::to_string(&request).expect("cannot serialize to params")
        ))
        .reply(&filter)
        .await;

    assert_eq!(res.status(), StatusCode::OK, "create should succeed");
    let reply: GetChangeSetAndEditSessionReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize node reply");
    assert_eq!(change_set, reply.change_set);
    assert_eq!(edit_session, reply.edit_session);
}

#[tokio::test]
async fn create_edit_session_and_get_change_set() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, event_log_fs, secret_key) = ctx.entries();
    let mut conn = pg.pool.get().await.expect("cannot get connection");
    let txn = conn.transaction().await.expect("cannot get transaction");
    let nats = nats_conn.transaction();

    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;

    txn.commit().await.expect("cannot commit txn");
    nats.commit().await.expect("cannot commit nats txn");

    let txn = conn.transaction().await.expect("cannot get transaction");
    let nats = nats_conn.transaction();

    let change_set = create_change_set(&txn, &nats, &nba).await;

    txn.commit().await.expect("cannot commit txn");
    nats.commit().await.expect("cannot commit nats txn");

    let token = login_user(&ctx, &nba).await;
    let filter = api(pg, nats_conn, veritech, event_log_fs, secret_key);

    let res = warp::test::request()
        .method("POST")
        .header("authorization", &token)
        .path("/applicationContextDal/createEditSessionAndGetChangeSet")
        .json(&CreateEditSessionAndGetChangeSetRequest {
            change_set_id: change_set.id.clone(),
        })
        .reply(&filter)
        .await;

    assert_eq!(res.status(), StatusCode::OK, "create should succeed");
    let reply: CreateEditSessionAndGetChangeSetReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize node reply");

    let txn = conn.transaction().await.expect("cannot get transaction");

    let expected_edit_session = EditSession::get(&txn, &reply.edit_session.id)
        .await
        .expect("cannot get edit session");

    assert_eq!(change_set, reply.change_set);
    assert_eq!(expected_edit_session, reply.edit_session);
}

#[tokio::test]
async fn create_edit_session_handler() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, event_log_fs, secret_key) = ctx.entries();
    let mut conn = pg.pool.get().await.expect("cannot get connection");
    let txn = conn.transaction().await.expect("cannot get transaction");
    let nats = nats_conn.transaction();

    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;

    txn.commit().await.expect("cannot commit txn");
    nats.commit().await.expect("cannot commit nats txn");

    let txn = conn.transaction().await.expect("cannot get transaction");
    let nats = nats_conn.transaction();

    let change_set = create_change_set(&txn, &nats, &nba).await;

    txn.commit().await.expect("cannot commit txn");
    nats.commit().await.expect("cannot commit nats txn");

    let token = login_user(&ctx, &nba).await;
    let filter = api(pg, nats_conn, veritech, event_log_fs, secret_key);

    let res = warp::test::request()
        .method("POST")
        .header("authorization", &token)
        .path("/applicationContextDal/createEditSession")
        .json(&CreateEditSessionRequest {
            change_set_id: change_set.id.clone(),
            workspace_id: nba.workspace.id.clone(),
        })
        .reply(&filter)
        .await;

    assert_eq!(res.status(), StatusCode::OK, "create should succeed");
    let reply: CreateEditSessionReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize node reply");

    let txn = conn.transaction().await.expect("cannot get transaction");

    let expected_edit_session = EditSession::get(&txn, &reply.edit_session.id)
        .await
        .expect("cannot get edit session");

    assert_eq!(expected_edit_session, reply.edit_session);
}

#[tokio::test]
async fn cancel_edit_session() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, event_log_fs, secret_key) = ctx.entries();
    let mut conn = pg.pool.get().await.expect("cannot get connection");
    let txn = conn.transaction().await.expect("cannot get transaction");
    let nats = nats_conn.transaction();

    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;

    txn.commit().await.expect("cannot commit txn");
    nats.commit().await.expect("cannot commit nats txn");

    let txn = conn.transaction().await.expect("cannot get transaction");
    let nats = nats_conn.transaction();

    let change_set = create_change_set(&txn, &nats, &nba).await;
    let edit_session = create_edit_session(&txn, &nats, &nba, &change_set).await;

    txn.commit().await.expect("cannot commit txn");
    nats.commit().await.expect("cannot commit nats txn");

    let token = login_user(&ctx, &nba).await;
    let filter = api(pg, nats_conn, veritech, event_log_fs, secret_key);

    let res = warp::test::request()
        .method("POST")
        .header("authorization", &token)
        .path("/applicationContextDal/cancelEditSession")
        .json(&CancelEditSessionRequest {
            edit_session_id: edit_session.id.clone(),
            workspace_id: nba.workspace.id.clone(),
        })
        .reply(&filter)
        .await;

    assert_eq!(res.status(), StatusCode::OK, "create should succeed");
    let reply: CancelEditSessionReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize node reply");

    let txn = conn.transaction().await.expect("cannot get transaction");

    let expected_edit_session = EditSession::get(&txn, &reply.edit_session.id)
        .await
        .expect("cannot get edit session");

    assert_eq!(expected_edit_session, reply.edit_session);
    assert_eq!(reply.edit_session.reverted, true);
}
