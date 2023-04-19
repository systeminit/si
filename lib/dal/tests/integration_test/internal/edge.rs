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
use pretty_assertions_sorted::assert_eq;

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
async fn create_delete_and_restore_edges(ctx: &DalContext) {
    let mut harness = SchemaBuiltinsTestHarness::new();

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

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
                && s.arity() == &SocketArity::One
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

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    // check that the value of the ec2 instance region
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "to",
                "color": "#FF9900",
                "type": "component",
                "protected": false,
            },
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
            "confirmation": {
                "si:confirmationResourceExists": {
                    "success": false,
                    "recommendedActions": [
                        "create",
                    ],
                },
                "si:confirmationResourceNeedsDeletion": {
                    "success": true,
                    "recommendedActions": [],
                },
                "si:confirmationResourceNeedsUpdate": {
                    "success": true,
                    "recommendedActions": [],
                },
            }
        }], // expected
        to_aws_ec2_instance
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value()
            .expect("could not convert to value") // actual
    );

    // delete the edge
    Connection::delete_for_edge(ctx, connection.id)
        .await
        .expect("Unable to delete connection");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    // check that the region of the ec2 instance is empty
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "to",
                "color": "#FF9900",
                "type": "component",
                "protected": false,
            },
            "domain": {
                "tags": {
                    "Name": "to",
                },
                "awsResourceType": "instance",
            },
            "code": {
                "si:generateAwsEc2JSON": {
                    "code": "{\n\t\"TagSpecifications\": [\n\t\t{\n\t\t\t\"ResourceType\": \"instance\",\n\t\t\t\"Tags\": [\n\t\t\t\t{\n\t\t\t\t\t\"Key\": \"Name\",\n\t\t\t\t\t\"Value\": \"to\"\n\t\t\t\t}\n\t\t\t]\n\t\t}\n\t]\n}",
                    "format": "json",
                },
            },
            "confirmation": {
                "si:confirmationResourceExists": {
                    "success": false,
                    "recommendedActions": [
                        "create",
                    ],
                },
                "si:confirmationResourceNeedsDeletion": {
                    "success": true,
                    "recommendedActions": [],
                },
                "si:confirmationResourceNeedsUpdate": {
                    "success": true,
                    "recommendedActions": [],
                },
            },
        }], // expected
        to_aws_ec2_instance
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value()
            .expect("could not convert to value") // actual
    );

    // restore the edge
    Connection::restore_for_edge(ctx, connection.id)
        .await
        .expect("Unable to restore connection");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    // check that the value of the ec2 instance region
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "to",
                "color": "#FF9900",
                "type": "component",
                "protected": false,
            },
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
            "confirmation": {
                "si:confirmationResourceExists": {
                    "success": false,
                    "recommendedActions": [
                        "create",
                    ],
                },
                "si:confirmationResourceNeedsDeletion": {
                    "success": true,
                    "recommendedActions": [],
                },
                "si:confirmationResourceNeedsUpdate": {
                    "success": true,
                    "recommendedActions": [],
                },
            },
        }], // expected
        to_aws_ec2_instance
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value()
            .expect("could not convert to value") // actual
    );
}

