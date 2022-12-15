use dal::DalContext;
use dal_test::{
    helpers::builtins::{Builtin, SchemaBuiltinsTestHarness},
    test,
};
use pretty_assertions_sorted::assert_eq;

#[test]
async fn docker_image_intra_component_update(ctx: &DalContext) {
    let mut harness = SchemaBuiltinsTestHarness::new();
    let soulrender_payload = harness
        .create_component(ctx, "soulrender", Builtin::DockerImage)
        .await;
    let bloodscythe_payload = harness
        .create_component(ctx, "bloodscythe", Builtin::DockerImage)
        .await;

    // Ensure that setup worked
    assert_eq!(
        serde_json::json![{
            "domain": {
                "image": "soulrender"
            },
            "si": {
                "name": "soulrender",
                "type": "component"
            },
        }], // expected
        soulrender_payload
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value() // actual
    );
    assert_eq!(
        serde_json::json![{
            "domain": {
                "image": "bloodscythe"
            },
            "si": {
                "name": "bloodscythe",
                "type": "component"
            },
        }], // expected
        bloodscythe_payload
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value() // actual
    );

    // Update the "/root/si/name" value for "bloodscythe", observe that it worked, and observe
    // that the "soulrender" component was not updated.
    bloodscythe_payload
        .update_attribute_value_for_prop_name(
            ctx,
            "/root/si/name",
            Some(serde_json::json!["bloodscythe-updated"]),
        )
        .await;

    assert_eq!(
        serde_json::json![{
            "domain": {
                "image": "soulrender"
            },
            "si": {
                "name": "soulrender",
                "type": "component"
            },
        }], // expected
        soulrender_payload
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value() // actual
    );

    assert_eq!(
        serde_json::json![{
            "domain": {
                "image": "bloodscythe-updated"
            },
            "si": {
                "name": "bloodscythe-updated",
                "type": "component"
            },
        }], // expected
        bloodscythe_payload
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value() // actual
    );

    // Now, the "/root/si/name" value for "soulrender", observe that it worked, and observe
    // that the "bloodscythe" component was not updated.
    soulrender_payload
        .update_attribute_value_for_prop_name(
            ctx,
            "/root/si/name",
            Some(serde_json::json!["soulrender-updated"]),
        )
        .await;

    assert_eq!(
        serde_json::json![{
            "domain": {
                "image": "soulrender-updated"
            },
            "si": {
                "name": "soulrender-updated",
                "type": "component"
            },
        }], // expected
        soulrender_payload
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value() // actual // actual
    );
    assert_eq!(
        serde_json::json![{
            "domain": {
                "image": "bloodscythe-updated"
            },
            "si": {
                "name": "bloodscythe-updated",
                "type": "component"
            },
        }], // expected
        bloodscythe_payload
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value() // actual
    );
}
