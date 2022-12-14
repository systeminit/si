use dal::{Component, DalContext, Edge, ExternalProvider, InternalProvider, StandardModel};
use dal_test::{
    helpers::builtins::{Builtin, SchemaBuiltinsTestHarness},
    test,
};
use pretty_assertions_sorted::assert_eq;

#[test]
async fn aws_region_to_aws_ec2_intelligence(ctx: &DalContext) {
    let mut harness = SchemaBuiltinsTestHarness::new();
    let ec2_payload = harness
        .create_component(ctx, "server", Builtin::AwsEc2)
        .await;
    let region_payload = harness
        .create_component(ctx, "region", Builtin::AwsRegion)
        .await;

    // Initialize the tail name field.
    region_payload
        .update_attribute_value_for_prop_name(
            ctx,
            "/root/domain/region",
            Some(serde_json::json!["us-east-2"]),
        )
        .await;

    // Ensure setup worked.
    assert_eq!(
        serde_json::json![{
            "domain": {
                "region": "us-east-2"
            },

            "si": {
                "name": "us-east-2",
                "type": "configurationFrame"
            }
        }], // expected
        region_payload
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .drop_validation()
            .to_value() // actual
    );
    assert_eq!(
        serde_json::json![{
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
            "si": {
                "name": "server",
                "type": "component"
            }
        }], // expected
        ec2_payload
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .drop_validation()
            .to_value() // actual
    );

    // Find the providers we need for connection.
    let region_external_provider = ExternalProvider::find_for_schema_variant_and_name(
        ctx,
        region_payload.schema_variant_id,
        "Region",
    )
    .await
    .expect("cannot find external provider")
    .expect("external provider not found");
    let ec2_explicit_internal_provider =
        InternalProvider::find_explicit_for_schema_variant_and_name(
            ctx,
            ec2_payload.schema_variant_id,
            "Region",
        )
        .await
        .expect("cannot find explicit internal provider")
        .expect("explicit internal provider not found");

    // Finally, create the inter component connection.
    Edge::connect_providers_for_components(
        ctx,
        *ec2_explicit_internal_provider.id(),
        ec2_payload.component_id,
        *region_external_provider.id(),
        region_payload.component_id,
    )
    .await
    .expect("could not connect providers");

    // Ensure the view did not drift.
    assert_eq!(
        serde_json::json![{
            "domain": {
                "region": "us-east-2"
            },

            "si": {
                "name": "us-east-2",
                "type": "configurationFrame"
            }
        }], // expected
        region_payload
            .component_view_properties(ctx)
            .await
            .drop_validation()
            .to_value() // actual
    );
    assert_eq!(
        serde_json::json![{
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
            "si": {
                "name": "server",
                "type": "component"
            }
        }], // expected
        ec2_payload
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .drop_validation()
            .to_value() // actual
    );

    // Perform update!
    region_payload
        .update_attribute_value_for_prop_name(
            ctx,
            "/root/domain/region",
            Some(serde_json::json!["us-west-2"]),
        )
        .await;

    // Observed that it worked.
    assert_eq!(
        serde_json::json![{
            "domain": {
                "region": "us-west-2"
            },

            "si": {
                "name": "us-west-2",
                "type": "configurationFrame"
            }
        }], // expected
        region_payload
            .component_view_properties(ctx)
            .await
            .drop_validation()
            .to_value() // actual
    );
    assert_eq!(
        serde_json::json![{
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
            "si": {
                "name": "server",
                "type": "component"
            }
        }], // expected
        ec2_payload
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .drop_validation()
            .to_value() // actual
    );
}

#[test]
async fn aws_region_field_validation(ctx: &DalContext) {
    let mut harness = SchemaBuiltinsTestHarness::new();
    let region_payload = harness
        .create_component(ctx, "region", Builtin::AwsRegion)
        .await;

    let _updated_region_attribute_value_id = region_payload
        .update_attribute_value_for_prop_name(
            ctx,
            "/root/domain/region",
            Some(serde_json::json!["us-poop-1"]),
        )
        .await;

    let validations = Component::list_validations(ctx, region_payload.component_id)
        .await
        .expect("able to fetch validations");

    assert_eq!(1, validations.len());

    let validation_error = "'us-poop-1' is not a valid AWS region";
    let validation = &validations[0];
    let validation_map_key = format!("{};si:validationIsValidRegion", validation.prop_id);

    assert_eq!(Some(validation_error.to_string()), validation.message);

    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "us-poop-1",
                "type": "configurationFrame"
            },

            "domain": {
                "region": "us-poop-1",
            },

            "validation": {
                &validation_map_key: {
                    "valid": false,
                    "message": validation_error,
                },
            },

        }], // actual
        region_payload.component_view_properties_raw(ctx).await // expected
    );

    let _updated_region_attribute_value_id = region_payload
        .update_attribute_value_for_prop_name(
            ctx,
            "/root/domain/region",
            Some(serde_json::json!["us-east-1"]),
        )
        .await;

    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "us-east-1",
                "type": "configurationFrame",
            },

            "domain": {
                "region": "us-east-1"
            },

            "validation": {
                &validation_map_key: {
                    "valid": true,
                },
            },
        }], // actual
        region_payload.component_view_properties_raw(ctx).await // expected
    );

    // assert valid = true
}