#[test]
async fn create_multiple_connections_and_delete(ctx: &DalContext) {
    let mut harness = SchemaBuiltinsTestHarness::new();

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    let nginx_container = harness
        .create_component(ctx, "nginx", Builtin::DockerImage)
        .await;
    let apache2_container = harness
        .create_component(ctx, "apache2", Builtin::DockerImage)
        .await;
    apache2_container
        .update_attribute_value_for_prop_name(
            ctx,
            "/root/domain/image",
            Some(serde_json::json!["apache2"]),
        )
        .await;
    let butane_instance = harness
        .create_component(ctx, "userdata", Builtin::CoreOsButane)
        .await;

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    let docker_image_schema_variant =
        SchemaVariant::get_by_id(ctx, &nginx_container.schema_variant_id)
            .await
            .expect("could not find schema variant by id")
            .expect("schema variant by id not found");
    let butane_schema_variant = SchemaVariant::get_by_id(ctx, &butane_instance.schema_variant_id)
        .await
        .expect("could not find schema variant by id")
        .expect("schema variant by id not found");

    let docker_image_sockets = docker_image_schema_variant
        .sockets(ctx)
        .await
        .expect("cannot fetch sockets");
    let butane_sockets = butane_schema_variant
        .sockets(ctx)
        .await
        .expect("cannot fetch sockets");

    let from_container_image_socket = docker_image_sockets
        .iter()
        .find(|s| {
            s.edge_kind() == &SocketEdgeKind::ConfigurationOutput
                && s.kind() == &SocketKind::Provider
                && s.diagram_kind() == &DiagramKind::Configuration
                && s.arity() == &SocketArity::Many
                && s.name() == "Container Image"
        })
        .expect("cannot find output socket");

    let to_container_image_socket = butane_sockets
        .iter()
        .find(|s| {
            s.edge_kind() == &SocketEdgeKind::ConfigurationInput
                && s.kind() == &SocketKind::Provider
                && s.diagram_kind() == &DiagramKind::Configuration
                && s.arity() == &SocketArity::Many
                && s.name() == "Container Image"
        })
        .expect("cannot find input socket");

    let connect_from_nginx = Connection::new(
        ctx,
        nginx_container.node_id,
        *from_container_image_socket.id(),
        butane_instance.node_id,
        *to_container_image_socket.id(),
        EdgeKind::Configuration,
    )
    .await
    .expect("could not create connection");

    let connect_from_apache2 = Connection::new(
        ctx,
        apache2_container.node_id,
        *from_container_image_socket.id(),
        butane_instance.node_id,
        *to_container_image_socket.id(),
        EdgeKind::Configuration,
    )
    .await
    .expect("could not create connection");

    // required to happen *AFTER* the connection to trigger a dependantValuesUpdate
    nginx_container
        .update_attribute_value_for_prop_name(
            ctx,
            "/root/domain/image",
            Some(serde_json::json!["nginx"]),
        )
        .await;

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    // check that the value of the butance instance
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "userdata",
                "color": "#e26b70",
                "type": "component",
                "protected": false,
            },
            "domain": {
                "systemd": {
                    "units": [
                        {
                            "name": "nginx.service",
                            "enabled": true,
                            "contents": "[Unit]\nDescription=Nginx\nAfter=network-online.target\nWants=network-online.target\n\n[Service]\nTimeoutStartSec=0\nExecStartPre=-/bin/podman kill nginx\nExecStartPre=-/bin/podman rm nginx\nExecStartPre=/bin/podman pull docker.io/library/nginx\nExecStart=/bin/podman run --name nginx docker.io/library/nginx\n\n[Install]\nWantedBy=multi-user.target",
                        },
                        {
                            "name": "apache2.service",
                            "enabled": true,
                            "contents": "[Unit]\nDescription=Apache2\nAfter=network-online.target\nWants=network-online.target\n\n[Service]\nTimeoutStartSec=0\nExecStartPre=-/bin/podman kill apache2\nExecStartPre=-/bin/podman rm apache2\nExecStartPre=/bin/podman pull docker.io/library/apache2\nExecStart=/bin/podman run --name apache2 docker.io/library/apache2\n\n[Install]\nWantedBy=multi-user.target",
                        },
                    ],
                },
                "variant": "fcos",
                "version": "1.4.0",
            },
            "code": {
                "si:generateButaneIgnition": {
                    "code": "{\n  \"ignition\": {\n    \"version\": \"3.3.0\"\n  },\n  \"systemd\": {\n    \"units\": [\n      {\n        \"contents\": \"[Unit]\\nDescription=Nginx\\nAfter=network-online.target\\nWants=network-online.target\\n\\n[Service]\\nTimeoutStartSec=0\\nExecStartPre=-/bin/podman kill nginx\\nExecStartPre=-/bin/podman rm nginx\\nExecStartPre=/bin/podman pull docker.io/library/nginx\\nExecStart=/bin/podman run --name nginx docker.io/library/nginx\\n\\n[Install]\\nWantedBy=multi-user.target\",\n        \"enabled\": true,\n        \"name\": \"nginx.service\"\n      },\n      {\n        \"contents\": \"[Unit]\\nDescription=Apache2\\nAfter=network-online.target\\nWants=network-online.target\\n\\n[Service]\\nTimeoutStartSec=0\\nExecStartPre=-/bin/podman kill apache2\\nExecStartPre=-/bin/podman rm apache2\\nExecStartPre=/bin/podman pull docker.io/library/apache2\\nExecStart=/bin/podman run --name apache2 docker.io/library/apache2\\n\\n[Install]\\nWantedBy=multi-user.target\",\n        \"enabled\": true,\n        \"name\": \"apache2.service\"\n      }\n    ]\n  }\n}",
                    "format": "json",
                },
            },
        }], // expected
        butane_instance
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value()
            .expect("could not convert to value") // actual
    );

    // delete the nginx connection
    Connection::delete_for_edge(ctx, connect_from_nginx.id)
        .await
        .expect("Deletion should work");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "userdata",
                "color": "#e26b70",
                "type": "component",
                "protected": false,
            },
            "domain": {
                "systemd": {
                    "units": [
                        {
                            "name": "apache2.service",
                            "enabled": true,
                            "contents": "[Unit]\nDescription=Apache2\nAfter=network-online.target\nWants=network-online.target\n\n[Service]\nTimeoutStartSec=0\nExecStartPre=-/bin/podman kill apache2\nExecStartPre=-/bin/podman rm apache2\nExecStartPre=/bin/podman pull docker.io/library/apache2\nExecStart=/bin/podman run --name apache2 docker.io/library/apache2\n\n[Install]\nWantedBy=multi-user.target",
                        },
                    ],
                },
                "variant": "fcos",
                "version": "1.4.0",
            },
            "code": {
                "si:generateButaneIgnition": {
                    "code": "{\n  \"ignition\": {\n    \"version\": \"3.3.0\"\n  },\n  \"systemd\": {\n    \"units\": [\n      {\n        \"contents\": \"[Unit]\\nDescription=Apache2\\nAfter=network-online.target\\nWants=network-online.target\\n\\n[Service]\\nTimeoutStartSec=0\\nExecStartPre=-/bin/podman kill apache2\\nExecStartPre=-/bin/podman rm apache2\\nExecStartPre=/bin/podman pull docker.io/library/apache2\\nExecStart=/bin/podman run --name apache2 docker.io/library/apache2\\n\\n[Install]\\nWantedBy=multi-user.target\",\n        \"enabled\": true,\n        \"name\": \"apache2.service\"\n      }\n    ]\n  }\n}",
                    "format": "json",
                },
            },
        }], // expected
        butane_instance
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value()
            .expect("could not convert to value") // actual
    );

    // delete the nginx connection
    let _result = Connection::delete_for_edge(ctx, connect_from_apache2.id).await;

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "userdata",
                "color": "#e26b70",
                "type": "component",
                "protected": false,
            },
            "domain": {
                "systemd": {
                    "units": [],
                },
                "variant": "fcos",
                "version": "1.4.0",
            },
            "code": {
                "si:generateButaneIgnition": {
                    "code": "{\n  \"ignition\": {\n    \"version\": \"3.3.0\"\n  }\n}",
                    "format": "json",
                },
            },
        }], // expected
        butane_instance
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value()
            .expect("could not convert to value") // actual
    );
}
