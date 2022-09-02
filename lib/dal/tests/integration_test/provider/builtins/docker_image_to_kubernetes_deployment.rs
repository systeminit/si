use dal::test::helpers::builtins::{Builtin, BuiltinsHarness};
use dal::{DalContext, Edge, ExternalProvider, InternalProvider, StandardModel};
use pretty_assertions_sorted::assert_eq_sorted;

use crate::dal::test;

#[test]
async fn docker_image_to_kubernetes_deployment_inter_component_update(
    ctx: &DalContext<'_, '_, '_>,
) {
    let mut harness = BuiltinsHarness::new();
    let tail_docker_image_payload = harness
        .create_component(ctx, "image", Builtin::DockerImage)
        .await;
    let head_deployment_payload = harness
        .create_component(ctx, "deployment", Builtin::KubernetesDeployment)
        .await;

    // Initialize the tail "/root/si/name" field.
    tail_docker_image_payload
        .update_attribute_value_for_prop_name(ctx, "/root/si/name", Some(serde_json::json!["tail"]))
        .await;

    // Ensure setup worked.
    assert_eq_sorted!(
        serde_json::json![{
            "domain": {
                "image": "tail"
            },
            "si": {
                "name": "tail"
            }
        }], // expected
        tail_docker_image_payload
            .component_view_properties(ctx)
            .await // actual
    );
    assert_eq_sorted!(
        serde_json::json![{
            "domain": {
                "apiVersion": "apps/v1",
                "kind": "Deployment",
            },
            "si": {
                "name": "deployment"
            }
        }], // expected
        head_deployment_payload.component_view_properties(ctx).await // actual
    );

    // Find the providers we need for connection.
    let tail_external_provider = ExternalProvider::find_for_schema_variant_and_name(
        ctx,
        tail_docker_image_payload.schema_variant_id,
        "docker_image",
    )
    .await
    .expect("cannot find external provider")
    .expect("external provider not found");
    let head_explicit_internal_provider =
        InternalProvider::find_explicit_for_schema_variant_and_name(
            ctx,
            head_deployment_payload.schema_variant_id,
            "docker_image",
        )
        .await
        .expect("cannot find explicit internal provider")
        .expect("explicit internal provider not found");

    // Finally, create the inter component connection.
    Edge::connect_providers_for_components(
        ctx,
        "identity",
        *head_explicit_internal_provider.id(),
        head_deployment_payload.component_id,
        *tail_external_provider.id(),
        tail_docker_image_payload.component_id,
    )
    .await
    .expect("could not connect providers");

    // Ensure the view did not drift.
    assert_eq_sorted!(
        serde_json::json![{
            "domain": {
                "image": "tail"
            },
            "si": {
                "name": "tail"
            }
        }], // expected
        tail_docker_image_payload
            .component_view_properties(ctx)
            .await // actual
    );
    assert_eq_sorted!(
        serde_json::json![{
            "domain": {
                "apiVersion": "apps/v1",
                "kind": "Deployment",
            },
            "si": {
                "name": "deployment"
            }
        }], // expected
        head_deployment_payload.component_view_properties(ctx).await // actual
    );

    // Perform update!
    tail_docker_image_payload
        .update_attribute_value_for_prop_name(
            ctx,
            "/root/si/name",
            Some(serde_json::json!["ironsides"]),
        )
        .await;

    // Observe that it worked.
    assert_eq_sorted!(
        serde_json::json![{
            "domain": {
                "image": "ironsides"
            },
            "si": {
                "name": "ironsides"
            }
        }], // expected
        tail_docker_image_payload
            .component_view_properties(ctx)
            .await // actual
    );

    assert_eq_sorted!(
        serde_json::json![{
            "domain": {
                "apiVersion": "apps/v1",
                "kind": "Deployment",
                "spec": {
                    "template": {
                        "spec": {
                            "containers": [
                                {
                                    "image": "ironsides",
                                    "name": "ironsides",
                                    "ports": [],
                                },
                            ],
                        },
                    },
                },
            },
            "si": {
                "name": "deployment"
            },
        }], // expected
        head_deployment_payload.component_view_properties(ctx).await // actual
    );
}
