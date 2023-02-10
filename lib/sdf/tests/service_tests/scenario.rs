//! This module contains all scenario tests. A scenario test is an `sdf` test that leverages
//! multiple endpoints to test an end-to-end user scenario.

use dal::qualification::QualificationSubCheckStatus;
use dal::{ChangeSet, Component, DalContext, Visibility};
use dal_test::test;
use pretty_assertions_sorted::assert_eq;

use crate::service_tests::scenario::harness::ScenarioHarness;
use crate::test_setup;

mod harness;

/// This test runs through the entire model flow and fix flow lifecycle with a non-trivial set
/// of [`Components`](dal::Component).
///
/// It is recommended to run this test with the following environment variable:
/// ```shell
/// SI_TEST_BUILTIN_SCHEMAS=aws region,aws ami,aws keypair,aws ec2,aws securitygroup,aws ingress,docker image,coreos butane
/// ```
#[test]
#[ignore]
async fn model_and_fix_flow() {
    test_setup!(
        _sdf_ctx,
        _secret_key,
        _pg,
        _conn,
        _txn,
        _nats_conn,
        _nats,
        veritech,
        encr_key,
        app,
        _nba,
        auth_token,
        ctx,
        _job_processor,
        _council_subject_prefix,
    );
    // Just borrow it the whole time because old habits die hard.
    let ctx: &mut DalContext = &mut ctx;

    // Setup the harness to start.
    let mut harness = ScenarioHarness::new(
        ctx,
        app,
        auth_token,
        &[
            "Region",
            "Key Pair",
            "EC2 Instance",
            "Security Group",
            "Ingress",
            "AMI",
            "Docker Image",
            "Butane",
        ],
    )
    .await;

    // Enter a new change set. We will not go through the routes for this.
    let new_change_set = ChangeSet::new(ctx, "bruce springsteen", None)
        .await
        .expect("could not create new change set");
    ctx.update_visibility(Visibility::new(new_change_set.pk, None));
    assert!(!ctx.visibility().is_head());

    // Create all AWS components.
    let region = harness.create_node(ctx, "Region", None).await;
    let ami = harness.create_node(ctx, "AMI", Some(region.node_id)).await;
    let key_pair = harness
        .create_node(ctx, "Key Pair", Some(region.node_id))
        .await;
    let ec2 = harness
        .create_node(ctx, "EC2 Instance", Some(region.node_id))
        .await;
    let security_group = harness
        .create_node(ctx, "Security Group", Some(region.node_id))
        .await;
    let ingress = harness
        .create_node(ctx, "Ingress", Some(region.node_id))
        .await;

    // Create all other components.
    let docker = harness.create_node(ctx, "Docker Image", None).await;
    let butane = harness.create_node(ctx, "Butane", None).await;

    // Connect Docker and Butane to the relevant AWS components.
    harness
        .create_connection(ctx, docker.node_id, butane.node_id, "Container Image")
        .await;
    harness
        .create_connection(ctx, docker.node_id, ingress.node_id, "Exposed Ports")
        .await;
    harness
        .create_connection(ctx, butane.node_id, ec2.node_id, "User Data")
        .await;

    // Connect AMI, Key Pair and Security Group to the relevant AWS components.
    harness
        .create_connection(ctx, ami.node_id, ec2.node_id, "Image ID")
        .await;
    harness
        .create_connection(ctx, key_pair.node_id, ec2.node_id, "Key Name")
        .await;
    harness
        .create_connection(
            ctx,
            security_group.node_id,
            ingress.node_id,
            "Security Group ID",
        )
        .await;
    harness
        .create_connection(
            ctx,
            security_group.node_id,
            ec2.node_id,
            "Security Group ID",
        )
        .await;

    // Update AMI, Key Pair and Security Group to start.
    harness
        .update_value(
            ctx,
            ami.component_id,
            &["si", "name"],
            Some(serde_json::json!["Fedora CoreOS"]),
        )
        .await;
    harness
        .update_value(
            ctx,
            ami.component_id,
            &["domain", "ImageId"],
            Some(serde_json::json!["ami-0bde60638be9bb870"]),
        )
        .await;
    harness
        .update_value(
            ctx,
            key_pair.component_id,
            &["si", "name"],
            Some(serde_json::json!["toddhoward-key"]),
        )
        .await;
    harness
        .update_value(
            ctx,
            key_pair.component_id,
            &["domain", "KeyType"],
            Some(serde_json::json!["rsa"]),
        )
        .await;
    harness
        .update_value(
            ctx,
            security_group.component_id,
            &["si", "name"],
            Some(serde_json::json!["toddhoward-sg"]),
        )
        .await;
    harness
        .update_value(
            ctx,
            security_group.component_id,
            &["domain", "Description"],
            Some(serde_json::json!["poop canoe"]),
        )
        .await;

    // Now, look at Ingress and EC2 Instance.
    harness
        .update_value(
            ctx,
            ingress.component_id,
            &["si", "name"],
            Some(serde_json::json!["toddhoward-ingress"]),
        )
        .await;
    harness
        .update_value(
            ctx,
            ec2.component_id,
            &["si", "name"],
            Some(serde_json::json!["toddhoward-server"]),
        )
        .await;
    harness
        .update_value(
            ctx,
            ec2.component_id,
            &["domain", "InstanceType"],
            Some(serde_json::json!["t3.micro"]),
        )
        .await;

    // Update Docker Image and Butane.
    harness
        .update_value(
            ctx,
            docker.component_id,
            &["si", "name"],
            Some(serde_json::json!["docker.io/systeminit/whiskers:latest"]),
        )
        .await;
    harness
        .insert_value(ctx, docker.component_id, &["domain", "ExposedPorts"], None)
        .await;
    harness
        .update_value(
            ctx,
            docker.component_id,
            &["domain", "ExposedPorts", "0"],
            Some(serde_json::json!["80/tcp"]),
        )
        .await;
    harness
        .update_value(
            ctx,
            butane.component_id,
            &["si", "name"],
            Some(serde_json::json!["Butane"]),
        )
        .await;

    // Finally, update Region.
    harness
        .update_value(
            ctx,
            region.component_id,
            &["domain", "region"],
            Some(serde_json::json!["us-east-2"]),
        )
        .await;

    // Check AMI, Key Pair and Security Group.
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "Fedora CoreOS",
                "type": "component",
                "protected": false,
            },
            "code": {
                "si:generateAwsAmiJSON": {
                    "code": "{\n\t\"ImageIds\": [\n\t\t\"ami-0bde60638be9bb870\"\n\t]\n}",
                    "format": "json",
                },
            },
            "domain": {
                "region": "us-east-2",
                "ImageId": "ami-0bde60638be9bb870",
            },
            "qualification": {
                "si:qualificationAmiExists": {
                    "result": "success",
                    "message": "Image exists",
                },
            },
        }], // expected
        ami.view(ctx).await.to_value(), // actual
    );
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "toddhoward-key",
                "type": "component",
                "protected": false,
            },
            "code": {
                "si:generateAwsKeyPairJSON": {
                    "code": "{\n\t\"KeyName\": \"toddhoward-key\",\n\t\"KeyType\": \"rsa\",\n\t\"TagSpecifications\": [\n\t\t{\n\t\t\t\"ResourceType\": \"key-pair\",\n\t\t\t\"Tags\": [\n\t\t\t\t{\n\t\t\t\t\t\"Key\": \"Name\",\n\t\t\t\t\t\"Value\": \"toddhoward-key\"\n\t\t\t\t}\n\t\t\t]\n\t\t}\n\t]\n}",
                    "format": "json",
                },
            },
            "domain": {
                "tags": {
                    "Name": "toddhoward-key",
                },
                "region": "us-east-2",
                "KeyName": "toddhoward-key",
                "KeyType": "rsa",
                "awsResourceType": "key-pair",
            },
            "qualification": {
                "si:qualificationKeyPairCanCreate": {
                    "result": "success",
                    "message": "component qualified",
                },
            },
        }], // expected
        key_pair.view(ctx).await.drop_confirmation().to_value(), // actual
    );
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "toddhoward-sg",
                "type": "component",
                "protected": false,
            },
            "code": {
                "si:generateAwsSecurityGroupJSON": {
                    "code": "{\n\t\"Description\": \"poop canoe\",\n\t\"GroupName\": \"toddhoward-sg\",\n\t\"TagSpecifications\": [\n\t\t{\n\t\t\t\"ResourceType\": \"security-group\",\n\t\t\t\"Tags\": [\n\t\t\t\t{\n\t\t\t\t\t\"Key\": \"Name\",\n\t\t\t\t\t\"Value\": \"toddhoward-sg\"\n\t\t\t\t}\n\t\t\t]\n\t\t}\n\t]\n}",
                    "format": "json",
                },
            },
            "domain": {
                "tags": {
                    "Name": "toddhoward-sg",
                },
                "region": "us-east-2",
                "GroupName": "toddhoward-sg",
                "Description": "poop canoe",
                "awsResourceType": "security-group",
            },
            "qualification": {
                "si:qualificationSecurityGroupCanCreate": {
                    "result": "success",
                    "message": "component qualified",
                },
            },
        }], // expected
        security_group
            .view(ctx)
            .await
            .drop_confirmation()
            .to_value(), // actual
    );

    // Check Ingress, EC2 Instance and Region.
    assert_eq!(
        serde_json::json![{
          "si": {
                "name": "toddhoward-ingress",
                "type": "component",
                "protected": false,
            },
            "code": {
                "si:generateAwsIngressJSON": {
                    "code": "{\n\t\"IpPermissions\": [\n\t\t{\n\t\t\t\"FromPort\": 80,\n\t\t\t\"ToPort\": 80,\n\t\t\t\"IpProtocol\": \"tcp\",\n\t\t\t\"IpRanges\": [\n\t\t\t\t{\n\t\t\t\t\t\"CidrIp\": \"0.0.0.0/0\"\n\t\t\t\t}\n\t\t\t]\n\t\t}\n\t],\n\t\"TagSpecifications\": [\n\t\t{\n\t\t\t\"ResourceType\": \"security-group-rule\",\n\t\t\t\"Tags\": [\n\t\t\t\t{\n\t\t\t\t\t\"Key\": \"Name\",\n\t\t\t\t\t\"Value\": \"toddhoward-ingress\"\n\t\t\t\t}\n\t\t\t]\n\t\t}\n\t]\n}",
                    "format": "json",
                },
            },
            "domain": {
                "tags": {
                    "Name": "toddhoward-ingress",
                },
                "region": "us-east-2",
                "IpPermissions": [
                    {
                        "CidrIp": "0.0.0.0/0",
                        "ToPort": "80",
                        "FromPort": "80",
                        "IpProtocol": "tcp",
                    },
                ],
                "awsResourceType": "security-group-rule",
            },
            "qualification": {
                "si:qualificationIngressCanCreate": {
                    "result": "success",
                    "message": "component qualified",
                },
            },
        }], // expected
        ingress.view(ctx).await.drop_confirmation().to_value(), // actual
    );
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "toddhoward-server",
                "type": "component",
                "protected": false,
            },
            "code": {
                "si:generateAwsEc2JSON": {
                    "code": "{\n\t\"ImageId\": \"ami-0bde60638be9bb870\",\n\t\"InstanceType\": \"t3.micro\",\n\t\"KeyName\": \"toddhoward-key\",\n\t\"UserData\": \"\",\n\t\"TagSpecifications\": [\n\t\t{\n\t\t\t\"ResourceType\": \"instance\",\n\t\t\t\"Tags\": [\n\t\t\t\t{\n\t\t\t\t\t\"Key\": \"Name\",\n\t\t\t\t\t\"Value\": \"toddhoward-server\"\n\t\t\t\t}\n\t\t\t]\n\t\t}\n\t]\n}",
                    "format": "json",
                },
            },
            "domain": {
                "tags": {
                    "Name": "toddhoward-server",
                },
                "region": "us-east-2",
                "ImageId": "ami-0bde60638be9bb870",
                "KeyName": "toddhoward-key",
                "UserData": "{\n  \"ignition\": {\n    \"version\": \"3.3.0\"\n  },\n  \"systemd\": {\n    \"units\": [\n      {\n        \"contents\": \"[Unit]\\nDescription=Docker-io-systeminit-whiskers-latest\\nAfter=network-online.target\\nWants=network-online.target\\n\\n[Service]\\nTimeoutStartSec=0\\nExecStartPre=-/bin/podman kill docker-io-systeminit-whiskers-latest\\nExecStartPre=-/bin/podman rm docker-io-systeminit-whiskers-latest\\nExecStartPre=/bin/podman pull docker.io/systeminit/whiskers:latest\\nExecStart=/bin/podman run --name docker-io-systeminit-whiskers-latest --publish 80:80 docker.io/systeminit/whiskers:latest\\n\\n[Install]\\nWantedBy=multi-user.target\",\n        \"enabled\": true,\n        \"name\": \"docker-io-systeminit-whiskers-latest.service\"\n      }\n    ]\n  }\n}",
                "InstanceType": "t3.micro",
                "awsResourceType": "instance",
            },
            "qualification": {
                "si:qualificationEc2CanRun": {
                    "result": "warning",
                    "message": "Key Pair must exist. It will be created by the fix flow after merging this change-set",
                },
            },
        }], // expected
        ec2.view(ctx).await.drop_confirmation().to_value(), // actual
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
        region.view(ctx).await.to_value(), // actual
    );

    // Finally, check Docker Image and Butane.
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "docker.io/systeminit/whiskers:latest",
                "type": "component",
                "protected": false,
            },
            "domain": {
                "image": "docker.io/systeminit/whiskers:latest",
                "ExposedPorts": [
                    "80/tcp",
                ],
            },
        }], // expected
        docker.view(ctx).await.drop_qualification().to_value(), // actual
    );

    // Evaluate Docker Image qualification(s) separately as they may contain a timestamp. Technically,
    // we should be doing this via an sdf route. However, this test focuses on the model and fix
    // flow and we only want to know that the qualification(s) passed.
    for qualification in Component::list_qualifications(ctx, docker.component_id)
        .await
        .expect("could not list qualifications")
    {
        assert_eq!(
            QualificationSubCheckStatus::Success,
            qualification.result.expect("no result found").status
        );
    }

    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "Butane",
                "type": "component",
                "protected": false,
            },
            "code": {
                "si:generateButaneIgnition": {
                    "code": "{\n  \"ignition\": {\n    \"version\": \"3.3.0\"\n  },\n  \"systemd\": {\n    \"units\": [\n      {\n        \"contents\": \"[Unit]\\nDescription=Docker-io-systeminit-whiskers-latest\\nAfter=network-online.target\\nWants=network-online.target\\n\\n[Service]\\nTimeoutStartSec=0\\nExecStartPre=-/bin/podman kill docker-io-systeminit-whiskers-latest\\nExecStartPre=-/bin/podman rm docker-io-systeminit-whiskers-latest\\nExecStartPre=/bin/podman pull docker.io/systeminit/whiskers:latest\\nExecStart=/bin/podman run --name docker-io-systeminit-whiskers-latest --publish 80:80 docker.io/systeminit/whiskers:latest\\n\\n[Install]\\nWantedBy=multi-user.target\",\n        \"enabled\": true,\n        \"name\": \"docker-io-systeminit-whiskers-latest.service\"\n      }\n    ]\n  }\n}",
                    "format": "json",
                },
            },
            "domain": {
                "systemd": {
                    "units": [
                        {
                            "name": "docker-io-systeminit-whiskers-latest.service",
                            "enabled": true,
                            "contents": "[Unit]\nDescription=Docker-io-systeminit-whiskers-latest\nAfter=network-online.target\nWants=network-online.target\n\n[Service]\nTimeoutStartSec=0\nExecStartPre=-/bin/podman kill docker-io-systeminit-whiskers-latest\nExecStartPre=-/bin/podman rm docker-io-systeminit-whiskers-latest\nExecStartPre=/bin/podman pull docker.io/systeminit/whiskers:latest\nExecStart=/bin/podman run --name docker-io-systeminit-whiskers-latest --publish 80:80 docker.io/systeminit/whiskers:latest\n\n[Install]\nWantedBy=multi-user.target",
                        },
                    ],
                },
                "variant": "fcos",
                "version": "1.4.0",
            },
            "qualification": {
                "si:qualificationButaneIsValidIgnition": {
                    "result": "success",
                    "message": "{\n  \"ignition\": {\n    \"version\": \"3.3.0\"\n  },\n  \"systemd\": {\n    \"units\": [\n      {\n        \"contents\": \"[Unit]\\nDescription=Docker-io-systeminit-whiskers-latest\\nAfter=network-online.target\\nWants=network-online.target\\n\\n[Service]\\nTimeoutStartSec=0\\nExecStartPre=-/bin/podman kill docker-io-systeminit-whiskers-latest\\nExecStartPre=-/bin/podman rm docker-io-systeminit-whiskers-latest\\nExecStartPre=/bin/podman pull docker.io/systeminit/whiskers:latest\\nExecStart=/bin/podman run --name docker-io-systeminit-whiskers-latest --publish 80:80 docker.io/systeminit/whiskers:latest\\n\\n[Install]\\nWantedBy=multi-user.target\",\n        \"enabled\": true,\n        \"name\": \"docker-io-systeminit-whiskers-latest.service\"\n      }\n    ]\n  }\n}",
                },
            },
        }], // expected
        butane.view(ctx).await.to_value(), // actual
    );

    // TODO(nick): continue this test starting with "change set apply" and then running the fix
    // flow. Fortunately, this should be much easier now that the groundwork has been laid for
    // authoring scenario tests.
}
