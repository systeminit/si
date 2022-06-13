use dal::builtins::helpers::{parent_prop, update_attribute_value_for_prop_and_context};
use dal::test::helpers::{find_prop_and_parent_by_name, ComponentPayload};
use dal::{
    AttributeReadContext, Connection, DalContext, ExternalProvider, InternalProvider, Schema,
    StandardModel,
};
use dal::{AttributeValue, Component};
use pretty_assertions_sorted::assert_eq_sorted;
use std::collections::HashMap;

use crate::dal::test;

#[test]
async fn kubernetes_namespace_to_kubernetes_deployment_inter_component_update() {
    let test_context = ::dal::test::TestContext::global().await;
    let nats_subject_prefix = ::dal::test::nats_subject_prefix();
    let services_context = test_context
        .create_services_context(&nats_subject_prefix)
        .await;
    let dal_context_builder = services_context.into_builder();
    let mut transactions_starter = dal_context_builder
        .transactions_starter()
        .await
        .expect("failed to build transactions starter");
    let transactions = transactions_starter
        .start()
        .await
        .expect("failed to start transactions");
    let (nba, auth_token) = ::dal::test::helpers::billing_account_signup(
        &dal_context_builder,
        &transactions,
        test_context.jwt_secret_key(),
    )
    .await;
    let application_id =
        ::dal::test::helpers::create_application(&dal_context_builder, &transactions, &nba).await;
    let application_id = {
        use dal::StandardModel;
        *application_id.id()
    };
    let default_dal_context = ::dal::test::helpers::create_ctx_for_new_change_set_and_edit_session(
        &dal_context_builder,
        &transactions,
        &nba,
        application_id,
    )
    .await;
    let veritech_server = ::dal::test::veritech_server_for_uds_cyclone(
        test_context.nats_config().clone(),
        nats_subject_prefix.clone(),
    )
    .await;
    ::tokio::spawn(veritech_server.run());
    let ctx = &default_dal_context;

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
    update_attribute_value_for_prop_and_context(
        ctx,
        tail_namespace_payload.get_prop_id("/root/domain/metadata/name"),
        Some(serde_json::json!["look-at-me-mom-i-updated"]),
        tail_namespace_payload.base_attribute_read_context,
    )
    .await
    .expect("could not update attribute value");

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

    dbg!("FOO");
    dbg!(
        "UPDATED VALUES",
        AttributeValue::find_for_context(
            ctx,
            tail_namespace_payload
                .attribute_read_context_with_prop_id("/root/domain/metadata/name"),
        )
        .await
        .unwrap(),
        AttributeValue::find_for_context(
            ctx,
            head_deployment_payload
                .attribute_read_context_with_prop_id("/root/domain/metadata/namespace"),
        )
        .await
        .unwrap(),
        AttributeValue::find_for_context(
            ctx,
            head_deployment_payload.attribute_read_context_with_prop_id(
                "/root/domain/spec/template/metadata/namespace"
            ),
        )
        .await
        .unwrap(),
    );

    transactions.commit().await.expect("failed to commit");
    panic!();
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

    let (name_prop_id, _metadata_prop_id) =
        find_prop_and_parent_by_name(ctx, "name", "metadata", None, *schema_variant_id)
            .await
            .expect("could not find prop (and its parent): /root/domain/metadata/name");

    // We only need to store the name prop.
    let mut prop_map = HashMap::new();
    prop_map.insert("/root/domain/metadata/name", name_prop_id);

    // Initialize the value corresponding to the "name" prop.
    update_attribute_value_for_prop_and_context(
        ctx,
        name_prop_id,
        Some(serde_json::json!["tail"]),
        base_attribute_read_context,
    )
    .await
    .expect("could not update attribute value");

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
    let mut prop_map = HashMap::new();

    // Initialize the value corresponding to the "domain namespace" prop.
    let (domain_namespace_prop_id, _domain_metadata_prop_id) = find_prop_and_parent_by_name(
        ctx,
        "namespace",
        "metadata",
        Some("domain"),
        *schema_variant_id,
    )
    .await
    .expect("could not find prop (and its parent): /root/domain/metadata/namespace");
    update_attribute_value_for_prop_and_context(
        ctx,
        domain_namespace_prop_id,
        Some(serde_json::json!["head-domain"]),
        base_attribute_read_context,
    )
    .await
    .expect("could not update attribute value");
    prop_map.insert("/root/domain/metadata/namespace", domain_namespace_prop_id);

    // Initialize the value corresponding to the "template namespace" prop.
    let (template_namespace_prop_id, _template_metadata_prop_id) = find_prop_and_parent_by_name(
        ctx,
        "namespace",
        "metadata",
        Some("template"),
        *schema_variant_id,
    )
    .await
    .expect("could not find prop (and its parent): /root/domain/metadata/namespace");
    update_attribute_value_for_prop_and_context(
        ctx,
        template_namespace_prop_id,
        Some(serde_json::json!["head-template"]),
        base_attribute_read_context,
    )
    .await
    .expect("could not update attribute value");
    prop_map.insert(
        "/root/domain/spec/template/metadata/namespace",
        template_namespace_prop_id,
    );

    ComponentPayload {
        schema_id: *schema.id(),
        schema_variant_id: *schema_variant_id,
        component_id: *component.id(),
        prop_map,
        base_attribute_read_context,
    }
}
