use dal::{
    edge::{EdgeKind, EdgeObjectId, VertexObjectKind},
    socket::SocketEdgeKind,
    DalContext, Edge, SchemaVariant, StandardModel,
};
use dal_test::{
    helpers::builtins::{Builtin, SchemaBuiltinsTestHarness},
    test,
};

#[test]
async fn new(ctx: &DalContext) {
    let mut harness = SchemaBuiltinsTestHarness::new();
    let credential_payload = harness
        .create_component(ctx, "tail", Builtin::DockerHubCredential)
        .await;
    let image_payload = harness
        .create_component(ctx, "head", Builtin::DockerImage)
        .await;

    let credential_schema_variant =
        SchemaVariant::get_by_id(ctx, &credential_payload.schema_variant_id)
            .await
            .expect("could not get schema variant by id")
            .expect("schema variant by id not found");
    let image_schema_variant = SchemaVariant::get_by_id(ctx, &image_payload.schema_variant_id)
        .await
        .expect("could not get schema variant by id")
        .expect("schema variant by id not found");

    let credential_sockets = credential_schema_variant
        .sockets(ctx)
        .await
        .expect("cannot fetch sockets");
    let image_sockets = image_schema_variant
        .sockets(ctx)
        .await
        .expect("cannot fetch sockets");

    let output_socket = credential_sockets
        .iter()
        .find(|s| {
            s.edge_kind() == &SocketEdgeKind::ConfigurationOutput
                && s.name() == "Docker Hub Credential"
        })
        .expect("cannot find output socket");
    let input_socket = image_sockets
        .iter()
        .find(|s| {
            s.edge_kind() == &SocketEdgeKind::ConfigurationInput
                && s.name() == "Docker Hub Credential"
        })
        .expect("cannot find input socket");

    let _edge = Edge::new(
        ctx,
        EdgeKind::Configuration,
        image_payload.node_id,
        VertexObjectKind::Configuration,
        EdgeObjectId::from(image_payload.component_id.into()),
        *input_socket.id(),
        credential_payload.node_id,
        VertexObjectKind::Configuration,
        EdgeObjectId::from(credential_payload.component_id.into()),
        *output_socket.id(),
    )
    .await
    .expect("cannot create new edge");

    let parents = Edge::list_parents_for_component(ctx, image_payload.component_id)
        .await
        .expect("unable to find component's parents");
    assert_eq!(parents.len(), 1);
    assert_eq!(parents[0], credential_payload.component_id);
}
