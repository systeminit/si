use dal::{Connection, DalContext, ExternalProvider, InternalProvider, StandardModel};

use pretty_assertions_sorted::assert_eq_sorted;

use crate::dal::test;
use crate::integration_test::provider::builtins::ProviderBuiltinsHarness;

// Oh yeah, it's big brain time.
#[test]
async fn kubernetes_deployment_intelligence(ctx: &DalContext<'_, '_>) {
    let mut harness = ProviderBuiltinsHarness::new();
    let tail_fedora_payload = harness.create_docker_image(ctx, "fedora").await;
    let tail_alpine_payload = harness.create_docker_image(ctx, "alpine").await;
    let tail_namespace_payload = harness.create_kubernetes_namespace(ctx, "namespace").await;
    let head_deployment_spongebob_payload =
        harness.create_kubernetes_deployment(ctx, "spongebob").await;
    let head_deployment_squidward_payload =
        harness.create_kubernetes_deployment(ctx, "squidward").await;

    // Initialize the tail component primary fields
    tail_fedora_payload
        .update_attribute_value_for_prop_name(
            ctx,
            "/root/si/name",
            Some(serde_json::json!["fedora"]),
        )
        .await;
    tail_alpine_payload
        .update_attribute_value_for_prop_name(
            ctx,
            "/root/si/name",
            Some(serde_json::json!["alpine"]),
        )
        .await;
    tail_namespace_payload
        .update_attribute_value_for_prop_name(
            ctx,
            "/root/domain/metadata/name",
            Some(serde_json::json!["rancher-system"]),
        )
        .await;
    ctx.run_enqueued_jobs()
        .await
        .expect("cannot run enqueued jobs");

    // Ensure setup worked.
    assert_eq_sorted!(
        serde_json::json![{
            "domain": {
                "image": "fedora"
            },
            "si": {
                "name": "fedora"
            }
        }], // expected
        tail_fedora_payload.component_view_properties(ctx).await // actual
    );
    assert_eq_sorted!(
        serde_json::json![{
            "domain": {
                "image": "alpine"
            },
            "si": {
                "name": "alpine"
            }
        }], // expected
        tail_alpine_payload.component_view_properties(ctx).await // actual
    );
    assert_eq_sorted!(
        serde_json::json![{
            "domain": {
                "metadata": {
                    "name": "rancher-system"
                }
            },
            "si": {
                "name": "namespace"
            }
        }], // expected
        tail_namespace_payload.component_view_properties(ctx).await // actual
    );
    assert_eq_sorted!(
        serde_json::json![{
            "domain": {
                "apiVersion": "apps/v1",
                "kind": "Deployment",
            },
            "si": {
                "name": "spongebob"
            }
        }], // expected
        head_deployment_spongebob_payload
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
                "name": "squidward"
            }
        }], // expected
        head_deployment_squidward_payload
            .component_view_properties(ctx)
            .await // actual
    );

    // Collect the explicit internal providers we want from the deployments.
    let head_deployment_spongebob_docker_image_provider =
        InternalProvider::find_explicit_for_schema_variant_and_name(
            ctx,
            head_deployment_spongebob_payload.schema_variant_id,
            "docker_image",
        )
        .await
        .expect("cannot find explicit internal provider")
        .expect("explicit internal provider not found");
    let head_deployment_kubernetes_namespace_provider =
        InternalProvider::find_explicit_for_schema_variant_and_name(
            ctx,
            head_deployment_spongebob_payload.schema_variant_id,
            "kubernetes_namespace",
        )
        .await
        .expect("cannot find explicit internal provider")
        .expect("explicit internal provider not found");
    let head_deployment_squidward_docker_image_provider =
        InternalProvider::find_explicit_for_schema_variant_and_name(
            ctx,
            head_deployment_squidward_payload.schema_variant_id,
            "docker_image",
        )
        .await
        .expect("cannot find explicit internal provider")
        .expect("explicit internal provider not found");

    // Connect fedora to the deployment.
    let tail_fedora_provider = ExternalProvider::find_for_schema_variant_and_name(
        ctx,
        tail_fedora_payload.schema_variant_id,
        "docker_image",
    )
    .await
    .expect("cannot find external provider")
    .expect("external provider not found");
    Connection::connect_providers(
        ctx,
        "identity".to_string(),
        *tail_fedora_provider.id(),
        tail_fedora_payload.component_id,
        *head_deployment_spongebob_docker_image_provider.id(),
        head_deployment_spongebob_payload.component_id,
    )
    .await
    .expect("could not connect providers");

    // Perform one update.
    tail_fedora_payload
        .update_attribute_value_for_prop_name(
            ctx,
            "/root/si/name",
            Some(serde_json::json!["fedora-updated"]),
        )
        .await;
    ctx.run_enqueued_jobs()
        .await
        .expect("cannot run enqueued jobs");

    // Check that the update worked.
    assert_eq_sorted!(
        serde_json::json![{
            "domain": {
                "image": "fedora-updated"
            },
            "si": {
                "name": "fedora-updated"
            }
        }], // expected
        tail_fedora_payload.component_view_properties(ctx).await // actual
    );
    assert_eq_sorted!(
        serde_json::json![{
            "domain": {
                "image": "alpine"
            },
            "si": {
                "name": "alpine"
            }
        }], // expected
        tail_alpine_payload.component_view_properties(ctx).await // actual
    );
    assert_eq_sorted!(
        serde_json::json![{
            "domain": {
                "metadata": {
                    "name": "rancher-system"
                }
            },
            "si": {
                "name": "namespace"
            }
        }], // expected
        tail_namespace_payload.component_view_properties(ctx).await // actual
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
        head_deployment_spongebob_payload
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
                "name": "squidward"
            }
        }], // expected
        head_deployment_squidward_payload
            .component_view_properties(ctx)
            .await // actual
    );

    // After the first update, let's connect alpine to the spongebob deployment.
    let tail_alpine_provider = ExternalProvider::find_for_schema_variant_and_name(
        ctx,
        tail_alpine_payload.schema_variant_id,
        "docker_image",
    )
    .await
    .expect("cannot find external provider")
    .expect("external provider not found");
    Connection::connect_providers(
        ctx,
        "identity".to_string(),
        *tail_alpine_provider.id(),
        tail_alpine_payload.component_id,
        *head_deployment_spongebob_docker_image_provider.id(),
        head_deployment_spongebob_payload.component_id,
    )
    .await
    .expect("could not connect providers");

    // Then, connect namespace to the deployment.
    let tail_namespace_provider = ExternalProvider::find_for_schema_variant_and_name(
        ctx,
        tail_namespace_payload.schema_variant_id,
        "kubernetes_namespace",
    )
    .await
    .expect("cannot find external provider")
    .expect("external provider not found");
    Connection::connect_providers(
        ctx,
        "identity".to_string(),
        *tail_namespace_provider.id(),
        tail_namespace_payload.component_id,
        *head_deployment_kubernetes_namespace_provider.id(),
        head_deployment_spongebob_payload.component_id,
    )
    .await
    .expect("could not connect providers");

    // Finally, connect fedora to the squidward deployment.
    Connection::connect_providers(
        ctx,
        "identity".to_string(),
        *tail_fedora_provider.id(),
        tail_fedora_payload.component_id,
        *head_deployment_squidward_docker_image_provider.id(),
        head_deployment_squidward_payload.component_id,
    )
    .await
    .expect("could not connect providers");

    // Perform three updates and assert afterwards.
    tail_alpine_payload
        .update_attribute_value_for_prop_name(
            ctx,
            "/root/si/name",
            Some(serde_json::json!["alpine-updated"]),
        )
        .await;
    tail_namespace_payload
        .update_attribute_value_for_prop_name(
            ctx,
            "/root/domain/metadata/name",
            Some(serde_json::json!["rancher-system-updated"]),
        )
        .await;
    tail_fedora_payload
        .update_attribute_value_for_prop_name(
            ctx,
            "/root/si/name",
            Some(serde_json::json!["fedora-updated-twice"]),
        )
        .await;
    ctx.run_enqueued_jobs()
        .await
        .expect("cannot run enqueued jobs");

    // Observed that it worked.
    assert_eq_sorted!(
        serde_json::json![{
            "domain": {
                "image": "fedora-updated-twice"
            },
            "si": {
                "name": "fedora-updated-twice"
            }
        }], // expected
        tail_fedora_payload.component_view_properties(ctx).await // actual
    );
    assert_eq_sorted!(
        serde_json::json![{
            "domain": {
                "image": "alpine-updated"
            },
            "si": {
                "name": "alpine-updated"
            }
        }], // expected
        tail_alpine_payload.component_view_properties(ctx).await // actual
    );
    assert_eq_sorted!(
        serde_json::json![{
            "domain": {
                "metadata": {
                    "name": "rancher-system-updated"
                }
            },
            "si": {
                "name": "namespace"
            }
        }], // expected
        tail_namespace_payload.component_view_properties(ctx).await // actual
    );
    // We cannot use "assert_eq_sorted" here because the containers array order should be stable,
    // but we should not assert a guaranteed order.
    assert_eq!(
        serde_json::json![{
            "domain": {
                "apiVersion": "apps/v1",
                "kind": "Deployment",
                "metadata": {
                    "namespace": "rancher-system-updated"
                },
                "spec": {
                    "template": {
                        "metadata": {
                            "namespace": "rancher-system-updated"
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
        head_deployment_spongebob_payload
            .component_view_properties(ctx)
            .await // actual
    );
    assert_eq!(
        serde_json::json![{
            "domain": {
                "apiVersion": "apps/v1",
                "kind": "Deployment",
                "metadata": {
                    "namespace": "rancher-system-updated"
                },
                "spec": {
                    "template": {
                        "metadata": {
                            "namespace": "rancher-system-updated"
                        },
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
                "name": "squidward"
            }
        }], // expected
        head_deployment_squidward_payload
            .component_view_properties(ctx)
            .await // actual
    );
}
