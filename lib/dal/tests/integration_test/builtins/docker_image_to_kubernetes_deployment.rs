use dal::{ChangeSet, DalContext, Edge, ExternalProvider, InternalProvider, StandardModel};
use dal_test::{
    helpers::builtins::{Builtin, SchemaBuiltinsTestHarness},
    test,
};
use pretty_assertions_sorted::assert_eq;

#[test]
async fn docker_image_to_kubernetes_deployment_inter_component_update(ctx: &DalContext) {
    let mut harness = SchemaBuiltinsTestHarness::new();
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
    assert_eq!(
        serde_json::json![{
            "domain": {
                "image": "tail"
            },

            "si": {
                "name": "tail",
                "type": "component"
            }
        }], // expected
        tail_docker_image_payload
            .component_view_properties(ctx)
            .await // actual
    );
    assert_eq!(
        serde_json::json![{
            "code": {
                "si:generateYAML": {
                    "code": "kind: Deployment\napiVersion: apps/v1\n",
                    "format": "yaml",
                },
            },
            "domain": {
                "apiVersion": "apps/v1",
                "kind": "Deployment",
            },
            "si": {
                "name": "deployment",
                "type": "component"
            }
        }], // expected
        head_deployment_payload.component_view_properties(ctx).await // actual
    );

    // Find the providers we need for connection.
    let tail_external_provider = ExternalProvider::find_for_schema_variant_and_name(
        ctx,
        tail_docker_image_payload.schema_variant_id,
        "Container Image",
    )
    .await
    .expect("cannot find external provider")
    .expect("external provider not found");
    let head_explicit_internal_provider =
        InternalProvider::find_explicit_for_schema_variant_and_name(
            ctx,
            head_deployment_payload.schema_variant_id,
            "Container Image",
        )
        .await
        .expect("cannot find explicit internal provider")
        .expect("explicit internal provider not found");

    // Finally, create the inter component connection.
    Edge::connect_providers_for_components(
        ctx,
        *head_explicit_internal_provider.id(),
        head_deployment_payload.component_id,
        *tail_external_provider.id(),
        tail_docker_image_payload.component_id,
    )
    .await
    .expect("could not connect providers");

    // Ensure the view did not drift.
    assert_eq!(
        serde_json::json![{

            "domain": {
                "image": "tail"
            },
            "si": {
                "name": "tail",
                "type": "component"
            }
        }], // expected
        tail_docker_image_payload
            .component_view_properties(ctx)
            .await // actual
    );
    assert_eq!(
        serde_json::json![{
            "code": {
                "si:generateYAML": {
                    "code": "kind: Deployment\napiVersion: apps/v1\n",
                    "format": "yaml",
                },
            },
            "domain": {
                "apiVersion": "apps/v1",
                "kind": "Deployment",
            },
            "si": {
                "name": "deployment",
                "type": "component"
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
    assert_eq!(
        serde_json::json![{

            "domain": {
                "image": "ironsides"
            },
            "si": {
                "name": "ironsides",
                "type": "component"
            }
        }], // expected
        tail_docker_image_payload
            .component_view_properties(ctx)
            .await // actual
    );

    assert_eq!(
        serde_json::json![{
            "code": {
                "si:generateYAML": {
                    "code": "kind: Deployment\nspec:\n  template:\n    spec:\n      containers:\n        - name: ironsides\n          image: ironsides\n          ports: []\napiVersion: apps/v1\n",
                    "format": "yaml",
                },
            },
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
                "name": "deployment",
                "type": "component"
            },
        }], // expected
        head_deployment_payload.component_view_properties(ctx).await // actual
    );

    let mut cs = ChangeSet::get_by_pk(ctx, &ctx.visibility().change_set_pk)
        .await
        .expect("unable to find changeset")
        .expect("no changeset found");
    cs.apply(ctx).await.expect("unable to apply changeset");
}
