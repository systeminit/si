use std::io::Read;

use dal::action::prototype::ActionKind;
use dal::func::FuncKind;
use dal::workspace_snapshot::content_address::ContentAddress;
use dal::workspace_snapshot::node_weight::category_node_weight::CategoryNodeKind;
use dal::workspace_snapshot::node_weight::traits::SiVersionedNodeWeight;
use dal::workspace_snapshot::node_weight::{
    ArgumentTargets, CategoryNodeWeight, NodeWeight, OrderingNodeWeight,
};
use dal::{
    ContentHash, DalContext, EdgeWeight, EdgeWeightKind, EdgeWeightKindDiscriminants,
    NodeWeightDiscriminants, SocketArity, WorkspaceSnapshotGraph, WorkspaceSnapshotGraphVCurrent,
};
use dal::{PropKind, Ulid};
use dal_test::test;
use si_events::EncryptedSecretKey;
use si_layer_cache::db::serialize;
use strum::IntoEnumIterator;

const CURRENT_SERIALIZED_GRAPH_DIR_PATH: &str = "./lib/dal/tests";
const CURRENT_SERIALIZED_GRAPH_FILENAME: &str = "serialization-test-data-2024-10-17.snapshot";

// If you're modifying this, you probably just added a new node or edge weight. Before you replace
// the snapshot with one that includes the new weights, ensure that your current code passes the
// deserialization test against the snapshot with the *old* weights. If it does, update the
// validation snapshot file.
#[allow(unused)]
fn make_me_one_with_everything(graph: &mut WorkspaceSnapshotGraphVCurrent) {
    let mut node_indexes = vec![];

    // For every enum that goes into a node weight, try to pick the last variant
    // in the enum, so on the verification pass we will break if anything is
    // added out of order
    for node_kind in NodeWeightDiscriminants::iter() {
        let weight = match node_kind {
            NodeWeightDiscriminants::Action => {
                NodeWeight::new_action(Ulid::new().into(), Ulid::new(), Ulid::new())
            }
            NodeWeightDiscriminants::ActionPrototype => NodeWeight::new_action_prototype(
                Ulid::new(),
                Ulid::new(),
                ActionKind::Update,
                "foo".into(),
                None,
            ),
            NodeWeightDiscriminants::AttributePrototypeArgument => {
                NodeWeight::new_attribute_prototype_argument(
                    Ulid::new(),
                    Ulid::new(),
                    Some(ArgumentTargets {
                        source_component_id: Ulid::new().into(),
                        destination_component_id: Ulid::new().into(),
                    }),
                )
            }
            NodeWeightDiscriminants::AttributeValue => NodeWeight::new_attribute_value(
                Ulid::new(),
                Ulid::new(),
                Some(ContentAddress::JsonValue(ContentHash::new(
                    "foo".as_bytes(),
                ))),
                Some(ContentAddress::JsonValue(ContentHash::new(
                    "foo".as_bytes(),
                ))),
            ),
            NodeWeightDiscriminants::Category => NodeWeight::Category(CategoryNodeWeight::new(
                Ulid::new(),
                Ulid::new(),
                CategoryNodeKind::DependentValueRoots,
            )),
            NodeWeightDiscriminants::Component => NodeWeight::new_component(
                Ulid::new(),
                Ulid::new(),
                ContentHash::new("bar".as_bytes()),
            ),
            NodeWeightDiscriminants::Content => NodeWeight::new_content(
                Ulid::new(),
                Ulid::new(),
                ContentAddress::ManagementPrototype(ContentHash::new("baz".as_bytes())),
            ),
            NodeWeightDiscriminants::DependentValueRoot => {
                NodeWeight::new_dependent_value_root(Ulid::new(), Ulid::new(), Ulid::new())
            }
            NodeWeightDiscriminants::Func => NodeWeight::new_func(
                Ulid::new(),
                Ulid::new(),
                "foo",
                FuncKind::Management,
                ContentHash::new("quux".as_bytes()),
            ),
            NodeWeightDiscriminants::FuncArgument => NodeWeight::new_func_argument(
                Ulid::new(),
                Ulid::new(),
                "arg",
                ContentHash::new("the attitude era".as_bytes()),
            ),
            NodeWeightDiscriminants::Ordering => {
                NodeWeight::Ordering(OrderingNodeWeight::new(Ulid::new(), Ulid::new()))
            }
            NodeWeightDiscriminants::Prop => NodeWeight::new_prop(
                Ulid::new(),
                Ulid::new(),
                PropKind::String,
                "foo",
                ContentHash::new("bar".as_bytes()),
            ),
            NodeWeightDiscriminants::Secret => NodeWeight::new_secret(
                Ulid::new(),
                Ulid::new(),
                EncryptedSecretKey::new("shhh".as_bytes()),
                ContentHash::new("content".as_bytes()),
            ),
            NodeWeightDiscriminants::FinishedDependentValueRoot => {
                NodeWeight::new_finished_dependent_value_root(Ulid::new(), Ulid::new(), Ulid::new())
            }
            NodeWeightDiscriminants::InputSocket => NodeWeight::new_input_socket(
                Ulid::new(),
                Ulid::new(),
                SocketArity::Many,
                ContentHash::new("bar".as_bytes()),
            ),
            NodeWeightDiscriminants::SchemaVariant => NodeWeight::new_schema_variant(
                Ulid::new(),
                Ulid::new(),
                false,
                ContentHash::new("variant".as_bytes()),
            ),
            NodeWeightDiscriminants::ManagementPrototype => NodeWeight::new_management_prototype(
                Ulid::new(),
                Ulid::new(),
                ContentHash::new("management".as_bytes()),
            ),
            NodeWeightDiscriminants::Geometry => NodeWeight::new_geometry(
                Ulid::new(),
                Ulid::new(),
                ContentHash::new("geometry".as_bytes()),
            ),
            NodeWeightDiscriminants::View => NodeWeight::new_view(
                Ulid::new(),
                Ulid::new(),
                ContentHash::new("geometry".as_bytes()),
            ),
        };

        let idx = graph.add_or_replace_node(weight).expect("add node");
        // Attach to root
        graph
            .add_edge(
                graph.root(),
                EdgeWeight::new(EdgeWeightKind::new_use()),
                idx,
            )
            .expect("add edge");
        node_indexes.push(idx);
    }

    let mut last_node = 0;

    for edge_kind in EdgeWeightKindDiscriminants::iter() {
        if last_node + 1 == node_indexes.len() {
            last_node = 0;
        };
        let edge_weight_kind = match edge_kind {
            EdgeWeightKindDiscriminants::Action => EdgeWeightKind::Action,
            EdgeWeightKindDiscriminants::ActionPrototype => EdgeWeightKind::ActionPrototype,
            EdgeWeightKindDiscriminants::AuthenticationPrototype => {
                EdgeWeightKind::AuthenticationPrototype
            }
            EdgeWeightKindDiscriminants::Contain => {
                EdgeWeightKind::Contain(Some("foo".to_string()))
            }
            EdgeWeightKindDiscriminants::FrameContains => EdgeWeightKind::FrameContains,
            EdgeWeightKindDiscriminants::Ordering => EdgeWeightKind::Ordering,
            EdgeWeightKindDiscriminants::Ordinal => EdgeWeightKind::Ordinal,
            EdgeWeightKindDiscriminants::Prop => EdgeWeightKind::Prop,
            EdgeWeightKindDiscriminants::Prototype => {
                EdgeWeightKind::Prototype(Some("bar".to_string()))
            }
            EdgeWeightKindDiscriminants::PrototypeArgument => EdgeWeightKind::PrototypeArgument,
            EdgeWeightKindDiscriminants::PrototypeArgumentValue => {
                EdgeWeightKind::PrototypeArgumentValue
            }
            EdgeWeightKindDiscriminants::Proxy => EdgeWeightKind::Proxy,
            EdgeWeightKindDiscriminants::Root => EdgeWeightKind::Root,
            EdgeWeightKindDiscriminants::Socket => EdgeWeightKind::Socket,
            EdgeWeightKindDiscriminants::SocketValue => EdgeWeightKind::SocketValue,
            EdgeWeightKindDiscriminants::Use => EdgeWeightKind::new_use(),
            EdgeWeightKindDiscriminants::ValidationOutput => EdgeWeightKind::ValidationOutput,
            EdgeWeightKindDiscriminants::ManagementPrototype => EdgeWeightKind::ManagementPrototype,
            EdgeWeightKindDiscriminants::Represents => EdgeWeightKind::Represents,
            EdgeWeightKindDiscriminants::Manages => EdgeWeightKind::Manages,
        };

        let edge_weight = EdgeWeight::new(edge_weight_kind);

        graph
            .add_edge(
                node_indexes[last_node],
                edge_weight,
                node_indexes[last_node + 1],
            )
            .expect("add edge");

        last_node += 1;
    }
}

