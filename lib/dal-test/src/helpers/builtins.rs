//! This module provides a harness for use in integration tests related to providers and builtin
//! [`Schema`s](dal::Schema) leveraging them. It will cache relevant information to reduce the
//! total number of queries to the database during a test.

use std::collections::HashMap;

use dal::{
    AttributeReadContext, Component, DalContext, PropId, SchemaId, SchemaVariantId, StandardModel,
};

use super::{
    component_payload::ComponentPayload, find_prop_and_parent_by_name,
    find_schema_and_default_variant_by_name,
};

/// A list of builtin schemas that can be used to create [`Component`](Component) for integration
/// tests.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub enum Builtin {
    AwsEc2,
    AwsIngress,
    AwsRegion,
    AwsSecurityGroup,
    CoreOsButane,
    DockerHubCredential,
    DockerImage,
    KubernetesDeployment,
    KubernetesNamespace,
}

impl Builtin {
    /// Returns its schema name.
    pub fn as_str(&self) -> &'static str {
        match &self {
            Builtin::AwsEc2 => "EC2 Instance",
            Builtin::AwsIngress => "Ingress",
            Builtin::AwsRegion => "Region",
            Builtin::AwsSecurityGroup => "Security Group",
            Builtin::CoreOsButane => "Butane",
            Builtin::DockerHubCredential => "Docker Hub Credential",
            Builtin::DockerImage => "Docker Image",
            Builtin::KubernetesDeployment => "Kubernetes Deployment",
            Builtin::KubernetesNamespace => "Kubernetes Namespace",
        }
    }
}

/// A private struct to provide helpful metadata for a given [`Builtin`].
#[derive(Clone)]
struct BuiltinMetadata {
    schema_id: SchemaId,
    schema_variant_id: SchemaVariantId,
    prop_map: PropMap,
}

/// A hash map of [`PropId`s](PropId) where the key is the JSON pointer to the [`Prop`] on the
/// [`SchemaVariant`].
type PropMap = HashMap<&'static str, PropId>;

/// This harness provides methods to create [`Component`s](Component) from builtin schemas. All
/// fields are private since they are purely used to reduce the number of total database queries.
#[derive(Default)]
pub struct SchemaBuiltinsTestHarness {
    builtins: HashMap<Builtin, BuiltinMetadata>,
}

