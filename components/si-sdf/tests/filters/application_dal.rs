use crate::filters::session_dal::login_user;
use si_model_test::{
    create_change_set, create_custom_entity, create_edit_session, generate_fake_name,
    one_time_setup, signup_new_billing_account, TestContext,
};
use si_sdf::filters::api;
use si_sdf::handlers::application_dal::{
    CreateApplicationReply, CreateApplicationRequest, ListApplicationsReply,
};
use warp::http::StatusCode;

#[tokio::test]
async fn create_application() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, event_log_fs, secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.get().await.expect("cannot get connection");
    let txn = conn.transaction().await.expect("cannot get transaction");
    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
    txn.commit().await.expect("cannot commit txn");

    let token = login_user(&ctx, &nba).await;
    let filter = api(pg, nats_conn, veritech, event_log_fs, secret_key);

    let app_name = generate_fake_name();
    let res = warp::test::request()
        .method("POST")
        .header("authorization", &token)
        .path("/applicationDal/createApplication")
        .json(&CreateApplicationRequest {
            workspace_id: nba.workspace.id.clone(),
            application_name: app_name.clone(),
        })
        .reply(&filter)
        .await;

    assert_eq!(res.status(), StatusCode::OK, "create should succeed");
    let reply: CreateApplicationReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize node reply");
    assert_eq!(reply.application.name, app_name);
}

#[tokio::test]
async fn list_applications() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, event_log_fs, secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.get().await.expect("cannot get connection");
    let txn = conn.transaction().await.expect("cannot get transaction");
    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
    txn.commit().await.expect("cannot commit txn");

    let token = login_user(&ctx, &nba).await;

    let txn = conn.transaction().await.expect("cannot get transaction");
    let mut change_set = create_change_set(&txn, &nats, &nba).await;
    let mut edit_session = create_edit_session(&txn, &nats, &nba, &change_set).await;
    txn.commit().await.expect("cannot commit txn");

    let txn = conn.transaction().await.expect("cannot get transaction");
    let app1 = create_custom_entity(
        &pg,
        &txn,
        &nats_conn,
        &nats,
        &veritech,
        &nba,
        &change_set,
        &edit_session,
        "application",
    )
    .await;
    let app2 = create_custom_entity(
        &pg,
        &txn,
        &nats_conn,
        &nats,
        &veritech,
        &nba,
        &change_set,
        &edit_session,
        "application",
    )
    .await;

    edit_session
        .save_session(&txn)
        .await
        .expect("cannot save edit session");
    change_set
        .apply(&txn)
        .await
        .expect("cannot apply changeset");
    txn.commit().await.expect("cannot commit txn");

    let filter = api(pg, nats_conn, veritech, event_log_fs, secret_key);

    let res = warp::test::request()
        .method("GET")
        .header("authorization", &token)
        .path(&format!(
            "/applicationDal/listApplications?workspaceId={}",
            &nba.workspace.id
        ))
        .reply(&filter)
        .await;

    assert_eq!(res.status(), StatusCode::OK, "create should succeed");
    let reply: ListApplicationsReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize node reply");
    assert_eq!(reply.list.len(), 2);
    assert!(
        reply.list.iter().any(|a| a.application.id == app1.id),
        "app1 appears in the list"
    );
    assert!(
        reply.list.iter().any(|a| a.application.id == app2.id),
        "app2 appears in the list"
    );
}
