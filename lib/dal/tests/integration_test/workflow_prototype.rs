use serde_json::json;

use dal::{
    func::binding::FuncBinding, workflow_prototype::WorkflowPrototypeContext, ComponentId,
    DalContext, Func, SchemaId, SchemaVariantId, StandardModel, SystemId, WorkflowPrototype,
    WorkflowTree, WorkflowView,
};

use crate::dal::test;

#[test]
async fn new(ctx: &DalContext<'_, '_, '_>) {
    let func_name = "si:poemWorkflow".to_string();
    let mut funcs = Func::find_by_attr(ctx, "name", &func_name)
        .await
        .expect("Error fetching builtin function");
    let func = funcs
        .pop()
        .expect("Missing builtin function si:poemWorkflow");

    let prototype_context = WorkflowPrototypeContext::new();
    let _prototype = WorkflowPrototype::new(
        ctx,
        *func.id(),
        serde_json::Value::Null,
        prototype_context,
        "prototype",
    )
    .await
    .expect("cannot create new prototype");
}

#[test]
async fn find_for_context(ctx: &DalContext<'_, '_, '_>) {
    let poem_func = Func::find_by_attr(ctx, "name", &"si:poemWorkflow".to_string())
        .await
        .expect("got func")
        .pop()
        .expect("cannot pop func off vec");

    let proto_context = WorkflowPrototypeContext::new();
    let new_prototype = WorkflowPrototype::new(
        ctx,
        *poem_func.id(),
        serde_json::Value::Null,
        proto_context,
        "prototype",
    )
    .await
    .expect("cannot create workflow_prototype");

    let found_prototypes = WorkflowPrototype::find_for_context(
        ctx,
        ComponentId::NONE,
        SchemaId::NONE,
        SchemaVariantId::NONE,
        SystemId::NONE,
    )
    .await
    .expect("could not find for context");

    let mut found_poem_prototype = false;
    let mut found_new_prototype = false;
    for prototype in found_prototypes {
        if prototype.func_id() == *poem_func.id() {
            found_poem_prototype = true;
            if found_new_prototype {
                break;
            }
        }
        if prototype.id() == new_prototype.id() {
            found_new_prototype = true;
            if found_poem_prototype {
                break;
            }
        }
    }
    assert!(found_poem_prototype);
    assert!(found_new_prototype);
}

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

    let prototype_context = WorkflowPrototypeContext::new();
    let prototype = WorkflowPrototype::new(
        ctx,
        *func.id(),
        serde_json::Value::Null,
        prototype_context,
        "prototype",
    )
    .await
    .expect("cannot create new prototype");
    let tree = prototype
        .resolve(ctx)
        .await
        .expect("unable to resolve prototype")
        .tree(ctx)
        .await
        .expect("unable to extract tree");

    // TODO: fix args propagation
    // TODO: confirm output
    tree.run(ctx).await.expect("unable to run workflow");
}
