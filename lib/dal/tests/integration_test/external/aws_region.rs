use dal::{ComponentType, DalContext, Edge, ExternalProvider, InternalProvider, StandardModel};
use dal_test::helpers::component_bag::ComponentBagger;
use dal_test::test;
use pretty_assertions_sorted::assert_eq;

#[test]
async fn aws_region_to_aws_ec2_intelligence(ctx: &DalContext) {
    let mut bagger = ComponentBagger::new();
    let ec2_bag = bagger.create_component(ctx, "server", "EC2 Instance").await;
    let region_bag = bagger.create_component(ctx, "region", "Region").await;

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    // Ensure the component type is a frame, which should be the default.
    let region_component = region_bag.component(ctx).await;
    let component_type = region_component
        .get_type(ctx)
        .await
        .expect("could not get type");
    assert_eq!(
        ComponentType::ConfigurationFrameDown, // expected
        component_type,                        // actual
    );

    // Initialize the tail name field.
    let region_prop_id = *region_bag
        .find_prop(ctx, &["root", "domain", "region"])
        .await
        .id();
    region_bag
        .update_attribute_value_for_prop(ctx, region_prop_id, Some(serde_json::json!["us-east-2"]))
        .await;

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    // Ensure setup worked.
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "us-east-2",
                "color": "#FF9900",
                "type": "configurationFrameDown",
                "protected": false,
            },
            "domain": {
                "region": "us-east-2",
            },
        }], // expected
        region_bag
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value()
            .expect("could not convert to value") // actual
    );
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "server",
                "color": "#FF9900",
                "type": "component",
                "protected": false,
            },
            "domain": {
                "awsResourceType": "instance",
                "tags": {
                    "Name": "server",
                },
            },
            "code": {
                "si:generateAwsEc2JSON": {
                    "code": "{\n\t\"TagSpecifications\": [\n\t\t{\n\t\t\t\"ResourceType\": \"instance\",\n\t\t\t\"Tags\": [\n\t\t\t\t{\n\t\t\t\t\t\"Key\": \"Name\",\n\t\t\t\t\t\"Value\": \"server\"\n\t\t\t\t}\n\t\t\t]\n\t\t}\n\t]\n}",
                    "format": "json",
                },
            },
        }], // expected
        ec2_bag
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value()
            .expect("could not convert to value") // actual
    );

    // Find the providers we need for connection.
    let region_external_provider = ExternalProvider::find_for_schema_variant_and_name(
        ctx,
        region_bag.schema_variant_id,
        "Region",
    )
    .await
    .expect("cannot find external provider")
    .expect("external provider not found");
    let ec2_explicit_internal_provider =
        InternalProvider::find_explicit_for_schema_variant_and_name(
            ctx,
            ec2_bag.schema_variant_id,
            "Region",
        )
        .await
        .expect("cannot find explicit internal provider")
        .expect("explicit internal provider not found");

    // Finally, create the inter component connection.
    Edge::connect_providers_for_components(
        ctx,
        *ec2_explicit_internal_provider.id(),
        ec2_bag.component_id,
        *region_external_provider.id(),
        region_bag.component_id,
    )
    .await
    .expect("could not connect providers");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    // Ensure the view did not drift.
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "us-east-2",
                "color": "#FF9900",
                "type": "configurationFrameDown",
                "protected": false,
            },
            "domain": {
                "region": "us-east-2"
            },
        }], // expected
        region_bag
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value()
            .expect("could not convert to value") // actual
    );
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "server",
                "color": "#FF9900",
                "type": "component",
                "protected": false,
            },
            "domain": {
                "awsResourceType": "instance",
                "tags": {
                    "Name": "server",
                },
            },
            "code": {
                "si:generateAwsEc2JSON": {
                    "code": "{\n\t\"TagSpecifications\": [\n\t\t{\n\t\t\t\"ResourceType\": \"instance\",\n\t\t\t\"Tags\": [\n\t\t\t\t{\n\t\t\t\t\t\"Key\": \"Name\",\n\t\t\t\t\t\"Value\": \"server\"\n\t\t\t\t}\n\t\t\t]\n\t\t}\n\t]\n}",
                    "format": "json",
                },
            },
        }], // expected
        ec2_bag
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value()
            .expect("could not convert to value") // actual
    );

    // Perform update!
    region_bag
        .update_attribute_value_for_prop(ctx, region_prop_id, Some(serde_json::json!["us-west-2"]))
        .await;

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    // Observed that it worked.
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "us-west-2",
                "color": "#FF9900",
                "type": "configurationFrameDown",
                "protected": false,
            },
            "domain": {
                "region": "us-west-2"
            },
        }], // expected
        region_bag
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value()
            .expect("could not convert to value") // actual
    );
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "server",
                "color": "#FF9900",
                "type": "component",
                "protected": false,
            },
            "domain": {
                "awsResourceType": "instance",
                "region": "us-west-2",
                "tags": {
                    "Name": "server",
                },
            },
            "code": {
                "si:generateAwsEc2JSON": {
                    "code": "{\n\t\"TagSpecifications\": [\n\t\t{\n\t\t\t\"ResourceType\": \"instance\",\n\t\t\t\"Tags\": [\n\t\t\t\t{\n\t\t\t\t\t\"Key\": \"Name\",\n\t\t\t\t\t\"Value\": \"server\"\n\t\t\t\t}\n\t\t\t]\n\t\t}\n\t]\n}",
                    "format": "json",
                },
            },
        }], // expected
        ec2_bag
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value()
            .expect("could not convert to value") // actual
    );
}

