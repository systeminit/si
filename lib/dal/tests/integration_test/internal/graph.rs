//! This test module contains mathematical-graph-related tests (i.e. not diagram-related tests)
//! when working with [`Edges`](dal::Edge) and [`Nodes`](dal::Node).

use dal::component::ComponentKind;
use dal::node::NodeId;
use dal::{
    edge::{EdgeKind, EdgeObjectId, VertexObjectKind},
    Component, DalContext, Edge, ExternalProvider, InternalProvider, Node, Schema, SchemaVariant,
    SchemaVariantId, SocketArity, SocketId, StandardModel,
};
use dal_test::helpers::setup_identity_func;
use dal_test::test;
use pretty_assertions_sorted::assert_eq;
use std::collections::BTreeMap;

/// The number of iterations to list topologically sorted configuration nodes with stable ordering
/// for where each call shuffles edges.
const ITERATIONS: i32 = 10;

/// Recommendation: run this test with the following environment variable:
/// ```shell
/// SI_TEST_BUILTIN_SCHEMAS=none
/// ```
#[test]
async fn ascending_creation_topologically_sorted_configuration_nodes_with_stable_ordering(
    ctx: &DalContext,
) {
    let constructor = ConfigurationGraphConstructor::new(ctx).await;

    // Creation order matters: "same level" nodes will be sorted by creation timestamp.
    let torrent_bag = constructor.create_node(ctx, "torrent").await;
    let tarnished_bag = constructor.create_node(ctx, "tarnished").await;
    let godrick_bag = constructor.create_node(ctx, "godrick").await;
    let rennala_bag = constructor.create_node(ctx, "rennala").await;
    let radahn_bag = constructor.create_node(ctx, "radahn").await;
    let morgott_bag = constructor.create_node(ctx, "morgott").await;
    let rykard_bag = constructor.create_node(ctx, "rykard").await;
    let malenia_bag = constructor.create_node(ctx, "malenia").await;
    let mohg_bag = constructor.create_node(ctx, "mohg").await;

    // Create a directed, acyclic graph manually.
    constructor.connect(ctx, &godrick_bag, &rennala_bag).await;
    constructor.connect(ctx, &godrick_bag, &radahn_bag).await;
    constructor.connect(ctx, &rennala_bag, &radahn_bag).await;
    constructor.connect(ctx, &radahn_bag, &morgott_bag).await;
    constructor.connect(ctx, &radahn_bag, &rykard_bag).await;
    constructor.connect(ctx, &morgott_bag, &malenia_bag).await;
    constructor.connect(ctx, &morgott_bag, &mohg_bag).await;
    constructor.connect(ctx, &rykard_bag, &mohg_bag).await;
    constructor.connect(ctx, &malenia_bag, &mohg_bag).await;

    // Created our expected order and contents (correct and stable).
    let expected = vec![
        torrent_bag.node_id,
        tarnished_bag.node_id,
        godrick_bag.node_id,
        rennala_bag.node_id,
        radahn_bag.node_id,
        morgott_bag.node_id,
        rykard_bag.node_id,
        malenia_bag.node_id,
        mohg_bag.node_id,
    ];

    // Ensure the list call is correct and stable. We don't need to compare the lengths in addition
    // to the lists themselves, but it is helpful for debugging failing tests. We use "BTreeMap"
    // to ensure ordering based on key.
    let mut expected_results = BTreeMap::new();
    let mut actual_results = BTreeMap::new();
    for index in 0..ITERATIONS {
        let actual =
            Node::list_topologically_sorted_configuration_nodes_with_stable_ordering(ctx, true)
                .await
                .expect("could not list nodes");
        expected_results.insert(index, (expected.len(), expected.clone()));
        actual_results.insert(index, (actual.len(), actual));
    }
    assert_eq!(
        expected_results, // expected
        actual_results    // actual
    );
}

