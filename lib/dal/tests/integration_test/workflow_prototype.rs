use dal::{
    workflow_prototype::WorkflowPrototypeContext, ComponentId, ComponentView, DalContext, Func,
    Schema, SchemaId, SchemaVariantId, StandardModel, WorkflowKind, WorkflowPrototype,
    WorkflowTreeStep,
};
use dal_test::{test, test_harness::create_component_for_schema};
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

    let found_prototypes = WorkflowPrototype::find_for_context(
        ctx,
        ComponentId::NONE,
        SchemaId::NONE,
        SchemaVariantId::NONE,
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

#[test]
async fn resolve(ctx: &DalContext) {
    let title = "Refresh Docker Image";
    let prototype = WorkflowPrototype::find_by_attr(ctx, "title", &title)
        .await
        .expect("unable to find workflow by attr")
        .pop()
        .expect("unable to find docker image resource refresh workflow prototype");

    let schema = Schema::find_by_attr(ctx, "name", &"Docker Image")
        .await
        .expect("unable to find docker image schema")
        .pop()
        .expect("unable to find docker image");
    let component = create_component_for_schema(ctx, schema.id()).await;
    let component_view = ComponentView::new(ctx, *component.id())
        .await
        .expect("unable to generate component view for docker image component");
    let mut tree = prototype
        .resolve(ctx, *component.id())
        .await
        .expect("unable to resolve prototype")
        .tree(ctx)
        .await
        .expect("unable to extract tree");

    assert_eq!("si:dockerImageRefreshWorkflow", tree.name);
    assert_eq!(WorkflowKind::Conditional, tree.kind);
    assert_eq!(1, tree.steps.len());
    let step = tree.steps.pop().expect("Cannot get first step");
    match step {
        WorkflowTreeStep::Workflow(_) => panic!("Expected workflow step to be of kind Command"),
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
    let title = "Refresh Docker Image";
    let prototype = WorkflowPrototype::find_by_attr(&ctx, "title", &title)
        .await
        .expect("unable to find workflow by attr")
        .pop()
        .expect("unable to find docker image resource refresh workflow prototype");

    let schema = Schema::find_by_attr(&ctx, "name", &"Docker Image")
        .await
        .expect("unable to find docker image schema")
        .pop()
        .expect("unable to find docker image");
    let component = create_component_for_schema(&ctx, schema.id()).await;

    let tree = prototype
        .resolve(&ctx, *component.id())
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
