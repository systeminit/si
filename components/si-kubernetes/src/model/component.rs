use si_cea::component::prelude::*;
use std::fmt;
use std::str::FromStr;

pub use crate::gen::kubernetes::kubernetes_deployment_component::{
    KubernetesDeploymentComponent, KubernetesDeploymentComponentConstraints,
};
pub use crate::protobuf::KubernetesDeploymentComponentConstraintsKubernetesVersion;

type Constraints = KubernetesDeploymentComponentConstraints;
type KubernetesVersion = KubernetesDeploymentComponentConstraintsKubernetesVersion;

impl KubernetesDeploymentComponent {
    pub async fn pick(db: &Db, constraints: &Constraints) -> CeaResult<(Constraints, Self)> {
        if let Some(found) = Self::pick_by_component_name(db, constraints).await? {
            return Ok(found);
        }
        if let Some(found) = Self::pick_by_component_display_name(db, constraints).await? {
            return Ok(found);
        }

        let mut implicit_constraints = Constraints::default();
        let mut query_items = Vec::new();

        let kubernetes_version = match constraints.kubernetes_version() {
            KubernetesVersion::Unknown => {
                let default = KubernetesVersion::default();
                implicit_constraints.set_kubernetes_version(default);
                default
            }
            value => value,
        };
        query_items.push(si_data::DataQueryItems::generate_expression_for_string(
            Field::KubernetesVersion.to_string(),
            si_data::DataQueryItemsExpressionComparison::Equals,
            kubernetes_version.to_string(),
        ));

        let component =
            Self::pick_by_expressions(db, query_items, si_data::DataQueryBooleanTerm::And).await?;

        Ok((implicit_constraints, component))
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
impl MigrateComponent for KubernetesDeploymentComponent {
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

        for kubernetes_version in KubernetesVersion::iterator() {
            let name = format!("AWS EKS Kubernetes {} Deployment", kubernetes_version);
            let mut c = Self {
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

// TODO(fnichol) Code gen this
impl KubernetesVersion {
    pub fn iterator() -> impl Iterator<Item = Self> {
        [Self::V112, Self::V113, Self::V114, Self::V115]
            .iter()
            .copied()
    }

    fn default() -> Self {
        Self::V115
    }
}

// TODO(fnichol) Code gen this
#[derive(thiserror::Error, Debug)]
#[error("invalid KubernetesVersion value: {0}")]
pub struct InvalidKubernetesVersionError(String);

// TODO(fnichol) Code gen this
impl FromStr for KubernetesVersion {
    type Err = InvalidKubernetesVersionError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let r = match s {
            "v1.12" => Self::V112,
            "v1.13" => Self::V113,
            "v1.14" => Self::V114,
            "v1.15" => Self::V115,
            invalid => return Err(InvalidKubernetesVersionError(invalid.to_string())),
        };
        Ok(r)
    }
}

// TODO(fnichol) Code gen this
impl fmt::Display for KubernetesVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            Self::Unknown => "<UNKNOWN>",
            Self::V112 => "v1.12",
            Self::V113 => "v1.13",
            Self::V114 => "v1.14",
            Self::V115 => "v1.15",
        };
        write!(f, "{}", msg)
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
