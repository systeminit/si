use dal::{
    func::binding::FuncBinding, workflow_resolver::WorkflowResolverContext, DalContext, Func,
    StandardModel, WorkflowPrototypeId, WorkflowResolver,
};
use dal_test::test;

#[test]
async fn new(ctx: &DalContext) {
    let func_name = "si:poemWorkflow".to_string();
    let mut funcs = Func::find_by_attr(ctx, "name", &func_name)
        .await
        .expect("Error fetching builtin function");
    let func = funcs
        .pop()
        .expect("Missing builtin function si:poemWorkflow");

    let args = serde_json::Value::Null;
    let func_binding = FuncBinding::new(
        ctx,
        serde_json::to_value(args).expect("cannot turn args into json"),
        *func.id(),
        *func.backend_kind(),
    )
    .await
    .expect("cannot create function binding");
    func_binding
        .execute(ctx)
        .await
        .expect("failed to execute func binding");

    let workflow_resolver_context = WorkflowResolverContext::new();
    let _workflow_resolver = WorkflowResolver::new(
        ctx,
        WorkflowPrototypeId::NONE,
        *func.id(),
        *func_binding.id(),
        workflow_resolver_context,
    )
    .await
    .expect("cannot create new workflow resolver");
}

#[test]
async fn find_for_prototype(ctx: &DalContext) {
    let func_name = "si:poemWorkflow".to_string();
    let mut funcs = Func::find_by_attr(ctx, "name", &func_name)
        .await
        .expect("Error fetching builtin function");
    let func = funcs
        .pop()
        .expect("Missing builtin function si:poemWorkflow");

    let args = serde_json::Value::Null;
    let func_binding = FuncBinding::new(
        ctx,
        serde_json::to_value(args.clone()).expect("cannot turn args into json"),
        *func.id(),
        *func.backend_kind(),
    )
    .await
    .expect("cannot create function binding");
    func_binding
        .execute(ctx)
        .await
        .expect("failed to execute func binding");

    let resolver_context = WorkflowResolverContext::new();
    let created = WorkflowResolver::new(
        ctx,
        WorkflowPrototypeId::NONE,
        *func.id(),
        *func_binding.id(),
        resolver_context,
    )
    .await
    .expect("cannot create new workflow resolver");

    let mut found_resolvers = WorkflowResolver::find_for_prototype(
        ctx,
        &WorkflowPrototypeId::NONE,
        WorkflowResolverContext::default(),
    )
    .await
    .expect("cannot find resolvers");
    assert_eq!(found_resolvers.len(), 1);
    let found = found_resolvers.pop().expect("found no workflow resolvers");
    assert_eq!(created, found);
}
