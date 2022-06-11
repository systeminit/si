use dal::test::helpers::{
    find_prop_and_parent_by_name, parent_prop, update_prop_attribute_value, ComponentPayload,
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
async fn kubernetes_namespace_to_kubernetes_deployment_inter_component_update(
    ctx: &DalContext<'_, '_>,
) {
    let tail_namespace_payload = setup_kubernetes_namespace(ctx).await;
    let head_deployment_payload = setup_kubernetes_deployment(ctx).await;

    // Ensure setup worked.
    assert_eq_sorted!(
        serde_json::json![{
            "domain": {
                "metadata": {
                    "name": "tail"
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
                "metadata": {
                    "namespace": "head-domain"
                },
                "spec": {
                    "template": {
                        "metadata": {
                            "namespace": "head-template"
                        }
                    }
                },
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
        tail_namespace_payload.schema_variant_id,
        "namespace-string-output".to_string(),
    )
    .await
    .expect("cannot find external provider")
    .expect("external provider not found");
    let head_explicit_internal_provider =
        InternalProvider::find_explicit_for_schema_variant_and_name(
            ctx,
            head_deployment_payload.schema_variant_id,
            "namespace-string-input".to_string(),
        )
        .await
        .expect("cannot find explicit internal provider")
        .expect("explicit internal provider not found");

    // Finally, create the inter component connection.
    Connection::connect_providers(
        ctx,
        "identity".to_string(),
        *tail_external_provider.id(),
        tail_namespace_payload.component_id,
        *head_explicit_internal_provider.id(),
        head_deployment_payload.component_id,
    )
    .await
    .expect("could not connect providers");

    // Ensure the view did not drift.
    assert_eq_sorted!(
        serde_json::json![{
            "domain": {
                "metadata": {
                    "name": "tail"
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
                "metadata": {
                    "namespace": "head-domain"
                },
                "spec": {
                    "template": {
                        "metadata": {
                            "namespace": "head-template"
                        }
                    }
                },
            },
            "si": {
                "name": "deployment"
            }
        }], // expected
        head_deployment_payload.component_view_properties(ctx).await // actual
    );

    // Perform update!
    update_prop_attribute_value(
        ctx,
        tail_namespace_payload.get_prop_id("/root/domain/metadata/name"),
        Some(serde_json::json!["look-at-me-mom-i-updated"]),
        tail_namespace_payload.base_attribute_read_context,
    )
    .await;

    // Observed that it worked.
    assert_eq_sorted!(
        serde_json::json![{
            "domain": {
                "metadata": {
                    "name": "look-at-me-mom-i-updated"
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
            "si": {
                "name": "deployment"
            }
        }], // expected
        head_deployment_payload.component_view_properties(ctx).await // actual
    );
}

async fn setup_kubernetes_namespace(ctx: &DalContext<'_, '_>) -> ComponentPayload {
    let schema_name = "kubernetes_namespace".to_string();
    let schema: Schema = Schema::find_by_attr(ctx, "name", &schema_name)
        .await
        .expect("could not find schema by name")
        .pop()
        .expect("schema not found");
    let schema_variant_id = schema
        .default_schema_variant_id()
        .expect("default schema variant id not found");

    let (component, _, _) = Component::new_for_schema_with_node(ctx, "namespace", schema.id())
        .await
        .expect("unable to create component");
    let base_attribute_read_context = AttributeReadContext {
        prop_id: None,
        schema_id: Some(*schema.id()),
        schema_variant_id: Some(*schema_variant_id),
        component_id: Some(*component.id()),
        ..AttributeReadContext::default()
    };

    let (name_prop_id, metadata_prop_id) =
        find_prop_and_parent_by_name(ctx, "name", "metadata", None, *schema_variant_id)
            .await
            .expect("could not find prop (and its parent): /root/domain/metadata/name");

    // We only need to store the name prop.
    let mut prop_map = HashMap::new();
    prop_map.insert("/root/domain/metadata/name", name_prop_id);

    // Initialize the value corresponding to the "name" prop.
    update_prop_attribute_value(
        ctx,
        metadata_prop_id,
        Some(serde_json::json![{}]),
        base_attribute_read_context,
    )
    .await;
    update_prop_attribute_value(
        ctx,
        name_prop_id,
        Some(serde_json::json!["tail"]),
        base_attribute_read_context,
    )
    .await;

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

    let (component, _, _) = Component::new_for_schema_with_node(ctx, "deployment", schema.id())
        .await
        .expect("unable to create component");
    let base_attribute_read_context = AttributeReadContext {
        prop_id: None,
        schema_id: Some(*schema.id()),
        schema_variant_id: Some(*schema_variant_id),
        component_id: Some(*component.id()),
        ..AttributeReadContext::default()
    };

    // Initialize the value corresponding to the "domain namespace" prop.
    let (domain_namespace_prop_id, domain_metadata_prop_id) = find_prop_and_parent_by_name(
        ctx,
        "namespace",
        "metadata",
        Some("domain"),
        *schema_variant_id,
    )
    .await
    .expect("could not find prop (and its parent): /root/domain/metadata/namespace");
    update_prop_attribute_value(
        ctx,
        domain_metadata_prop_id,
        Some(serde_json::json![{}]),
        base_attribute_read_context,
    )
    .await;
    update_prop_attribute_value(
        ctx,
        domain_namespace_prop_id,
        Some(serde_json::json!["head-domain"]),
        base_attribute_read_context,
    )
    .await;

    // Initialize the value corresponding to the "template namespace" prop.
    let (template_namespace_prop_id, template_metadata_prop_id) = find_prop_and_parent_by_name(
        ctx,
        "namespace",
        "metadata",
        Some("template"),
        *schema_variant_id,
    )
    .await
    .expect("could not find prop (and its parent): /root/domain/metadata/namespace");
    let template_prop_id = parent_prop(ctx, template_metadata_prop_id).await;
    let spec_prop_id = parent_prop(ctx, template_prop_id).await;

    update_prop_attribute_value(
        ctx,
        spec_prop_id,
        Some(serde_json::json![{}]),
        base_attribute_read_context,
    )
    .await;
    update_prop_attribute_value(
        ctx,
        template_prop_id,
        Some(serde_json::json![{}]),
        base_attribute_read_context,
    )
    .await;
    update_prop_attribute_value(
        ctx,
        template_metadata_prop_id,
        Some(serde_json::json![{}]),
        base_attribute_read_context,
    )
    .await;
    update_prop_attribute_value(
        ctx,
        template_namespace_prop_id,
        Some(serde_json::json!["head-template"]),
        base_attribute_read_context,
    )
    .await;

    ComponentPayload {
        schema_id: *schema.id(),
        schema_variant_id: *schema_variant_id,
        component_id: *component.id(),
        prop_map: HashMap::new(),
        base_attribute_read_context,
    }
}
