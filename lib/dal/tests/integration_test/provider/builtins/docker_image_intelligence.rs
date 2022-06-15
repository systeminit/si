use dal::test::helpers::provider::ProviderBuiltinsHarness;
use dal::DalContext;
use pretty_assertions_sorted::assert_eq_sorted;

use crate::dal::test;

#[test]
async fn docker_image_intra_component_update(ctx: &DalContext<'_, '_>) {
    let mut harness = ProviderBuiltinsHarness::new();
    let soulrender_payload = harness.create_docker_image(ctx, "soulrender").await;
    let bloodscythe_payload = harness.create_docker_image(ctx, "bloodscythe").await;

    // Ensure that setup worked
    assert_eq_sorted!(
        serde_json::json![{
            "domain": {
                "image": "soulrender"
            },
            "si": {
                "name": "soulrender",
            },
        }], // expected
        soulrender_payload.component_view_properties(ctx).await // actual
    );
    assert_eq_sorted!(
        serde_json::json![{
            "domain": {
                "image": "bloodscythe"
            },
            "si": {
                "name": "bloodscythe",
            },
        }], // expected
        bloodscythe_payload.component_view_properties(ctx).await // actual
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

    assert_eq_sorted!(
        serde_json::json![{
            "domain": {
                "image": "soulrender"
            },
            "si": {
                "name": "soulrender",
            },
        }], // expected
        soulrender_payload.component_view_properties(ctx).await // actual
    );

    assert_eq_sorted!(
        serde_json::json![{
            "domain": {
                "image": "bloodscythe-updated"
            },
            "si": {
                "name": "bloodscythe-updated",
            },
        }], // expected
        bloodscythe_payload.component_view_properties(ctx).await // actual
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

    assert_eq_sorted!(
        serde_json::json![{
            "domain": {
                "image": "soulrender-updated"
            },
            "si": {
                "name": "soulrender-updated",
            },
        }], // expected
        soulrender_payload.component_view_properties(ctx).await // actual
    );
    assert_eq_sorted!(
        serde_json::json![{
            "domain": {
                "image": "bloodscythe-updated"
            },
            "si": {
                "name": "bloodscythe-updated",
            },
        }], // expected
        bloodscythe_payload.component_view_properties(ctx).await // actual
    );
}
