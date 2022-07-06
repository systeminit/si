use dal::{
    edge::{EdgeKind, VertexObjectKind},
    node::ApplicationId,
    socket::SocketEdgeKind,
    test::{
        helpers::{
            create_system_with_node, find_schema_and_default_variant_by_name, find_schema_by_name,
        },
        DalContextHeadRef,
    },
    Component, DalContext, Edge, StandardModel, Visibility, WorkspaceId,
};

use crate::dal::test;

#[test]
async fn new(ctx: &DalContext<'_, '_>) {
    let (service_schema, service_schema_variant) =
        find_schema_and_default_variant_by_name(ctx, "service").await;

    let sockets = service_schema_variant
        .sockets(ctx)
        .await
        .expect("cannot fetch sockets");

    let input_socket = sockets
        .iter()
        .find(|s| s.edge_kind() == &SocketEdgeKind::Configures && s.name() == "service")
        .expect("cannot find input socket");

    let output_socket = sockets
        .iter()
        .find(|s| s.edge_kind() == &SocketEdgeKind::Output && s.name() == "service")
        .expect("cannot find output socket");

    let (head_component, head_node) =
        Component::new_for_schema_with_node(ctx, "head", service_schema.id())
            .await
            .expect("cannot create component and node for service");

    let (tail_component, tail_node) =
        Component::new_for_schema_with_node(ctx, "head", service_schema.id())
            .await
            .expect("cannot create component and node for service");

    let _edge = Edge::new(
        ctx,
        EdgeKind::Configures,
        *head_node.id(),
        VertexObjectKind::Component,
        (*head_component.id()).into(),
        *input_socket.id(),
        *tail_node.id(),
        VertexObjectKind::Component,
        (*tail_component.id()).into(),
        *output_socket.id(),
    )
    .await
    .expect("cannot create new edge");

    let parents = Edge::find_component_configuration_parents(ctx, head_component.id())
        .await
        .expect("unable to find component's parents");
    assert_eq!(parents.len(), 1);
    assert_eq!(parents[0], *tail_component.id());
}

#[test]
async fn include_component_in_system(
    DalContextHeadRef(ctx): DalContextHeadRef<'_, '_, '_>,
    application_node_id: ApplicationId,
    wid: WorkspaceId,
) {
    let (system, system_node) = create_system_with_node(ctx, &wid).await;

    let service_schema = find_schema_by_name(ctx, "service").await;

    let (first_component, first_component_node) =
        Component::new_for_schema_with_node(ctx, "first", service_schema.id())
            .await
            .expect("cannot create component and node for service");

    let edges = Edge::find_by_attr(ctx, "kind", &"includes".to_string())
        .await
        .expect("cannot retrieve edges from edit session");
    assert_eq!(edges.len(), 1);

    let _ = first_component
        .add_to_system(ctx, system.id())
        .await
        .expect("failed to add component to system");

    let edges = Edge::find_by_attr(ctx, "kind", &"includes".to_string())
        .await
        .expect("cannot retrieve edges from edit session");
    assert_eq!(edges.len(), 2);

    let (second_component, second_component_node) =
        Component::new_for_schema_with_node(ctx, "second", service_schema.id())
            .await
            .expect("cannot create component and node for service");

    let edges = Edge::find_by_attr(ctx, "kind", &"includes".to_string())
        .await
        .expect("cannot retrieve edges from edit session");
    assert_eq!(edges.len(), 3);

    let _ = second_component
        .add_to_system(ctx, system.id())
        .await
        .expect("failed to add component to system");

    let edges = Edge::find_by_attr(ctx, "kind", &"includes".to_string())
        .await
        .expect("cannot retrieve edges from edit session");
    assert_eq!(edges.len(), 4);

    assert_eq!(edges[0].head_node_id(), *first_component_node.id());
    assert_eq!(edges[0].head_object_kind(), &VertexObjectKind::Component);
    assert_eq!(edges[0].tail_node_id(), application_node_id);
    assert_eq!(edges[0].tail_object_kind(), &VertexObjectKind::Component);

    assert_eq!(edges[1].head_node_id(), *first_component_node.id());
    assert_eq!(edges[1].head_object_kind(), &VertexObjectKind::Component);
    assert_eq!(edges[1].tail_node_id(), *system_node.id());
    assert_eq!(edges[1].tail_object_kind(), &VertexObjectKind::System);

    assert_eq!(edges[2].head_node_id(), *second_component_node.id());
    assert_eq!(edges[2].head_object_kind(), &VertexObjectKind::Component);
    assert_eq!(edges[2].tail_node_id(), application_node_id);
    assert_eq!(edges[2].tail_object_kind(), &VertexObjectKind::Component);

    assert_eq!(edges[3].head_node_id(), *second_component_node.id());
    assert_eq!(edges[3].head_object_kind(), &VertexObjectKind::Component);
    assert_eq!(edges[3].tail_node_id(), *system_node.id());
    assert_eq!(edges[3].tail_object_kind(), &VertexObjectKind::System);
}

