use crate::filters::change_sets::create_change_set;
use crate::filters::edit_sessions::create_edit_session;
use crate::filters::nodes::create_node;
use crate::DB;
use crate::{test_cleanup, test_setup};

use si_sdf::models::{Edge, EdgeKind, Entity, Node};

#[tokio::test]
async fn calculate_configures() {
    let test_account = test_setup().await.expect("failed to setup test");

    let change_set_id = create_change_set(&test_account).await;
    let edit_session_id = create_edit_session(&test_account, &change_set_id).await;
    let node_reply = create_node(&test_account, &change_set_id, &edit_session_id, "server").await;
    let node = node_reply.item;
    let entity: Entity = node
        .get_object_projection(&DB, &change_set_id)
        .await
        .expect("cannot get new nodes entity for this change set");
    let mut edges = Edge::by_kind_and_tail_node_id(&DB, EdgeKind::Configures, &node.id)
        .await
        .expect("cannot get edges for node");
    assert_eq!(edges.len(), 1, "should have one edge");
    let configures_edge = edges.pop().expect("should have an edge to pop!");
    assert_eq!(
        configures_edge.kind,
        EdgeKind::Configures,
        "is a configures edge"
    );
    assert_eq!(
        configures_edge.tail_vertex.node_id, node.id,
        "edge has this node as the tail vertex"
    );

    let configured_node = Node::get(
        &DB,
        &configures_edge.head_vertex.node_id,
        &node.si_storable.billing_account_id,
    )
    .await
    .expect("cannot fetch configured node");
    assert_eq!(
        configured_node.object_type, "ubuntu",
        "object type should be an ubuntu os"
    );

    let configured_entity: Entity = configured_node
        .get_object_projection(&DB, &change_set_id)
        .await
        .expect("cannot get change set id projection for entity");
    assert_eq!(
        configured_entity.name,
        format!("{} OS", entity.name),
        "new node should have the right name"
    );

    test_cleanup(test_account)
        .await
        .expect("failed to finish test");
}
