use crate::dal::test;
use dal::qualification_prototype::QualificationPrototypeContext;
use dal::{
    component::ComponentKind,
    func::backend::validation::FuncBackendValidateStringValueArgs,
    generate_name,
    qualification_prototype::QualificationPrototypeContextField,
    socket::{Socket, SocketArity, SocketEdgeKind, SocketKind},
    validation_prototype::{ValidationPrototype, ValidationPrototypeContext},
    Component, DalContext, Func, FuncBackendKind, FuncBackendResponseType, Prop, PropKind,
    QualificationPrototype, Schema, SchemaError, SchemaKind, SchemaVariant, SchematicKind,
    StandardModel, SystemId,
};

#[test]
async fn new(ctx: &DalContext<'_, '_>) {
    let name = "docker_image".to_string();
    let schema = Schema::find_by_attr(ctx, "name", &name)
        .await
        .expect("cannot find docker image")
        .pop()
        .expect("no docker image found");
    let (component, _node) = Component::new_for_schema_with_node(ctx, &name, schema.id())
        .await
        .expect("could not create component");

    let func_name = "si:qualificationDockerImageNameInspect".to_string();
    let mut funcs = Func::find_by_attr(ctx, "name", &func_name)
        .await
        .expect("Error fetching builtin function");
    let func = funcs
        .pop()
        .expect("Missing builtin function si:qualificationDockerImageNameInspect");

    let mut prototype_context = QualificationPrototypeContext::new();
    prototype_context.set_component_id(*component.id());
    let _prototype = QualificationPrototype::new(ctx, *func.id(), prototype_context)
        .await
        .expect("cannot create new prototype");
}

#[test]
async fn find_for_component(ctx: &DalContext<'_, '_>) {
    // TODO: This test is brittle, because it relies on the behavior of docker_image. I'm okay
    // with that for now, but not for long. If it breaks before we fix it - future person, I'm
    // sorry. ;)

    let name = "docker_image".to_string();
    let schema = Schema::find_by_attr(ctx, "name", &name)
        .await
        .expect("cannot find docker image")
        .pop()
        .expect("no docker image found");
    let default_schema_variant_id = schema
        .default_schema_variant_id()
        .expect("cannot get default schema variant id");

    let (component, _node) = Component::new_for_schema_with_node(ctx, "silverado", schema.id())
        .await
        .expect("cannot create new component");

    let func = Func::find_by_attr(
        ctx,
        "name",
        &"si:qualificationDockerImageNameInspect".to_string(),
    )
    .await
    .expect("got func")
    .pop()
    .expect("cannot pop func off vec");

    let mut proto_context = QualificationPrototypeContext::new();
    proto_context.set_component_id(*component.id());
    let _second_proto = QualificationPrototype::new(ctx, *func.id(), proto_context)
        .await
        .expect("cannot create qualification_prototype");

    let mut found_prototypes = QualificationPrototype::find_for_component(
        ctx,
        *component.id(),
        *schema.id(),
        *default_schema_variant_id,
        SystemId::NONE,
    )
    .await
    .expect("could not create component qualification view");
    assert_eq!(found_prototypes.len(), 1);

    let found = found_prototypes
        .pop()
        .expect("found no qualification prototypes");

    assert_eq!(found.func_id(), *func.id());
}

