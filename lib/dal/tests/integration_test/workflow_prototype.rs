use serde_json::json;

use dal::{
    func::binding::FuncBinding, test_harness::create_component_for_schema,
    workflow_prototype::WorkflowPrototypeContext, AttributeReadContext, ComponentId, ComponentView,
    DalContext, Func, Schema, SchemaId, SchemaVariantId, StandardModel, SystemId,
    WorkflowPrototype,
};
use pretty_assertions_sorted::assert_eq;

use crate::dal::test;

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
    let title = "Docker Image Resource Create";
    let prototype = WorkflowPrototype::find_by_attr(ctx, "title", &title)
        .await
        .expect("unable to find workflow by attr")
        .pop()
        .expect("unable to find docker image resource create workflow prototype");

    let schema = Schema::find_by_attr(ctx, "name", &"Docker Image")
        .await
        .expect("unable to find docker image schema")
        .pop()
        .expect("unable to find docker image");
    let schema_variant = schema
        .default_variant(ctx)
        .await
        .expect("unable to find default schema variant");
    let component = create_component_for_schema(ctx, schema.id()).await;

    let context = AttributeReadContext {
        prop_id: None,
        schema_id: Some(*schema.id()),
        schema_variant_id: Some(*schema_variant.id()),
        component_id: Some(*component.id()),
        system_id: Some(SystemId::NONE),
        ..AttributeReadContext::default()
    };

    let component_view = ComponentView::for_context(ctx, context)
        .await
        .expect("unable to generate component view for docker image component");
    let tree = prototype
        .resolve(ctx, *component.id())
        .await
        .expect("unable to resolve prototype")
        .tree(ctx)
        .await
        .expect("unable to extract tree");

    let expected_json = json!({
        "name": "si:dockerImageCreateWorkflow",
        "kind": "conditional",
        "steps": [
            { "func_binding": fb(ctx, "si:dockerImageCreateCommand", json!([ serde_json::to_value(component_view).expect("unable to serialize component view") ])).await },
        ],
    });
    assert_eq!(
        serde_json::to_value(tree).expect("unable to serialize tree"),
        expected_json
    );
}

#[test]
async fn run(ctx: &DalContext) {
    let title = "Docker Image Resource Create";
    let prototype = WorkflowPrototype::find_by_attr(ctx, "title", &title)
        .await
        .expect("unable to find workflow by attr")
        .pop()
        .expect("unable to find docker image resource create workflow prototype");

    let schema = Schema::find_by_attr(ctx, "name", &"Docker Image")
        .await
        .expect("unable to find docker image schema")
        .pop()
        .expect("unable to find docker image");
    let component = create_component_for_schema(ctx, schema.id()).await;

    let tree = prototype
        .resolve(ctx, *component.id())
        .await
        .expect("unable to resolve prototype")
        .tree(ctx)
        .await
        .expect("unable to extract tree");

    tree.run(ctx, 0).await.expect("unable to run workflow");
}
