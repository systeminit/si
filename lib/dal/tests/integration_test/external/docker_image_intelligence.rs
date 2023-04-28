use dal::{DalContext, StandardModel};
use dal_test::helpers::component_bag::ComponentBagger;
use dal_test::test;
use pretty_assertions_sorted::assert_eq;

#[test]
async fn docker_image_intra_component_update(ctx: &DalContext) {
    let mut bagger = ComponentBagger::new();
    let soulrender_bag = bagger
        .create_component(ctx, "soulrender", "Docker Image")
        .await;
    let bloodscythe_bag = bagger
        .create_component(ctx, "bloodscythe", "Docker Image")
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
        soulrender_bag
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
        bloodscythe_bag
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value()
            .expect("could not convert to value") // actual
    );

    // Update the "/root/si/name" value for "bloodscythe", observe that it worked, and observe
    // that the "soulrender" component was not updated.
    let name_prop = bloodscythe_bag
        .find_prop(ctx, &["root", "si", "name"])
        .await;
    bloodscythe_bag
        .update_attribute_value_for_prop(
            ctx,
            *name_prop.id(),
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
        soulrender_bag
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
        bloodscythe_bag
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value()
            .expect("could not convert to value") // actual
    );

    // Now, the "/root/si/name" value for "soulrender", observe that it worked, and observe
    // that the "bloodscythe" component was not updated.
    soulrender_bag
        .update_attribute_value_for_prop(
            ctx,
            *name_prop.id(),
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
        soulrender_bag
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
        bloodscythe_bag
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value()
            .expect("could not convert to value") // actual
    );
}
