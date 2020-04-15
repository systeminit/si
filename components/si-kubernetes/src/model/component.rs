pub use crate::gen::kubernetes_deployment::component::{
    Component, Constraints, PickComponentRequest,
};

use si_cea::component::prelude::*;
use si_data::Db;
use std::fmt;

// NOTE(fnichol): this for sure should not live here long term; it needs to be rolled up into
// top module level info, metadata, etc.
const DEFAULT_KUBERNETES_VERSION: &str = "1.15";
const VALID_KUBERNETES_VERSION_VALUES: &[&str] = &["1.15", "1.14", "1.13", "1.12"];

impl Component {
    pub async fn pick(db: &Db, req: &PickComponentRequest) -> CeaResult<(Constraints, Self)> {
        match &req.constraints {
            None => Err(CeaError::InvalidPickMissingConstraints),
            Some(constraints) => {
                if let Some(found) = Self::pick_by_component_name(db, &constraints).await? {
                    return Ok(found);
                }
                if let Some(found) = Self::pick_by_component_display_name(db, &constraints).await? {
                    return Ok(found);
                }

                let mut implicit_constraints = Constraints::default();
                let mut query_items = Vec::new();

                let kubernetes_version = match &constraints.kubernetes_version {
                    Some(value) => {
                        if Field::is_valid_kubernetes_version(&value) {
                            value.clone()
                        } else {
                            return Err(CeaError::PickComponent(format!(
                                "invalid {}: {}",
                                Field::KubernetesVersion,
                                value
                            )));
                        }
                    }
                    None => {
                        implicit_constraints.kubernetes_version =
                            Some(DEFAULT_KUBERNETES_VERSION.to_string());
                        DEFAULT_KUBERNETES_VERSION.to_string()
                    }
                };
                query_items.push(si_data::QueryItems::generate_expression_for_string(
                    Field::KubernetesVersion.to_string(),
                    si_data::QueryItemsExpressionComparison::Equals,
                    kubernetes_version,
                ));

                let component =
                    Self::pick_by_expressions(db, query_items, si_data::QueryBooleanTerm::And)
                        .await?;

                Ok((implicit_constraints, component))
            }
        }
    }

    // NOTE(fnichol): This can likely be common/cea code?
    async fn pick_by_component_name(
        db: &Db,
        req: &Constraints,
    ) -> CeaResult<Option<(Constraints, Self)>> {
        match &req.component_name {
            Some(name) => {
                match Self::pick_by_string_field(db, Field::Name.to_string(), name).await? {
                    Some(component) => Ok(Some((Constraints::default(), component))),
                    None => Ok(None),
                }
            }
            None => Ok(None),
        }
    }

    // NOTE(fnichol): This can likely be common/cea code?
    async fn pick_by_component_display_name(
        db: &Db,
        req: &Constraints,
    ) -> CeaResult<Option<(Constraints, Self)>> {
        match &req.component_display_name {
            Some(display_name) => {
                match Self::pick_by_string_field(db, Field::DisplayName.to_string(), display_name)
                    .await?
                {
                    Some(component) => Ok(Some((Constraints::default(), component))),
                    None => Ok(None),
                }
            }
            None => Ok(None),
        }
    }
}

#[async_trait::async_trait]
impl MigrateComponent for Component {
    async fn migrate(db: &Db) -> CeaResult<()> {
        // Should these be internal model calls? Pretty sure they def should.
        let aws_integration: Integration =
            db.lookup_by_natural_key("global:integration:aws").await?;

        let aws_eks_integration_service_id = format!(
            "global:{}:integration_service:eks_kubernetes",
            aws_integration.get_id()
        );

        let aws_eks_integration_service: IntegrationService = db
            .lookup_by_natural_key(aws_eks_integration_service_id)
            .await?;

        for kubernetes_version in VALID_KUBERNETES_VERSION_VALUES {
            let name = format!("AWS EKS Kubernetes {} Deployment", kubernetes_version);
            let mut c = Component {
                name: Some(name.clone()),
                display_name: Some(name.clone()),
                description: Some(name.clone()),

                // integration_id: aws_integration.get_id().to_string(),
                // integration_service_id: aws_eks_integration_service.get_id().to_string(),
                // version: 2,
                // kubernetes_version: kubernetes_version.to_string(),
                ..Default::default()
            };
            c.add_to_tenant_ids("global".to_string());
            c.add_to_tenant_ids(aws_integration.get_id().to_string());
            c.add_to_tenant_ids(aws_eks_integration_service.get_id().to_string());
            db.migrate(&mut c).await?;
        }

        Ok(())
    }
}

enum Field {
    DisplayName,
    KubernetesVersion,
    Name,
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            Self::DisplayName => "displayName",
            Self::KubernetesVersion => "kubernetesVersion",
            Self::Name => "name",
        };
        write!(f, "{}", msg)
    }
}

impl Field {
    fn is_valid_kubernetes_version(s: &str) -> bool {
        VALID_KUBERNETES_VERSION_VALUES.contains(&s)
    }
}