#[test]
async fn associate_prototypes_with_func_and_objects(ctx: &DalContext<'_, '_>) {
    let mut schema = Schema::new(
        ctx,
        "dingue",
        &SchemaKind::Concrete,
        &ComponentKind::Standard,
    )
    .await
    .expect("could not create schema");

    let (farfelu, farfelu_root) = SchemaVariant::new(ctx, *schema.id(), "farfelu")
        .await
        .expect("could not create schema variant");
    let (cinglé, cinglé_root) = SchemaVariant::new(ctx, *schema.id(), "cinglé")
        .await
        .expect("could not create second schema variant");
    let (fou, fou_root) = SchemaVariant::new(ctx, *schema.id(), "fou")
        .await
        .expect("could not create third variant");

    schema
        .set_default_schema_variant_id(ctx, Some(*fou.id()))
        .await
        .expect("could not set default schema variant");

    for (variant, root_prop) in [
        (&farfelu, &farfelu_root),
        (&cinglé, &cinglé_root),
        (&fou, &fou_root),
    ] {
        // Is this the minimum for a schema variant that can have a component?
        let func_name = "si:validateStringEquals".to_string();
        let mut funcs = Func::find_by_attr(ctx, "name", &func_name).await.unwrap();
        let func = funcs
            .pop()
            .ok_or(SchemaError::MissingFunc(func_name))
            .unwrap();
        let mut validation_prototype_ctx = ValidationPrototypeContext::default();
        validation_prototype_ctx.set_schema_id(*schema.id());
        validation_prototype_ctx.set_schema_variant_id(*variant.id());
        let text_prop = Prop::new(ctx, "text", PropKind::String).await.unwrap();
        text_prop
            .set_parent_prop(ctx, root_prop.domain_prop_id)
            .await
            .unwrap();
        validation_prototype_ctx.set_prop_id(*text_prop.id());
        let _prototype = ValidationPrototype::new(
            ctx,
            *func.id(),
            serde_json::to_value(&FuncBackendValidateStringValueArgs::new(
                None,
                "Fou!".to_string(),
            ))
            .unwrap(),
            validation_prototype_ctx.clone(),
        )
        .await
        .unwrap();
        variant.finalize(ctx).await.expect("finalize variant");

        let includes_socket = Socket::new(
            ctx,
            "includes",
            SocketKind::Provider,
            &SocketEdgeKind::Includes,
            &SocketArity::Many,
            &SchematicKind::Component,
        )
        .await
        .expect("create includes socket");
        variant
            .add_socket(ctx, includes_socket.id())
            .await
            .expect("add socket");
    }

    let (component_farfelu, _) = Component::new_for_schema_variant_with_node(
        ctx,
        "Espérer, c'est démentir l'avenir",
        farfelu.id(),
    )
    .await
    .expect("could not create first component");

    let (component_cinglé, _) = Component::new_for_schema_variant_with_node(
        ctx,
        "La \"vie\" est une occupation d'insecte",
        cinglé.id(),
    )
    .await
    .expect("could not create second component");

    let (component_fou, _) = Component::new_for_schema_variant_with_node(
        ctx,
        "Le tort de la philosophie est d'être trop supportable",
        fou.id(),
    )
    .await
    .expect("could not create third component");

    let mut func = Func::new(
        ctx,
        generate_name(None),
        FuncBackendKind::JsQualification,
        FuncBackendResponseType::Qualification,
    )
    .await
    .expect("could not create func");

    func.set_handler(ctx, Some("qualification".to_owned()))
        .await
        .expect("could not set handler on func");

    func.set_code_plaintext(
        ctx,
        Some("function qualification(){return {qualified: true}}"),
    )
    .await
    .expect("could not set code on func");

    assert!(QualificationPrototype::find_for_func(ctx, func.id())
        .await
        .expect("could not get protos for func")
        .is_empty());

    // Ensure we delete old prototypes when we associate new ones
    for (variant, component) in [
        (&farfelu, &component_farfelu),
        (&cinglé, &component_cinglé),
        (&fou, &component_fou),
    ] {
        let associations: Vec<QualificationPrototypeContextField> =
            vec![(*variant.id()).into(), (*component.id()).into()];

        let _ = QualificationPrototype::associate_prototypes_with_func_and_objects(
            ctx,
            func.id(),
            &associations,
        )
        .await
        .expect("could not associate");

        let prototypes = QualificationPrototype::find_for_func(ctx, func.id())
            .await
            .expect("could not get protos for func");

        assert_eq!(2, prototypes.len());
        assert_eq!(*variant.id(), prototypes[0].schema_variant_id());
        assert_eq!(*component.id(), prototypes[1].component_id());
    }

    // clear prototypes
    let associations: Vec<QualificationPrototypeContextField> = vec![];
    let _ = QualificationPrototype::associate_prototypes_with_func_and_objects(
        ctx,
        func.id(),
        &associations,
    )
    .await
    .expect("could not clear associations");
    assert!(QualificationPrototype::find_for_func(ctx, func.id())
        .await
        .expect("could not get protos for func")
        .is_empty());
}
