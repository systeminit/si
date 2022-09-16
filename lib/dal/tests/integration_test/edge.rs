use dal::test::helpers::builtins::{Builtin, BuiltinsHarness};
use dal::test::helpers::{create_system_with_node, find_schema_by_name};
use dal::test::DalContextHeadRef;
use dal::{
    edge::{EdgeKind, VertexObjectKind},
    socket::SocketEdgeKind,
    Component, DalContext, Edge, SchemaVariant, StandardModel, Visibility, WorkspaceId,
};

use crate::dal::test;

#[test]
async fn new(ctx: &DalContext) {
    let mut harness = BuiltinsHarness::new();
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
                && s.name() == "docker_hub_credential"
        })
        .expect("cannot find output socket");
    let input_socket = image_sockets
        .iter()
        .find(|s| {
            s.edge_kind() == &SocketEdgeKind::ConfigurationInput
                && s.name() == "docker_hub_credential"
        })
        .expect("cannot find input socket");

    let _edge = Edge::new(
        ctx,
        EdgeKind::Configuration,
        image_payload.node_id,
        VertexObjectKind::Configuration,
        image_payload.component_id.into(),
        *input_socket.id(),
        credential_payload.node_id,
        VertexObjectKind::Configuration,
        credential_payload.component_id.into(),
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

#[test]
async fn include_component_in_system_on_head(
    DalContextHeadRef(ctx): DalContextHeadRef<'_>,
    wid: WorkspaceId,
) {
    let edge_kind = "system".to_string();
    let (system, system_node) = create_system_with_node(ctx, &wid).await;

    let schema = find_schema_by_name(ctx, "docker_image").await;

    let (first_component, first_component_node) =
        Component::new_for_schema_with_node(ctx, "first", schema.id())
            .await
            .expect("cannot create component and node");

    let edges = Edge::find_by_attr(ctx, "kind", &edge_kind)
        .await
        .expect("cannot retrieve edges from edit session");
    assert_eq!(edges.len(), 0);

    first_component
        .add_to_system(ctx, *system.id())
        .await
        .expect("failed to add component to system");

    let edges = Edge::find_by_attr(ctx, "kind", &edge_kind)
        .await
        .expect("cannot retrieve edges from edit session");
    assert_eq!(edges.len(), 1);

    let (second_component, second_component_node) =
        Component::new_for_schema_with_node(ctx, "second", schema.id())
            .await
            .expect("cannot create component and node for service");

    let edges = Edge::find_by_attr(ctx, "kind", &edge_kind)
        .await
        .expect("cannot retrieve edges from edit session");
    assert_eq!(edges.len(), 1);

    second_component
        .add_to_system(ctx, *system.id())
        .await
        .expect("failed to add component to system");

    let edges = Edge::find_by_attr(ctx, "kind", &edge_kind)
        .await
        .expect("cannot retrieve edges from edit session");
    assert_eq!(edges.len(), 2);

    assert_eq!(edges[0].head_node_id(), *first_component_node.id());
    assert_eq!(
        edges[0].head_object_kind(),
        &VertexObjectKind::Configuration
    );
    assert_eq!(edges[0].tail_node_id(), *system_node.id());
    assert_eq!(edges[0].tail_object_kind(), &VertexObjectKind::System);

    assert_eq!(edges[1].head_node_id(), *second_component_node.id());
    assert_eq!(
        edges[1].head_object_kind(),
        &VertexObjectKind::Configuration
    );
    assert_eq!(edges[1].tail_node_id(), *system_node.id());
    assert_eq!(edges[1].tail_object_kind(), &VertexObjectKind::System);
}

#[test]
async fn include_component_in_system_with_edit_sessions(ctx: &DalContext, wid: WorkspaceId) {
    let edge_kind = "system".to_string();
    let (system, system_node) = create_system_with_node(ctx, &wid).await;

    let schema = find_schema_by_name(ctx, "docker_image").await;

    let (first_component, first_component_node) =
        Component::new_for_schema_with_node(ctx, "first", schema.id())
            .await
            .expect("cannot create component and node");

    let edges = Edge::find_by_attr(ctx, "kind", &edge_kind)
        .await
        .expect("cannot retrieve edges from edit session");
    assert_eq!(edges.len(), 0);

    first_component
        .add_to_system(ctx, *system.id())
        .await
        .expect("failed to add component to system");

    let edges = Edge::find_by_attr(ctx, "kind", &edge_kind)
        .await
        .expect("cannot retrieve edges from edit session");
    assert_eq!(edges.len(), 1);

    let (second_component, second_component_node) =
        Component::new_for_schema_with_node(ctx, "second", schema.id())
            .await
            .expect("cannot create component and node for service");

    let edges = Edge::find_by_attr(ctx, "kind", &edge_kind)
        .await
        .expect("cannot retrieve edges from edit session");
    assert_eq!(edges.len(), 1);

    second_component
        .add_to_system(ctx, *system.id())
        .await
        .expect("failed to add component to system");

    let edges = Edge::find_by_attr(ctx, "kind", &edge_kind)
        .await
        .expect("cannot retrieve edges from edit session");
    assert_eq!(edges.len(), 2);

    assert_eq!(edges[0].head_node_id(), *first_component_node.id());
    assert_eq!(
        edges[0].head_object_kind(),
        &VertexObjectKind::Configuration
    );
    assert_eq!(edges[0].tail_node_id(), *system_node.id());
    assert_eq!(edges[0].tail_object_kind(), &VertexObjectKind::System);

    assert_eq!(edges[1].head_node_id(), *second_component_node.id());
    assert_eq!(
        edges[1].head_object_kind(),
        &VertexObjectKind::Configuration
    );
    assert_eq!(edges[1].tail_node_id(), *system_node.id());
    assert_eq!(edges[1].tail_object_kind(), &VertexObjectKind::System);

    let head_ctx = ctx.clone_with_new_visibility(Visibility::new_head(false));
    let edges = Edge::find_by_attr(&head_ctx, "kind", &edge_kind)
        .await
        .expect("cannot retrieve edges from HEAD");
    assert_eq!(edges.len(), 0);
}
