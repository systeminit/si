use pretty_assertions_sorted::assert_eq;

use dal::test::helpers::builtins::{Builtin, SchemaBuiltinsTestHarness};
use dal::{DalContext, Edge, ExternalProvider, InternalProvider, StandardModel};

use crate::dal::test;

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
                "name": "region"
            }
        }], // expected
        region_payload.component_view_properties(ctx).await // actual
    );
    assert_eq!(
        serde_json::json![{
            "domain": {},
            "si": {
                "name": "server"
            }
        }], // expected
        ec2_payload.component_view_properties(ctx).await // actual
    );

    // Find the providers we need for connection.
    let region_external_provider = ExternalProvider::find_for_schema_variant_and_name(
        ctx,
        region_payload.schema_variant_id,
        "region",
    )
    .await
    .expect("cannot find external provider")
    .expect("external provider not found");
    let ec2_explicit_internal_provider =
        InternalProvider::find_explicit_for_schema_variant_and_name(
            ctx,
            ec2_payload.schema_variant_id,
            "region",
        )
        .await
        .expect("cannot find explicit internal provider")
        .expect("explicit internal provider not found");

    // Finally, create the inter component connection.
    Edge::connect_providers_for_components(
        ctx,
        "identity",
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
                "name": "region"
            }
        }], // expected
        region_payload.component_view_properties(ctx).await // actual
    );
    assert_eq!(
        serde_json::json![{
            "domain": {},
            "si": {
                "name": "server"
            }
        }], // expected
        ec2_payload.component_view_properties(ctx).await // actual
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
                "name": "region"
            }
        }], // expected
        region_payload.component_view_properties(ctx).await // actual
    );
    assert_eq!(
        serde_json::json![{
            "domain": {
                "region": "us-west-2"
            },
            "si": {
                "name": "server"
            }
        }], // expected
        ec2_payload.component_view_properties(ctx).await // actual
    );
}
