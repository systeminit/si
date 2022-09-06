use crate::dal::test;
use dal::{
    DalContext, Func, FuncBackendError, FuncBinding, FuncBindingError, StandardModel,
    WorkflowError, WorkflowTree, WorkflowView,
};
use serde_json::json;
use veritech::FunctionResult;

async fn fb(
    ctx: &DalContext<'_, '_, '_>,
    name: &str,
    args: serde_json::Value,
) -> serde_json::Value {
    let func = Func::find_by_attr(ctx, "name", &name)
        .await
        .expect("unable to find func")
        .pop()
        .unwrap_or_else(|| panic!("function not found: {}", name));
    let fb = FuncBinding::find_or_create(ctx, args, *func.id(), *func.backend_kind())
        .await
        .expect("unable to find_or_create func binding")
        .0;
    serde_json::to_value(fb).expect("unable to serialize func binding")
}

#[test]
async fn resolve(ctx: &DalContext<'_, '_, '_>) {
    let name = "si:poemWorkflow";
    let func = Func::find_by_attr(ctx, "name", &name)
        .await
        .expect("unable to find func")
        .pop()
        .unwrap_or_else(|| panic!("function not found: {}", name));
    let tree = WorkflowView::resolve(ctx, &func)
        .await
        .expect("unable to resolve workflow");
    // TODO: fix args propagation
    let expected_json = json!({
        "name": "si:poemWorkflow",
        "kind": "conditional",
        "steps": [
            //{
            //    "name": "si:exceptionalWorkflow",
            //    "kind": "exceptional",
            //    "steps": [
            //        { "func_binding": fb(ctx, "si:leroLeroTitle1Command", json!([])).await },
            //        { "func_binding": fb(ctx, "si:leroLeroTitle2Command", json!([])).await },
            //    ],
            //},
            { "func_binding": fb(ctx, "si:leroLeroStanza1Command", json!([])).await },
            { "func_binding": fb(ctx, "si:leroLeroStanza2Command", json!([])).await },
            { "func_binding": fb(ctx, "si:leroLeroStanza3Command", json!([])).await },
            { "func_binding": fb(ctx, "si:leroLeroStanza4Command", json!([])).await },
            { "func_binding": fb(ctx, "si:leroLeroStanza5Command", json!([])).await },
            { "func_binding": fb(ctx, "si:leroLeroStanza6Command", json!([])).await },
            { "func_binding": fb(ctx, "si:leroLeroStanza7Command", json!([])).await },
            {
                "name": "si:finalizingWorkflow",
                "kind": "parallel",
                "steps": [
                    { "func_binding": fb(ctx, "si:leroLeroQuestionCommand", json!([null])).await },
                    { "func_binding": fb(ctx, "si:leroLeroByeCommand", json!([])).await },
                ],
            },
        ],
    });
    let expected: WorkflowTree =
        serde_json::from_value(expected_json).expect("unable to serialize expected workflow tree");
    assert_eq!(tree, expected);
}

#[test]
async fn run(ctx: &DalContext<'_, '_, '_>) {
    let name = "si:poemWorkflow";
    let func = Func::find_by_attr(ctx, "name", &name)
        .await
        .expect("unable to find func")
        .pop()
        .unwrap_or_else(|| panic!("function not found: {}", name));
    let tree = WorkflowView::resolve(ctx, &func)
        .await
        .expect("unable to resolve workflow");
    // TODO: fix args propagation
    // TODO: confirm output
    tree.run(ctx).await.expect("unable to run workflow");
}

#[test]
async fn fail(ctx: &DalContext<'_, '_, '_>) {
    let name = "si:failureWorkflow";
    let func = Func::find_by_attr(ctx, "name", &name)
        .await
        .expect("unable to find func")
        .pop()
        .unwrap_or_else(|| panic!("function not found: {}", name));
    let tree = WorkflowView::resolve(ctx, &func)
        .await
        .expect("unable to resolve workflow");

    // TODO: fix args propagation
    // TODO: confirm output
    let err = tree.run(ctx).await.expect_err("no error found");
    if let WorkflowError::FuncBinding(FuncBindingError::FuncBackend(
        FuncBackendError::FunctionResultCommandRun(FunctionResult::Failure(e)),
    )) = &err
    {
        assert_eq!(e.error.message.as_str(), "oopsie!");
    } else {
        panic!("error found, but is unexpected {0:?}", err);
    }
}
