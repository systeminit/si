use super::session_dal::login_user;
use si_model::Resource;
use si_model::test::{
    create_change_set, create_custom_entity, create_edit_session, one_time_setup,
    signup_new_billing_account, TestContext,
};
use si_sdf::{
    filters::api,
    handlers::resource_dal::{GetResourceReply, GetResourceRequest},
};
use warp::http::StatusCode;

#[tokio::test]
async fn get_resource() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, event_log_fs, secret_key) = ctx.entries();
    let mut conn = pg.get().await.expect("cannot get connection");
    let txn = conn.transaction().await.expect("cannot get transaction");
    let nats = nats_conn.transaction();

    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;

    txn.commit().await.expect("cannot commit txn");
    nats.commit().await.expect("cannot commit nats txn");

    let txn = conn.transaction().await.expect("cannot get transaction");
    let nats = nats_conn.transaction();

    let mut change_set = create_change_set(&txn, &nats, &nba).await;
    let mut edit_session = create_edit_session(&txn, &nats, &nba, &change_set).await;

    txn.commit().await.expect("cannot commit txn");
    nats.commit().await.expect("cannot commit nats txn");

    let txn = conn.transaction().await.expect("cannot get transaction");
    let nats = nats_conn.transaction();

    let system = create_custom_entity(
        &pg,
        &txn,
        &nats_conn,
        &nats,
        &veritech,
        &nba,
        &change_set,
        &edit_session,
        "system",
    )
    .await;
    let entity = create_custom_entity(
        &pg,
        &txn,
        &nats_conn,
        &nats,
        &veritech,
        &nba,
        &change_set,
        &edit_session,
        "service",
    )
    .await;

    edit_session
        .save_session(&txn)
        .await
        .expect("failed to save edit session");
    change_set
        .apply(&txn)
        .await
        .expect("failed to apply change set");

    txn.commit().await.expect("cannot commit txn");
    nats.commit().await.expect("cannot commit nats txn");

    let resource = Resource::new(
        &pg,
        &nats_conn,
        serde_json::json!({ "cool": "beans" }),
        &entity.id,
        &system.id,
        &nba.workspace.id,
    )
    .await
    .expect("failed to create resource");

    let token = login_user(&ctx, &nba).await;
    let filter = api(pg, nats_conn, veritech, event_log_fs, secret_key);

    let request = GetResourceRequest {
        entity_id: entity.id.clone(),
        system_id: system.id.clone(),
        workspace_id: nba.workspace.id.clone(),
    };

    let res = warp::test::request()
        .method("GET")
        .header("authorization", &token)
        .path(&format!(
            "/resourceDal/getResource?{}",
            serde_urlencoded::to_string(&request).expect("cannot serialize to params")
        ))
        .reply(&filter)
        .await;

    assert_eq!(res.status(), StatusCode::OK, "create should succeed");
    let reply: GetResourceReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize node reply");
    assert_eq!(Some(resource), reply.resource);
}

#[tokio::test]
async fn get_resource_no_resource() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, event_log_fs, secret_key) = ctx.entries();
    let mut conn = pg.get().await.expect("cannot get connection");
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

    let txn = conn.transaction().await.expect("cannot get transaction");
    let nats = nats_conn.transaction();

    let system = create_custom_entity(
        &pg,
        &txn,
        &nats_conn,
        &nats,
        &veritech,
        &nba,
        &change_set,
        &edit_session,
        "system",
    )
    .await;
    let entity = create_custom_entity(
        &pg,
        &txn,
        &nats_conn,
        &nats,
        &veritech,
        &nba,
        &change_set,
        &edit_session,
        "service",
    )
    .await;

    txn.commit().await.expect("cannot commit txn");
    nats.commit().await.expect("cannot commit nats txn");

    let token = login_user(&ctx, &nba).await;
    let filter = api(pg, nats_conn, veritech, event_log_fs, secret_key);

    let request = GetResourceRequest {
        entity_id: entity.id.clone(),
        system_id: system.id.clone(),
        workspace_id: nba.workspace.id.clone(),
    };

    let res = warp::test::request()
        .method("GET")
        .header("authorization", &token)
        .path(&format!(
            "/resourceDal/getResource?{}",
            serde_urlencoded::to_string(&request).expect("cannot serialize to params")
        ))
        .reply(&filter)
        .await;

    assert_eq!(res.status(), StatusCode::OK, "create should succeed");
    let reply: GetResourceReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize node reply");
    assert_eq!(None, reply.resource);
}
