use crate::protobuf::deployment::{
    ImplicitConstraint, ListComponentsReply, ListComponentsRequest, PickComponentRequest,
};
use si_cea::component::prelude::*;
use si_data::Db;
use std::fmt;

pub use crate::protobuf::deployment::Component;

// NOTE(fnichol): this for sure should not live here long term; it needs to be rolled up into
// top module level info, metadata, etc.
const DEFAULT_KUBERNETES_VERSION: &str = "1.15";
const VALID_KUBERNETES_VERSION_VALUES: &[&str] = &["1.15", "1.14", "1.13", "1.12"];

gen_component!(
    type_name: "kubernetes_deployment",
    order_by_fields: [
        "kubernetesVersion"
    ],
    validate_fn: |self| {
        if self.display_name.is_empty() {
            return Err(DataError::ValidationError(format!("missing {}", Field::DisplayName)));
        }
        if self.name.is_empty() {
            return Err(DataError::ValidationError(format!("missing {}", Field::Name)));
        }
        // validate the version is right? seems real.
        Ok(())
    }
);

impl Component {
    pub async fn pick(
        db: &Db,
        req: &PickComponentRequest,
    ) -> CeaResult<(ImplicitConstraints, Self)> {
        let mut implicit_constraints = ImplicitConstraints::new();

        if let Some(result) =
            Self::pick_by_string_field(db, Field::Name.to_string(), req.name.clone()).await?
        {
            return Ok((implicit_constraints, result));
        }
        if let Some(result) =
            Self::pick_by_string_field(db, Field::DisplayName.to_string(), req.display_name.clone())
                .await?
        {
            return Ok((implicit_constraints, result));
        }

        let kubernetes_version = {
            let rkv = &req.kubernetes_version;

            if rkv.is_empty() {
                implicit_constraints.add(
                    Field::KubernetesVersion.to_string(),
                    DEFAULT_KUBERNETES_VERSION,
                );
                DEFAULT_KUBERNETES_VERSION.to_string()
            } else if Field::is_valid_kubernetes_version(&req.kubernetes_version) {
                rkv.clone()
            } else {
                return Err(CeaError::PickComponent(format!(
                    "invalid {}: {}",
                    Field::KubernetesVersion,
                    rkv
                )));
            }
        };

        let query_expressions = vec![si_data::Query::generate_expression_for_string(
            Field::KubernetesVersion.to_string(),
            si_data::QueryComparison::Equals,
            kubernetes_version,
        )];
        let component =
            Component::pick_by_expressions(db, query_expressions, si_data::QueryBooleanLogic::And)
                .await?;

        Ok((implicit_constraints, component))
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
                display_name: name.clone(),
                description: name.clone(),
                integration_id: aws_integration.get_id().to_string(),
                integration_service_id: aws_eks_integration_service.get_id().to_string(),
                display_type_name: "AWS EKS Kubernetes {} Deployment".to_string(),
                name,
                version: 2,
                kubernetes_version: kubernetes_version.to_string(),
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