// Run this test to produce a serialized version of the graph. Do this any time
// a breaking change in serialization occurs.
//
// cd to the root of "si"
// then run:
//
// $ buck2 run //lib/dal:test-integration -- write_deserialization_data --ignored
//
// Then delete the old copy of the graph, and replace the constant
// `CURRENT_SERIALIZED_GRAPH_FILENAME` with the filename of the new graph.
#[test]
#[ignore = "only run this when you want to produce a new serialized graph"]
async fn write_deserialization_data(ctx: &DalContext) {
    let mut graph = WorkspaceSnapshotGraphVCurrent::new(ctx)
        .await
        .expect("make new");

    make_me_one_with_everything(&mut graph);

    graph.cleanup_and_merkle_tree_hash().expect("hash it");

    let real_graph = WorkspaceSnapshotGraph::V4(graph);
    let serialized = serialize::to_vec(&real_graph).expect("serialize");

    let date = chrono::Utc::now().format("%Y-%m-%d").to_string();

    let mut file = std::fs::File::create(format!(
        "{CURRENT_SERIALIZED_GRAPH_DIR_PATH}/serialization-test-data-{date}.snapshot"
    ))
    .expect("create file");
    file.write_all(&serialized).expect("write file");
}

#[test]
async fn graph_can_be_deserialized(_ctx: &DalContext) {
    let mut file = std::fs::File::open(format!(
        "{CURRENT_SERIALIZED_GRAPH_DIR_PATH}/{CURRENT_SERIALIZED_GRAPH_FILENAME}"
    ))
    .expect("open file");
    let mut bytes = vec![];
    file.read_to_end(&mut bytes).expect("able to read bytes");

    let graph: WorkspaceSnapshotGraph = serialize::from_bytes(&bytes).expect("deserialize");

    assert_eq!(31, graph.node_count());

    // Where we can, verify that the enums on the node weights match what we expect
    for (node_weight, _) in graph.nodes() {
        match node_weight {
            NodeWeight::Action(_) => {}
            NodeWeight::ActionPrototype(action_prototype_node_weight) => {
                assert_eq!(ActionKind::Update, action_prototype_node_weight.kind());
            }
            NodeWeight::AttributePrototypeArgument(_) => {}
            NodeWeight::AttributeValue(_) => {}
            NodeWeight::Category(_) => {}
            NodeWeight::Component(_) => {}
            NodeWeight::Content(_) => {}
            NodeWeight::DependentValueRoot(_) => {}
            NodeWeight::Func(func_node_weight) => {
                assert_eq!(FuncKind::Management, func_node_weight.func_kind());
            }
            NodeWeight::FuncArgument(_) => {}
            NodeWeight::Ordering(_) => {}
            NodeWeight::Prop(prop_node_weight) => {
                assert_eq!(PropKind::String, prop_node_weight.kind());
            }
            NodeWeight::Secret(_) => {}
            NodeWeight::FinishedDependentValueRoot(_) => {}
            NodeWeight::InputSocket(input_socket_node_weight) => {
                assert_eq!(SocketArity::Many, input_socket_node_weight.inner().arity());
            }
            NodeWeight::SchemaVariant(_) => {}
            NodeWeight::ManagementPrototype(_) => {}
            NodeWeight::Geometry(_) => {}
            NodeWeight::View(_) => {}
        }
    }
}