/// Recommendation: run this test with the following environment variable:
/// ```shell
/// SI_TEST_BUILTIN_SCHEMAS=none
/// ```
#[test]
async fn unordered_creation_topologically_sorted_configuration_nodes_with_stable_ordering(
    ctx: &DalContext,
) {
    let constructor = ConfigurationGraphConstructor::new(ctx).await;

    // Just like the "ascending creation" version of this test, creation order matters: "same level"
    // nodes will be sorted by creation timestamp. However, we will create them in a random order
    // this time. The nodes themselves are the same as those in the aforementioned test.
    let godrick_bag = constructor.create_node(ctx, "godrick").await;
    let rennala_bag = constructor.create_node(ctx, "rennala").await;
    let malenia_bag = constructor.create_node(ctx, "malenia").await;
    let mohg_bag = constructor.create_node(ctx, "mohg").await;
    let torrent_bag = constructor.create_node(ctx, "torrent").await;
    let radahn_bag = constructor.create_node(ctx, "radahn").await;
    let rykard_bag = constructor.create_node(ctx, "rykard").await;
    let morgott_bag = constructor.create_node(ctx, "morgott").await;
    let tarnished_bag = constructor.create_node(ctx, "tarnished").await;

    // Just like the "ascending creation" version of this test, we create a directed, acyclic graph
    // manually. However, we will create the edges in a random order this time. The edges themselves
    // are the same as those in the aforementioned test.
    constructor.connect(ctx, &godrick_bag, &radahn_bag).await;
    constructor.connect(ctx, &rykard_bag, &mohg_bag).await;
    constructor.connect(ctx, &morgott_bag, &malenia_bag).await;
    constructor.connect(ctx, &rennala_bag, &radahn_bag).await;
    constructor.connect(ctx, &malenia_bag, &mohg_bag).await;
    constructor.connect(ctx, &radahn_bag, &morgott_bag).await;
    constructor.connect(ctx, &radahn_bag, &rykard_bag).await;
    constructor.connect(ctx, &morgott_bag, &mohg_bag).await;
    constructor.connect(ctx, &godrick_bag, &rennala_bag).await;

    // The expected order will change slightly compared to the "ascending creation" version of this
    // test because the siblings at each level are sorted by node id (i.e. that sort is dependent
    // on the order of creation for the nodes).
    let expected = vec![
        godrick_bag.node_id,
        torrent_bag.node_id,
        tarnished_bag.node_id,
        rennala_bag.node_id,
        radahn_bag.node_id,
        rykard_bag.node_id,
        morgott_bag.node_id,
        malenia_bag.node_id,
        mohg_bag.node_id,
    ];

    // Ensure the list call is correct and stable. We don't need to compare the lengths in addition
    // to the lists themselves, but it is helpful for debugging failing tests. We use "BTreeMap"
    // to ensure ordering based on key.
    let mut expected_results = BTreeMap::new();
    let mut actual_results = BTreeMap::new();
    for index in 0..ITERATIONS {
        let actual =
            Node::list_topologically_sorted_configuration_nodes_with_stable_ordering(ctx, true)
                .await
                .expect("could not list nodes");
        expected_results.insert(index, (expected.len(), expected.clone()));
        actual_results.insert(index, (actual.len(), actual));
    }
    assert_eq!(
        expected_results, // expected
        actual_results    // actual
    );
}

/// A constructor for creating and connecting [`Nodes`](dal::Node) of the same
/// [`SchemaVariant`](dal::SchemaVariant), input [`Socket`](dal::Socket) and output
/// [`Socket`](dal::Socket). Creating a [`constructor`](Self) results in the creation of a
/// [`SchemaVariant`](dal::SchemaVariant) and relevant [`Sockets`](dal::Socket).
struct ConfigurationGraphConstructor {
    schema_variant_id: SchemaVariantId,
    input_socket_id: SocketId,
    output_socket_id: SocketId,
}

impl ConfigurationGraphConstructor {
    async fn new(ctx: &DalContext) -> Self {
        let mut schema = Schema::new(ctx, "fromsoft", &ComponentKind::Standard)
            .await
            .expect("could not create schema");
        let (mut schema_variant, _root_prop) = SchemaVariant::new(ctx, *schema.id(), "v0")
            .await
            .expect("could not create schema variant");
        schema
            .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
            .await
            .expect("could not set default variant");

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
                "Input",
                identity_func_id,
                identity_func_binding_id,
                identity_func_binding_return_value_id,
                "Input",
                SocketArity::Many,
                false,
            )
            .await
            .expect("could not create explicit internal provider with socket");

        let (_schema_external_provider, output_socket) = ExternalProvider::new_with_socket(
            ctx,
            *schema.id(),
            *schema_variant.id(),
            "Output",
            None,
            identity_func_id,
            identity_func_binding_id,
            identity_func_binding_return_value_id,
            "Output",
            SocketArity::Many,
            false,
        )
        .await
        .expect("could not create external provider with socket");

        schema_variant
            .finalize(ctx, None)
            .await
            .expect("could not finalize schema variant");

        Self {
            schema_variant_id: *schema_variant.id(),
            input_socket_id: *input_socket.id(),
            output_socket_id: *output_socket.id(),
        }
    }

    async fn create_node(&self, ctx: &DalContext, name: &str) -> ConfigurationNodeBag {
        let (component, node) = Component::new(ctx, name, self.schema_variant_id)
            .await
            .expect("could not create component");
        ConfigurationNodeBag {
            object_id: EdgeObjectId::from(*component.id()),
            node_id: *node.id(),
        }
    }

    async fn connect(
        &self,
        ctx: &DalContext,
        source_node: &ConfigurationNodeBag,
        destination_node: &ConfigurationNodeBag,
    ) {
        Edge::new(
            ctx,
            EdgeKind::Configuration,
            destination_node.node_id,
            VertexObjectKind::Configuration,
            destination_node.object_id,
            self.input_socket_id,
            source_node.node_id,
            VertexObjectKind::Configuration,
            source_node.object_id,
            self.output_socket_id,
        )
        .await
        .expect("unable to create edge");
    }
}

/// The bag of a given [`Node`](dal::Node) created by the [`NodeConstructor`].
struct ConfigurationNodeBag {
    object_id: EdgeObjectId,
    node_id: NodeId,
}
