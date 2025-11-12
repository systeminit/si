use std::{
    collections::HashSet,
    sync::Arc,
    time::Duration,
};

use chrono::Utc;
use si_events::{
    Actor,
    ChangeSetId,
    ContentHash,
    FuncBackendKind,
    FuncBackendResponseType,
    FuncKind,
    FuncRun,
    FuncRunBuilder,
    FuncRunId,
    FuncRunState,
    Tenancy,
    UserPk,
    WorkspacePk,
};
use si_layer_cache::{
    LayerDb,
    db::serialize,
};
use tokio::time::Instant;
use tokio_util::sync::CancellationToken;

use crate::integration_test::{
    make_test_layerdb_config,
    setup_compute_executor,
    setup_nats_client,
    setup_pg_db,
};

type TestLayerDb = LayerDb<String, String, String, String, String, String, String>;

#[tokio::test]
async fn write_to_db() {
    let token = CancellationToken::new();

    let (ldb, _): (TestLayerDb, _) = LayerDb::from_services(
        make_test_layerdb_config(),
        setup_pg_db("func_run_write_to_db").await,
        setup_nats_client(Some("func_run_write_to_db".to_string())).await,
        setup_compute_executor(),
        token,
    )
    .await
    .expect("cannot create layerdb");
    ldb.pg_migrate().await.expect("migrate layer db");

    let (tenancy, actor) = (
        Tenancy::new(WorkspacePk::new(), ChangeSetId::new()),
        Actor::User(UserPk::new()),
    );

    let func_run = create_func_run(actor, tenancy, "joshua graham");
    let key_str: Arc<str> = func_run.id().to_string().into();
    let value: Arc<FuncRun> = Arc::new(func_run);

    ldb.func_run()
        .write(value.clone(), None, tenancy, actor)
        .await
        .expect("failed to write to layerdb");

    // Are we in memory?
    let in_memory = ldb.func_run().cache.cache().get(key_str.clone()).await;
    assert_eq!(value.id(), in_memory.expect("func run not in memory").id());

    // Are we in pg?
    let in_pg_postcard = ldb
        .func_run()
        .cache
        .pg()
        .get(&key_str)
        .await
        .expect("error getting data from pg")
        .expect("no cas object in pg");
    let in_pg: FuncRun =
        serialize::from_bytes(&in_pg_postcard[..]).expect("cannot deserialize data");
    assert_eq!(value.id(), in_pg.id());
}

#[tokio::test]
async fn update() {
    let token = CancellationToken::new();

    let db = setup_pg_db("func_run_update_to_db").await;
    let (ldb, _): (TestLayerDb, _) = LayerDb::from_services(
        make_test_layerdb_config(),
        db.clone(),
        setup_nats_client(Some("func_run_update_to_db".to_string())).await,
        setup_compute_executor(),
        token.clone(),
    )
    .await
    .expect("cannot create layerdb");
    ldb.pg_migrate().await.expect("migrate layer db");

    let (ldb_remote, _): (TestLayerDb, _) = LayerDb::from_services(
        make_test_layerdb_config(),
        db,
        setup_nats_client(Some("func_run_update_to_db".to_string())).await,
        setup_compute_executor(),
        token,
    )
    .await
    .expect("cannot create layerdb");

    let (tenancy, actor) = (
        Tenancy::new(WorkspacePk::new(), ChangeSetId::new()),
        Actor::User(UserPk::new()),
    );

    let func_run = create_func_run(actor, tenancy, "lindsey buckingham");
    let key_str: Arc<str> = func_run.id().to_string().into();
    let value: Arc<FuncRun> = Arc::new(func_run);

    ldb.func_run()
        .write(value.clone(), None, tenancy, actor)
        .await
        .expect("failed to write to layerdb");

    // Are we in memory?
    let in_memory = ldb.func_run().cache.cache().get(key_str.clone()).await;
    assert_eq!(value.id(), in_memory.expect("func run not in memory").id());

    // Are we in pg?
    let in_pg_postcard = ldb
        .func_run()
        .cache
        .pg()
        .get(&key_str)
        .await
        .expect("error getting data from pg")
        .expect("no cas object in pg");
    let in_pg: FuncRun =
        serialize::from_bytes(&in_pg_postcard[..]).expect("cannot deserialize data");
    assert_eq!(value.id(), in_pg.id());
    assert_eq!(value.state(), in_pg.state());

    // Update the state to success
    let mut update_func_run_inner = Arc::unwrap_or_clone(value);
    update_func_run_inner.set_state(FuncRunState::Success);
    let update_func_run = Arc::new(update_func_run_inner);

    ldb.func_run()
        .write(update_func_run.clone(), None, tenancy, actor)
        .await
        .expect("failed to write to layerdb");

    // Are we in memory?
    let in_memory = ldb.func_run().cache.cache().get(key_str.clone()).await;
    assert_eq!(
        update_func_run.state(),
        in_memory.expect("func run not in memory").state(),
        "updated in memory state"
    );

    // Are we in pg?
    let in_pg_postcard = ldb
        .func_run()
        .cache
        .pg()
        .get(&key_str)
        .await
        .expect("error getting data from pg")
        .expect("no cas object in pg");
    let in_pg: FuncRun =
        serialize::from_bytes(&in_pg_postcard[..]).expect("cannot deserialize data");
    assert_eq!(update_func_run.state(), in_pg.state());

    let max_check_count = 10;
    let mut memory_check_count = 0;
    while memory_check_count <= max_check_count {
        let in_memory = ldb_remote
            .func_run()
            .cache
            .cache()
            .get(key_str.clone())
            .await;
        match in_memory {
            Some(value) => {
                assert_eq!(update_func_run.state(), value.state());
                break;
            }
            None => {
                memory_check_count += 1;
                tokio::time::sleep_until(Instant::now() + Duration::from_millis(1)).await;
            }
        }
    }
    assert_ne!(
        max_check_count, memory_check_count,
        "value did not arrive in the remote memory cache within 10ms"
    );
}

