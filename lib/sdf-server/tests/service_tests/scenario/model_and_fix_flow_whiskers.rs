use axum::Router;
use dal::{
    qualification::QualificationSubCheckStatus, ActionKind, Component, FixCompletionStatus, Func,
    StandardModel,
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

    let random_name = ScenarioHarness::generate_fake_name();
    println!("Starting test run for: {}", random_name);

    // Enter a new change set. We will not go through the routes for this.
    harness
        .create_change_set_and_update_ctx(&mut ctx, ScenarioHarness::generate_fake_name())
        .await;

    let mut func = Func::find_by_attr(&ctx, "name", &"si:awsEc2DeleteAction")
        .await
        .expect("unable to find ec2 delete exists")
        .pop()
        .expect("unable to find one ec2 delete exists");
    func.set_code_plaintext(
        &ctx,
        Some("async function deleteResource(component: Input): Promise < Output > {
    const resource = component.properties.resource?.payload;

    if (!resource.InstanceId)
        return {
            status: 'error',
            payload: resource,
            message: 'No EC2 instance id found',
        };

    const child = await siExec.waitUntilEnd('aws', [
        'ec2',
        'terminate-instances',
        '--region',
        component.properties.domain?.region,
        '--instance-ids',
        resource.InstanceId,
    ]);

    if (child.exitCode !== 0) {
        console.error(child.stderr);
        return {
            status: 'error',
            payload: resource,
            message: `Unable to delete Ec2 Instance, AWS CLI 2 exited with non zero code: ${child.exitCode}`,
        };
    }

    await new Promise((resolve) => setTimeout(resolve, 2 * 60 * 1000);

    return {
        payload: null,
        status: 'ok'
    };
}"
        ),
    )
        .await
        .expect("unable to patch function");

    ctx.commit().await.expect("unable to commit");

    // Create all AWS components.
    let region = harness.create_node(ctx.visibility(), "Region", None).await;
    let ami = harness
        .create_node(ctx.visibility(), "AMI", Some(region.node_id))
        .await;
    let key_pair = harness
        .create_node(ctx.visibility(), "Key Pair", Some(region.node_id))
        .await;
    let ec2 = harness
        .create_node(ctx.visibility(), "EC2 Instance", Some(region.node_id))
        .await;
    let security_group = harness
        .create_node(ctx.visibility(), "Security Group", Some(region.node_id))
        .await;
    let ingress = harness
        .create_node(ctx.visibility(), "Ingress", Some(region.node_id))
        .await;

    // Create all other components.
    let docker = harness
        .create_node(ctx.visibility(), "Docker Image", None)
        .await;
    let butane = harness.create_node(ctx.visibility(), "Butane", None).await;

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
            Some(serde_json::json![format!("{random_name}-key")]),
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
            Some(serde_json::json![format!("{random_name}-sg")]),
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
            Some(serde_json::json![format!("{random_name}-ingress")]),
        )
        .await;
    harness
        .update_value(
            &ctx,
            ec2.component_id,
            &["si", "name"],
            Some(serde_json::json![format!("{random_name}-server")]),
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
                "name": format!("{random_name}-key"),
                "type": "component",
                "color": "#FF9900",
                "protected": false,
            },
            "code": {
                "si:generateAwsKeyPairJSON": {
                    "code": format!("{{\n\t\"KeyName\": \"{random_name}-key\",\n\t\"KeyType\": \"rsa\",\n\t\"TagSpecifications\": [\n\t\t{{\n\t\t\t\"ResourceType\": \"key-pair\",\n\t\t\t\"Tags\": [\n\t\t\t\t{{\n\t\t\t\t\t\"Key\": \"Name\",\n\t\t\t\t\t\"Value\": \"{random_name}-key\"\n\t\t\t\t}}\n\t\t\t]\n\t\t}}\n\t]\n}}"),
                    "format": "json",
                },
            },
            "domain": {
                "tags": {
                    "Name": format!("{random_name}-key"),
                },
                "region": "us-east-2",
                "KeyName": format!("{random_name}-key"),
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
            .to_value()
            .expect("could not convert to value"), // actual
    );
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": format!("{random_name}-sg"),
                "type": "component",
                "color": "#FF9900",
                "protected": false,
            },
            "code": {
                "si:generateAwsSecurityGroupJSON": {
                    "code": format!("{{\n\t\"Description\": \"poop canoe\",\n\t\"GroupName\": \"{random_name}-sg\",\n\t\"TagSpecifications\": [\n\t\t{{\n\t\t\t\"ResourceType\": \"security-group\",\n\t\t\t\"Tags\": [\n\t\t\t\t{{\n\t\t\t\t\t\"Key\": \"Name\",\n\t\t\t\t\t\"Value\": \"{random_name}-sg\"\n\t\t\t\t}}\n\t\t\t]\n\t\t}}\n\t]\n}}"),
                    "format": "json",
                },
            },
            "domain": {
                "tags": {
                    "Name": format!("{random_name}-sg"),
                },
                "region": "us-east-2",
                "GroupName": format!("{random_name}-sg"),
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
            .to_value()
            .expect("could not convert to value"), // actual
    );

    // // Check Ingress, EC2 Instance and Region.
    assert_eq!(
        serde_json::json![{
          "si": {
                "name": format!("{random_name}-ingress"),
                "type": "component",
                "color": "#FF9900",
                "protected": false,
            },
            "domain": {
                "tags": {
                    "Name": format!("{random_name}-ingress"),
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
            "code": {
                "si:generateAwsIngressJSON": {
                    "code": format!("{{\n\t\"IpPermissions\": [\n\t\t{{\n\t\t\t\"FromPort\": 80,\n\t\t\t\"ToPort\": 80,\n\t\t\t\"IpProtocol\": \"tcp\",\n\t\t\t\"IpRanges\": [\n\t\t\t\t{{\n\t\t\t\t\t\"CidrIp\": \"0.0.0.0/0\"\n\t\t\t\t}}\n\t\t\t]\n\t\t}}\n\t],\n\t\"TagSpecifications\": [\n\t\t{{\n\t\t\t\"ResourceType\": \"security-group-rule\",\n\t\t\t\"Tags\": [\n\t\t\t\t{{\n\t\t\t\t\t\"Key\": \"Name\",\n\t\t\t\t\t\"Value\": \"{random_name}-ingress\"\n\t\t\t\t}}\n\t\t\t]\n\t\t}}\n\t]\n}}"),
                    "format": "json",
                },
            },
            "qualification": {
                "si:qualificationIngressCanCreate": {
                    "result": "success",
                    "message": "GroupId must be set. If a Security Group is connected to this component the id will be automatically set when the fix flow creates the security group after merging this change-set",
                },
            },
        }], // expected
        ingress
            .view(&ctx)
            .await
            .to_value()
            .expect("could not convert to value"), // actual
    );
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": format!("{random_name}-server"),
                "type": "component",
                "color": "#FF9900",
                "protected": false,
            },
            "code": {
                "si:generateAwsEc2JSON": {
                    "code": format!("{{\n\t\"ImageId\": \"ami-0bde60638be9bb870\",\n\t\"InstanceType\": \"t3.micro\",\n\t\"KeyName\": \"{random_name}-key\",\n\t\"UserData\": \"{{\\n  \\\"ignition\\\": {{\\n    \\\"version\\\": \\\"3.3.0\\\"\\n  }},\\n  \\\"systemd\\\": {{\\n    \\\"units\\\": [\\n      {{\\n        \\\"contents\\\": \\\"[Unit]\\\\nDescription=Docker-io-systeminit-whiskers\\\\nAfter=network-online.target\\\\nWants=network-online.target\\\\n\\\\n[Service]\\\\nTimeoutStartSec=0\\\\nExecStartPre=-/bin/podman kill docker-io-systeminit-whiskers\\\\nExecStartPre=-/bin/podman rm docker-io-systeminit-whiskers\\\\nExecStartPre=/bin/podman pull docker.io/systeminit/whiskers\\\\nExecStart=/bin/podman run --name docker-io-systeminit-whiskers --publish 80:80 docker.io/systeminit/whiskers\\\\n\\\\n[Install]\\\\nWantedBy=multi-user.target\\\",\\n        \\\"enabled\\\": true,\\n        \\\"name\\\": \\\"docker-io-systeminit-whiskers.service\\\"\\n      }}\\n    ]\\n  }}\\n}}\",\n\t\"TagSpecifications\": [\n\t\t{{\n\t\t\t\"ResourceType\": \"instance\",\n\t\t\t\"Tags\": [\n\t\t\t\t{{\n\t\t\t\t\t\"Key\": \"Name\",\n\t\t\t\t\t\"Value\": \"{random_name}-server\"\n\t\t\t\t}}\n\t\t\t]\n\t\t}}\n\t]\n}}"),
                    "format": "json",
                },
            },
            "domain": {
                "tags": {
                    "Name": format!("{random_name}-server"),
                },
                "region": "us-east-2",
                "ImageId": "ami-0bde60638be9bb870",
                "KeyName": format!("{random_name}-key"),
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

    let actions = harness
        .list_actions_for_changeset(ctx.visibility())
        .await
        .actions;

    let expected_actions_and_parents = [
        (key_pair.component_id, ActionKind::Create, Vec::new()),
        (security_group.component_id, ActionKind::Create, Vec::new()),
        (
            ingress.component_id,
            ActionKind::Create,
            vec![(security_group.component_id, ActionKind::Create)],
        ),
        (
            ec2.component_id,
            ActionKind::Create,
            vec![
                (key_pair.component_id, ActionKind::Create),
                (security_group.component_id, ActionKind::Create),
            ],
        ),
    ];
    dbg!(&actions, &expected_actions_and_parents);

    'outer: for (expected_comp_id, expected_kind, expected_parents) in &expected_actions_and_parents
    {
        for action in actions.values() {
            if *expected_comp_id == action.component_id
                && *expected_kind == action.kind
                && action.parents.len() == expected_parents.len()
            {
                'parent_outer: for (expected_comp_id, expected_kind) in expected_parents {
                    for action in actions.values() {
                        if *expected_comp_id == action.component_id && *expected_kind == action.kind
                        {
                            continue 'parent_outer;
                        }
                    }

                    panic!(
                        "Expected parent action not found: {:?} ({:#?} {:#?})",
                        (expected_comp_id, expected_kind),
                        actions,
                        expected_actions_and_parents
                    );
                }

                continue 'outer;
            }
        }

        panic!(
            "Expected action not found: {:?} ({:#?} {:#?})",
            (expected_comp_id, expected_kind),
            actions,
            expected_actions_and_parents
        );
    }

    assert_eq!(actions.len(), expected_actions_and_parents.len());

    let fix_batch_history_views = harness.list_fixes(ctx.visibility()).await;
    assert!(fix_batch_history_views.is_empty());

    // Apply the change set and get rolling!
    harness
        .apply_change_set_and_update_ctx_visibility_to_head(&mut ctx)
        .await;

    // Check that they succeeded.
    let mut fix_batch_history_views = harness.list_fixes(ctx.visibility()).await;
    let fix_batch_history_view = fix_batch_history_views.pop().expect("no fix batches found");
    assert_eq!(
        Some(FixCompletionStatus::Success), // expected
        fix_batch_history_view.status
    );

    ctx.rollback().await.expect("unable to rollback ctx");

    let key_pair_comp = Component::get_by_id(&ctx, &key_pair.component_id)
        .await
        .expect("unable to get key pair")
        .expect("unable to get a key pair");
    assert!(key_pair_comp
        .resource(&ctx)
        .await
        .expect("unable to get resource")
        .payload
        .is_some());

    let ec2_comp = Component::get_by_id(&ctx, &ec2.component_id)
        .await
        .expect("unable to get ec2")
        .expect("unable to get a ec2");
    assert!(ec2_comp
        .resource(&ctx)
        .await
        .expect("unable to get resource")
        .payload
        .is_some());

    let security_group_comp = Component::get_by_id(&ctx, &security_group.component_id)
        .await
        .expect("unable to get security_group")
        .expect("unable to get a security_group");
    assert!(security_group_comp
        .resource(&ctx)
        .await
        .expect("unable to get resource")
        .payload
        .is_some());

    let ingress_comp = Component::get_by_id(&ctx, &ingress.component_id)
        .await
        .expect("unable to get ingress")
        .expect("unable to get a ingress");
    assert!(ingress_comp
        .resource(&ctx)
        .await
        .expect("unable to get resource")
        .payload
        .is_some());

    // // let's refresh the resources and check what they are
    // harness.refresh_resources(&mut ctx).await;
    //
    // // check that we have some resources
    // let _x = dbg!(docker
    //     .view(&ctx)
    //     .await
    //     .drop_qualification()
    //     .drop_code()
    //     .to_value());

    // Create a new change set to delete the components
    harness
        .create_change_set_and_update_ctx(&mut ctx, ScenarioHarness::generate_fake_name())
        .await;

    // delete AWS components
    harness
        .delete_component(ctx.visibility(), key_pair.component_id)
        .await;
    harness
        .delete_component(ctx.visibility(), ec2.component_id)
        .await;
    harness
        .delete_component(ctx.visibility(), security_group.component_id)
        .await;
    harness
        .delete_component(ctx.visibility(), ingress.component_id)
        .await;

    let actions = harness
        .list_actions_for_changeset(ctx.visibility())
        .await
        .actions;

    let expected_actions_and_parents = [
        (
            key_pair.component_id,
            ActionKind::Delete,
            vec![(ec2.component_id, ActionKind::Delete)],
        ),
        (
            security_group.component_id,
            ActionKind::Delete,
            vec![
                (ec2.component_id, ActionKind::Delete),
                (ingress.component_id, ActionKind::Delete),
            ],
        ),
        (ingress.component_id, ActionKind::Delete, Vec::new()),
        (ec2.component_id, ActionKind::Delete, Vec::new()),
    ];
    dbg!(&actions, &expected_actions_and_parents);

    'outer: for (expected_comp_id, expected_kind, expected_parents) in &expected_actions_and_parents
    {
        for action in actions.values() {
            if *expected_comp_id == action.component_id
                && *expected_kind == action.kind
                && action.parents.len() == expected_parents.len()
            {
                'parent_outer: for (expected_comp_id, expected_kind) in expected_parents {
                    for action in actions.values() {
                        if *expected_comp_id == action.component_id && *expected_kind == action.kind
                        {
                            continue 'parent_outer;
                        }
                    }

                    panic!(
                        "Expected parent action not found: {:?} ({:#?} {:#?})",
                        (expected_comp_id, expected_kind),
                        actions,
                        expected_actions_and_parents
                    );
                }

                continue 'outer;
            }
        }

        panic!(
            "Expected action not found: {:?} ({:#?} {:#?})",
            (expected_comp_id, expected_kind),
            actions,
            expected_actions_and_parents
        );
    }

    assert_eq!(actions.len(), expected_actions_and_parents.len());

    let num_of_fix_batch_history_views = harness.list_fixes(ctx.visibility()).await.len();

    harness
        .apply_change_set_and_update_ctx_visibility_to_head(&mut ctx)
        .await;

    // Check that they succeeded.
    let mut fix_batch_history_views = dbg!(harness.list_fixes(ctx.visibility()).await);
    assert_ne!(
        fix_batch_history_views.len(),
        num_of_fix_batch_history_views
    );

    let fix_batch_history_view = fix_batch_history_views.pop().expect("no fix batches found");
    assert_eq!(
        Some(FixCompletionStatus::Success), // expected
        fix_batch_history_view.status
    );

    ctx.rollback().await.expect("unable to rollback ctx");

    let deleted_ctx = &ctx.clone_with_delete_visibility();
    let key_pair_comp = Component::get_by_id(deleted_ctx, &key_pair.component_id)
        .await
        .expect("unable to get key pair")
        .expect("unable to get a key pair");
    assert!(key_pair_comp.is_destroyed());

    let ec2_comp = Component::get_by_id(deleted_ctx, &ec2.component_id)
        .await
        .expect("unable to get ec2")
        .expect("unable to get a ec2");
    assert!(ec2_comp.is_destroyed());

    let security_group_comp = Component::get_by_id(deleted_ctx, &security_group.component_id)
        .await
        .expect("unable to get security_group")
        .expect("unable to get a security_group");
    assert!(security_group_comp.is_destroyed());

    let ingress_comp = Component::get_by_id(deleted_ctx, &ingress.component_id)
        .await
        .expect("unable to get ingress")
        .expect("unable to get a ingress");
    assert!(ingress_comp.is_destroyed());

    // TODO(nick): mix in creation and deletion actions as well as scenarios where not
    // all fixes are ran all at once.
}
