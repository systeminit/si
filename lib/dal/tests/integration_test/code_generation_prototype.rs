use dal::{BillingAccountSignup, DalContext};

use crate::dal::test;
use dal::code_generation_prototype::CodeGenerationPrototypeContext;
use dal::func::backend::js_code_generation::FuncBackendJsCodeGenerationArgs;
use dal::{
    code_generation_prototype::UNSET_ID_VALUE, prototype_context::PrototypeContext,
    CodeGenerationPrototype, CodeLanguage, Component, Func, Schema, StandardModel,
};

#[test]
async fn new(ctx: &DalContext) {
    let name = "Kubernetes Deployment".to_string();
    let schema = Schema::find_by_attr(ctx, "name", &name)
        .await
        .expect("cannot find kubernetes deployment")
        .pop()
        .expect("no kubernetes deployment found");
    let (component, _node) = Component::new_for_schema_with_node(ctx, &name, schema.id())
        .await
        .expect("could not create component");

    let func_name = "si:generateYAML".to_string();
    let mut funcs = Func::find_by_attr(ctx, "name", &func_name)
        .await
        .expect("Error fetching builtin function");
    let func = funcs
        .pop()
        .expect("Missing builtin function si:generateYAML");

    let args = FuncBackendJsCodeGenerationArgs {
        component: component
            .veritech_code_generation_component(ctx, UNSET_ID_VALUE.into())
            .await
            .expect("could not create component code_generation view"),
    };

    let mut prototype_context = CodeGenerationPrototypeContext::new();
    prototype_context.set_component_id(*component.id());
    let _prototype = CodeGenerationPrototype::new(
        ctx,
        *func.id(),
        serde_json::to_value(&args).expect("serialization failed"),
        CodeLanguage::Yaml,
        prototype_context,
    )
    .await
    .expect("cannot create new prototype");
}

#[test]
async fn find_for_component(ctx: &DalContext, _nba: BillingAccountSignup) {
    // TODO: This test is brittle, because it relies on the behavior of kubernetes_deployment. I'm okay
    // with that for now, but not for long. If it breaks before we fix it - future person, I'm
    // sorry. ;)
    let name = "Kubernetes Deployment".to_string();
    let schema = Schema::find_by_attr(ctx, "name", &name)
        .await
        .expect("cannot find kubernetes deployment")
        .pop()
        .expect("no kubernetes deployment found");
    let default_schema_variant_id = schema
        .default_schema_variant_id()
        .expect("cannot get default schema variant id");

    let (component, _node) = Component::new_for_schema_with_node(ctx, "silverado", schema.id())
        .await
        .expect("cannot create new component");

    let mut found_prototype = CodeGenerationPrototype::find_for_component(
        ctx,
        *component.id(),
        *schema.id(),
        *default_schema_variant_id,
        UNSET_ID_VALUE.into(),
    )
    .await
    .expect("could not create component code_generation view");
    let _found = found_prototype
        .pop()
        .expect("found no code_generation prototypes");
}