#[tokio::test]
async fn write_and_read_many_for_workspace_id() {
    let token = CancellationToken::new();

    let (ldb, _): (TestLayerDb, _) = LayerDb::from_services(
        make_test_layerdb_config(),
        setup_pg_db("func_run_write_and_read_many_for_workspace_id").await,
        setup_nats_client(Some(
            "fun_run_write_and_read_many_for_workspace_id".to_string(),
        ))
        .await,
        setup_compute_executor(),
        token,
    )
    .await
    .expect("cannot create layerdb");
    ldb.pg_migrate().await.expect("migrate ldb");

    let (tenancy, actor) = (
        Tenancy::new(WorkspacePk::new(), ChangeSetId::new()),
        Actor::User(UserPk::new()),
    );

    let values = vec![
        Arc::new(create_func_run(actor, tenancy, "dead money")),
        Arc::new(create_func_run(actor, tenancy, "honest hearts")),
        Arc::new(create_func_run(actor, tenancy, "old world blues")),
        Arc::new(create_func_run(actor, tenancy, "lonesome road")),
    ];
    let expected: HashSet<FuncRunId> = HashSet::from_iter(values.iter().map(|v| v.id()));

    for value in values {
        ldb.func_run()
            .write(value.clone(), None, tenancy, actor)
            .await
            .expect("failed to write to layerdb");
    }

    let read_many_in_workspace_values = ldb
        .func_run()
        .read_many_for_workspace(tenancy.workspace_pk)
        .await
        .expect("error getting data from pg")
        .expect("should be able to read");

    assert_eq!(
        expected,
        HashSet::from_iter(read_many_in_workspace_values.iter().map(|v| v.id()))
    );
}

