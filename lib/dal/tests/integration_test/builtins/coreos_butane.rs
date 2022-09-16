use crate::dal::test;
use dal::test::helpers::builtins::{Builtin, BuiltinsHarness};

use dal::{Component, DalContext, StandardModel, SystemId};
use pretty_assertions_sorted::assert_eq_sorted;

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

// NOTE(nick): this was nasty inline. Trust me, you want this file. Doesn't matter where, you
// absolutely want it though.
const EXPECTED_IGNITION_OUTPUT: &str = include_str!("expected.ign");

#[test]
async fn butane_is_valid_ignition(ctx: &DalContext) {
    let mut harness = BuiltinsHarness::new();
    let butane_payload = harness
        .create_component(ctx, "butane", Builtin::CoreOsButane)
        .await;

    // "Fill out" the entire component.
    butane_payload
        .update_attribute_value_for_prop_name(
            ctx,
            "/root/si/domain/variant",
            Some(serde_json::json!["fcos"]),
        )
        .await;
    butane_payload
        .update_attribute_value_for_prop_name(
            ctx,
            "/root/si/domain/version",
            Some(serde_json::json!["1.4.0"]),
        )
        .await;
    let element_attribute_value_id = butane_payload
        .insert_array_object_element(
            ctx,
            "/root/si/domain/systemd/units",
            "/root/si/domain/systemd/units/unit",
        )
        .await;
    butane_payload
        .update_attribute_value_for_prop_name_and_parent_element_attribute_value_id(
            ctx,
            "/root/si/domain/systemd/units/unit/name",
            Some(serde_json::json!["whiskers.service"]),
            element_attribute_value_id,
        )
        .await;
    butane_payload
        .update_attribute_value_for_prop_name_and_parent_element_attribute_value_id(
            ctx,
            "/root/si/domain/systemd/units/unit/enabled",
            Some(serde_json::json![true]),
            element_attribute_value_id,
        )
        .await;

    // Ensure setup worked.
    assert_eq_sorted!(
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
            "/root/si/domain/systemd/units/unit/contents",
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
    assert_eq_sorted!(
        SYSTEMD_UNIT_FILE_PAYLOAD, // expected
        contents                   // actual
    );

    // Check the ignition qualification.
    let component = Component::get_by_id(ctx, &butane_payload.component_id)
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

    // Perform final assertions based on the output.
    let line = filtered_stream_view_lines
        .pop()
        .expect("filtered streams are empty");
    assert!(filtered_stream_view_lines.is_empty());
    assert_eq_sorted!(
        &line,                    // actual
        EXPECTED_IGNITION_OUTPUT  // expected
    );
}
