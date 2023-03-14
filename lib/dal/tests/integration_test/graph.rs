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

/// Recommendation: run this test with the following environment variable:
/// ```shell
/// SI_TEST_BUILTIN_SCHEMAS=none
/// ```
#[test]
async fn topologically_ish_sorted_configuration_nodes(ctx: &DalContext) {
    let constructor = ConfigurationGraphConstructor::new(ctx).await;

    let torrent_payload = constructor.create_node(ctx, "torrent").await;
    let tarnished_payload = constructor.create_node(ctx, "tarnished").await;
    let godrick_payload = constructor.create_node(ctx, "godrick").await;
    let rennala_payload = constructor.create_node(ctx, "rennala").await;
    let radahn_payload = constructor.create_node(ctx, "radahn").await;
    let morgott_payload = constructor.create_node(ctx, "morgott").await;
    let rykard_payload = constructor.create_node(ctx, "rykard").await;
    let malenia_payload = constructor.create_node(ctx, "malenia").await;
    let mohg_payload = constructor.create_node(ctx, "mohg").await;

    // Create a directed, acyclic graph manually.
    constructor
        .connect(ctx, &godrick_payload, &rennala_payload)
        .await;
    constructor
        .connect(ctx, &godrick_payload, &radahn_payload)
        .await;
    constructor
        .connect(ctx, &rennala_payload, &radahn_payload)
        .await;
    constructor
        .connect(ctx, &radahn_payload, &morgott_payload)
        .await;
    constructor
        .connect(ctx, &radahn_payload, &rykard_payload)
        .await;
    constructor
        .connect(ctx, &morgott_payload, &malenia_payload)
        .await;
    constructor
        .connect(ctx, &morgott_payload, &mohg_payload)
        .await;
    constructor
        .connect(ctx, &rykard_payload, &mohg_payload)
        .await;
    constructor
        .connect(ctx, &malenia_payload, &mohg_payload)
        .await;

    // Perform the sort five times to ensure that we are not dependent on order.
    for _ in 0..5 {
        let sorted_nodes = Node::list_topologically_ish_sorted_configuration_nodes(ctx, true)
            .await
            .expect("could not list nodes");
        assert_eq!(9, sorted_nodes.len());

        // Prepare unstable items to find.
        let mut found_torrent_node_id = None;
        let mut found_tarnished_node_id = None;
        let mut found_morgott_node_id = None;

        for (index, sorted_node_id) in sorted_nodes.iter().enumerate() {
            let sorted_node_id = *sorted_node_id;

            // Check our stable items across every run.
            match index {
                2 => assert_eq!(godrick_payload.node_id, sorted_node_id),
                3 => assert_eq!(rennala_payload.node_id, sorted_node_id),
                4 => assert_eq!(radahn_payload.node_id, sorted_node_id),
                8 => assert_eq!(mohg_payload.node_id, sorted_node_id),
                _ => {}
            }

            // For our unstable items, check that their ordering is correct across every run.
            if sorted_node_id == torrent_payload.node_id {
                found_torrent_node_id = Some(sorted_node_id);
                match found_tarnished_node_id {
                    Some(_) => assert_eq!(index, 1),
                    None => assert_eq!(index, 0),
                }
            } else if sorted_node_id == tarnished_payload.node_id {
                found_tarnished_node_id = Some(sorted_node_id);
                match found_torrent_node_id {
                    Some(_) => assert_eq!(index, 1),
                    None => assert_eq!(index, 0),
                }
            } else if sorted_node_id == morgott_payload.node_id {
                found_morgott_node_id = Some(sorted_node_id);
                assert!(index > 4 && index < 8);
            } else if sorted_node_id == malenia_payload.node_id {
                assert!(found_morgott_node_id.is_some());
                assert!(index > 4 && index < 8);
            }
        }
    }
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

    async fn create_node(&self, ctx: &DalContext, name: &str) -> ConfigurationNodePayload {
        let (component, node) = Component::new(ctx, name, self.schema_variant_id)
            .await
            .expect("could not create component");
        ConfigurationNodePayload {
            object_id: EdgeObjectId::from(*component.id()),
            node_id: *node.id(),
        }
    }

    async fn connect(
        &self,
        ctx: &DalContext,
        source_node: &ConfigurationNodePayload,
        destination_node: &ConfigurationNodePayload,
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

/// The payload of a given [`Node`](dal::Node) created by the [`NodeConstructor`].
struct ConfigurationNodePayload {
    object_id: EdgeObjectId,
    node_id: NodeId,
}
