use dal::{DalContext, Edge, ExternalProvider, InternalProvider, StandardModel};
use dal_test::{
    helpers::builtins::{Builtin, SchemaBuiltinsTestHarness},
    test,
};

use pretty_assertions_sorted::assert_eq;

/// This test simulates a usability study test from June 2021 that showcased the
/// [`providers`](dal::provider) and [`attribute`](dal::attribute) work with extended
/// "intelligence" functionality across multiple [`Components`](dal::Component).
#[test]
async fn kubernetes_deployment_intelligence(octx: DalContext) {
    let ctx = &octx;
    let mut harness = SchemaBuiltinsTestHarness::new();
    let fedora_docker_image_payload = harness
        .create_component(ctx, "fedora", Builtin::DockerImage)
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
    let alpine_docker_image_payload = harness
        .create_component(ctx, "alpine", Builtin::DockerImage)
        .await;

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

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

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    // Ensure setup worked.
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "fedora",
                "color": "#4695E7",
                "type": "component",
                "protected": false,
            },
            "domain": {
                "image": "fedora",
            },
        }], // expected
        fedora_docker_image_payload
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value()
            .expect("could not convert to value") // actual
    );
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "alpine",
                "color": "#4695E7",
                "type": "component",
                "protected": false,
            },
            "domain": {
                "image": "alpine",
            },
        }], // expected
        alpine_docker_image_payload
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value()
            .expect("could not convert to value") // actual
    );
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "namespace",
                "color": "#30BA78",
                "type": "component",
                "protected": false,
            },
            "code": {
                "si:generateYAML": {
                    "code": "metadata:\n  name: squidward-system\n",
                    "format": "yaml",
                },
            },
            "domain": {
                "metadata": {
                    "name": "squidward-system",
                },
            },
        }], // expected
        namespace_payload.component_view_properties_raw(ctx).await // actual
    );
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "spongebob",
                "color": "#30BA78",
                "type": "component",
                "protected": false,
            },
            "code": {
                "si:generateYAML": {
                    "code": "kind: Deployment\napiVersion: apps/v1\n",
                    "format": "yaml",
                },
            },
            "domain": {
                "kind": "Deployment",
                "apiVersion": "apps/v1",
            },
        }], // expected
        spongebob_deployment_payload
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value()
            .expect("could not convert to value") // actual
    );
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "patrick",
                "color": "#30BA78",
                "type": "component",
                "protected": false,
            },
            "code": {
                "si:generateYAML": {
                    "code": "kind: Deployment\napiVersion: apps/v1\n",
                    "format": "yaml",
                },
            },
            "domain": {
                "kind": "Deployment",
                "apiVersion": "apps/v1",
            },
        }], // expected
        patrick_deployment_payload
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value()
            .expect("could not convert to value") // actual
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

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    // Check that the update worked.
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "fedora-updated",
                "color": "#4695E7",
                "type": "component",
                "protected": false,
            },
            "domain": {
                "image": "fedora-updated",
            },
        }], // expected
        fedora_docker_image_payload
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value()
            .expect("could not convert to value") // actual
    );
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "alpine",
                "color": "#4695E7",
                "type": "component",
                "protected": false,
            },
            "domain": {
                "image": "alpine",
            },
        }], // expected
        alpine_docker_image_payload
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value()
            .expect("could not convert to value") // actual
    );
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "namespace",
                "color": "#30BA78",
                "type": "component",
                "protected": false,
            },
            "code": {
                "si:generateYAML": {
                    "code": "metadata:\n  name: squidward-system\n",
                    "format": "yaml",
                },
            },
            "domain": {
                "metadata": {
                    "name": "squidward-system",
                },
            },
        }], // expected
        namespace_payload.component_view_properties_raw(ctx).await // actual
    );
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "spongebob",
                "color": "#30BA78",
                "type": "component",
                "protected": false,
            },
            "code": {
                "si:generateYAML": {
                    "code": "kind: Deployment\nspec:\n  template:\n    spec:\n      containers:\n        - name: fedora-updated\n          image: fedora-updated\n          ports: []\napiVersion: apps/v1\n",
                    "format": "yaml",
                },
            },
            "domain": {
                "kind": "Deployment",
                "spec": {
                    "template": {
                        "spec": {
                            "containers": [
                                {
                                    "name": "fedora-updated",
                                    "image": "fedora-updated",
                                    "ports": [],
                                },
                            ],
                        },
                    }
                },
                "apiVersion": "apps/v1",
            },
        }], // expected
        spongebob_deployment_payload
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value()
            .expect("could not convert to value") // actual
    );
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "patrick",
                "color": "#30BA78",
                "type": "component",
                "protected": false,
            },
            "code": {
                "si:generateYAML": {
                    "code": "kind: Deployment\napiVersion: apps/v1\n",
                    "format": "yaml",
                },
            },
            "domain": {
                "kind": "Deployment",
                "apiVersion": "apps/v1",
            },
        }], // expected
        patrick_deployment_payload
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value()
            .expect("could not convert to value") // actual
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

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    // Observed that it worked.
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "fedora-updated-twice",
                "color": "#4695E7",
                "type": "component",
                "protected": false,
            },

            "domain": {
                "image": "fedora-updated-twice",
            },
        }], // expected
        fedora_docker_image_payload
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value()
            .expect("could not convert to value") // actual
    );
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "alpine-updated",
                "color": "#4695E7",
                "type": "component",
                "protected": false,
            },
            "domain": {
                "image": "alpine-updated"
            },
        }], // expected
        alpine_docker_image_payload
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value()
            .expect("could not convert to value") // actual
    );
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "namespace",
                "color": "#30BA78",
                "type": "component",
                "protected": false,
            },
            "code": {
                "si:generateYAML": {
                    "code": "metadata:\n  name: squidward-system-updated\n",
                    "format": "yaml",
                },
            },
            "domain": {
                "metadata": {
                    "name": "squidward-system-updated",
                },
            },
        }], // expected
        namespace_payload.component_view_properties_raw(ctx).await // actual
    );
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "spongebob",
                "color": "#30BA78",
                "type": "component",
                "protected": false,
            },
            "code": {
                "si:generateYAML": {
                    "code": "kind: Deployment\nspec:\n  template:\n    spec:\n      containers:\n        - name: fedora-updated-twice\n          image: fedora-updated-twice\n          ports: []\n        - name: alpine-updated\n          image: alpine-updated\n          ports: []\n    metadata:\n      namespace: squidward-system-updated\nmetadata:\n  namespace: squidward-system-updated\napiVersion: apps/v1\n",
                    "format": "yaml",
                },
            },
            "domain": {
                "kind": "Deployment",
                "spec": {
                    "template": {
                        "spec": {
                            "containers": [
                                {
                                    "name": "fedora-updated-twice",
                                    "image": "fedora-updated-twice",
                                    "ports": [],
                                },
                                {
                                    "name": "alpine-updated",
                                    "image": "alpine-updated",
                                    "ports": [],
                                },
                            ],
                        },
                        "metadata": {
                            "namespace": "squidward-system-updated",
                        },
                    }
                },
                "metadata": {
                    "namespace": "squidward-system-updated",
                },
                "apiVersion": "apps/v1",
            },
        }], // expected
        spongebob_deployment_payload
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value()
            .expect("could not convert to value") // actual
    );
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "patrick",
                "color": "#30BA78",
                "type": "component",
                "protected": false,
            },
            "code": {
                "si:generateYAML": {
                    "code": "kind: Deployment\nspec:\n  template:\n    spec:\n      containers:\n        - name: fedora-updated-twice\n          image: fedora-updated-twice\n          ports: []\napiVersion: apps/v1\n",
                    "format": "yaml",
                },
            },
            "domain": {
                "kind": "Deployment",
                "spec": {
                    "template": {
                        "spec": {
                            "containers": [
                                {
                                    "name": "fedora-updated-twice",
                                    "image": "fedora-updated-twice",
                                    "ports": [],
                                },
                            ],
                        },
                    }
                },
                "apiVersion": "apps/v1",
            },
        }], // expected
        patrick_deployment_payload
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value()
            .expect("could not convert to value") // actual
    );
}
