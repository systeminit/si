use dal::{ChangeSet, DalContext, Edge, ExternalProvider, InternalProvider, StandardModel};
use dal_test::{
    helpers::builtins::{Builtin, SchemaBuiltinsTestHarness},
    test,
};
use pretty_assertions_sorted::assert_eq;

#[test]
async fn docker_image_to_kubernetes_deployment_inter_component_update(ctx: &mut DalContext) {
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

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    // Ensure setup worked.
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "tail",
                "color": "#4695E7",
                "type": "component",
                "protected": false,
            },
            "domain": {
                "image": "tail",
            },
        }], // expected
        tail_docker_image_payload
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value()
            .expect("could not convert to value") // actual
    );
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "deployment",
                "color": "#30BA78",
                "type": "component",
                "protected": false,
            },
            "domain": {
                "apiVersion": "apps/v1",
                "kind": "Deployment",
            },
            "code": {
                "si:generateYAML": {
                    "code": "kind: Deployment\napiVersion: apps/v1\n",
                    "format": "yaml",
                },
            },
        }], // expected
        head_deployment_payload
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value()
            .expect("could not convert to value") // actual
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

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    // Ensure the view did not drift.
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "tail",
                "color": "#4695E7",
                "type": "component",
                "protected": false,
            },
            "domain": {
                "image": "tail",
            },
        }], // expected
        tail_docker_image_payload
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value()
            .expect("could not convert to value") // actual
    );
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "deployment",
                "color": "#30BA78",
                "type": "component",
                "protected": false,
            },
            "domain": {
                "apiVersion": "apps/v1",
                "kind": "Deployment",
            },
            "code": {
                "si:generateYAML": {
                    "code": "kind: Deployment\napiVersion: apps/v1\n",
                    "format": "yaml",
                },
            },
        }], // expected
        head_deployment_payload
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value()
            .expect("could not convert to value") // actual
    );

    // Perform update!
    tail_docker_image_payload
        .update_attribute_value_for_prop_name(
            ctx,
            "/root/si/name",
            Some(serde_json::json!["ironsides"]),
        )
        .await;

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    // Observe that it worked.
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "ironsides",
                "color": "#4695E7",
                "type": "component",
                "protected": false,
            },
            "domain": {
                "image": "ironsides"
            },
        }], // expected
        tail_docker_image_payload
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value()
            .expect("could not convert to value") // actual
    );

    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "deployment",
                "color": "#30BA78",
                "type": "component",
                "protected": false,
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
            "code": {
                "si:generateYAML": {
                    "code": "kind: Deployment\nspec:\n  template:\n    spec:\n      containers:\n        - name: ironsides\n          image: ironsides\n          ports: []\napiVersion: apps/v1\n",
                    "format": "yaml",
                },
            },
        }], // expected
        head_deployment_payload
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value()
            .expect("could not convert to value") // actual
    );

    let mut cs = ChangeSet::get_by_pk(ctx, &ctx.visibility().change_set_pk)
        .await
        .expect("unable to find changeset")
        .expect("no changeset found");
    cs.apply(ctx).await.expect("unable to apply changeset");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");
}
