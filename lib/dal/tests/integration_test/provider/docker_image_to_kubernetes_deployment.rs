use dal::test::helpers::{
    find_prop_and_parent_by_name, process_job_queue, update_attribute_value_for_prop_and_context,
    ComponentPayload,
};
use dal::Component;
use dal::{
    AttributeReadContext, Connection, DalContext, ExternalProvider, InternalProvider, Schema,
    StandardModel,
};
use pretty_assertions_sorted::assert_eq_sorted;
use std::collections::HashMap;

use crate::dal::test;

#[test]
async fn docker_image_to_kubernetes_deployment_inter_component_update(ctx: &DalContext<'_, '_>) {
    let tail_docker_image_payload = setup_docker_image(ctx).await;
    let head_deployment_payload = setup_kubernetes_deployment(ctx).await;

    // Initialize the tail "/root/si/name" field.
    update_attribute_value_for_prop_and_context(
        ctx,
        tail_docker_image_payload.get_prop_id("/root/si/name"),
        Some(serde_json::json!["tail"]),
        tail_docker_image_payload.base_attribute_read_context,
    )
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
    Connection::connect_providers(
        ctx,
        "identity",
        *tail_external_provider.id(),
        tail_docker_image_payload.component_id,
        *head_explicit_internal_provider.id(),
        head_deployment_payload.component_id,
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
    update_attribute_value_for_prop_and_context(
        ctx,
        tail_docker_image_payload.get_prop_id("/root/si/name"),
        Some(serde_json::json!["ironsides"]),
        tail_docker_image_payload.base_attribute_read_context,
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

async fn setup_docker_image(ctx: &DalContext<'_, '_>) -> ComponentPayload {
    let schema_name = "docker_image".to_string();
    let schema: Schema = Schema::find_by_attr(ctx, "name", &schema_name)
        .await
        .expect("could not find schema by name")
        .pop()
        .expect("schema not found");
    let schema_variant_id = schema
        .default_schema_variant_id()
        .expect("default schema variant id not found");

    let (component, _) = Component::new_for_schema_with_node(ctx, "image", schema.id())
        .await
        .expect("unable to create component");
    process_job_queue(ctx).await;

    let base_attribute_read_context = AttributeReadContext {
        prop_id: None,
        schema_id: Some(*schema.id()),
        schema_variant_id: Some(*schema_variant_id),
        component_id: Some(*component.id()),
        ..AttributeReadContext::default()
    };
    let mut prop_map = HashMap::new();

    let (name_prop_id, _si_prop_id) =
        find_prop_and_parent_by_name(ctx, "name", "si", None, *schema_variant_id)
            .await
            .expect("could not find prop (and its parent)");
    prop_map.insert("/root/si/name", name_prop_id);

    ComponentPayload {
        schema_id: *schema.id(),
        schema_variant_id: *schema_variant_id,
        component_id: *component.id(),
        prop_map,
        base_attribute_read_context,
    }
}

async fn setup_kubernetes_deployment(ctx: &DalContext<'_, '_>) -> ComponentPayload {
    let schema_name = "kubernetes_deployment".to_string();
    let schema: Schema = Schema::find_by_attr(ctx, "name", &schema_name)
        .await
        .expect("could not find schema by name")
        .pop()
        .expect("schema not found");
    let schema_variant_id = schema
        .default_schema_variant_id()
        .expect("default schema variant id not found");

    let (component, _) = Component::new_for_schema_with_node(ctx, "deployment", schema.id())
        .await
        .expect("unable to create component");
    process_job_queue(ctx).await;

    let base_attribute_read_context = AttributeReadContext {
        prop_id: None,
        schema_id: Some(*schema.id()),
        schema_variant_id: Some(*schema_variant_id),
        component_id: Some(*component.id()),
        ..AttributeReadContext::default()
    };
    let prop_map = HashMap::new();

    ComponentPayload {
        schema_id: *schema.id(),
        schema_variant_id: *schema_variant_id,
        component_id: *component.id(),
        prop_map,
        base_attribute_read_context,
    }
}
