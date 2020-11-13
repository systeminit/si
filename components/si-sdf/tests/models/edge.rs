use crate::filters::change_sets::create_change_set;
use crate::filters::edit_sessions::create_edit_session;
use crate::filters::nodes::create_node;
use crate::{test_cleanup, test_setup, DB, NATS};

use si_sdf::models::{Edge, EdgeKind, Node};

#[test]
fn all_predecessor_edges_by_node_id() {
    tokio_test::block_on(Box::pin(async move {
        let test_account = test_setup().await.expect("failed to setup test");

        let change_set_id = create_change_set(&test_account).await;
        let edit_session_id = create_edit_session(&test_account, &change_set_id).await;

        let mut created_nodes: Vec<Node> = Vec::new();
        for _n in 0..20 as usize {
            let create_node =
                create_node(&test_account, &change_set_id, &edit_session_id, "service").await;
            created_nodes.push(create_node.item);
        }

        // The graph is 0 -> 1..10
        //                1 -> 11..12
        //                2 -> 13..15
        //                   15 -> 16..19

        // Node 0 -(configures)-> Node 1..10
        for n in 1..=10 {
            created_nodes[0]
                .configure_node(&DB, &NATS, &created_nodes[n].id)
                .await
                .expect("failed to add edge to configure node");
        }
        // Node 1 -(configures)-> Node 11,12
        for n in 11..=12 {
            created_nodes[1]
                .configure_node(&DB, &NATS, &created_nodes[n].id)
                .await
                .expect("failed to add edge to configure node");
        }
        // Node 2 -(configures)-> Node 13,14,15
        for n in 13..=15 {
            created_nodes[2]
                .configure_node(&DB, &NATS, &created_nodes[n].id)
                .await
                .expect("failed to add edge to configure node");
        }
        // Node 15 -(configures)-> Node 16,17,18,19
        for n in 16..=19 {
            created_nodes[15]
                .configure_node(&DB, &NATS, &created_nodes[n].id)
                .await
                .expect("failed to add edge to configure node");
        }

        // Node 0 should have no predecessor edges
        let pedges =
            Edge::all_predecessor_edges_by_node_id(&DB, EdgeKind::Configures, &created_nodes[0].id)
                .await
                .expect("cannot get predecessor edges");
        assert_eq!(pedges.len(), 0, "has no predecessors");

        // Node 1..10 should have Node 0 as a predecessor edge
        for n in 1..=10 {
            let pedges = Edge::all_predecessor_edges_by_node_id(
                &DB,
                EdgeKind::Configures,
                &created_nodes[n].id,
            )
            .await
            .expect("cannot get predecessor edges");
            assert_eq!(pedges.len(), 1, "has 1 predecessor");
            assert_eq!(
                &pedges[0].tail_vertex.node_id, &created_nodes[0].id,
                "predecessor is node 0"
            );
        }

        // Node 11..12 should have Node 0 and Node 1 as predecessor edges
        for n in 11..=12 {
            let pedges = Edge::all_predecessor_edges_by_node_id(
                &DB,
                EdgeKind::Configures,
                &created_nodes[n].id,
            )
            .await
            .expect("cannot get predecessor edges");
            assert_eq!(pedges.len(), 2, "has 2 predecessors");
            assert_eq!(
                &pedges[0].tail_vertex.node_id, &created_nodes[1].id,
                "first predecessor is node 1"
            );
            assert_eq!(
                &pedges[1].tail_vertex.node_id, &created_nodes[0].id,
                "second predecessor is node 0"
            );
        }

        // Node 19 should have 3 predecessors, 19->15->2->0
        let pedges = Edge::all_predecessor_edges_by_node_id(
            &DB,
            EdgeKind::Configures,
            &created_nodes[19].id,
        )
        .await
        .expect("cannot get predecessor edges");
        assert_eq!(pedges.len(), 3, "has 3 predecessors");
        assert_eq!(
            &pedges[0].tail_vertex.node_id, &created_nodes[15].id,
            "first predecessor is node 15"
        );
        assert_eq!(
            &pedges[1].tail_vertex.node_id, &created_nodes[2].id,
            "second predecessor is node 2"
        );
        assert_eq!(
            &pedges[2].tail_vertex.node_id, &created_nodes[0].id,
            "third predecessor is node 0"
        );

        test_cleanup(test_account)
            .await
            .expect("failed to finish test");
    }));
}
