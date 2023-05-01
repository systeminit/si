use dal::{
    workflow_prototype::WorkflowPrototypeContext, DalContext, Func, StandardModel, WorkflowKind,
    WorkflowPrototype, WorkflowTreeStep,
};
use dal_test::helpers::component_bag::ComponentBagger;
use dal_test::test;
use pretty_assertions_sorted::assert_eq;
use serde_json::json;

#[test]
async fn new(ctx: &DalContext) {
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
async fn find_for_context(ctx: &DalContext) {
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

    let found_prototypes =
        WorkflowPrototype::find_for_context(ctx, WorkflowPrototypeContext::default())
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

#[test]
async fn resolve(ctx: &DalContext) {
    let title = "Refresh Starfield";
    let prototype = WorkflowPrototype::find_by_attr(ctx, "title", &title)
        .await
        .expect("unable to find workflow by attr")
        .pop()
        .expect("unable to find refresh workflow prototype");

    let mut bagger = ComponentBagger::new();
    let component_bag = bagger.create_component(ctx, "starfield", "starfield").await;

    let component_view = component_bag.component_view(ctx).await;

    let mut tree = prototype
        .resolve(ctx, component_bag.component_id)
        .await
        .expect("unable to resolve prototype")
        .tree(ctx)
        .await
        .expect("unable to extract tree");

    assert_eq!("test:refreshWorkflowStarfield", tree.name);
    assert_eq!(WorkflowKind::Conditional, tree.kind);
    assert_eq!(1, tree.steps.len());
    let step = tree.steps.pop().expect("cannot get first step");
    match step {
        WorkflowTreeStep::Workflow(_) => panic!("expected workflow step to be of kind Command"),
        WorkflowTreeStep::Command { func_binding } => {
            assert_eq!(
                json!([serde_json::to_value(component_view)
                    .expect("unable to serialize component view")]),
                *func_binding.args(),
            );
        }
    }
}

#[test]
async fn run(ctx: DalContext) {
    let title = "Refresh Starfield";
    let prototype = WorkflowPrototype::find_by_attr(&ctx, "title", &title)
        .await
        .expect("unable to find workflow by attr")
        .pop()
        .expect("unable to find refresh workflow prototype");

    let mut bagger = ComponentBagger::new();
    let component_bag = bagger
        .create_component(&ctx, "starfield", "starfield")
        .await;

    let tree = prototype
        .resolve(&ctx, component_bag.component_id)
        .await
        .expect("unable to resolve prototype")
        .tree(&ctx)
        .await
        .expect("unable to extract tree");

    // Needed as workflow run create new transactions
    ctx.blocking_commit()
        .await
        .expect("unable to commit transaction");

    tree.run(&ctx, 0).await.expect("unable to run workflow");
}
