use dal::{DalContext, Edge, ExternalProvider, InternalProvider, StandardModel};
use dal_test::helpers::component_bag::ComponentBagger;
use dal_test::test;
use pretty_assertions_sorted::assert_eq;

#[test]
async fn kubernetes_namespace_to_kubernetes_deployment_inter_component_update(ctx: &DalContext) {
    let mut bagger = ComponentBagger::new();
    let tail_namespace_bag = bagger
        .create_component(ctx, "tail", "Kubernetes Namespace")
        .await;
    let head_deployment_bag = bagger
        .create_component(ctx, "head", "Kubernetes Deployment")
        .await;
    let namespace_name_prop = tail_namespace_bag
        .find_prop(ctx, &["root", "si", "name"])
        .await;

    // Initialize the tail name field.
    tail_namespace_bag
        .update_attribute_value_for_prop(
            ctx,
            *namespace_name_prop.id(),
            Some(serde_json::json!["tail"]),
        )
        .await;

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    // Ensure setup worked.
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "tail",
                "color": "#30BA78",
                "type": "component",
                "protected": false,
            },
            "code": {
                "si:generateYAML": {
                    "code": "metadata:\n  name: tail\n",
                    "format": "yaml",
                },
            },
            "domain": {
                "metadata": {
                    "name": "tail",
                },
            },
        }], // expected
        tail_namespace_bag.component_view_properties_raw(ctx).await // actual
    );
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "head",
                "color": "#30BA78",
                "type": "component",
                "protected": false,
            },
            "domain": {
                "kind": "Deployment",
                "apiVersion": "apps/v1",
            },
            "code": {
                "si:generateYAML": {
                    "code": "kind: Deployment\napiVersion: apps/v1\n",
                    "format": "yaml",
                },
            },
        }], // expected
        head_deployment_bag
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value()
            .expect("could not convert to value") // actual
    );

    // Find the providers we need for connection.
    let tail_external_provider = ExternalProvider::find_for_schema_variant_and_name(
        ctx,
        tail_namespace_bag.schema_variant_id,
        "Kubernetes Namespace",
    )
    .await
    .expect("cannot find external provider")
    .expect("external provider not found");
    let head_explicit_internal_provider =
        InternalProvider::find_explicit_for_schema_variant_and_name(
            ctx,
            head_deployment_bag.schema_variant_id,
            "Kubernetes Namespace",
        )
        .await
        .expect("cannot find explicit internal provider")
        .expect("explicit internal provider not found");

    // Finally, create the inter component connection.
    Edge::connect_providers_for_components(
        ctx,
        *head_explicit_internal_provider.id(),
        head_deployment_bag.component_id,
        *tail_external_provider.id(),
        tail_namespace_bag.component_id,
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
                "color": "#30BA78",
                "type": "component",
                "protected": false,
            },
            "domain": {
                "metadata": {
                    "name": "tail"
                }
            },
            "code": {
                "si:generateYAML": {
                    "code": "metadata:\n  name: tail\n",
                    "format": "yaml",
                },
            },
        }], // expected
        tail_namespace_bag.component_view_properties_raw(ctx).await // actual
    );
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "head",
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
        head_deployment_bag
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value()
            .expect("could not convert to value") // actual
    );

    // Perform update!
    tail_namespace_bag
        .update_attribute_value_for_prop(
            ctx,
            *namespace_name_prop.id(),
            Some(serde_json::json!["look-at-me-mom-i-updated"]),
        )
        .await;

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    // Observed that it worked.
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "look-at-me-mom-i-updated",
                "color": "#30BA78",
                "type": "component",
                "protected": false,
            },
            "domain": {
                "metadata": {
                    "name": "look-at-me-mom-i-updated"
                }
            },
            "code": {
                "si:generateYAML": {
                    "code": "metadata:\n  name: look-at-me-mom-i-updated\n",
                    "format": "yaml",
                },
            },
        }], // expected
        tail_namespace_bag.component_view_properties_raw(ctx).await // actual
    );

    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "head",
                "color": "#30BA78",
                "type": "component",
                "protected": false,
            },
            "domain": {
                "apiVersion": "apps/v1",
                "kind": "Deployment",
                "metadata": {
                    "namespace": "look-at-me-mom-i-updated"
                },
                "spec": {
                    "template": {
                        "metadata": {
                            "namespace": "look-at-me-mom-i-updated"
                        }
                    }
                },
            },
            "code": {
                "si:generateYAML": {
                    "code": "kind: Deployment\nspec:\n  template:\n    metadata:\n      namespace: look-at-me-mom-i-updated\nmetadata:\n  namespace: look-at-me-mom-i-updated\napiVersion: apps/v1\n",
                    "format": "yaml",
                },
            },
        }], // expected
        head_deployment_bag
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value()
            .expect("could not convert to value") // actual
    );
}
