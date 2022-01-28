use crate::test_setup;

use dal::func::execution::FuncExecution;
use dal::{
    func::{
        backend::FuncBackendStringArgs, binding::FuncBinding,
        binding_return_value::FuncBindingReturnValue,
    },
    test_harness::{
        create_change_set, create_edit_session, create_func, create_func_binding,
        create_visibility_edit_session,
    },
    Func, FuncBackendKind, FuncBackendResponseType, HistoryActor, StandardModel, Tenancy,
    Visibility, NO_CHANGE_SET_PK, NO_EDIT_SESSION_PK,
};

#[tokio::test]
async fn new() {
    test_setup!(
        ctx,
        _secret_key,
        _pg,
        _conn,
        txn,
        _nats_conn,
        nats,
        _veritech,
    );
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let _func = Func::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        "poop",
        FuncBackendKind::String,
        FuncBackendResponseType::String,
    )
    .await
    .expect("cannot create func");
}

#[tokio::test]
async fn func_binding_new() {
    test_setup!(
        ctx,
        _secret_key,
        _pg,
        _conn,
        txn,
        _nats_conn,
        nats,
        _veritech,
    );
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let func = create_func(&txn, &nats, &tenancy, &visibility, &history_actor).await;
    let args = FuncBackendStringArgs::new("floop".to_string());
    let args_json = serde_json::to_value(args).expect("cannot serialize args to json");
    let _func_binding = FuncBinding::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        args_json,
        *func.id(),
        *func.backend_kind(),
    )
    .await
    .expect("cannot create func binding");
}

#[tokio::test]
async fn func_binding_find_or_create_head() {
    test_setup!(
        ctx,
        _secret_key,
        _pg,
        _conn,
        txn,
        _nats_conn,
        nats,
        _veritech,
    );
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let func = create_func(&txn, &nats, &tenancy, &visibility, &history_actor).await;
    let args = FuncBackendStringArgs::new("floop".to_string());
    let args_json = serde_json::to_value(args).expect("cannot serialize args to json");
    let (_func_binding, created) = FuncBinding::find_or_create(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        args_json.clone(),
        *func.id(),
        *func.backend_kind(),
    )
    .await
    .expect("cannot create func binding");
    assert_eq!(
        created, true,
        "must create a new func binding when one is absent"
    );

    let (_func_binding, created) = FuncBinding::find_or_create(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        args_json,
        *func.id(),
        *func.backend_kind(),
    )
    .await
    .expect("cannot create func binding");
    assert_eq!(
        created, false,
        "must not create a new func binding when one is present"
    );
}

#[tokio::test]
async fn func_binding_find_or_create_edit_session() {
    test_setup!(
        ctx,
        _secret_key,
        _pg,
        _conn,
        txn,
        _nats_conn,
        nats,
        _veritech,
    );
    let tenancy = Tenancy::new_universal();
    let history_actor = HistoryActor::SystemInit;
    let mut change_set = create_change_set(&txn, &nats, &tenancy, &history_actor).await;
    let mut edit_session = create_edit_session(&txn, &nats, &history_actor, &change_set).await;
    let edit_session_visibility = create_visibility_edit_session(&change_set, &edit_session);

    let func = create_func(
        &txn,
        &nats,
        &tenancy,
        &edit_session_visibility,
        &history_actor,
    )
    .await;
    let args = FuncBackendStringArgs::new("floop".to_string());
    let args_json = serde_json::to_value(args).expect("cannot serialize args to json");
    let (edit_session_func_binding, created) = FuncBinding::find_or_create(
        &txn,
        &nats,
        &tenancy,
        &edit_session_visibility,
        &history_actor,
        args_json.clone(),
        *func.id(),
        *func.backend_kind(),
    )
    .await
    .expect("cannot create func binding");
    assert_eq!(
        created, true,
        "must create a new func binding when one is absent"
    );

    let (edit_session_func_binding_again, created) = FuncBinding::find_or_create(
        &txn,
        &nats,
        &tenancy,
        &edit_session_visibility,
        &history_actor,
        args_json.clone(),
        *func.id(),
        *func.backend_kind(),
    )
    .await
    .expect("cannot create func binding");
    assert_eq!(
        created, false,
        "must not create a new func binding when one is present"
    );
    assert_eq!(
        edit_session_func_binding, edit_session_func_binding_again,
        "should return the identical func binding"
    );

    edit_session
        .save(&txn, &nats, &history_actor)
        .await
        .expect("cannot save edit session");

    let second_edit_session = create_edit_session(&txn, &nats, &history_actor, &change_set).await;
    let second_edit_session_visibility =
        create_visibility_edit_session(&change_set, &second_edit_session);
    let (change_set_func_binding, created) = FuncBinding::find_or_create(
        &txn,
        &nats,
        &tenancy,
        &second_edit_session_visibility,
        &history_actor,
        args_json.clone(),
        *func.id(),
        *func.backend_kind(),
    )
    .await
    .expect("cannot create func binding");
    assert_eq!(
        created, false,
        "must not create a new func binding when one is present"
    );
    assert_eq!(
        change_set_func_binding.visibility().edit_session_pk,
        NO_EDIT_SESSION_PK,
        "should return the identical func binding"
    );

    change_set
        .apply(&txn, &nats, &history_actor)
        .await
        .expect("cannot apply change set");

    let final_change_set = create_change_set(&txn, &nats, &tenancy, &history_actor).await;
    let final_edit_session =
        create_edit_session(&txn, &nats, &history_actor, &final_change_set).await;
    let final_visibility = create_visibility_edit_session(&final_change_set, &final_edit_session);
    let (head_func_binding, created) = FuncBinding::find_or_create(
        &txn,
        &nats,
        &tenancy,
        &final_visibility,
        &history_actor,
        args_json,
        *func.id(),
        *func.backend_kind(),
    )
    .await
    .expect("cannot create func binding");
    assert_eq!(
        created, false,
        "must not create a new func binding when one is present"
    );
    assert_eq!(
        head_func_binding.visibility().edit_session_pk,
        NO_EDIT_SESSION_PK,
        "should not have an edit session"
    );
    assert_eq!(
        head_func_binding.visibility().change_set_pk,
        NO_CHANGE_SET_PK,
        "should not have a change set"
    );
}

