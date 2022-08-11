use dal::test_harness::create_visibility_change_set;
use dal::{ChangeSet, DalContext};

use crate::dal::test;
use dal::func::execution::FuncExecution;
use dal::{
    func::{
        backend::string::FuncBackendStringArgs, binding::FuncBinding,
        binding_return_value::FuncBindingReturnValue,
    },
    test_harness::{create_change_set, create_func, create_func_binding},
    Func, FuncBackendKind, FuncBackendResponseType, HistoryActor, StandardModel, Visibility,
    WriteTenancy, NO_CHANGE_SET_PK,
};

#[test]
async fn new(ctx: &DalContext<'_, '_>) {
    let _write_tenancy = WriteTenancy::new_universal();
    let _visibility = Visibility::new_head(false);
    let _history_actor = HistoryActor::SystemInit;
    let _func = Func::new(
        ctx,
        "poop",
        FuncBackendKind::String,
        FuncBackendResponseType::String,
    )
    .await
    .expect("cannot create func");
}

#[test]
async fn func_binding_new(ctx: &DalContext<'_, '_>) {
    let _write_tenancy = WriteTenancy::new_universal();
    let _visibility = Visibility::new_head(false);
    let _history_actor = HistoryActor::SystemInit;
    let func = create_func(ctx).await;
    let args = FuncBackendStringArgs::new("floop".to_string());
    let args_json = serde_json::to_value(args).expect("cannot serialize args to json");
    let _func_binding = FuncBinding::new(ctx, args_json, *func.id(), *func.backend_kind())
        .await
        .expect("cannot create func binding");
}

#[test]
async fn func_binding_find_or_create_head(ctx: &DalContext<'_, '_>) {
    let _write_tenancy = WriteTenancy::new_universal();
    let _visibility = Visibility::new_head(false);
    let _history_actor = HistoryActor::SystemInit;
    let func = create_func(ctx).await;
    let args = FuncBackendStringArgs::new("floop".to_string());
    let args_json = serde_json::to_value(args).expect("cannot serialize args to json");
    let (_func_binding, created) =
        FuncBinding::find_or_create(ctx, args_json.clone(), *func.id(), *func.backend_kind())
            .await
            .expect("cannot create func binding");
    assert!(created, "must create a new func binding when one is absent");

    let (_func_binding, created) =
        FuncBinding::find_or_create(ctx, args_json, *func.id(), *func.backend_kind())
            .await
            .expect("cannot create func binding");
    assert!(
        !created,
        "must not create a new func binding when one is present"
    );
}

#[test]
async fn func_binding_find_or_create_change_set(ctx: &DalContext<'_, '_>) {
    let func = create_func(ctx).await;
    let args = FuncBackendStringArgs::new("floop".to_string());
    let args_json = serde_json::to_value(args).expect("cannot serialize args to json");
    let (change_set_func_binding, created) =
        FuncBinding::find_or_create(ctx, args_json.clone(), *func.id(), *func.backend_kind())
            .await
            .expect("cannot create func binding");
    assert!(created, "must create a new func binding when one is absent");

    let (change_set_func_binding_again, created) =
        FuncBinding::find_or_create(ctx, args_json.clone(), *func.id(), *func.backend_kind())
            .await
            .expect("cannot create func binding");
    assert!(
        !created,
        "must not create a new func binding when one is present"
    );
    assert_eq!(
        change_set_func_binding, change_set_func_binding_again,
        "should return the identical func binding"
    );

    let mut change_set = ChangeSet::get_by_pk(ctx, &ctx.visibility().change_set_pk)
        .await
        .expect("cannot get change set by pk; bug")
        .expect("expected a change set, but none were found");
    change_set
        .apply(ctx)
        .await
        .expect("cannot apply change set");

    let final_change_set = create_change_set(ctx).await;
    let final_visibility = create_visibility_change_set(&final_change_set);
    let foctx = ctx.clone_with_new_visibility(final_visibility);
    let ctx = &foctx;

    let (head_func_binding, created) =
        FuncBinding::find_or_create(ctx, args_json, *func.id(), *func.backend_kind())
            .await
            .expect("cannot create func binding");
    assert!(
        !created,
        "must not create a new func binding when one is present"
    );
    assert_eq!(
        head_func_binding.visibility().change_set_pk,
        NO_CHANGE_SET_PK,
        "should not have a change set"
    );
}

#[test]
async fn func_binding_return_value_new(ctx: &DalContext<'_, '_>) {
    let func = create_func(ctx).await;
    let args = FuncBackendStringArgs::new("funky".to_string());
    let args_json = serde_json::to_value(args).expect("cannot serialize args to json");

    let func_binding = create_func_binding(ctx, args_json, *func.id(), *func.backend_kind()).await;

    let execution = FuncExecution::new(ctx, &func, &func_binding)
        .await
        .expect("cannot create a new func execution");

    let _func_binding_return_value = FuncBindingReturnValue::new(
        ctx,
        Some(serde_json::json!("funky")),
        Some(serde_json::json!("funky")),
        *func.id(),
        *func_binding.id(),
        execution.pk(),
    )
    .await
    .expect("failed to create return value");
}

#[test]
async fn func_binding_execute(ctx: &DalContext<'_, '_>) {
    let func = create_func(ctx).await;
    let args = serde_json::to_value(FuncBackendStringArgs::new("funky".to_string()))
        .expect("cannot serialize args to json");

    let func_binding = create_func_binding(ctx, args, *func.id(), *func.backend_kind()).await;

    let return_value = func_binding
        .execute(ctx)
        .await
        .expect("failed to execute func binding");
    assert_eq!(return_value.value(), Some(&serde_json::json!["funky"]));
    assert_eq!(
        return_value.unprocessed_value(),
        Some(&serde_json::json!["funky"])
    );
}

#[test]
async fn func_binding_execute_unset(ctx: &DalContext<'_, '_>) {
    let name = dal::test_harness::generate_fake_name();
    let func = Func::new(
        ctx,
        name,
        FuncBackendKind::Unset,
        FuncBackendResponseType::Unset,
    )
    .await
    .expect("cannot create func");
    let args = serde_json::json!({});

    let func_binding = create_func_binding(ctx, args, *func.id(), *func.backend_kind()).await;

    let return_value = func_binding
        .execute(ctx)
        .await
        .expect("failed to execute func binding");
    assert_eq!(return_value.value(), None);
    assert_eq!(return_value.unprocessed_value(), None,);
}
