use axum::Router;
use dal::qualification::QualificationSubCheckStatus;
use dal::Component;
use dal_test::{sdf_test, AuthToken, DalContextHead};
use pretty_assertions_sorted::assert_eq;

use crate::service_tests::scenario::ScenarioHarness;

/// This test runs through the model flow for ensuring
/// [Ignition](https://coreos.github.io/ignition/) data is populated correctly to the "user data"
/// field for an [AWS EC2 Instance](https://www.amazonaws.cn/en/ec2). This scenario test includes
/// testing an array of values being propagated and transformed between
/// [`Components`](dal::Component) as well as a "code generation" result being propagated to
/// another [`Component`](dal::Component).
///
/// It is recommended to run this test with the following environment variable:
/// ```shell
/// SI_TEST_BUILTIN_SCHEMAS=docker image,coreos butane,aws region,aws ec2
/// ```
#[sdf_test]
#[ignore]
async fn model_flow_fedora_coreos_ignition(
    DalContextHead(mut ctx): DalContextHead,
    app: Router,
    AuthToken(auth_token): AuthToken,
) {
    // Setup the harness to start.
    let mut harness = ScenarioHarness::new(
        &ctx,
        app,
        auth_token,
        &["Region", "EC2 Instance", "Docker Image", "Butane"],
    )
    .await;

    // Enter a new change set. We will not go through the routes for this.
    harness.create_change_set_and_update_ctx(&mut ctx, "").await;

    // Create all components.
    let region = harness.create_node(&ctx, "Region", None).await;
    let ec2 = harness
        .create_node(&ctx, "EC2 Instance", Some(region.node_id))
        .await;
    let docker = harness.create_node(&ctx, "Docker Image", None).await;
    let butane = harness.create_node(&ctx, "Butane", None).await;

    // Make all connections.
    harness
        .create_connection(&ctx, docker.node_id, butane.node_id, "Container Image")
        .await;
    harness
        .create_connection(&ctx, butane.node_id, ec2.node_id, "User Data")
        .await;

    // Update all components, as needed.
    harness
        .update_value(
            &ctx,
            ec2.component_id,
            &["si", "name"],
            Some(serde_json::json!["toddhoward-server"]),
        )
        .await;
    harness
        .update_value(
            &ctx,
            docker.component_id,
            &["si", "name"],
            Some(serde_json::json!["docker.io/systeminit/whiskers"]),
        )
        .await;
    harness
        .insert_value(&ctx, docker.component_id, &["domain", "ExposedPorts"], None)
        .await;
    harness
        .update_value(
            &ctx,
            docker.component_id,
            &["domain", "ExposedPorts", "0"],
            Some(serde_json::json!["80/tcp"]),
        )
        .await;
    harness
        .insert_value(&ctx, docker.component_id, &["domain", "ExposedPorts"], None)
        .await;
    harness
        .update_value(
            &ctx,
            docker.component_id,
            &["domain", "ExposedPorts", "1"],
            Some(serde_json::json!["443/tcp"]),
        )
        .await;
    harness
        .update_value(
            &ctx,
            butane.component_id,
            &["si", "name"],
            Some(serde_json::json!["Butane"]),
        )
        .await;
    harness
        .update_value(
            &ctx,
            region.component_id,
            &["domain", "region"],
            Some(serde_json::json!["us-east-2"]),
        )
        .await;

    // Ensure everything worked.
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "toddhoward-server",
                "type": "component",
                "protected": false,
            },
            "domain": {
                "tags": {
                    "Name": "toddhoward-server",
                },
                "region": "us-east-2",
                "UserData": "{\n  \"ignition\": {\n    \"version\": \"3.3.0\"\n  },\n  \"systemd\": {\n    \"units\": [\n      {\n        \"contents\": \"[Unit]\\nDescription=Docker-io-systeminit-whiskers\\nAfter=network-online.target\\nWants=network-online.target\\n\\n[Service]\\nTimeoutStartSec=0\\nExecStartPre=-/bin/podman kill docker-io-systeminit-whiskers\\nExecStartPre=-/bin/podman rm docker-io-systeminit-whiskers\\nExecStartPre=/bin/podman pull docker.io/systeminit/whiskers\\nExecStart=/bin/podman run --name docker-io-systeminit-whiskers --publish 80:80 --publish 443:443 docker.io/systeminit/whiskers\\n\\n[Install]\\nWantedBy=multi-user.target\",\n        \"enabled\": true,\n        \"name\": \"docker-io-systeminit-whiskers.service\"\n      }\n    ]\n  }\n}",
                "awsResourceType": "instance",
            },
        }], // expected
        ec2.view(&ctx)
            .await
            .drop_confirmation()
            .drop_code()
            .drop_qualification()
            .to_value()
            .expect("could not convert to value"), // actual
    );
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "Butane",
                "type": "component",
                "protected": false,
            },
            "domain": {
                "systemd": {
                    "units": [
                        {
                            "name": "docker-io-systeminit-whiskers.service",
                            "enabled": true,
                            "contents": "[Unit]\nDescription=Docker-io-systeminit-whiskers\nAfter=network-online.target\nWants=network-online.target\n\n[Service]\nTimeoutStartSec=0\nExecStartPre=-/bin/podman kill docker-io-systeminit-whiskers\nExecStartPre=-/bin/podman rm docker-io-systeminit-whiskers\nExecStartPre=/bin/podman pull docker.io/systeminit/whiskers\nExecStart=/bin/podman run --name docker-io-systeminit-whiskers --publish 80:80 --publish 443:443 docker.io/systeminit/whiskers\n\n[Install]\nWantedBy=multi-user.target",
                        },
                    ],
                },
                "variant": "fcos",
                "version": "1.4.0",
            },
            "code": {
                "si:generateButaneIgnition": {
                    "code": "{\n  \"ignition\": {\n    \"version\": \"3.3.0\"\n  },\n  \"systemd\": {\n    \"units\": [\n      {\n        \"contents\": \"[Unit]\\nDescription=Docker-io-systeminit-whiskers\\nAfter=network-online.target\\nWants=network-online.target\\n\\n[Service]\\nTimeoutStartSec=0\\nExecStartPre=-/bin/podman kill docker-io-systeminit-whiskers\\nExecStartPre=-/bin/podman rm docker-io-systeminit-whiskers\\nExecStartPre=/bin/podman pull docker.io/systeminit/whiskers\\nExecStart=/bin/podman run --name docker-io-systeminit-whiskers --publish 80:80 --publish 443:443 docker.io/systeminit/whiskers\\n\\n[Install]\\nWantedBy=multi-user.target\",\n        \"enabled\": true,\n        \"name\": \"docker-io-systeminit-whiskers.service\"\n      }\n    ]\n  }\n}",
                    "format": "json",
                },
            },
            "qualification": {
                "si:qualificationButaneIsValidIgnition": {
                    "result": "success",
                    "message": "{\n  \"ignition\": {\n    \"version\": \"3.3.0\"\n  },\n  \"systemd\": {\n    \"units\": [\n      {\n        \"contents\": \"[Unit]\\nDescription=Docker-io-systeminit-whiskers\\nAfter=network-online.target\\nWants=network-online.target\\n\\n[Service]\\nTimeoutStartSec=0\\nExecStartPre=-/bin/podman kill docker-io-systeminit-whiskers\\nExecStartPre=-/bin/podman rm docker-io-systeminit-whiskers\\nExecStartPre=/bin/podman pull docker.io/systeminit/whiskers\\nExecStart=/bin/podman run --name docker-io-systeminit-whiskers --publish 80:80 --publish 443:443 docker.io/systeminit/whiskers\\n\\n[Install]\\nWantedBy=multi-user.target\",\n        \"enabled\": true,\n        \"name\": \"docker-io-systeminit-whiskers.service\"\n      }\n    ]\n  }\n}",
                },
            },
        }], // expected
        butane
            .view(&ctx)
            .await
            .to_value()
            .expect("could not convert to value"), // actual
    );
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "us-east-2",
                "type": "configurationFrame",
                "protected": false,
            },
            "domain": {
                "region": "us-east-2",
            },
        }], // expected
        region
            .view(&ctx)
            .await
            .to_value()
            .expect("could not convert to value"), // actual
    );
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "docker.io/systeminit/whiskers",
                "type": "component",
                "protected": false,
            },
            "domain": {
                "image": "docker.io/systeminit/whiskers",
                "ExposedPorts": [
                    "80/tcp",
                    "443/tcp",
                ],
            },
        }], // expected
        docker
            .view(&ctx)
            .await
            .drop_qualification()
            .to_value()
            .expect("could not convert to value"), // actual
    );

    // Evaluate Docker Image qualification(s) separately as they may contain a timestamp.
    // Technically, we should be doing this via an sdf route. However, this test focuses on the
    // model flow and we only want to know that the qualification(s) passed.
    for qualification in Component::list_qualifications(&ctx, docker.component_id)
        .await
        .expect("could not list qualifications")
    {
        assert_eq!(
            QualificationSubCheckStatus::Success,
            qualification.result.expect("no result found").status
        );
    }
}
