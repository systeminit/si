use dal::socket::SocketKind;
use dal::{
    edge::{EdgeKind, EdgeObjectId, VertexObjectKind},
    socket::SocketEdgeKind,
    Connection, DalContext, DiagramKind, Edge, Schema, SchemaVariant, SocketArity, StandardModel,
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
        EdgeObjectId::from(image_payload.component_id),
        *input_socket.id(),
        credential_payload.node_id,
        VertexObjectKind::Configuration,
        EdgeObjectId::from(credential_payload.component_id),
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
async fn create_and_delete_edges(ctx: &DalContext) {
    let mut harness = SchemaBuiltinsTestHarness::new();
    let from_aws_region = harness
        .create_component(ctx, "from", Builtin::AwsRegion)
        .await;
    let to_aws_ec2_instance = harness.create_component(ctx, "to", Builtin::AwsEc2).await;

    let _from_schema = Schema::get_by_id(ctx, &from_aws_region.schema_id)
        .await
        .expect("could not find schema by id")
        .expect("schema by id not found");
    let _to_schema = Schema::get_by_id(ctx, &to_aws_ec2_instance.schema_id)
        .await
        .expect("could not find schema by id")
        .expect("schema by id not found");

    let from_schema_variant = SchemaVariant::get_by_id(ctx, &from_aws_region.schema_variant_id)
        .await
        .expect("could not find schema variant by id")
        .expect("schema variant by id not found");
    let to_schema_variant = SchemaVariant::get_by_id(ctx, &to_aws_ec2_instance.schema_variant_id)
        .await
        .expect("could not find schema variant by id")
        .expect("schema variant by id not found");

    let from_sockets = from_schema_variant
        .sockets(ctx)
        .await
        .expect("cannot fetch sockets");
    let to_sockets = to_schema_variant
        .sockets(ctx)
        .await
        .expect("cannot fetch sockets");

    let output_socket = from_sockets
        .iter()
        .find(|s| {
            s.edge_kind() == &SocketEdgeKind::ConfigurationOutput
                && s.kind() == &SocketKind::Provider
                && s.diagram_kind() == &DiagramKind::Configuration
                && s.arity() == &SocketArity::Many
                && s.name() == "Region"
        })
        .expect("cannot find output socket");

    let input_socket = to_sockets
        .iter()
        .find(|s| {
            s.edge_kind() == &SocketEdgeKind::ConfigurationInput
                && s.kind() == &SocketKind::Provider
                && s.diagram_kind() == &DiagramKind::Configuration
                && s.arity() == &SocketArity::Many
                && s.name() == "Region"
        })
        .expect("cannot find input socket");

    let connection = Connection::new(
        ctx,
        from_aws_region.node_id,
        *output_socket.id(),
        to_aws_ec2_instance.node_id,
        *input_socket.id(),
        EdgeKind::Configuration,
    )
    .await
    .expect("could not create connection");

    // Update the region to be us-east-2
    from_aws_region
        .update_attribute_value_for_prop_name(
            ctx,
            "/root/domain/region",
            Some(serde_json::json!["us-east-2"]),
        )
        .await;

    // check that the value of the ec2 instance region
    assert_eq!(
        serde_json::json![{
            "domain": {
                "awsResourceType": "instance",
                "region": "us-east-2",
                "tags": {
                    "Name": "to",
                },
            },
            "code": {
                "si:generateAwsEc2JSON": {
                    "code": "{\n\t\"TagSpecifications\": [\n\t\t{\n\t\t\t\"ResourceType\": \"instance\",\n\t\t\t\"Tags\": [\n\t\t\t\t{\n\t\t\t\t\t\"Key\": \"Name\",\n\t\t\t\t\t\"Value\": \"to\"\n\t\t\t\t}\n\t\t\t]\n\t\t}\n\t]\n}",
                    "format": "json",
                },
            },
            "si": {
                "name": "to",
                "type": "component"
            }
        }], // expected
        to_aws_ec2_instance
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value() // actual
    );

    // delete the edge
    let _result = Connection::delete_for_edge(ctx, connection.id).await;

    // check that the region of the ec2 instance is empty
    assert_eq!(
        serde_json::json![{
            "domain": {
                "awsResourceType": "instance",
                "tags": {
                    "Name": "to",
                },
            },
            "code": {
                "si:generateAwsEc2JSON": {
                    "code": "{\n\t\"TagSpecifications\": [\n\t\t{\n\t\t\t\"ResourceType\": \"instance\",\n\t\t\t\"Tags\": [\n\t\t\t\t{\n\t\t\t\t\t\"Key\": \"Name\",\n\t\t\t\t\t\"Value\": \"to\"\n\t\t\t\t}\n\t\t\t]\n\t\t}\n\t]\n}",
                    "format": "json",
                },
            },
            "si": {
                "name": "to",
                "type": "component"
            }
        }], // expected
        to_aws_ec2_instance
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value() // actual
    );
}