#[tokio::test]
async fn read_many_for_workspace_paginated() {
    let token = CancellationToken::new();

    let (ldb, _): (TestLayerDb, _) = LayerDb::from_services(
        make_test_layerdb_config(),
        setup_pg_db("fun_run_write_and_read_many_for_workspace_paginated").await,
        setup_nats_client(Some(
            "fun_run_write_and_read_many_for_workspace_paginated".to_string(),
        ))
        .await,
        setup_compute_executor(),
        token,
    )
    .await
    .expect("cannot create layerdb");
    ldb.pg_migrate().await.expect("migrate ldb");

    let (tenancy, actor) = (
        Tenancy::new(WorkspacePk::new(), ChangeSetId::new()),
        Actor::User(UserPk::new()),
    );

    let values = vec![
        Arc::new(create_func_run(actor, tenancy, "dead money")),
        Arc::new(create_func_run(actor, tenancy, "honest hearts")),
        Arc::new(create_func_run(actor, tenancy, "old world blues")),
        Arc::new(create_func_run(actor, tenancy, "lonesome road")),
    ];
    // create func runs
    for value in values {
        ldb.func_run()
            .write(value.clone(), None, tenancy, actor)
            .await
            .expect("failed to write to layerdb");
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }

    let read_many_in_workspace_values = ldb
        .func_run()
        .read_many_for_workspace(tenancy.workspace_pk)
        .await
        .expect("error getting data from pg")
        .expect("should be able to read");
    dbg!(&read_many_in_workspace_values);
    // let's get the very first value (no cursor, limit 1)
    let first_value = ldb
        .func_run()
        .read_many_for_workspace_paginated(tenancy.workspace_pk, tenancy.change_set_id, 1, None)
        .await
        .expect("error getting data from pg")
        .expect("should be able to read")
        .pop()
        .expect("has one entry");

    // the first returned should be the last func
    let cursor = first_value.id();
    assert_eq!("lonesome road", first_value.function_name());

    // now let's get the next 2 values (cursor is the first result's RunId and a limit of 2)
    let next_two_values = ldb
        .func_run()
        .read_many_for_workspace_paginated(
            tenancy.workspace_pk,
            tenancy.change_set_id,
            2,
            Some(cursor),
        )
        .await
        .expect("error getting data from pg")
        .expect("should be able to read");
    dbg!(&next_two_values);
    assert_eq!(2, next_two_values.len());
    let next_two_expected: HashSet<FuncRunId> =
        HashSet::from_iter(next_two_values.iter().map(|v| v.id()));

    // now let's start at the same cursor (the first result returned) and fetch a limit of 10 (which should be all of the remaining funcs)
    let all_remaining = ldb
        .func_run()
        .read_many_for_workspace_paginated(
            tenancy.workspace_pk,
            tenancy.change_set_id,
            10,
            Some(cursor),
        )
        .await
        .expect("error getting data from pg")
        .expect("should be able to read");
    dbg!(&all_remaining);
    assert_eq!(3, all_remaining.len());

    // Create some more runs - simulating a user who has a cursor ID but hasn't fetched all latest yet

    let values = vec![
        Arc::new(create_func_run(actor, tenancy, "dead money part 2")),
        Arc::new(create_func_run(actor, tenancy, "honest hearts part 2")),
        Arc::new(create_func_run(actor, tenancy, "old world blues part 2")),
        Arc::new(create_func_run(actor, tenancy, "lonesome road part 2")),
    ];
    // create func runs
    for value in values {
        ldb.func_run()
            .write(value.clone(), None, tenancy, actor)
            .await
            .expect("failed to write to layerdb");
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }

    // repeat the same, get the next 2 from the previous cursor
    let next_two_values = ldb
        .func_run()
        .read_many_for_workspace_paginated(
            tenancy.workspace_pk,
            tenancy.change_set_id,
            2,
            Some(cursor),
        )
        .await
        .expect("error getting data from pg")
        .expect("should be able to read");
    dbg!(&next_two_values);
    assert_eq!(2, next_two_values.len());
    assert_eq!(
        next_two_expected,
        HashSet::from_iter(next_two_values.iter().map(|v| v.id()))
    );

    // ensure the results are the same when getting a page of funcs older than the last cursor
    let all_remaining = ldb
        .func_run()
        .read_many_for_workspace_paginated(
            tenancy.workspace_pk,
            tenancy.change_set_id,
            10,
            Some(cursor),
        )
        .await
        .expect("error getting data from pg")
        .expect("should be able to read");
    dbg!(&all_remaining);
    assert_eq!(3, all_remaining.len());

    // But if I fetch without a cursor, I get the new funcs now!
    let first_value = ldb
        .func_run()
        .read_many_for_workspace_paginated(tenancy.workspace_pk, tenancy.change_set_id, 1, None)
        .await
        .expect("error getting data from pg")
        .expect("should be able to read")
        .pop()
        .expect("has one entry");

    // the first returned should be the last func
    let cursor = first_value.id();
    assert_eq!("lonesome road part 2", first_value.function_name());

    // now fetch all remaining from this new cursor, which should yield all func runs
    let all_remaining = ldb
        .func_run()
        .read_many_for_workspace_paginated(
            tenancy.workspace_pk,
            tenancy.change_set_id,
            10,
            Some(cursor),
        )
        .await
        .expect("error getting data from pg")
        .expect("should be able to read");
    dbg!(&all_remaining);
    assert_eq!(7, all_remaining.len());
}

fn create_func_run(actor: Actor, tenancy: Tenancy, function_name: impl Into<String>) -> FuncRun {
    let func_run_create_time = Utc::now();
    FuncRunBuilder::default()
        .actor(actor)
        .tenancy(tenancy)
        .component_id(None)
        .attribute_value_id(None)
        .backend_kind(FuncBackendKind::JsAction)
        .backend_response_type(FuncBackendResponseType::Action)
        .function_name(function_name.into())
        .function_kind(FuncKind::Action)
        .function_args_cas_address(ContentHash::default())
        .function_code_cas_address(ContentHash::default())
        .created_at(func_run_create_time)
        .updated_at(func_run_create_time)
        .build()
        .expect("could not build func run")
}
