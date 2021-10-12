use si_model::{system::assign_entity_to_system_by_name, Entity, Workflow, WorkflowContext};
use si_model::test::{
    create_change_set, create_custom_entity, create_edit_session, one_time_setup,
    signup_new_billing_account, TestContext,
};

#[tokio::test]
async fn new() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;

    let system =
        Entity::get_head_by_name_and_entity_type(&txn, "production", "system", &nba.workspace.id)
            .await
            .expect("failed to fetch production system")
            .pop();
    let change_set = create_change_set(&txn, &nats, &nba).await;
    let edit_session = create_edit_session(&txn, &nats, &nba, &change_set).await;
    let entity = create_custom_entity(
        &pg,
        &txn,
        &nats_conn,
        &nats,
        &veritech,
        &nba,
        &change_set,
        &edit_session,
        "dockerImage",
    )
    .await;
    assign_entity_to_system_by_name(&txn, &nats, "production", &entity)
        .await
        .expect("failed to assign entity to system");

    txn.commit().await.expect("failed to commit transaction");
    nats.commit().await.expect("failed to commit nats");

    let txn = conn.transaction().await.expect("cannot create txn");

    let ctx = WorkflowContext {
        dry_run: true,
        entity: Some(entity),
        system,
        selection: vec![],
        strategy: None,
        fail_if_missing: None,
        inputs: None,
        args: None,
        output: None,
        store: None,
        workspace: nba.workspace.clone(),
    };

    let _run = Workflow::get_by_name(&txn, "universal:deploy")
        .await
        .expect("failed to get workflow")
        .invoke_and_wait(&pg, &nats_conn, &veritech, ctx)
        .await
        .expect("failed to invoke workflow");
}
