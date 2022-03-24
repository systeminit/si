use dal::{
    edge::{EdgeKind, VertexObjectKind},
    test::{
        helpers::{
            find_or_create_production_system, find_schema_and_default_variant_by_name,
            find_schema_by_name,
        },
        DalContextHeadRef,
    },
    Component, DalContext, Edge, StandardModel, System, Visibility,
};

use crate::dal::test;

#[test]
async fn new(ctx: &DalContext<'_, '_>) {
    let _ = find_or_create_production_system(ctx).await;

    let (service_schema, service_schema_variant) =
        find_schema_and_default_variant_by_name(ctx, "service").await;

    let sockets = service_schema_variant
        .sockets(ctx.pg_txn(), ctx.visibility())
        .await
        .expect("cannot fetch sockets");

    let input_socket = sockets
        .iter()
        .find(|s| s.name() == "input")
        .expect("cannot find input socket");

    let output_socket = sockets
        .iter()
        .find(|s| s.name() == "output")
        .expect("cannot find output socket");

    let (head_component, head_node) = Component::new_for_schema_with_node(
        ctx.pg_txn(),
        ctx.nats_txn(),
        ctx.veritech().clone(),
        ctx.encryption_key(),
        &ctx.write_tenancy().into(),
        ctx.visibility(),
        ctx.history_actor(),
        "head",
        service_schema.id(),
    )
    .await
    .expect("cannot create component and node for service");

    let (tail_component, tail_node) = Component::new_for_schema_with_node(
        ctx.pg_txn(),
        ctx.nats_txn(),
        ctx.veritech().clone(),
        ctx.encryption_key(),
        &ctx.write_tenancy().into(),
        ctx.visibility(),
        ctx.history_actor(),
        "head",
        service_schema.id(),
    )
    .await
    .expect("cannot create component and node for service");

    let _edge = Edge::new(
        ctx.pg_txn(),
        ctx.nats_txn(),
        ctx.write_tenancy(),
        ctx.visibility(),
        ctx.history_actor(),
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

    let parents = Edge::find_component_configuration_parents(
        ctx.pg_txn(),
        ctx.read_tenancy(),
        ctx.visibility(),
        head_component.id(),
    )
    .await
    .expect("unable to find component's parents");
    assert_eq!(parents.len(), 1);
    assert_eq!(parents[0], *tail_component.id());
}

#[test]
async fn include_component_in_system(DalContextHeadRef(ctx): DalContextHeadRef<'_, '_, '_>) {
    let (_system, system_node) = System::new_with_node(
        ctx.pg_txn(),
        ctx.nats_txn(),
        ctx.write_tenancy(),
        &Visibility::new_head(false),
        ctx.history_actor(),
        "production",
    )
    .await
    .expect("cannot create production system");

    let service_schema = find_schema_by_name(ctx, "service").await;

    let (_first_component, first_component_node) = Component::new_for_schema_with_node(
        ctx.pg_txn(),
        ctx.nats_txn(),
        ctx.veritech().clone(),
        ctx.encryption_key(),
        &ctx.write_tenancy().into(),
        ctx.visibility(),
        ctx.history_actor(),
        "first",
        service_schema.id(),
    )
    .await
    .expect("cannot create component and node for service");

    let (_second_component, second_component_node) = Component::new_for_schema_with_node(
        ctx.pg_txn(),
        ctx.nats_txn(),
        ctx.veritech().clone(),
        ctx.encryption_key(),
        &ctx.write_tenancy().into(),
        ctx.visibility(),
        ctx.history_actor(),
        "second",
        service_schema.id(),
    )
    .await
    .expect("cannot create component and node for service");

    let edges = Edge::find_by_attr(
        ctx.pg_txn(),
        &ctx.read_tenancy().into(),
        ctx.visibility(),
        "kind",
        &"includes".to_string(),
    )
    .await
    .expect("cannot retrieve edges from edit session");

    assert_eq!(edges.len(), 2);

    assert_eq!(edges[0].head_node_id(), *first_component_node.id());
    assert_eq!(edges[0].head_object_kind(), &VertexObjectKind::Component);
    assert_eq!(edges[0].tail_node_id(), *system_node.id());
    assert_eq!(edges[0].tail_object_kind(), &VertexObjectKind::System);

    assert_eq!(edges[1].head_node_id(), *second_component_node.id());
    assert_eq!(edges[1].head_object_kind(), &VertexObjectKind::Component);
    assert_eq!(edges[1].tail_node_id(), *system_node.id());
    assert_eq!(edges[1].tail_object_kind(), &VertexObjectKind::System);
}

#[test]
async fn include_component_in_system_with_edit_sessions(ctx: &DalContext<'_, '_>) {
    let (_system, system_node) = System::new_with_node(
        ctx.pg_txn(),
        ctx.nats_txn(),
        ctx.write_tenancy(),
        &Visibility::new_head(false),
        ctx.history_actor(),
        "production",
    )
    .await
    .expect("cannot create production system");

    let service_schema = find_schema_by_name(ctx, "service").await;

    let (_first_component, first_component_node) = Component::new_for_schema_with_node(
        ctx.pg_txn(),
        ctx.nats_txn(),
        ctx.veritech().clone(),
        ctx.encryption_key(),
        &ctx.write_tenancy().into(),
        ctx.visibility(),
        ctx.history_actor(),
        "first",
        service_schema.id(),
    )
    .await
    .expect("cannot create component and node for service");

    let (_second_component, second_component_node) = Component::new_for_schema_with_node(
        ctx.pg_txn(),
        ctx.nats_txn(),
        ctx.veritech().clone(),
        ctx.encryption_key(),
        &ctx.write_tenancy().into(),
        ctx.visibility(),
        ctx.history_actor(),
        "second",
        service_schema.id(),
    )
    .await
    .expect("cannot create component and node for service");

    let edges = Edge::find_by_attr(
        ctx.pg_txn(),
        &ctx.read_tenancy().into(),
        &Visibility::new_head(false),
        "kind",
        &"includes".to_string(),
    )
    .await
    .expect("cannot retrieve edges from HEAD");
    assert_eq!(edges.len(), 0);

    let edges = Edge::find_by_attr(
        ctx.pg_txn(),
        &ctx.read_tenancy().into(),
        ctx.visibility(),
        "kind",
        &"includes".to_string(),
    )
    .await
    .expect("cannot retrieve edges from edit session");
    assert_eq!(edges.len(), 2);

    assert_eq!(edges[0].head_node_id(), *first_component_node.id());
    assert_eq!(edges[0].head_object_kind(), &VertexObjectKind::Component);
    assert_eq!(edges[0].tail_node_id(), *system_node.id());
    assert_eq!(edges[0].tail_object_kind(), &VertexObjectKind::System);

    assert_eq!(edges[1].head_node_id(), *second_component_node.id());
    assert_eq!(edges[1].head_object_kind(), &VertexObjectKind::Component);
    assert_eq!(edges[1].tail_node_id(), *system_node.id());
    assert_eq!(edges[1].tail_object_kind(), &VertexObjectKind::System);
}
