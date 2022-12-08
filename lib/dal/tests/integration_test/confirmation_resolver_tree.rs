use dal::action_prototype::ActionKind;
use dal::component::ComponentKind;
use dal::{
    edge::EdgeKind, edge::EdgeObjectId, edge::VertexObjectKind, socket::SocketArity,
    ActionPrototype, ActionPrototypeContext, BuiltinsResult, ConfirmationPrototype,
    ConfirmationPrototypeContext, ConfirmationResolverTree, DalContext, DiagramKind, Edge,
    ExternalProvider, Func, HasPrototypeContext, InternalProvider, Schema, SchemaError, SchemaKind,
    SchemaVariant, Socket, StandardModel, WorkflowPrototypeId,
};
use dal_test::helpers::setup_identity_func;
use dal_test::{helpers::create_component_and_node_for_schema, test};

async fn create_dummy_schema(ctx: &DalContext) -> BuiltinsResult<(Schema, Socket, Socket)> {
    let mut schema = Schema::new(
        ctx,
        "Dummy Schema",
        &SchemaKind::Configuration,
        &ComponentKind::Standard,
    )
    .await?;
    let (schema_variant, _root_prop) = SchemaVariant::new(ctx, *schema.id(), "v0").await?;
    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await?;

    let (
        identity_func_id,
        identity_func_binding_id,
        identity_func_binding_return_value_id,
        _identity_func_identity_arg_id,
    ) = setup_identity_func(ctx).await;

    let (_schema_explicit_internal_provider, input_socket) =
        InternalProvider::new_explicit_with_socket(
            ctx,
            *schema_variant.id(),
            "Schema",
            identity_func_id,
            identity_func_binding_id,
            identity_func_binding_return_value_id,
            SocketArity::Many,
            DiagramKind::Configuration,
        )
        .await?;

    let (_schema_external_provider, output_socket) = ExternalProvider::new_with_socket(
        ctx,
        *schema.id(),
        *schema_variant.id(),
        "Schema",
        None,
        identity_func_id,
        identity_func_binding_id,
        identity_func_binding_return_value_id,
        SocketArity::Many,
        DiagramKind::Configuration,
    )
    .await?;

    schema_variant.finalize(ctx).await?;

    let func_name = "si:resourceExistsConfirmation";
    let func = Func::find_by_attr(ctx, "name", &func_name)
        .await?
        .pop()
        .ok_or_else(|| SchemaError::FuncNotFound(func_name.to_owned()))?;
    let context = ConfirmationPrototypeContext {
        schema_id: *schema.id(),
        schema_variant_id: *schema_variant.id(),
        ..Default::default()
    };
    ConfirmationPrototype::new(ctx, "Has resource?", *func.id(), context).await?;

    let name = "create";
    let context = ActionPrototypeContext {
        schema_id: *schema.id(),
        schema_variant_id: *schema_variant.id(),
        ..Default::default()
    };
    ActionPrototype::new(
        ctx,
        WorkflowPrototypeId::NONE,
        name,
        ActionKind::Other,
        context,
    )
    .await?;

    Ok((schema, input_socket, output_socket))
}

#[test]
async fn new(ctx: &DalContext) {
    let (schema, input_socket, output_socket) = create_dummy_schema(ctx)
        .await
        .expect("unable to create dummy schema");

    let (first, first_node) = create_component_and_node_for_schema(ctx, schema.id()).await;
    let (first2, _first2_node) = create_component_and_node_for_schema(ctx, schema.id()).await;
    let (second, second_node) = create_component_and_node_for_schema(ctx, schema.id()).await;
    let (second2, second2_node) = create_component_and_node_for_schema(ctx, schema.id()).await;
    let (third, third_node) = create_component_and_node_for_schema(ctx, schema.id()).await;
    let (third2, third2_node) = create_component_and_node_for_schema(ctx, schema.id()).await;

    Edge::new(
        ctx,
        EdgeKind::Configuration,
        *second_node.id(),
        VertexObjectKind::Configuration,
        EdgeObjectId::from(*second.id()),
        *output_socket.id(),
        *first_node.id(),
        VertexObjectKind::Configuration,
        EdgeObjectId::from(*first.id()),
        *input_socket.id(),
    )
    .await
    .expect("unable to create connection");
    Edge::new(
        ctx,
        EdgeKind::Configuration,
        *second2_node.id(),
        VertexObjectKind::Configuration,
        EdgeObjectId::from(*second2.id()),
        *output_socket.id(),
        *first_node.id(),
        VertexObjectKind::Configuration,
        EdgeObjectId::from(*first.id()),
        *input_socket.id(),
    )
    .await
    .expect("unable to create connection");
    Edge::new(
        ctx,
        EdgeKind::Configuration,
        *third_node.id(),
        VertexObjectKind::Configuration,
        EdgeObjectId::from(*third.id()),
        *output_socket.id(),
        *first_node.id(),
        VertexObjectKind::Configuration,
        EdgeObjectId::from(*second.id()),
        *input_socket.id(),
    )
    .await
    .expect("unable to create connection");
    Edge::new(
        ctx,
        EdgeKind::Configuration,
        *third2_node.id(),
        VertexObjectKind::Configuration,
        EdgeObjectId::from(*third2.id()),
        *output_socket.id(),
        *first_node.id(),
        VertexObjectKind::Configuration,
        EdgeObjectId::from(*second2.id()),
        *input_socket.id(),
    )
    .await
    .expect("unable to create connection");

    let prototypes = ConfirmationPrototype::list(ctx)
        .await
        .expect("unable to list confirmation prototypes");

    let mut resolvers = Vec::new();
    for prototype in prototypes {
        if prototype.context().schema_id == *schema.id() {
            // Creates in random order so we ensure there is no dependency on the creation order
            resolvers.push(
                prototype
                    .run(ctx, *third2.id())
                    .await
                    .expect("unable to run prototype"),
            );
            resolvers.push(
                prototype
                    .run(ctx, *first.id())
                    .await
                    .expect("unable to run prototype"),
            );
            resolvers.push(
                prototype
                    .run(ctx, *second2.id())
                    .await
                    .expect("unable to run prototype"),
            );
            resolvers.push(
                prototype
                    .run(ctx, *first2.id())
                    .await
                    .expect("unable to run prototype"),
            );
            resolvers.push(
                prototype
                    .run(ctx, *third.id())
                    .await
                    .expect("unable to run prototype"),
            );
            resolvers.push(
                prototype
                    .run(ctx, *second.id())
                    .await
                    .expect("unable to run prototype"),
            );
        }
    }

    let tree = ConfirmationResolverTree::build(ctx, resolvers)
        .await
        .expect("unable to build confirmation resolver tree");
    let ids: Vec<_> = tree
        .into_vec()
        .expect("unable to convert tree into vec")
        .into_iter()
        .map(|r| r.context().component_id)
        .collect();
    assert_eq!(
        ids,
        vec![
            *first.id(),
            *first2.id(),
            *second2.id(),
            *second.id(),
            *third2.id(),
            *third.id(),
        ]
    );
}
