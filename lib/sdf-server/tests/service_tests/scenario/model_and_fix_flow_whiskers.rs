use std::collections::HashSet;
use std::{thread, time};

use axum::Router;
use dal::{
    component::confirmation::view::ConfirmationStatus, qualification::QualificationSubCheckStatus,
    Component,
};
use dal_test::{sdf_test, AuthToken, DalContextHead};
use pretty_assertions_sorted::assert_eq;

use crate::service_tests::scenario::ScenarioHarness;

/// This test runs through the entire model flow and fix flow lifecycle with a non-trivial set
/// of [`Components`](dal::Component).
///
/// It is recommended to run this test with the following environment variable:
/// ```shell
/// SI_TEST_BUILTIN_SCHEMAS=aws region,aws ami,aws keypair,aws ec2,aws securitygroup,aws ingress,docker image,coreos butane
/// ```
#[sdf_test]
#[ignore]
async fn model_and_fix_flow_whiskers(
    DalContextHead(mut ctx): DalContextHead,
    app: Router,
    AuthToken(auth_token): AuthToken,
) {
    // Setup the harness to start.
    let mut harness = ScenarioHarness::new(
        &ctx,
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
    harness
        .create_change_set_and_update_ctx(&mut ctx, "bruce springsteen")
        .await;

    // Create all AWS components.
    let region = harness.create_node(&ctx, "Region", None).await;
    let ami = harness.create_node(&ctx, "AMI", Some(region.node_id)).await;
    let key_pair = harness
        .create_node(&ctx, "Key Pair", Some(region.node_id))
        .await;
    let ec2 = harness
        .create_node(&ctx, "EC2 Instance", Some(region.node_id))
        .await;
    let security_group = harness
        .create_node(&ctx, "Security Group", Some(region.node_id))
        .await;
    let ingress = harness
        .create_node(&ctx, "Ingress", Some(region.node_id))
        .await;

    // Create all other components.
    let docker = harness.create_node(&ctx, "Docker Image", None).await;
    let butane = harness.create_node(&ctx, "Butane", None).await;

    // Connect Docker and Butane to the relevant AWS components.
    harness
        .create_connection(&ctx, docker.node_id, butane.node_id, "Container Image")
        .await;
    harness
        .create_connection(&ctx, docker.node_id, ingress.node_id, "Exposed Ports")
        .await;
    harness
        .create_connection(&ctx, butane.node_id, ec2.node_id, "User Data")
        .await;

    // Connect AMI, Key Pair and Security Group to the relevant AWS components.
    harness
        .create_connection(&ctx, ami.node_id, ec2.node_id, "Image ID")
        .await;
    harness
        .create_connection(&ctx, key_pair.node_id, ec2.node_id, "Key Name")
        .await;
    harness
        .create_connection(
            &ctx,
            security_group.node_id,
            ingress.node_id,
            "Security Group ID",
        )
        .await;
    harness
        .create_connection(
            &ctx,
            security_group.node_id,
            ec2.node_id,
            "Security Group ID",
        )
        .await;

    // Update AMI, Key Pair and Security Group to start.
    harness
        .update_value(
            &ctx,
            ami.component_id,
            &["si", "name"],
            Some(serde_json::json!["Fedora CoreOS"]),
        )
        .await;
    harness
        .update_value(
            &ctx,
            ami.component_id,
            &["domain", "ImageId"],
            Some(serde_json::json!["ami-0bde60638be9bb870"]),
        )
        .await;
    harness
        .update_value(
            &ctx,
            key_pair.component_id,
            &["si", "name"],
            Some(serde_json::json!["toddhoward-key"]),
        )
        .await;
    harness
        .update_value(
            &ctx,
            key_pair.component_id,
            &["domain", "KeyType"],
            Some(serde_json::json!["rsa"]),
        )
        .await;
    harness
        .update_value(
            &ctx,
            security_group.component_id,
            &["si", "name"],
            Some(serde_json::json!["toddhoward-sg"]),
        )
        .await;
    harness
        .update_value(
            &ctx,
            security_group.component_id,
            &["domain", "Description"],
            Some(serde_json::json!["poop canoe"]),
        )
        .await;

    // Now, look at Ingress and EC2 Instance.
    harness
        .update_value(
            &ctx,
            ingress.component_id,
            &["si", "name"],
            Some(serde_json::json!["toddhoward-ingress"]),
        )
        .await;
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
            ec2.component_id,
            &["domain", "InstanceType"],
            Some(serde_json::json!["t3.micro"]),
        )
        .await;

    // Update Docker Image and Butane.
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
        .update_value(
            &ctx,
            butane.component_id,
            &["si", "name"],
            Some(serde_json::json!["Butane"]),
        )
        .await;

    // Finally, update Region.
    harness
        .update_value(
            &ctx,
            region.component_id,
            &["domain", "region"],
            Some(serde_json::json!["us-east-2"]),
        )
        .await;

    // This is a temporary measure to allow dependent values updates to finish
    let pinga_sleep = time::Duration::from_secs(90);
    thread::sleep(pinga_sleep);

    // Check AMI, Key Pair and Security Group.
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "Fedora CoreOS",
                "type": "component",
                "color": "#FF9900",
                "protected": false,
            },
            "code": {
                "si:generateAwsAmiJSON": {
                    "code": "{\n\t\"Filters\": [\n\t\t{\n\t\t\t\"Name\": \"image-id\",\n\t\t\t\"Values\": [\n\t\t\t\t\"ami-0bde60638be9bb870\"\n\t\t\t]\n\t\t}\n\t]\n}",
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
        ami.view(&ctx)
            .await
            .to_value()
            .expect("could not convert to value"), // actual
    );
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "toddhoward-key",
                "type": "component",
                "color": "#FF9900",
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
        key_pair
            .view(&ctx)
            .await
            .drop_confirmation()
            .to_value()
            .expect("could not convert to value"), // actual
    );
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "toddhoward-sg",
                "type": "component",
                "color": "#FF9900",
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
            .view(&ctx)
            .await
            .drop_confirmation()
            .to_value()
            .expect("could not convert to value"), // actual
    );

    // // Check Ingress, EC2 Instance and Region.
    assert_eq!(
        serde_json::json![{
          "si": {
                "name": "toddhoward-ingress",
                "type": "component",
                "color": "#FF9900",
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
                    "result": "warning",
                    "message": "GroupId must be set. If a Security Group is connected to this component the id will be automatically set when the fix flow creates the security group after merging this change-set",
                },
            },
        }], // expected
        ingress
            .view(&ctx)
            .await
            .drop_confirmation()
            .to_value()
            .expect("could not convert to value"), // actual
    );
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "toddhoward-server",
                "type": "component",
                "color": "#FF9900",
                "protected": false,
            },
            "code": {
                "si:generateAwsEc2JSON": {
                    "code": "{\n\t\"ImageId\": \"ami-0bde60638be9bb870\",\n\t\"InstanceType\": \"t3.micro\",\n\t\"KeyName\": \"toddhoward-key\",\n\t\"UserData\": \"{\\n  \\\"ignition\\\": {\\n    \\\"version\\\": \\\"3.3.0\\\"\\n  },\\n  \\\"systemd\\\": {\\n    \\\"units\\\": [\\n      {\\n        \\\"contents\\\": \\\"[Unit]\\\\nDescription=Docker-io-systeminit-whiskers\\\\nAfter=network-online.target\\\\nWants=network-online.target\\\\n\\\\n[Service]\\\\nTimeoutStartSec=0\\\\nExecStartPre=-/bin/podman kill docker-io-systeminit-whiskers\\\\nExecStartPre=-/bin/podman rm docker-io-systeminit-whiskers\\\\nExecStartPre=/bin/podman pull docker.io/systeminit/whiskers\\\\nExecStart=/bin/podman run --name docker-io-systeminit-whiskers --publish 80:80 docker.io/systeminit/whiskers\\\\n\\\\n[Install]\\\\nWantedBy=multi-user.target\\\",\\n        \\\"enabled\\\": true,\\n        \\\"name\\\": \\\"docker-io-systeminit-whiskers.service\\\"\\n      }\\n    ]\\n  }\\n}\",\n\t\"TagSpecifications\": [\n\t\t{\n\t\t\t\"ResourceType\": \"instance\",\n\t\t\t\"Tags\": [\n\t\t\t\t{\n\t\t\t\t\t\"Key\": \"Name\",\n\t\t\t\t\t\"Value\": \"toddhoward-server\"\n\t\t\t\t}\n\t\t\t]\n\t\t}\n\t]\n}",
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
                "UserData": "{\n  \"ignition\": {\n    \"version\": \"3.3.0\"\n  },\n  \"systemd\": {\n    \"units\": [\n      {\n        \"contents\": \"[Unit]\\nDescription=Docker-io-systeminit-whiskers\\nAfter=network-online.target\\nWants=network-online.target\\n\\n[Service]\\nTimeoutStartSec=0\\nExecStartPre=-/bin/podman kill docker-io-systeminit-whiskers\\nExecStartPre=-/bin/podman rm docker-io-systeminit-whiskers\\nExecStartPre=/bin/podman pull docker.io/systeminit/whiskers\\nExecStart=/bin/podman run --name docker-io-systeminit-whiskers --publish 80:80 docker.io/systeminit/whiskers\\n\\n[Install]\\nWantedBy=multi-user.target\",\n        \"enabled\": true,\n        \"name\": \"docker-io-systeminit-whiskers.service\"\n      }\n    ]\n  }\n}",
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
        ec2.view(&ctx)
            .await
            .drop_confirmation()
            .to_value()
            .expect("could not convert to value"), // actual
    );
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "us-east-2",
                "type": "configurationFrame",
                "color": "#FF9900",
                "protected": false,
            },
            "domain": {
                "region": "us-east-2",
            },
            "qualification": {
                "si:qualificationAwsRegionHasRegionSet": {
                    "result": "success",
                },
            },
        }], // expected
        region
            .view(&ctx)
            .await
            .to_value()
            .expect("could not convert to value"), // actual
    );

    // Finally, check Docker Image and Butane.
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "docker.io/systeminit/whiskers",
                "type": "component",
                "color": "#4695E7",
                "protected": false,
            },
            "domain": {
                "image": "docker.io/systeminit/whiskers",
                "ExposedPorts": [
                    "80/tcp",
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

    // Evaluate Docker Image qualification(s) separately as they may contain a timestamp. Technically,
    // we should be doing this via an sdf route. However, this test focuses on the model and fix
    // flow and we only want to know that the qualification(s) passed.
    for qualification in Component::list_qualifications(&ctx, docker.component_id)
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
                "color": "#e26b70",
                "protected": false,
            },
            "code": {
                "si:generateButaneIgnition": {
                    "code": "{\n  \"ignition\": {\n    \"version\": \"3.3.0\"\n  },\n  \"systemd\": {\n    \"units\": [\n      {\n        \"contents\": \"[Unit]\\nDescription=Docker-io-systeminit-whiskers\\nAfter=network-online.target\\nWants=network-online.target\\n\\n[Service]\\nTimeoutStartSec=0\\nExecStartPre=-/bin/podman kill docker-io-systeminit-whiskers\\nExecStartPre=-/bin/podman rm docker-io-systeminit-whiskers\\nExecStartPre=/bin/podman pull docker.io/systeminit/whiskers\\nExecStart=/bin/podman run --name docker-io-systeminit-whiskers --publish 80:80 docker.io/systeminit/whiskers\\n\\n[Install]\\nWantedBy=multi-user.target\",\n        \"enabled\": true,\n        \"name\": \"docker-io-systeminit-whiskers.service\"\n      }\n    ]\n  }\n}",
                    "format": "json",
                },
            },
            "domain": {
                "systemd": {
                    "units": [
                        {
                            "name": "docker-io-systeminit-whiskers.service",
                            "enabled": true,
                            "contents": "[Unit]\nDescription=Docker-io-systeminit-whiskers\nAfter=network-online.target\nWants=network-online.target\n\n[Service]\nTimeoutStartSec=0\nExecStartPre=-/bin/podman kill docker-io-systeminit-whiskers\nExecStartPre=-/bin/podman rm docker-io-systeminit-whiskers\nExecStartPre=/bin/podman pull docker.io/systeminit/whiskers\nExecStart=/bin/podman run --name docker-io-systeminit-whiskers --publish 80:80 docker.io/systeminit/whiskers\n\n[Install]\nWantedBy=multi-user.target",
                        },
                    ],
                },
                "variant": "fcos",
                "version": "1.4.0",
            },
            "qualification": {
                "si:qualificationButaneIsValidIgnition": {
                    "result": "success",
                    "message": "{\n  \"ignition\": {\n    \"version\": \"3.3.0\"\n  },\n  \"systemd\": {\n    \"units\": [\n      {\n        \"contents\": \"[Unit]\\nDescription=Docker-io-systeminit-whiskers\\nAfter=network-online.target\\nWants=network-online.target\\n\\n[Service]\\nTimeoutStartSec=0\\nExecStartPre=-/bin/podman kill docker-io-systeminit-whiskers\\nExecStartPre=-/bin/podman rm docker-io-systeminit-whiskers\\nExecStartPre=/bin/podman pull docker.io/systeminit/whiskers\\nExecStart=/bin/podman run --name docker-io-systeminit-whiskers --publish 80:80 docker.io/systeminit/whiskers\\n\\n[Install]\\nWantedBy=multi-user.target\",\n        \"enabled\": true,\n        \"name\": \"docker-io-systeminit-whiskers.service\"\n      }\n    ]\n  }\n}",
                },
            },
        }], // expected
        butane
            .view(&ctx)
            .await
            .to_value()
            .expect("could not convert to value"), // actual
    );

    // Apply the change set and get rolling!
    harness
        .apply_change_set_and_update_ctx_visibility_to_head(&mut ctx)
        .await;

    // This is a temporary measure to allow dependent values updates to finish
    thread::sleep(pinga_sleep);

    // Prepare the fixes
    let (confirmations, recommendations) = harness.list_confirmations(&mut ctx).await;
    //let mut requests = Vec::new();

    assert_eq!(
        8, // expected - there are matching exists/ needs deleted confirmations for each components
        confirmations.len(), // actual
    );
    let mut failing_targets = HashSet::new();
    let mut passing_targets = HashSet::new();
    for confirmation in confirmations {
        match confirmation.status {
            ConfirmationStatus::Failure => {
                failing_targets.insert(confirmation.attribute_value_id);
            }
            ConfirmationStatus::Success => {
                passing_targets.insert(confirmation.attribute_value_id);
            }
            _ => {}
        }
    }
    assert_eq!(
        4, // there should be 4 passing confirmations - these are the "needs deletion"
        passing_targets.len()
    );
    assert_eq!(
        4, // there should be 4 failing confirmations - these are the "needs creation"
        failing_targets.len()
    );

    println!("Create delete changeset");

    // Create a new change set to delete the components
    harness
        .create_change_set_and_update_ctx(&mut ctx, "phil collins")
        .await;

    println!("Delete Components");

    // delete AWS components
    harness.delete_component(&ctx, ami.component_id).await;
    harness.delete_component(&ctx, key_pair.component_id).await;
    harness.delete_component(&ctx, ec2.component_id).await;
    harness
        .delete_component(&ctx, security_group.component_id)
        .await;
    harness.delete_component(&ctx, ingress.component_id).await;

    // This is a temporary measure to allow dependent values updates to finish
    thread::sleep(pinga_sleep);
    println!("Here");

    dbg!(security_group
        .view(&ctx)
        .await
        .drop_confirmation()
        .to_value()
        .expect("could not convert to value")); // actual)

    // Let's apply the changeset with the deleted values and check the confirmations
    harness
        .apply_change_set_and_update_ctx_visibility_to_head(&mut ctx)
        .await;

    // This is a temporary measure to allow dependent values updates to finish
    thread::sleep(pinga_sleep);

    // Prepare the fixes
    let (_confirmations, _recommendations) = dbg!(harness.list_confirmations(&mut ctx).await);

    assert_eq!(
        4, // there should be 4 recommendations to apply
        recommendations.len(),
    );

    // Select the recommendations we want.
    // for recommendation in recommendations {
    //     if targets.contains(&recommendation.confirmation_attribute_value_id) {
    //         requests.push(FixRunRequest {
    //             attribute_value_id: recommendation.confirmation_attribute_value_id,
    //             component_id: recommendation.component_id,
    //             action_name: recommendation.recommended_action,
    //         })
    //     }
    // }
    // assert_eq!(
    //     3,              // expected
    //     requests.len(), // actual
    // );
    //
    // // Run fixes from the requests.
    // let first_fix_batch_id = harness.run_fixes(&mut ctx, requests).await;
    //
    // // This is a temporary measure to allow dependent values updates to finish
    // let sixty_secs = time::Duration::from_secs(60);
    // thread::sleep(sixty_secs);
    //
    // // Check that they succeeded.
    // let mut fix_batch_history_views = harness.list_fixes(&mut ctx).await;
    // let fix_batch_history_view = fix_batch_history_views.pop().expect("no fix batches found");
    // assert!(fix_batch_history_views.is_empty());
    // assert_eq!(
    //     first_fix_batch_id,        // expected
    //     fix_batch_history_view.id, // actual
    // );
    // assert_eq!(
    //     Some(FixCompletionStatus::Success), // expected
    //     fix_batch_history_view.status
    // );
    //
    // // Now, run the fix for EC2 .
    // let (confirmations, mut recommendations) = harness.list_confirmations(&mut ctx).await;
    // let mut requests = Vec::new();
    // assert_eq!(
    //     4,                   // expected
    //     confirmations.len(), // actual
    // );
    // for confirmation in confirmations {
    //     if confirmation.title == "EC2 Instance Exists?" {
    //         assert_eq!(
    //             ConfirmationStatus::Failure, // expected
    //             confirmation.status,         // actual
    //         );
    //     } else {
    //         // Ensure that previous confirmations succeeded.
    //         assert_eq!(
    //             ConfirmationStatus::Success, // expected
    //             confirmation.status,         // actual
    //         );
    //     }
    // }
    //
    // // Create the EC2 instance.
    // let recommendation = recommendations.pop().expect("no recommendations found");
    // assert!(recommendations.is_empty());
    // requests.push(FixRunRequest {
    //     attribute_value_id: recommendation.confirmation_attribute_value_id,
    //     component_id: recommendation.component_id,
    //     action_name: recommendation.recommended_action,
    // });
    //
    // assert_eq!(
    //     1,              // expected
    //     requests.len(), // actual
    // );
    //
    // // Run the EC2 fix.
    // let second_fix_batch_id = harness.run_fixes(&mut ctx, requests).await;
    //
    // // This is a temporary measure to allow dependent values updates to finish
    // let sixty_secs = time::Duration::from_secs(60);
    // thread::sleep(sixty_secs);
    //
    // // Check that they succeeded.
    // let fix_batch_history_views = harness.list_fixes(&mut ctx).await;
    // assert_eq!(
    //     2,                             // expected
    //     fix_batch_history_views.len(), // actual
    // );
    // let mut found_fix_batches: HashMap<FixBatchId, FixCompletionStatus> = HashMap::new();
    // for view in fix_batch_history_views {
    //     if let Some(status) = view.status {
    //         found_fix_batches.insert(view.id, status);
    //     }
    // }
    // assert_eq!(
    //     2,                              // expected
    //     found_fix_batches.keys().len(), // actual
    // );
    // assert_eq!(
    //     FixCompletionStatus::Success, // expected
    //     *found_fix_batches
    //         .get(&first_fix_batch_id)
    //         .expect("no status for first fix batch id"), // actual
    // );
    // assert_eq!(
    //     FixCompletionStatus::Success, // expected
    //     *found_fix_batches
    //         .get(&second_fix_batch_id)
    //         .expect("no status for second fix batch id"), // actual
    // );
    //
    // // Check that all confirmations are passing.
    // let (confirmations, recommendations) = harness.list_confirmations(&mut ctx).await;
    // assert_eq!(
    //     4,                   // expected
    //     confirmations.len(), // actual
    // );
    // for confirmation in confirmations {
    //     assert_eq!(
    //         ConfirmationStatus::Success, // expected
    //         confirmation.status,         // actual
    //     );
    // }
    // assert!(recommendations.is_empty())
}
