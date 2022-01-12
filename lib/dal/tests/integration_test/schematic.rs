use crate::test_setup;
use dal::{
    Component, HistoryActor, NodePosition, Schema, Schematic, SchematicKind, StandardModel,
    SystemId, Tenancy, Visibility,
};

#[tokio::test]
async fn get_schematic() {
    test_setup!(ctx, _secret_key, _pg, conn, txn, _nats_conn, nats);
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;

    let application_schema = Schema::find_by_attr(
        &txn,
        &tenancy,
        &visibility,
        "name",
        &"application".to_string(),
    )
    .await
    .expect("cannot find application schema")
    .pop()
    .expect("no application schema found");
    let (_component, root_node) = Component::new_for_schema_with_node(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        "sc-component-root-get_schematic",
        application_schema.id(),
    )
    .await
    .expect("unable to create component for schema");

    let service_schema =
        Schema::find_by_attr(&txn, &tenancy, &visibility, "name", &"service".to_string())
            .await
            .expect("cannot find service schema")
            .pop()
            .expect("no service schema found");
    let (_component, node) = Component::new_for_schema_with_node(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        "sc-component-get_schematic",
        service_schema.id(),
    )
    .await
    .expect("unable to create component for schema");

    let node_position = NodePosition::upsert_by_node_id(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        SchematicKind::Component,
        &Some(SystemId::from(1)),
        *root_node.id(),
        *node.id(),
        "123",
        "-10",
    )
    .await
    .expect("cannot upsert node position");

    let schematic = Schematic::find(
        &txn,
        &tenancy,
        &visibility,
        Some(SystemId::from(1)),
        *root_node.id(),
    )
    .await
    .expect("cannot find schematic");
    assert_eq!(schematic.nodes()[0].id(), root_node.id());
    assert_eq!(schematic.nodes()[1].id(), node.id());
    assert_eq!(schematic.nodes()[1].positions()[0].x(), node_position.x());
    assert_eq!(schematic.nodes()[1].positions()[0].y(), node_position.y());
}
