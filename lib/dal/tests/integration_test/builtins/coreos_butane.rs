use dal::{
    qualification::QualificationSubCheckStatus, Component, ComponentId, DalContext, Edge,
    ExternalProvider, InternalProvider, StandardModel,
};
use dal_test::{
    helpers::builtins::{Builtin, SchemaBuiltinsTestHarness},
    test,
};
use pretty_assertions_sorted::assert_eq;

const SYSTEMD_UNIT_FILE_PAYLOAD: &str = "\
[Unit]
Description=Whiskers
After=network-online.target
Wants=network-online.target

[Service]
TimeoutStartSec=0
ExecStartPre=-/bin/podman kill whiskers1
ExecStartPre=-/bin/podman rm whiskers1
ExecStartPre=/bin/podman pull docker.io/systeminit/whiskers
ExecStart=/bin/podman run --name whiskers1 --publish 80:80 docker.io/systeminit/whiskers

[Install]
WantedBy=multi-user.target";

// NOTE(nick): these were nasty inline. Trust me, you want these files. Doesn't matter where, you
// absolutely want them though.
const EXPECTED_BUTANE_TO_EC2_IGNITION: &str = include_str!("ignition/butane-to-ec2.ign");
const EXPECTED_DOCKER_TO_BUTANE_IGNITION: &str = include_str!("ignition/docker-to-butane.ign");