#[test]
async fn aws_region_to_aws_ec2_intelligence_switch_component_type(ctx: &DalContext) {
    let mut bagger = ComponentBagger::new();
    let ec2_bag = bagger.create_component(ctx, "server", "EC2 Instance").await;
    let region_bag = bagger.create_component(ctx, "region", "Region").await;

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    // Switch the component type to a component, which should not be the default.
    let region_component = region_bag.component(ctx).await;
    let component_type = region_component
        .get_type(ctx)
        .await
        .expect("could not get type");
    assert_eq!(
        ComponentType::ConfigurationFrameDown, // expected
        component_type,                        // actual
    );
    region_component
        .set_type(ctx, ComponentType::Component)
        .await
        .expect("could not set component type");
    let updated_component_type = region_component
        .get_type(ctx)
        .await
        .expect("could not get type");
    assert_eq!(
        ComponentType::Component, // expected
        updated_component_type,   // actual
    );

    // Initialize the tail name field.
    let region_prop_id = *region_bag
        .find_prop(ctx, &["root", "domain", "region"])
        .await
        .id();
    region_bag
        .update_attribute_value_for_prop(ctx, region_prop_id, Some(serde_json::json!["us-east-2"]))
        .await;

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    // Ensure setup worked.
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "us-east-2",
                "color": "#FF9900",
                "type": "component",
                "protected": false,
            },
            "domain": {
                "region": "us-east-2",
            },
        }], // expected
        region_bag
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value()
            .expect("could not convert to value") // actual
    );
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "server",
                "type": "component",
                "color": "#FF9900",
                "protected": false,
            },
            "domain": {
                "awsResourceType": "instance",
                "tags": {
                    "Name": "server",
                },
            },
            "code": {
                "si:generateAwsEc2JSON": {
                    "code": "{\n\t\"TagSpecifications\": [\n\t\t{\n\t\t\t\"ResourceType\": \"instance\",\n\t\t\t\"Tags\": [\n\t\t\t\t{\n\t\t\t\t\t\"Key\": \"Name\",\n\t\t\t\t\t\"Value\": \"server\"\n\t\t\t\t}\n\t\t\t]\n\t\t}\n\t]\n}",
                    "format": "json",
                },
            },
        }], // expected
        ec2_bag
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value()
            .expect("could not convert to value") // actual
    );

    // Find the providers we need for connection.
    let region_external_provider = ExternalProvider::find_for_schema_variant_and_name(
        ctx,
        region_bag.schema_variant_id,
        "Region",
    )
    .await
    .expect("cannot find external provider")
    .expect("external provider not found");
    let ec2_explicit_internal_provider =
        InternalProvider::find_explicit_for_schema_variant_and_name(
            ctx,
            ec2_bag.schema_variant_id,
            "Region",
        )
        .await
        .expect("cannot find explicit internal provider")
        .expect("explicit internal provider not found");

    // Finally, create the inter component connection.
    Edge::connect_providers_for_components(
        ctx,
        *ec2_explicit_internal_provider.id(),
        ec2_bag.component_id,
        *region_external_provider.id(),
        region_bag.component_id,
    )
    .await
    .expect("could not connect providers");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    // Ensure the view did not drift.
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "us-east-2",
                "color": "#FF9900",
                "type": "component",
                "protected": false,
            },
            "domain": {
                "region": "us-east-2"
            },
        }], // expected
        region_bag
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value()
            .expect("could not convert to value") // actual
    );
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "server",
                "color": "#FF9900",
                "type": "component",
                "protected": false,
            },
            "domain": {
                "awsResourceType": "instance",
                "tags": {
                    "Name": "server",
                },
            },
            "code": {
                "si:generateAwsEc2JSON": {
                    "code": "{\n\t\"TagSpecifications\": [\n\t\t{\n\t\t\t\"ResourceType\": \"instance\",\n\t\t\t\"Tags\": [\n\t\t\t\t{\n\t\t\t\t\t\"Key\": \"Name\",\n\t\t\t\t\t\"Value\": \"server\"\n\t\t\t\t}\n\t\t\t]\n\t\t}\n\t]\n}",
                    "format": "json",
                },
            },
        }], // expected
        ec2_bag
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value()
            .expect("could not convert to value") // actual
    );

    // Perform update!
    region_bag
        .update_attribute_value_for_prop(ctx, region_prop_id, Some(serde_json::json!["us-west-2"]))
        .await;

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    // Observed that it worked.
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "us-west-2",
                "color": "#FF9900",
                "type": "component",
                "protected": false,
            },
            "domain": {
                "region": "us-west-2"
            },
        }], // expected
        region_bag
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value()
            .expect("could not convert to value") // actual
    );
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "server",
                "color": "#FF9900",
                "type": "component",
                "protected": false,
            },
            "domain": {
                "awsResourceType": "instance",
                "region": "us-west-2",
                "tags": {
                    "Name": "server",
                },
            },
            "code": {
                "si:generateAwsEc2JSON": {
                    "code": "{\n\t\"TagSpecifications\": [\n\t\t{\n\t\t\t\"ResourceType\": \"instance\",\n\t\t\t\"Tags\": [\n\t\t\t\t{\n\t\t\t\t\t\"Key\": \"Name\",\n\t\t\t\t\t\"Value\": \"server\"\n\t\t\t\t}\n\t\t\t]\n\t\t}\n\t]\n}",
                    "format": "json",
                },
            },
        }], // expected
        ec2_bag
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value()
            .expect("could not convert to value") // actual
    );
}
