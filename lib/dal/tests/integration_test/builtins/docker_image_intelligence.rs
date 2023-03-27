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

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    // Ensure that setup worked
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "soulrender",
                "color": "#4695E7",
                "type": "component",
                "protected": false,
            },
            "domain": {
                "image": "soulrender",
            },
        }], // expected
        soulrender_payload
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value()
            .expect("could not convert to value") // actual
    );
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "bloodscythe",
                "color": "#4695E7",
                "type": "component",
                "protected": false,
            },
            "domain": {
                "image": "bloodscythe",
            },
        }], // expected
        bloodscythe_payload
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value()
            .expect("could not convert to value") // actual
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

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "soulrender",
                "color": "#4695E7",
                "type": "component",
                "protected": false,
            },
            "domain": {
                "image": "soulrender"
            },
        }], // expected
        soulrender_payload
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value()
            .expect("could not convert to value") // actual
    );

    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "bloodscythe-updated",
                "color": "#4695E7",
                "type": "component",
                "protected": false,
            },
            "domain": {
                "image": "bloodscythe-updated"
            },
        }], // expected
        bloodscythe_payload
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value()
            .expect("could not convert to value") // actual
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

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "soulrender-updated",
                "color": "#4695E7",
                "type": "component",
                "protected": false,
            },
            "domain": {
                "image": "soulrender-updated",
            },
        }], // expected
        soulrender_payload
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value()
            .expect("could not convert to value") // actual
    );
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "bloodscythe-updated",
                "color": "#4695E7",
                "type": "component",
                "protected": false,
            },
            "domain": {
                "image": "bloodscythe-updated",
            },
        }], // expected
        bloodscythe_payload
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value()
            .expect("could not convert to value") // actual
    );
}