#[test]
async fn butane_to_ec2_user_data_is_valid_ignition(ctx: &DalContext) {
    let mut harness = SchemaBuiltinsTestHarness::new();
    let butane_payload = harness
        .create_component(ctx, "Mimic Tear", Builtin::CoreOsButane)
        .await;
    let ec2_payload = harness
        .create_component(ctx, "Regal Ancestor Spirit", Builtin::AwsEc2)
        .await;

    // First, connect the two components together.
    let ec2_user_data_explicit_internal_provider =
        InternalProvider::find_explicit_for_schema_variant_and_name(
            ctx,
            ec2_payload.schema_variant_id,
            "User Data",
        )
        .await
        .expect("could not perform explicit internal provider find")
        .expect("no explicit internal provider found");
    let butane_user_data_external_provider = ExternalProvider::find_for_schema_variant_and_name(
        ctx,
        butane_payload.schema_variant_id,
        "User Data",
    )
    .await
    .expect("could not perform external provider find")
    .expect("no external provider found");
    Edge::connect_providers_for_components(
        ctx,
        *ec2_user_data_explicit_internal_provider.id(),
        ec2_payload.component_id,
        *butane_user_data_external_provider.id(),
        butane_payload.component_id,
    )
    .await
    .expect("could not connect providers for components");

    // Update all required fields for generating ignition.
    let element_attribute_value_id = butane_payload
        .insert_array_object_element(
            ctx,
            "/root/domain/systemd/units",
            "/root/domain/systemd/units/unit",
        )
        .await;
    butane_payload
        .update_attribute_value_for_prop_name_and_parent_element_attribute_value_id(
            ctx,
            "/root/domain/systemd/units/unit/name",
            Some(serde_json::json!["whiskers.service"]),
            element_attribute_value_id,
        )
        .await;
    butane_payload
        .update_attribute_value_for_prop_name_and_parent_element_attribute_value_id(
            ctx,
            "/root/domain/systemd/units/unit/enabled",
            Some(serde_json::json![true]),
            element_attribute_value_id,
        )
        .await;
    butane_payload
        .update_attribute_value_for_prop_name_and_parent_element_attribute_value_id(
            ctx,
            "/root/domain/systemd/units/unit/contents",
            Some(serde_json::json![SYSTEMD_UNIT_FILE_PAYLOAD]),
            element_attribute_value_id,
        )
        .await;

    // Ensure everything looks as expected.
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "Mimic Tear",
                "color": "#e26b70",
                "type": "component",
                "protected": false,
            },
            "domain": {
                "systemd": {
                    "units": [
                        {
                            "name": "whiskers.service",
                            "enabled": true,
                            "contents": "[Unit]\nDescription=Whiskers\nAfter=network-online.target\nWants=network-online.target\n\n[Service]\nTimeoutStartSec=0\nExecStartPre=-/bin/podman kill whiskers1\nExecStartPre=-/bin/podman rm whiskers1\nExecStartPre=/bin/podman pull docker.io/systeminit/whiskers\nExecStart=/bin/podman run --name whiskers1 --publish 80:80 docker.io/systeminit/whiskers\n\n[Install]\nWantedBy=multi-user.target",
                        },
                    ],
                },
                "variant": "fcos",
                "version": "1.4.0",
            },
            "code": {
                "si:generateButaneIgnition": {
                    "code": "{\n  \"ignition\": {\n    \"version\": \"3.3.0\"\n  },\n  \"systemd\": {\n    \"units\": [\n      {\n        \"contents\": \"[Unit]\\nDescription=Whiskers\\nAfter=network-online.target\\nWants=network-online.target\\n\\n[Service]\\nTimeoutStartSec=0\\nExecStartPre=-/bin/podman kill whiskers1\\nExecStartPre=-/bin/podman rm whiskers1\\nExecStartPre=/bin/podman pull docker.io/systeminit/whiskers\\nExecStart=/bin/podman run --name whiskers1 --publish 80:80 docker.io/systeminit/whiskers\\n\\n[Install]\\nWantedBy=multi-user.target\",\n        \"enabled\": true,\n        \"name\": \"whiskers.service\"\n      }\n    ]\n  }\n}",
                    "format": "json",
                },
            },
        }], // expected
        butane_payload
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value()
            .expect("could not convert to value") // actual
    );

    // FIXME(nick): there is a race here where the "generateAwsEc2JSON" function needs to run
    // again to include the new "UserData" in its code generation output. Sometimes, the "UserData"
    // populated when we generate the view. Other times, it is not. Thus, this test _temporarily_
    // looks at the contents of the "UserData" field alone, rather than the entire object.
    let ec2_component_view_properties = ec2_payload.component_view_properties_raw(ctx).await;
    let ec2_properties = ec2_component_view_properties
        .as_object()
        .expect("could not convert ec2 component view properties to object");
    let ec2_domain = ec2_properties["domain"]
        .as_object()
        .expect("could not find domain object off ec2 component view properties");
    let ec2_user_data = ec2_domain["UserData"].clone();
    assert_eq!(
        serde_json::json!["{\n  \"ignition\": {\n    \"version\": \"3.3.0\"\n  },\n  \"systemd\": {\n    \"units\": [\n      {\n        \"contents\": \"[Unit]\\nDescription=Whiskers\\nAfter=network-online.target\\nWants=network-online.target\\n\\n[Service]\\nTimeoutStartSec=0\\nExecStartPre=-/bin/podman kill whiskers1\\nExecStartPre=-/bin/podman rm whiskers1\\nExecStartPre=/bin/podman pull docker.io/systeminit/whiskers\\nExecStart=/bin/podman run --name whiskers1 --publish 80:80 docker.io/systeminit/whiskers\\n\\n[Install]\\nWantedBy=multi-user.target\",\n        \"enabled\": true,\n        \"name\": \"whiskers.service\"\n      }\n    ]\n  }\n}"],
        ec2_user_data
    );
    // assert_eq!(
    //     serde_json::json![{
    //         "si": {
    //             "name": "Regal Ancestor Spirit",
    //         },
    //         "code": {
    //             "si:generateAwsEc2JSON": {
    //                 "code": "{\n\t\"UserData\": \"\",\n\t\"TagSpecifications\": [\n\t\t{\n\t\t\t\"ResourceType\": \"instance\",\n\t\t\t\"Tags\": [\n\t\t\t\t{\n\t\t\t\t\t\"Key\": \"Name\",\n\t\t\t\t\t\"Value\": \"Regal Ancestor Spirit\"\n\t\t\t\t}\n\t\t\t]\n\t\t}\n\t]\n}",
    //                 "format": "json",
    //             },
    //         },
    //         "domain": {
    //             "tags": {
    //                 "Name": "Regal Ancestor Spirit",
    //             },
    //             "UserData": "{\n  \"ignition\": {\n    \"version\": \"3.3.0\"\n  },\n  \"systemd\": {\n    \"units\": [\n      {\n        \"contents\": \"[Unit]\\nDescription=Whiskers\\nAfter=network-online.target\\nWants=network-online.target\\n\\n[Service]\\nTimeoutStartSec=0\\nExecStartPre=-/bin/podman kill whiskers1\\nExecStartPre=-/bin/podman rm whiskers1\\nExecStartPre=/bin/podman pull docker.io/systeminit/whiskers\\nExecStart=/bin/podman run --name whiskers1 --publish 80:80 docker.io/systeminit/whiskers\\n\\n[Install]\\nWantedBy=multi-user.target\",\n        \"enabled\": true,\n        \"name\": \"whiskers.service\"\n      }\n    ]\n  }\n}",
    //             "awsResourceType": "instance",
    //         },
    //     }], // expected
    //     ec2_payload.component_view_properties(ctx).await // actual
    // );

    // Finally, check the ignition qualification.
    let ignition = get_ignition_from_qualification_output(ctx, &butane_payload.component_id).await;
    assert_eq!(
        &ignition,                       // actual
        EXPECTED_BUTANE_TO_EC2_IGNITION  // expected
    );
}

