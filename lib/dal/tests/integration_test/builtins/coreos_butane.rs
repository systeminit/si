use dal::{
    Component, ComponentId, DalContext, Edge, ExternalProvider, InternalProvider, StandardModel,
    SystemId,
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
const EXPECTED_CONNECTED_IGNITION_OUTPUT: &str = include_str!("ignition/connected.ign");
const EXPECTED_DISCRETE_IGNITION_OUTPUT: &str = include_str!("ignition/discrete.ign");

#[test]
async fn butane_is_valid_ignition(ctx: &DalContext) {
    let mut harness = SchemaBuiltinsTestHarness::new();
    let butane_payload = harness
        .create_component(ctx, "butane", Builtin::CoreOsButane)
        .await;

    // "Fill out" the entire component.
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

    // Ensure setup worked.
    assert_eq!(
        serde_json::json![{
            "domain": {
                "variant": "fcos",
                "version": "1.4.0",
                "systemd": {
                    "units": [
                        {
                            "name": "whiskers.service",
                            "enabled": true,
                        }
                    ]
                }
            },
            "code": {
                "si:generateButaneIgnition": {
                    "code": "{\n  \"ignition\": {\n    \"version\": \"3.3.0\"\n  },\n  \"systemd\": {\n    \"units\": [\n      {\n        \"enabled\": true,\n        \"name\": \"whiskers.service\"\n      }\n    ]\n  }\n}",
                   "format": "json",
                },
            },
            "si": {
                "name": "butane"
            }
        }], // expected
        butane_payload.component_view_properties(ctx).await // actual
    );

    // Add the huge string and ensure serialization worked.
    butane_payload
        .update_attribute_value_for_prop_name_and_parent_element_attribute_value_id(
            ctx,
            "/root/domain/systemd/units/unit/contents",
            Some(serde_json::json![SYSTEMD_UNIT_FILE_PAYLOAD]),
            element_attribute_value_id,
        )
        .await;
    let view = butane_payload.component_view_properties(ctx).await;
    let contents = view
        .get("domain")
        .and_then(|v| v.get("systemd"))
        .and_then(|v| v.get("units"))
        .and_then(|v| v.as_array())
        .and_then(|v| {
            assert_eq!(v.len(), 1);
            v.first()
        })
        .and_then(|v| v.get("contents"))
        .and_then(|v| v.as_str())
        .expect("could not get contents from view");
    assert_eq!(
        SYSTEMD_UNIT_FILE_PAYLOAD, // expected
        contents                   // actual
    );

    // Check the ignition qualification.
    let ignition = get_ignition_from_qualification_output(ctx, &butane_payload.component_id).await;
    assert_eq!(
        &ignition,                         // actual
        EXPECTED_DISCRETE_IGNITION_OUTPUT  // expected
    );
}

#[test]
async fn connected_butane_is_valid_ignition(ctx: &DalContext) {
    let mut harness = SchemaBuiltinsTestHarness::new();
    let butane_payload = harness
        .create_component(ctx, "butane", Builtin::CoreOsButane)
        .await;
    let alpine_payload = harness
        .create_component(ctx, "alpine", Builtin::DockerImage)
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
        EXPECTED_CONNECTED_IGNITION_OUTPUT  // expected
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
    component
        .check_qualifications(ctx, SystemId::NONE)
        .await
        .expect("cannot check qualifications");
    let qualifications = component
        .list_qualifications(ctx, SystemId::NONE)
        .await
        .expect("could not list qualifications");
    let mut filtered_stream_view_lines = qualifications
        .iter()
        .filter(|qv| qv.title == "Verify Butane config is valid Ignition")
        .map(|qv| {
            // First, ensure the qualification contained a successful result.
            let qualification_result = qv
                .result
                .clone()
                .expect("could not get result from qualification view");
            assert!(qualification_result.success);

            // Then, find the return line. This should be the "pretty" ignition output.
            let mut lines_from_stream_views = qv
                .output
                .iter()
                .filter(|sv| sv.stream == "return")
                .map(|sv| sv.line.clone())
                .collect::<Vec<String>>();
            let line = lines_from_stream_views
                .pop()
                .expect("lines from filtered stream views are empty");
            assert!(lines_from_stream_views.is_empty());
            line
        })
        .collect::<Vec<String>>();

    // Return the ignition.
    let ignition = filtered_stream_view_lines
        .pop()
        .expect("filtered streams are empty");
    assert!(filtered_stream_view_lines.is_empty());
    ignition
}
