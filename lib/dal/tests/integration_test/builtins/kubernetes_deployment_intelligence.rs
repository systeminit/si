use dal::{DalContext, Edge, ExternalProvider, InternalProvider, StandardModel};
use dal_test::{
    helpers::builtins::{Builtin, SchemaBuiltinsTestHarness},
    test,
};

use pretty_assertions_sorted::assert_eq;

// Oh yeah, it's big brain time.
#[ignore]
#[test]
async fn kubernetes_deployment_intelligence(octx: DalContext) {
    let ctx = &octx;
    let mut harness = SchemaBuiltinsTestHarness::new();
    let fedora_docker_image_payload = harness
        .create_component(ctx, "fedora", Builtin::DockerImage)
        .await;
    let alpine_docker_image_payload = harness
        .create_component(ctx, "alpine", Builtin::DockerImage)
        .await;
    let namespace_payload = harness
        .create_component(ctx, "namespace", Builtin::KubernetesNamespace)
        .await;
    let spongebob_deployment_payload = harness
        .create_component(ctx, "spongebob", Builtin::KubernetesDeployment)
        .await;
    let patrick_deployment_payload = harness
        .create_component(ctx, "patrick", Builtin::KubernetesDeployment)
        .await;

    // Cache schema variants to increase clarity in the test structure.
    assert_eq!(
        spongebob_deployment_payload.schema_variant_id,
        patrick_deployment_payload.schema_variant_id
    );
    let kubernetes_namespace_schema_variant_id = spongebob_deployment_payload.schema_variant_id;
    assert_eq!(
        fedora_docker_image_payload.schema_variant_id,
        alpine_docker_image_payload.schema_variant_id
    );
    let docker_image_schema_variant_id = fedora_docker_image_payload.schema_variant_id;

    // First, collect all the external providers that we need
    // (correspond to "output sockets" on the configuration diagram).
    let docker_image_external_provider = ExternalProvider::find_for_schema_variant_and_name(
        ctx,
        docker_image_schema_variant_id,
        "Container Image",
    )
    .await
    .expect("cannot find external provider")
    .expect("external provider not found");
    let namespace_external_provider = ExternalProvider::find_for_schema_variant_and_name(
        ctx,
        namespace_payload.schema_variant_id,
        "Kubernetes Namespace",
    )
    .await
    .expect("cannot find external provider")
    .expect("external provider not found");

    // First, collect all the explicit internal providers that we need
    // (correspond to "input sockets" on the configuration diagram).
    let kubernetes_deployment_explicit_internal_provider_for_container_image =
        InternalProvider::find_explicit_for_schema_variant_and_name(
            ctx,
            kubernetes_namespace_schema_variant_id,
            "Container Image",
        )
        .await
        .expect("cannot find explicit internal provider")
        .expect("explicit internal provider not found");
    let kubernetes_deployment_explicit_internal_provider_for_namespace =
        InternalProvider::find_explicit_for_schema_variant_and_name(
            ctx,
            kubernetes_namespace_schema_variant_id,
            "Kubernetes Namespace",
        )
        .await
        .expect("cannot find explicit internal provider")
        .expect("explicit internal provider not found");

    // Initialize the tail component primary fields.
    fedora_docker_image_payload
        .update_attribute_value_for_prop_name(
            ctx,
            "/root/si/name",
            Some(serde_json::json!["fedora"]),
        )
        .await;
    alpine_docker_image_payload
        .update_attribute_value_for_prop_name(
            ctx,
            "/root/si/name",
            Some(serde_json::json!["alpine"]),
        )
        .await;
    namespace_payload
        .update_attribute_value_for_prop_name(
            ctx,
            "/root/domain/metadata/name",
            Some(serde_json::json!["squidward-system"]),
        )
        .await;

    // Ensure setup worked.
    assert_eq!(
        serde_json::json![{
            "domain": {
                "image": "fedora"
            },
            "si": {
                "name": "fedora"
            }
        }], // expected
        fedora_docker_image_payload
            .component_view_properties(ctx)
            .await // actual
    );
    assert_eq!(
        serde_json::json![{
            "domain": {
                "image": "alpine"
            },
            "si": {
                "name": "alpine"
            }
        }], // expected
        alpine_docker_image_payload
            .component_view_properties(ctx)
            .await // actual
    );
    assert_eq!(
        serde_json::json![{
            "domain": {
                "metadata": {
                    "name": "squidward-system"
                }
            },
            "si": {
                "name": "namespace"
            }
        }], // expected
        namespace_payload.component_view_properties(ctx).await // actual
    );
    assert_eq!(
        serde_json::json![{
            "domain": {
                "apiVersion": "apps/v1",
                "kind": "Deployment",
            },
            "si": {
                "name": "spongebob"
            }
        }], // expected
        spongebob_deployment_payload
            .component_view_properties(ctx)
            .await // actual
    );
    assert_eq!(
        serde_json::json![{
            "domain": {
                "apiVersion": "apps/v1",
                "kind": "Deployment",
            },
            "si": {
                "name": "patrick"
            }
        }], // expected
        patrick_deployment_payload
            .component_view_properties(ctx)
            .await // actual
    );

    // Connect the fedora docker image to the spongebob deployment.
    Edge::connect_providers_for_components(
        ctx,
        *kubernetes_deployment_explicit_internal_provider_for_container_image.id(),
        spongebob_deployment_payload.component_id,
        *docker_image_external_provider.id(),
        fedora_docker_image_payload.component_id,
    )
    .await
    .expect("could not connect providers");

    // Perform one update for the fedora docker image.
    fedora_docker_image_payload
        .update_attribute_value_for_prop_name(
            ctx,
            "/root/si/name",
            Some(serde_json::json!["fedora-updated"]),
        )
        .await;

    // Check that the update worked.
    assert_eq!(
        serde_json::json![{
            "domain": {
                "image": "fedora-updated"
            },
            "si": {
                "name": "fedora-updated"
            }
        }], // expected
        fedora_docker_image_payload
            .component_view_properties(ctx)
            .await // actual
    );
    assert_eq!(
        serde_json::json![{
            "domain": {
                "image": "alpine"
            },
            "si": {
                "name": "alpine"
            }
        }], // expected
        alpine_docker_image_payload
            .component_view_properties(ctx)
            .await // actual
    );
    assert_eq!(
        serde_json::json![{
            "domain": {
                "metadata": {
                    "name": "squidward-system"
                }
            },
            "si": {
                "name": "namespace"
            }
        }], // expected
        namespace_payload.component_view_properties(ctx).await // actual
    );
    assert_eq!(
        serde_json::json![{
            "domain": {
                "apiVersion": "apps/v1",
                "kind": "Deployment",
                "spec": {
                    "template": {
                        "spec": {
                            "containers": [
                                {
                                    "image": "fedora-updated",
                                    "name": "fedora-updated",
                                    "ports": [],
                                },
                            ],
                        },
                    }
                },
            },
            "si": {
                "name": "spongebob"
            }
        }], // expected
        spongebob_deployment_payload
            .component_view_properties(ctx)
            .await // actual
    );
    assert_eq!(
        serde_json::json![{
            "domain": {
                "apiVersion": "apps/v1",
                "kind": "Deployment",
            },
            "si": {
                "name": "patrick"
            }
        }], // expected
        patrick_deployment_payload
            .component_view_properties(ctx)
            .await // actual
    );

    // After the first update, let's connect alpine to the spongebob deployment.
    Edge::connect_providers_for_components(
        ctx,
        *kubernetes_deployment_explicit_internal_provider_for_container_image.id(),
        spongebob_deployment_payload.component_id,
        *docker_image_external_provider.id(),
        alpine_docker_image_payload.component_id,
    )
    .await
    .expect("could not connect providers");

    // Then, connect namespace to the spongebob deployment.
    Edge::connect_providers_for_components(
        ctx,
        *kubernetes_deployment_explicit_internal_provider_for_namespace.id(),
        spongebob_deployment_payload.component_id,
        *namespace_external_provider.id(),
        namespace_payload.component_id,
    )
    .await
    .expect("could not connect providers");

    // Finally, connect fedora and the namespace to the patrick deployment, but do not connect
    // the namespace nor alpine to it.
    Edge::connect_providers_for_components(
        ctx,
        *kubernetes_deployment_explicit_internal_provider_for_container_image.id(),
        patrick_deployment_payload.component_id,
        *docker_image_external_provider.id(),
        fedora_docker_image_payload.component_id,
    )
    .await
    .expect("could not connect providers");

    // Perform updates before assertions.
    alpine_docker_image_payload
        .update_attribute_value_for_prop_name(
            ctx,
            "/root/si/name",
            Some(serde_json::json!["alpine-updated"]),
        )
        .await;
    namespace_payload
        .update_attribute_value_for_prop_name(
            ctx,
            "/root/domain/metadata/name",
            Some(serde_json::json!["squidward-system-updated"]),
        )
        .await;
    fedora_docker_image_payload
        .update_attribute_value_for_prop_name(
            ctx,
            "/root/si/name",
            Some(serde_json::json!["fedora-updated-twice"]),
        )
        .await;

    // Observed that it worked.
    assert_eq!(
        serde_json::json![{
            "domain": {
                "image": "fedora-updated-twice"
            },
            "si": {
                "name": "fedora-updated-twice"
            }
        }], // expected
        fedora_docker_image_payload
            .component_view_properties(ctx)
            .await // actual
    );
    assert_eq!(
        serde_json::json![{
            "domain": {
                "image": "alpine-updated"
            },
            "si": {
                "name": "alpine-updated"
            }
        }], // expected
        alpine_docker_image_payload
            .component_view_properties(ctx)
            .await // actual
    );
    assert_eq!(
        serde_json::json![{
            "domain": {
                "metadata": {
                    "name": "squidward-system-updated"
                }
            },
            "si": {
                "name": "namespace"
            }
        }], // expected
        namespace_payload.component_view_properties(ctx).await // actual
    );
    assert_eq!(
        serde_json::json![{
            "domain": {
                "apiVersion": "apps/v1",
                "kind": "Deployment",
                "metadata": {
                    "namespace": "squidward-system-updated"
                },
                "spec": {
                    "template": {
                        "metadata": {
                            "namespace": "squidward-system-updated"
                        },
                        "spec": {
                            "containers": [
                                {
                                    "image": "fedora-updated-twice",
                                    "name": "fedora-updated-twice",
                                    "ports": [],
                                },
                                {
                                    "image": "alpine-updated",
                                    "name": "alpine-updated",
                                    "ports": [],
                                },
                            ],
                        },
                    }
                },
            },
            "si": {
                "name": "spongebob"
            }
        }], // expected
        spongebob_deployment_payload
            .component_view_properties(ctx)
            .await // actual
    );
    assert_eq!(
        serde_json::json![{
            "domain": {
                "apiVersion": "apps/v1",
                "kind": "Deployment",
                "spec": {
                    "template": {
                        "spec": {
                            "containers": [
                                {
                                    "image": "fedora-updated-twice",
                                    "name": "fedora-updated-twice",
                                    "ports": [],
                                },
                            ],
                        },
                    }
                },
            },
            "si": {
                "name": "patrick"
            }
        }], // expected
        patrick_deployment_payload
            .component_view_properties(ctx)
            .await // actual
    );
}
