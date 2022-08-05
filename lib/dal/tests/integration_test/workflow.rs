use crate::dal::test;
use dal::{DalContext, WorkflowTree, WorkflowView};

#[test]
async fn resolve(ctx: &DalContext<'_, '_>) {
    let tree = WorkflowView::resolve(ctx, "si:poem")
        .await
        .expect("unable to resolve workflow");
    // TODO: fix args propagation
    let expected: WorkflowTree = serde_json::from_value(serde_json::json!({
        "name": "si:poem",
        "kind": "conditional",
        "steps": [
            {
                "name": "si:exceptional",
                "kind": "exceptional",
                "steps": [
                    { "func": "si:title" },
                    { "func": "si:title2" },
                ],
            },
            { "func": "si:firstStanza" },
            { "func": "si:secondStanza" },
            { "func": "si:thirdStanza" },
            { "func": "si:fourthStanza" },
            { "func": "si:fifthStanza" },
            { "func": "si:sixthStanza" },
            { "func": "si:seventhStanza" },
            {
                "name": "si:finalizing",
                "kind": "parallel",
                "args": [null],
                "steps": [
                    { "func": "si:question", "args": [null] },
                    { "func": "si:bye" },
                ],
            },
        ],
    }))
    .expect("unable to serialize expected workflow tree");
    assert_eq!(tree, expected);
}
