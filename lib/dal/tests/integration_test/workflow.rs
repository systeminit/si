use crate::dal::test;
use dal::{DalContext, Func, FuncBinding, StandardModel, WorkflowTree, WorkflowView};
use pretty_assertions_sorted::assert_eq_sorted;
use serde_json::json;

async fn fb(ctx: &DalContext<'_, '_>, name: &str, args: serde_json::Value) -> serde_json::Value {
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
async fn resolve(ctx: &DalContext<'_, '_>) {
    let tree = WorkflowView::resolve(ctx, "si:poem")
        .await
        .expect("unable to resolve workflow");
    // TODO: fix args propagation
    let expected_json = json!({
        "name": "si:poem",
        "kind": "conditional",
        "steps": [
            //{
            //    "name": "si:exceptional",
            //    "kind": "exceptional",
            //    "steps": [
            //        { "func_binding": fb(ctx, "si:title", json!([])).await },
            //        { "func_binding": fb(ctx, "si:title2", json!([])).await },
            //    ],
            //},
            { "func_binding": fb(ctx, "si:firstStanza", json!([])).await },
            { "func_binding": fb(ctx, "si:secondStanza", json!([])).await },
            { "func_binding": fb(ctx, "si:thirdStanza", json!([])).await },
            { "func_binding": fb(ctx, "si:fourthStanza", json!([])).await },
            { "func_binding": fb(ctx, "si:fifthStanza", json!([])).await },
            { "func_binding": fb(ctx, "si:sixthStanza", json!([])).await },
            { "func_binding": fb(ctx, "si:seventhStanza", json!([])).await },
            {
                "name": "si:finalizing",
                "kind": "parallel",
                "steps": [
                    { "func_binding": fb(ctx, "si:question", json!([null])).await },
                    { "func_binding": fb(ctx, "si:bye", json!([])).await },
                ],
            },
        ],
    });
    assert_eq_sorted!(
        &expected_json,
        &serde_json::to_value(tree.clone()).expect("bro")
    );
    let expected: WorkflowTree =
        serde_json::from_value(expected_json).expect("unable to serialize expected workflow tree");
    assert_eq!(tree, expected);
}

#[test]
async fn run(ctx: &DalContext<'_, '_>) {
    let tree = WorkflowView::resolve(ctx, "si:poem")
        .await
        .expect("unable to resolve workflow");
    // TODO: fix args propagation
    // TODO: confirm output
    tree.run(ctx).await.expect("unable to run workflow");
}