impl SchemaBuiltinsTestHarness {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a [`ComponentPayload`] (a [`Component`] with contextual metadata) for a given
    /// [`Builtin`].
    pub async fn create_component(
        &mut self,
        ctx: &DalContext,
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
                // If metadata has not been cached, we need to do that.
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

    /// Private method to create a [`Component`] and assemble a [`ComponentPayload`].
    async fn perform_component_creation_and_payload_assembly(
        ctx: &DalContext,
        schema_id: SchemaId,
        schema_variant_id: SchemaVariantId,
        prop_map: HashMap<&'static str, PropId>,
        name: impl AsRef<str>,
    ) -> ComponentPayload {
        let (component, node) = Component::new(ctx, name, schema_variant_id)
            .await
            .expect("unable to create component");

        ComponentPayload {
            schema_id,
            schema_variant_id,
            component_id: *component.id(),
            prop_map,
            node_id: *node.id(),
            base_attribute_read_context: AttributeReadContext {
                prop_id: None,
                component_id: Some(*component.id()),
                ..AttributeReadContext::default()
            },
        }
    }

    /// Private function to build a [`PropMap`] for a given [`Builtin`]. This function will
    /// populate the map differently depending on the `Builtin` provided.
    async fn build_prop_map(
        ctx: &DalContext,
        builtin: Builtin,
        schema_variant_id: SchemaVariantId,
    ) -> PropMap {
        let mut prop_map = match builtin {
            Builtin::AwsIngress => Self::cache_props_for_aws_ingress(ctx, schema_variant_id).await,
            Builtin::AwsRegion => Self::cache_props_for_aws_region(ctx, schema_variant_id).await,
            Builtin::AwsSecurityGroup => {
                Self::cache_props_for_aws_security_group(ctx, schema_variant_id).await
            }
            Builtin::CoreOsButane => {
                Self::cache_props_for_coreos_butane(ctx, schema_variant_id).await
            }
            Builtin::DockerImage => {
                Self::cache_props_for_docker_image(ctx, schema_variant_id).await
            }
            Builtin::KubernetesNamespace => {
                Self::cache_props_for_kubernetes_namespace(ctx, schema_variant_id).await
            }
            _ => PropMap::new(),
        };

        // Always provide "/root/si/name" for all builtins.
        let (si_name_prop_id, _) =
            find_prop_and_parent_by_name(ctx, "name", "si", None, schema_variant_id)
                .await
                .expect("could not find prop and/or parent");
        prop_map.insert("/root/si/name", si_name_prop_id);

        prop_map
    }

    async fn cache_props_for_aws_ingress(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> PropMap {
        let mut prop_map = PropMap::new();
        let (prop_id, _) =
            find_prop_and_parent_by_name(ctx, "GroupId", "domain", None, schema_variant_id)
                .await
                .expect("could not find prop and/or parent");
        prop_map.insert("/root/domain/GroupId", prop_id);
        prop_map
    }

    async fn cache_props_for_aws_region(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> PropMap {
        let mut prop_map = PropMap::new();
        let (prop_id, _) =
            find_prop_and_parent_by_name(ctx, "region", "domain", None, schema_variant_id)
                .await
                .expect("could not find prop and/or parent");
        prop_map.insert("/root/domain/region", prop_id);
        prop_map
    }

    async fn cache_props_for_aws_security_group(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> PropMap {
        let mut prop_map = PropMap::new();

        let (prop_id, _) =
            find_prop_and_parent_by_name(ctx, "Description", "domain", None, schema_variant_id)
                .await
                .expect("could not find prop and/or parent");
        prop_map.insert("/root/domain/Description", prop_id);
        let (prop_id, _) =
            find_prop_and_parent_by_name(ctx, "GroupName", "domain", None, schema_variant_id)
                .await
                .expect("could not find prop and/or parent");
        prop_map.insert("/root/domain/VpcId", prop_id);
        let (prop_id, _) =
            find_prop_and_parent_by_name(ctx, "VpcId", "domain", None, schema_variant_id)
                .await
                .expect("could not find prop and/or parent");
        prop_map.insert("/root/domain/VpcId", prop_id);
        let (prop_id, _) =
            find_prop_and_parent_by_name(ctx, "region", "domain", None, schema_variant_id)
                .await
                .expect("could not find prop and/or parent");
        prop_map.insert("/root/domain/region", prop_id);

        prop_map
    }

    async fn cache_props_for_coreos_butane(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> PropMap {
        let mut prop_map = PropMap::new();

        // All fields including and above "unit".
        let (variant_prop_id, _) =
            find_prop_and_parent_by_name(ctx, "variant", "domain", None, schema_variant_id)
                .await
                .expect("could not find prop and/or parent");
        prop_map.insert("/root/domain/variant", variant_prop_id);
        let (version_prop_id, _) =
            find_prop_and_parent_by_name(ctx, "version", "domain", None, schema_variant_id)
                .await
                .expect("could not find prop and/or parent");
        prop_map.insert("/root/domain/version", version_prop_id);
        let (systemd_prop_id, _) =
            find_prop_and_parent_by_name(ctx, "systemd", "domain", None, schema_variant_id)
                .await
                .expect("could not find prop and/or parent");
        prop_map.insert("/root/domain/systemd", systemd_prop_id);
        let (units_prop_id, _) =
            find_prop_and_parent_by_name(ctx, "units", "systemd", None, schema_variant_id)
                .await
                .expect("could not find prop and/or parent");
        prop_map.insert("/root/domain/systemd/units", units_prop_id);
        let (unit_prop_id, _) =
            find_prop_and_parent_by_name(ctx, "unit", "units", None, schema_variant_id)
                .await
                .expect("could not find prop and/or parent");
        prop_map.insert("/root/domain/systemd/units/unit", unit_prop_id);

        // All fields under "unit".
        let (unit_name_prop_id, _) =
            find_prop_and_parent_by_name(ctx, "name", "unit", None, schema_variant_id)
                .await
                .expect("could not find prop and/or parent");
        prop_map.insert("/root/domain/systemd/units/unit/name", unit_name_prop_id);
        let (unit_enabled_prop_id, _) =
            find_prop_and_parent_by_name(ctx, "enabled", "unit", None, schema_variant_id)
                .await
                .expect("could not find prop and/or parent");
        prop_map.insert(
            "/root/domain/systemd/units/unit/enabled",
            unit_enabled_prop_id,
        );
        let (unit_contents_prop_id, _) =
            find_prop_and_parent_by_name(ctx, "contents", "unit", None, schema_variant_id)
                .await
                .expect("could not find prop and/or parent");
        prop_map.insert(
            "/root/domain/systemd/units/unit/contents",
            unit_contents_prop_id,
        );
        prop_map
    }

    async fn cache_props_for_docker_image(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> PropMap {
        let mut prop_map = PropMap::new();
        let (image_prop_id, _) =
            find_prop_and_parent_by_name(ctx, "image", "domain", None, schema_variant_id)
                .await
                .expect("could not find prop and/or parent");
        prop_map.insert("/root/domain/image", image_prop_id);
        let (exposed_ports_prop_id, _) =
            find_prop_and_parent_by_name(ctx, "ExposedPorts", "domain", None, schema_variant_id)
                .await
                .expect("could not find prop and/or parent");
        prop_map.insert("/root/domain/exposed-ports", exposed_ports_prop_id);
        let (exposed_port_prop_id, _) = find_prop_and_parent_by_name(
            ctx,
            "ExposedPort",
            "ExposedPorts",
            None,
            schema_variant_id,
        )
        .await
        .expect("could not find prop and/or parent");
        prop_map.insert(
            "/root/domain/exposed-ports/exposed-port",
            exposed_port_prop_id,
        );
        prop_map
    }

    async fn cache_props_for_kubernetes_namespace(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> PropMap {
        let mut prop_map = PropMap::new();
        let (metadata_name_prop_id, _) =
            find_prop_and_parent_by_name(ctx, "name", "metadata", None, schema_variant_id)
                .await
                .expect("could not find prop and/or parent");
        prop_map.insert("/root/domain/metadata/name", metadata_name_prop_id);
        prop_map
    }
}
