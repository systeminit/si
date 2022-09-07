use crate::dal::test;
use dal::{DalContext, Func, FuncBinding, StandardModel, WorkflowView};
use pretty_assertions_sorted::assert_eq;
use serde_json::json;

async fn fb(ctx: &DalContext, name: &str, args: serde_json::Value) -> serde_json::Value {
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
async fn resolve(ctx: &DalContext) {
    let name = "si:poemWorkflow";
    let func = Func::find_by_attr(ctx, "name", &name)
        .await
        .expect("unable to find func")
        .pop()
        .unwrap_or_else(|| panic!("function not found: {}", name));
    let tree = WorkflowView::resolve(
        ctx,
        &func,
        serde_json::Value::String("Domingos Passos".to_owned()),
    )
    .await
    .expect("unable to resolve workflow");
    let expected_json = json!({
        "name": "si:poemWorkflow",
        "kind": "conditional",
        "steps": [
            //{
            //    "name": "si:exceptionalWorkflow",
            //    "kind": "exceptional",
            //    "steps": [
            //        { "func_binding": fb(ctx, "si:leroLeroTitle1Command", serde_json::Value::Null).await },
            //        { "func_binding": fb(ctx, "si:leroLeroTitle2Command", serde_json::Value::Null).await },
            //    ],
            //},
            { "func_binding": fb(ctx, "si:leroLeroStanza1Command", serde_json::Value::Null).await },
            { "func_binding": fb(ctx, "si:leroLeroStanza2Command", serde_json::Value::Null).await },
            { "func_binding": fb(ctx, "si:leroLeroStanza3Command", serde_json::Value::Null).await },
            { "func_binding": fb(ctx, "si:leroLeroStanza4Command", serde_json::Value::Null).await },
            { "func_binding": fb(ctx, "si:leroLeroStanza5Command", serde_json::Value::Null).await },
            { "func_binding": fb(ctx, "si:leroLeroStanza6Command", serde_json::Value::Null).await },
            { "func_binding": fb(ctx, "si:leroLeroStanza7Command", serde_json::Value::Null).await },
            {
                "name": "si:finalizingWorkflow",
                "kind": "parallel",
                "steps": [
                    { "func_binding": fb(ctx, "si:leroLeroQuestionCommand", json!(["Domingos Passos".to_owned()])).await },
                    { "func_binding": fb(ctx, "si:leroLeroByeCommand", serde_json::Value::Null).await },
                ],
            },
        ],
    });
    assert_eq!(
        expected_json,
        serde_json::to_value(tree).expect("unable to serialize tree")
    );
}

#[test]
async fn run(ctx: &DalContext) {
    let name = "si:poemWorkflow";
    let func = Func::find_by_attr(ctx, "name", &name)
        .await
        .expect("unable to find func")
        .pop()
        .unwrap_or_else(|| panic!("function not found: {}", name));
    let tree = WorkflowView::resolve(
        ctx,
        &func,
        serde_json::Value::String("Domingos Passos".to_owned()),
    )
    .await
    .expect("unable to resolve workflow");

    // Text output is checked at WorkflowRunner tests as they actually order it
    tree.run(ctx).await.expect("unable to run workflow");
}
