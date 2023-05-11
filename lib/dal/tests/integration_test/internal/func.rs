use dal::{
    func::{
        argument::{FuncArgument, FuncArgumentKind},
        backend::string::FuncBackendStringArgs,
        binding::FuncBinding,
        binding_return_value::FuncBindingReturnValue,
        execution::FuncExecution,
    },
    generate_name, ChangeSetPk, DalContext, Func, FuncBackendKind, FuncBackendResponseType, FuncId,
    StandardModel, Visibility,
};
use dal_test::{
    test,
    test_harness::{create_func, create_func_binding},
};
use strum::IntoEnumIterator;

mod description;
mod reconciliation;

#[test]
async fn new(ctx: &DalContext) {
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
    let func = create_func(ctx).await;
    let args = FuncBackendStringArgs::new("floop".to_string());
    let args_json = serde_json::to_value(args).expect("cannot serialize args to json");
    let _func_binding = FuncBinding::new(ctx, args_json, *func.id(), *func.backend_kind())
        .await
        .expect("cannot create func binding");
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
    let func_id = FuncId::generate();
    for (index, kind) in FuncArgumentKind::iter().enumerate() {
        FuncArgument::new(ctx, format!("poop {index}"), kind, None, func_id)
            .await
            .expect("Could not create function argument with null argument kind");
        FuncArgument::new(ctx, format!("canoe {index}"), kind, Some(kind), func_id)
            .await
            .expect("Could not create function argument with element kind");
    }
}

#[test]
async fn func_argument_list_for_func(ctx: &DalContext) {
    let func_id = FuncId::generate();
    for kind in FuncArgumentKind::iter() {
        FuncArgument::new(ctx, generate_name(), kind, None, func_id)
            .await
            .expect("Could not create function argument with null argument kind");
    }

    let funcs = FuncArgument::list_for_func(ctx, func_id)
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
        FuncArgument::find_by_name_for_func(&ctx, name, func_id)
            .await
            .expect("could not find_by_name_for_func")
    );

    assert!(
        FuncArgument::new(&ctx, name, FuncArgumentKind::String, None, func_id,)
            .await
            .expect("Could not create argument in head")
            .visibility()
            .is_head()
    );

    ctx.update_visibility(Visibility::new_change_set(ChangeSetPk::generate(), false));

    FuncArgument::find_by_name_for_func(&ctx, name, func_id)
        .await
        .expect("could not find_by_name_for_func")
        .expect("should have found a func");

    let arg = FuncArgument::new(&ctx, name, FuncArgumentKind::String, None, func_id)
        .await
        .expect("Could not create argument in head");

    assert!(arg.visibility().in_change_set());
    assert_eq!(name, arg.name());
    assert_eq!(func_id, arg.func_id());
}
