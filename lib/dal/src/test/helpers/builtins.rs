//! This module provides [`BuiltinsHarness`], for use in integration tests related to
//! providers and builtin [`Schemas`](crate::Schema) leveraging them.

use std::collections::HashMap;

use crate::test::helpers::{
    find_prop_and_parent_by_name, find_schema_and_default_variant_by_name, ComponentPayload,
};
use crate::{
    AttributeReadContext, Component, DalContext, PropId, SchemaId, SchemaVariantId, StandardModel,
};

/// A list of builtin [`Schemas`](crate::Schema) that can be used to create
/// [`Components`](crate::Component) for integration tests.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub enum Builtin {
    DockerHubCredential,
    DockerImage,
    KubernetesDeployment,
    KubernetesNamespace,
}

impl Builtin {
    /// Converts a [`Builtin`](Self) to its [`Schema`](crate::Schema) name.
    pub fn as_str(&self) -> &'static str {
        match &self {
            Builtin::DockerHubCredential => "docker_hub_credential",
            Builtin::DockerImage => "docker_image",
            Builtin::KubernetesDeployment => "kubernetes_deployment",
            Builtin::KubernetesNamespace => "kubernetes_namespace",
        }
    }
}

/// A private struct to provide helpful metadata for a given [`Builtin`](Builtin).
#[derive(Clone)]
struct BuiltinMetadata {
    schema_id: SchemaId,
    schema_variant_id: SchemaVariantId,
    prop_map: PropMap,
}

/// A hash map of [`PropIds`](crate::Prop) where the key is the JSON pointer to the
/// [`Prop`](crate::Prop) on the [`SchemaVariant`](crate::SchemaVariant).
type PropMap = HashMap<&'static str, PropId>;

/// This harness provides methods to create [`Components`](crate::Component) from builtin
/// [`Schemas`](crate::Schema). All fields are private since they are purely used to reduce the
/// number of total database queries.
#[derive(Default)]
pub struct BuiltinsHarness {
    builtins: HashMap<Builtin, BuiltinMetadata>,
}

impl BuiltinsHarness {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a [`ComponentPayload`](crate::test::helpers::ComponentPayload) (a
    /// [`Component`](crate::Component) with contextual metadata) for a given
    /// [`Builtin`](Builtin).
    pub async fn create_component(
        &mut self,
        ctx: &DalContext<'_, '_>,
        name: impl AsRef<str>,
        builtin: Builtin,
    ) -> ComponentPayload {
        match self.builtins.get(&builtin) {
            Some(populated_builtin_metadata) => {
                Self::perform_component_creation_and_payload_assembly(
                    ctx,
                    populated_builtin_metadata.schema_id,
                    populated_builtin_metadata.schema_variant_id,
                    populated_builtin_metadata.prop_map.clone(),
                    name,
                )
                .await
            }
            None => {
                let (schema, schema_variant) =
                    find_schema_and_default_variant_by_name(ctx, builtin.as_str()).await;
                let prop_map = Self::build_prop_map(ctx, builtin, *schema_variant.id()).await;

                let new_builtin_metadata = BuiltinMetadata {
                    schema_id: *schema.id(),
                    schema_variant_id: *schema_variant.id(),
                    prop_map,
                };
                self.builtins.insert(builtin, new_builtin_metadata.clone());

                Self::perform_component_creation_and_payload_assembly(
                    ctx,
                    new_builtin_metadata.schema_id,
                    new_builtin_metadata.schema_variant_id,
                    new_builtin_metadata.prop_map.clone(),
                    name,
                )
                .await
            }
        }
    }

    /// Private function to build a [`PropMap`](PropMap) for a given [`Builtin`](Builtin). This
    /// function will populate the map differently depending on the [`Builtin`](Builtin) provided.
    async fn build_prop_map(
        ctx: &DalContext<'_, '_>,
        builtin: Builtin,
        schema_variant_id: SchemaVariantId,
    ) -> PropMap {
        let mut prop_map = HashMap::new();

        // For kubernetes namespaces, we also want "/root/si/metadata/name". We can add more
        // Builtin-specific props to collect too in the future!
        if let Builtin::KubernetesNamespace = builtin {
            let (metadata_name_prop_id, _) =
                find_prop_and_parent_by_name(ctx, "name", "metadata", None, schema_variant_id)
                    .await
                    .expect("could not find prop and/or parent");
            prop_map.insert("/root/si/metadata/name", metadata_name_prop_id);
        }

        // Always provide "/root/si/name".
        let (si_name_prop_id, _) =
            find_prop_and_parent_by_name(ctx, "name", "si", None, schema_variant_id)
                .await
                .expect("could not find prop and/or parent");
        prop_map.insert("/root/si/name", si_name_prop_id);

        prop_map
    }

    /// Private method to create a [`Component`](crate::Component) and assemble a
    /// [`ComponentPayload`](crate::test::helpers::ComponentPayload).
    async fn perform_component_creation_and_payload_assembly(
        ctx: &DalContext<'_, '_>,
        schema_id: SchemaId,
        schema_variant_id: SchemaVariantId,
        prop_map: HashMap<&'static str, PropId>,
        name: impl AsRef<str>,
    ) -> ComponentPayload {
        let (component, node) = Component::new_for_schema_with_node(ctx, name, &schema_id)
            .await
            .expect("unable to create component");

        ComponentPayload {
            schema_id,
            schema_variant_id,
            component_id: *component.id(),
            prop_map,
            node,
            base_attribute_read_context: AttributeReadContext {
                prop_id: None,
                schema_id: Some(schema_id),
                schema_variant_id: Some(schema_variant_id),
                component_id: Some(*component.id()),
                ..AttributeReadContext::default()
            },
        }
    }
}
