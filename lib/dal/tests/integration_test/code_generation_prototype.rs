use dal::func::argument::FuncArgument;
use dal::{
    CodeGenerationPrototype, CodeLanguage, Component, DalContext, Func, Schema, StandardModel,
};
use dal_test::test;

#[test]
async fn new_and_list_for_schema_variant(ctx: &DalContext) {
    let name = "Kubernetes Deployment".to_string();
    let schema = Schema::find_by_attr(ctx, "name", &name)
        .await
        .expect("cannot find kubernetes deployment")
        .pop()
        .expect("no kubernetes deployment found");
    let (component, _node) = Component::new_for_schema_with_node(ctx, &name, schema.id())
        .await
        .expect("could not create component");
    let schema_variant = component
        .schema_variant(ctx)
        .await
        .expect("cannot find schema variant for component")
        .expect("no schema variant for component");

    let func_name = "si:generateYAML".to_string();
    let mut funcs = Func::find_by_attr(ctx, "name", &func_name)
        .await
        .expect("Error fetching builtin function");
    let func = funcs
        .pop()
        .expect("Missing builtin function si:generateYAML");
    let code_generation_func_argument =
        FuncArgument::find_by_name_for_func(ctx, "domain", *func.id())
            .await
            .expect("could not perform func argument find")
            .expect("no func argument found");

    CodeGenerationPrototype::new(
        ctx,
        *func.id(),
        *code_generation_func_argument.id(),
        *schema_variant.id(),
        CodeLanguage::Yaml,
    )
    .await
    .expect("cannot create new prototype");

    let mut found_prototypes =
        CodeGenerationPrototype::list_for_schema_variant(ctx, *schema_variant.id())
            .await
            .expect("could not create component code_generation view");
    let _found = found_prototypes.pop().expect("found no code prototypes");
}