#[tokio::test]
async fn func_binding_return_value_new() {
    test_setup!(
        ctx,
        _secret_key,
        _pg,
        _conn,
        txn,
        _nats_conn,
        nats,
        _veritech,
    );
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let func = create_func(&txn, &nats, &tenancy, &visibility, &history_actor).await;
    let args = FuncBackendStringArgs::new("funky".to_string());
    let args_json = serde_json::to_value(args).expect("cannot serialize args to json");

    let func_binding = create_func_binding(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        args_json,
        *func.id(),
        *func.backend_kind(),
    )
    .await;

    let execution = FuncExecution::new(&txn, &nats, &tenancy, &func, &func_binding)
        .await
        .expect("cannot create a new func execution");

    let _func_binding_return_value = FuncBindingReturnValue::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        Some(serde_json::json!("funky")),
        Some(serde_json::json!("funky")),
        *func.id(),
        *func_binding.id(),
        execution.pk(),
    )
    .await
    .expect("failed to create return value");
}

#[tokio::test]
async fn func_binding_execute() {
    test_setup!(ctx, _secret_key, _pg, _conn, txn, nats_conn, nats, veritech,);
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let func = create_func(&txn, &nats, &tenancy, &visibility, &history_actor).await;
    let args = serde_json::to_value(FuncBackendStringArgs::new("funky".to_string()))
        .expect("cannot serialize args to json");

    let func_binding = create_func_binding(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        args,
        *func.id(),
        *func.backend_kind(),
    )
    .await;

    let return_value = func_binding
        .execute(&txn, &nats, veritech)
        .await
        .expect("failed to execute func binding");
    assert_eq!(return_value.value(), Some(&serde_json::json!["funky"]));
    assert_eq!(
        return_value.unprocessed_value(),
        Some(&serde_json::json!["funky"])
    );
}

#[tokio::test]
async fn func_binding_execute_unset() {
    test_setup!(ctx, _secret_key, _pg, _conn, txn, nats_conn, nats, veritech,);
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let name = dal::test_harness::generate_fake_name();
    let func = Func::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        name,
        FuncBackendKind::Unset,
        FuncBackendResponseType::Unset,
    )
    .await
    .expect("cannot create func");
    let args = serde_json::json!({});

    let func_binding = create_func_binding(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        args,
        *func.id(),
        *func.backend_kind(),
    )
    .await;

    let return_value = func_binding
        .execute(&txn, &nats, veritech)
        .await
        .expect("failed to execute func binding");
    assert_eq!(return_value.value(), None);
    assert_eq!(return_value.unprocessed_value(), None,);
}