#[test]
async fn docker_to_butane_is_valid_ignition(ctx: &DalContext) {
    let mut harness = SchemaBuiltinsTestHarness::new();
    let alpine_payload = harness
        .create_component(ctx, "alpine", Builtin::DockerImage)
        .await;
    let butane_payload = harness
        .create_component(ctx, "butane", Builtin::CoreOsButane)
        .await;
    let nginx_payload = harness
        .create_component(ctx, "nginx", Builtin::DockerImage)
        .await;

    // Collect the providers needed to perform the two connections from each docker image to butane.
    let alpine_provider = ExternalProvider::find_for_schema_variant_and_name(
        ctx,
        alpine_payload.schema_variant_id,
        "Container Image",
    )
    .await
    .expect("cannot find external provider")
    .expect("external provider not found");
    let nginx_provider = ExternalProvider::find_for_schema_variant_and_name(
        ctx,
        nginx_payload.schema_variant_id,
        "Container Image",
    )
    .await
    .expect("cannot find external provider")
    .expect("external provider not found");
    let butane_provider = InternalProvider::find_explicit_for_schema_variant_and_name(
        ctx,
        butane_payload.schema_variant_id,
        "Container Image",
    )
    .await
    .expect("cannot find explicit internal provider")
    .expect("explicit internal provider not found");

    // Perform the two connections.
    Edge::connect_providers_for_components(
        ctx,
        *butane_provider.id(),
        butane_payload.component_id,
        *alpine_provider.id(),
        alpine_payload.component_id,
    )
    .await
    .expect("could not connect providers for components");
    Edge::connect_providers_for_components(
        ctx,
        *butane_provider.id(),
        butane_payload.component_id,
        *nginx_provider.id(),
        nginx_payload.component_id,
    )
    .await
    .expect("could not connect providers for components");

    // Set values required for butane.
    alpine_payload
        .update_attribute_value_for_prop_name(
            ctx,
            "/root/si/name",
            Some(serde_json::json!["alpine8675"]),
        )
        .await;
    let alpine_image_value =
        serde_json::to_value("docker.io/library/alpine").expect("could not convert to value");
    alpine_payload
        .update_attribute_value_for_prop_name(ctx, "/root/domain/image", Some(alpine_image_value))
        .await;
    nginx_payload
        .update_attribute_value_for_prop_name(
            ctx,
            "/root/si/name",
            Some(serde_json::json!["nginx309"]),
        )
        .await;
    let nginx_image_value =
        serde_json::to_value("docker.io/library/nginx").expect("could not convert to value");
    nginx_payload
        .update_attribute_value_for_prop_name(ctx, "/root/domain/image", Some(nginx_image_value))
        .await;
    let nginx_port80_value = serde_json::to_value("80/tcp").expect("could not convert to value");
    nginx_payload
        .insert_array_primitive_element(
            ctx,
            "/root/domain/exposed-ports",
            "/root/domain/exposed-ports/exposed-port",
            nginx_port80_value,
        )
        .await;
    let nginx_port443_value = serde_json::to_value("443/tcp").expect("could not convert to value");
    nginx_payload
        .insert_array_primitive_element(
            ctx,
            "/root/domain/exposed-ports",
            "/root/domain/exposed-ports/exposed-port",
            nginx_port443_value,
        )
        .await;

    // Check the ignition qualification.
    let ignition = get_ignition_from_qualification_output(ctx, &butane_payload.component_id).await;
    assert_eq!(
        &ignition,                          // actual
        EXPECTED_DOCKER_TO_BUTANE_IGNITION  // expected
    );
}

/// Combs through the qualifications for a Butane [Component](crate::Component) and returns the
/// Ignition output from the relevant qualification.
async fn get_ignition_from_qualification_output(
    ctx: &DalContext,
    butane_component_id: &ComponentId,
) -> String {
    let component = Component::get_by_id(ctx, butane_component_id)
        .await
        .expect("could not find component by id")
        .expect("component not found by id");
    let qualifications = Component::list_qualifications(ctx, *component.id())
        .await
        .expect("could not list qualifications");
    let mut messages = qualifications
        .iter()
        .filter(|qv| qv.title == "Verify Butane config is valid Ignition")
        .map(|qv| {
            // First, ensure the qualification contained a successful result.
            let mut qualification_result = qv
                .result
                .clone()
                .expect("could not get result from qualification view");
            assert_eq!(
                qualification_result.status,
                QualificationSubCheckStatus::Success
            );

            // Find the output in the sub check. Ensure there's only one sub check.
            let sub_check = qualification_result
                .sub_checks
                .pop()
                .expect("no sub checks found");
            assert!(qualification_result.sub_checks.is_empty());
            sub_check.description

            // TODO(nick): decide if we want to see the output stream the same way now that
            // qualifications are on the prop tree.
            // Then, find the return line. This should be the "pretty" ignition output.
            // let mut lines_from_stream_views = qv
            //     .output
            //     .iter()
            //     .filter(|sv| sv.stream == "return")
            //     .map(|sv| sv.line.clone())
            //     .collect::<Vec<String>>();
            // let line = lines_from_stream_views
            //     .pop()
            //     .expect("lines from filtered stream views are empty");
            // assert!(lines_from_stream_views.is_empty());
            // line
        })
        .collect::<Vec<String>>();

    // Return the ignition.
    let ignition = messages.pop().expect("messages are empty");
    assert!(messages.is_empty());
    ignition
}
