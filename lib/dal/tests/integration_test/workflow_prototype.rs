use crate::dal::test;
use dal::{
    func::binding::FuncBinding, workflow_prototype::WorkflowPrototypeContext, ComponentId,
    DalContext, Func, SchemaId, SchemaVariantId, StandardModel, SystemId, WorkflowPrototype,
    WorkflowTree, WorkflowView,
};
use serde_json::json;

#[test]
async fn new(ctx: &DalContext<'_, '_, '_>) {
    let func_name = "si:poem".to_string();
    let mut funcs = Func::find_by_attr(ctx, "name", &func_name)
        .await
        .expect("Error fetching builtin function");
    let func = funcs.pop().expect("Missing builtin function si:poem");

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
    let func = Func::find_by_attr(ctx, "name", &"si:poem".to_string())
        .await
        .expect("got func")
        .pop()
        .expect("cannot pop func off vec");

    let proto_context = WorkflowPrototypeContext::new();
    let _second_proto = WorkflowPrototype::new(
        ctx,
        *func.id(),
        serde_json::Value::Null,
        proto_context,
        "prototype",
    )
    .await
    .expect("cannot create workflow_prototype");

    let mut found_prototypes = WorkflowPrototype::find_for_context(
        ctx,
        ComponentId::NONE,
        SchemaId::NONE,
        SchemaVariantId::NONE,
        SystemId::NONE,
    )
    .await
    .expect("could not find for context");
    assert_eq!(found_prototypes.len(), 2);

    let found = found_prototypes
        .pop()
        .expect("found no workflow prototypes");

    assert_eq!(found.func_id(), *func.id());
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
    let name = "si:poem";
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
    let expected: WorkflowTree =
        serde_json::from_value(expected_json).expect("unable to serialize expected workflow tree");
    assert_eq!(tree, expected);
}

#[test]
async fn run(ctx: &DalContext<'_, '_, '_>) {
    let name = "si:poem";
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