#[test]
async fn include_component_in_system_with_edit_sessions(
    ctx: &DalContext<'_, '_>,
    application_node_id: ApplicationId,
    wid: WorkspaceId,
) {
    let (system, system_node) = create_system_with_node(ctx, &wid).await;

    let service_schema = find_schema_by_name(ctx, "service").await;

    let (first_component, first_component_node) =
        Component::new_for_schema_with_node(ctx, "first", service_schema.id())
            .await
            .expect("cannot create component and node for service");

    let edges = Edge::find_by_attr(ctx, "kind", &"includes".to_string())
        .await
        .expect("cannot retrieve edges from edit session");
    assert_eq!(edges.len(), 1);

    let _ = first_component
        .add_to_system(ctx, system.id())
        .await
        .expect("failed to add component to system");

    let edges = Edge::find_by_attr(ctx, "kind", &"includes".to_string())
        .await
        .expect("cannot retrieve edges from edit session");
    assert_eq!(edges.len(), 2);

    let (second_component, second_component_node) =
        Component::new_for_schema_with_node(ctx, "second", service_schema.id())
            .await
            .expect("cannot create component and node for service");

    let edges = Edge::find_by_attr(ctx, "kind", &"includes".to_string())
        .await
        .expect("cannot retrieve edges from edit session");
    assert_eq!(edges.len(), 3);

    let _ = second_component
        .add_to_system(ctx, system.id())
        .await
        .expect("failed to add component to system");

    let edges = Edge::find_by_attr(ctx, "kind", &"includes".to_string())
        .await
        .expect("cannot retrieve edges from edit session");
    assert_eq!(edges.len(), 4);

    assert_eq!(edges[0].head_node_id(), *first_component_node.id());
    assert_eq!(edges[0].head_object_kind(), &VertexObjectKind::Component);
    assert_eq!(edges[0].tail_node_id(), application_node_id);
    assert_eq!(edges[0].tail_object_kind(), &VertexObjectKind::Component);

    assert_eq!(edges[1].head_node_id(), *first_component_node.id());
    assert_eq!(edges[1].head_object_kind(), &VertexObjectKind::Component);
    assert_eq!(edges[1].tail_node_id(), *system_node.id());
    assert_eq!(edges[1].tail_object_kind(), &VertexObjectKind::System);

    assert_eq!(edges[2].head_node_id(), *second_component_node.id());
    assert_eq!(edges[2].head_object_kind(), &VertexObjectKind::Component);
    assert_eq!(edges[2].tail_node_id(), application_node_id);
    assert_eq!(edges[2].tail_object_kind(), &VertexObjectKind::Component);

    assert_eq!(edges[3].head_node_id(), *second_component_node.id());
    assert_eq!(edges[3].head_object_kind(), &VertexObjectKind::Component);
    assert_eq!(edges[3].tail_node_id(), *system_node.id());
    assert_eq!(edges[3].tail_object_kind(), &VertexObjectKind::System);

    let head_ctx = ctx.clone_with_new_visibility(Visibility::new_head(false));
    let edges = Edge::find_by_attr(&head_ctx, "kind", &"includes".to_string())
        .await
        .expect("cannot retrieve edges from HEAD");
    assert_eq!(edges.len(), 0);
}
