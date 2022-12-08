use dal::{
    func::{
        argument::{FuncArgument, FuncArgumentKind},
        backend::string::FuncBackendStringArgs,
        binding::FuncBinding,
        binding_return_value::FuncBindingReturnValue,
        execution::FuncExecution,
    },
    generate_name, ChangeSet, ChangeSetPk, DalContext, Func, FuncBackendKind,
    FuncBackendResponseType, FuncId, HistoryActor, StandardModel, Visibility, WriteTenancy,
};
use dal_test::{
    test,
    test_harness::{
        create_change_set, create_func, create_func_binding, create_visibility_change_set,
    },
};
use strum::IntoEnumIterator;

#[test]
async fn new(ctx: &DalContext) {
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
async fn func_binding_new(ctx: &DalContext) {
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
async fn func_binding_find_or_create_head(ctx: &DalContext) {
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
async fn func_binding_find_or_create_change_set(ctx: &DalContext) {
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
        ChangeSetPk::NONE,
        "should not have a change set"
    );
}

#[test]
async fn func_binding_find_or_create_sha_diff_between_changeset_and_head(ctx: &DalContext) {
    let code_1 = Some("beep boop");
    let code_2 = Some("beep boop beep");
    let code_3 = Some("Ones and zeros");

    let mut func = create_func(ctx).await;
    func.set_code_plaintext(ctx, code_1)
        .await
        .expect("set func code");

    let args_str = "andara a Roma senza vedere il papa";
    let args_json = serde_json::to_value(FuncBackendStringArgs::new(args_str.into()))
        .expect("create args json");

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

    func.set_code_plaintext(ctx, code_2)
        .await
        .expect("change func code");
    let (change_set_func_binding_2, created) =
        FuncBinding::find_or_create(ctx, args_json.clone(), *func.id(), *func.backend_kind())
            .await
            .expect("cannot create func binding");
    assert!(
        created,
        "must create a new func binding when code has changed"
    );

    func.set_code_plaintext(ctx, code_1)
        .await
        .expect("change code");
    let (change_set_func_binding_again_2, created) =
        FuncBinding::find_or_create(ctx, args_json.clone(), *func.id(), *func.backend_kind())
            .await
            .expect("cannot create func binding");
    assert!(
        !created,
        "must not create a new func binding when one is present"
    );
    assert_ne!(
        change_set_func_binding_2, change_set_func_binding_again_2,
        "should return different func binding"
    );
    assert_eq!(
        change_set_func_binding, change_set_func_binding_again_2,
        "change sets match for matching funcs"
    );

    // Now we want to apply the changeset and confirm that after a code change we do
    // not get the previous func bindings
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
    assert_ne!(final_visibility.change_set_pk, ChangeSetPk::NONE);
    let foctx = ctx.clone_with_new_visibility(final_visibility);
    let ctx = &foctx;

    let (head_func_binding, created) =
        FuncBinding::find_or_create(ctx, args_json.clone(), *func.id(), *func.backend_kind())
            .await
            .expect("cannot create func binding");
    assert!(
        !created,
        "must not create a new func binding when one is present"
    );
    assert_eq!(
        head_func_binding.visibility().change_set_pk,
        ChangeSetPk::NONE,
        "should not have a change set"
    );

    dbg!(head_func_binding.code_sha256());

    let mut func = Func::get_by_id(ctx, func.id())
        .await
        .expect("refetch func")
        .expect("cannot got func");

    func.set_code_plaintext(ctx, code_3)
        .await
        .expect("cannot change code");

    let (change_set_func_binding, created) =
        FuncBinding::find_or_create(ctx, args_json.clone(), *func.id(), *func.backend_kind())
            .await
            .expect("cannot create func binding");

    dbg!(change_set_func_binding.code_sha256());
    assert!(
        created,
        "must create a new func binding when code has changed, even if one exists in head"
    );
}

#[test]
async fn func_binding_return_value_new(ctx: &DalContext) {
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
async fn func_binding_execute(ctx: &DalContext) {
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
async fn func_binding_execute_unset(ctx: &DalContext) {
    let name = dal_test::test_harness::generate_fake_name();
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

#[test]
async fn func_argument_new(ctx: &DalContext) {
    for kind in FuncArgumentKind::iter() {
        FuncArgument::new(ctx, generate_name(), kind, None, FuncId::generate())
            .await
            .expect("Could not create function argument with null argument kind");
        FuncArgument::new(ctx, generate_name(), kind, Some(kind), FuncId::generate())
            .await
            .expect("Could not create function argument with element kind");
    }
}

#[test]
async fn func_argument_list_for_func(ctx: &DalContext) {
    for kind in FuncArgumentKind::iter() {
        FuncArgument::new(ctx, generate_name(), kind, None, FuncId::generate())
            .await
            .expect("Could not create function argument with null argument kind");
    }

    let funcs = FuncArgument::list_for_func(ctx, FuncId::generate())
        .await
        .expect("Could not list func arguments for func");
    assert_eq!(7, funcs.len());
}

#[test]
async fn func_argument_find_by_name_for_func(ctx: &DalContext) {
    let mut ctx = ctx.clone_with_head();
    ctx.update_to_head();

    let name = "an_argument";
    let func_id = FuncId::generate();

    assert_eq!(
        None,
        FuncArgument::find_by_name_for_func(&ctx, name, func_id,)
            .await
            .expect("could not find_by_name_for_func")
    );

    assert!(FuncArgument::new(
        &ctx,
        name,
        FuncArgumentKind::String,
        None,
        FuncId::generate()
    )
    .await
    .expect("Could not create argument in head")
    .visibility()
    .is_head());

    ctx.update_visibility(Visibility::new_change_set(ChangeSetPk::generate(), false));

    FuncArgument::find_by_name_for_func(&ctx, name, func_id)
        .await
        .expect("could not find_by_name_for_func")
        .expect("should have found a func");

    let arg = FuncArgument::new(
        &ctx,
        name,
        FuncArgumentKind::String,
        None,
        FuncId::generate(),
    )
    .await
    .expect("Could not create argument in head");

    assert!(arg.visibility().in_change_set());
    assert_eq!(name, arg.name());
    assert_eq!(func_id, arg.func_id());
}
