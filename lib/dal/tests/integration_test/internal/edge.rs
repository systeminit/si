use dal::{
    edge::{EdgeKind, EdgeObjectId, VertexObjectKind},
    socket::SocketEdgeKind,
    Connection, DalContext, Edge, Socket, StandardModel,
};
use dal_test::helpers::component_bag::ComponentBagger;
use dal_test::test;
use pretty_assertions_sorted::assert_eq;

#[test]
async fn new(ctx: &DalContext) {
    let mut bagger = ComponentBagger::new();
    let fallout_bag = bagger.create_component(ctx, "tail", "fallout").await;
    let starfield_bag = bagger.create_component(ctx, "head", "starfield").await;

    let output_socket = Socket::find_by_name_for_edge_kind_and_node(
        ctx,
        "bethesda",
        SocketEdgeKind::ConfigurationOutput,
        fallout_bag.node_id,
    )
    .await
    .expect("could not perform socket find'")
    .expect("could not find socket");
    let input_socket = Socket::find_by_name_for_edge_kind_and_node(
        ctx,
        "bethesda",
        SocketEdgeKind::ConfigurationInput,
        starfield_bag.node_id,
    )
    .await
    .expect("could not perform socket find'")
    .expect("could not find socket");

    let _edge = Edge::new(
        ctx,
        EdgeKind::Configuration,
        starfield_bag.node_id,
        VertexObjectKind::Configuration,
        EdgeObjectId::from(starfield_bag.component_id),
        *input_socket.id(),
        fallout_bag.node_id,
        VertexObjectKind::Configuration,
        EdgeObjectId::from(fallout_bag.component_id),
        *output_socket.id(),
    )
    .await
    .expect("cannot create new edge");

    let parents = Edge::list_parents_for_component(ctx, starfield_bag.component_id)
        .await
        .expect("unable to find component's parents");
    assert_eq!(parents.len(), 1);
    assert_eq!(parents[0], fallout_bag.component_id);
}

#[test]
async fn create_delete_and_restore_edges(ctx: &DalContext) {
    let mut bagger = ComponentBagger::new();

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    let from_aws_region = bagger.create_component(ctx, "from", "Region").await;
    let to_aws_ec2_instance = bagger.create_component(ctx, "to", "EC2 Instance").await;

    let output_socket = Socket::find_by_name_for_edge_kind_and_node(
        ctx,
        "Region",
        SocketEdgeKind::ConfigurationOutput,
        from_aws_region.node_id,
    )
    .await
    .expect("could not perform socket find'")
    .expect("could not find socket");
    let input_socket = Socket::find_by_name_for_edge_kind_and_node(
        ctx,
        "Region",
        SocketEdgeKind::ConfigurationInput,
        to_aws_ec2_instance.node_id,
    )
    .await
    .expect("could not perform socket find'")
    .expect("could not find socket");

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
    let region_prop = from_aws_region
        .find_prop(ctx, &["root", "domain", "region"])
        .await;
    from_aws_region
        .update_attribute_value_for_prop(
            ctx,
            *region_prop.id(),
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
    let mut bagger = ComponentBagger::new();

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    let nginx_container = bagger.create_component(ctx, "nginx", "Docker Image").await;
    let apache2_container = bagger
        .create_component(ctx, "apache2", "Docker Image")
        .await;
    let docker_image_prop = nginx_container
        .find_prop(ctx, &["root", "domain", "image"])
        .await;

    apache2_container
        .update_attribute_value_for_prop(
            ctx,
            *docker_image_prop.id(),
            Some(serde_json::json!["apache2"]),
        )
        .await;
    let butane_instance = bagger.create_component(ctx, "userdata", "Butane").await;

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    let from_container_image_socket = Socket::find_by_name_for_edge_kind_and_node(
        ctx,
        "Container Image",
        SocketEdgeKind::ConfigurationOutput,
        nginx_container.node_id,
    )
    .await
    .expect("could not perform socket find'")
    .expect("could not find socket");
    let to_container_image_socket = Socket::find_by_name_for_edge_kind_and_node(
        ctx,
        "Container Image",
        SocketEdgeKind::ConfigurationInput,
        butane_instance.node_id,
    )
    .await
    .expect("could not perform socket find'")
    .expect("could not find socket");

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
        .update_attribute_value_for_prop(
            ctx,
            *docker_image_prop.id(),
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
