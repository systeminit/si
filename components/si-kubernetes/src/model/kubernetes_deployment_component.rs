use crate::protobuf::{
    KubernetesDeploymentComponentConstraints,
    KubernetesDeploymentComponentConstraintsKubernetesVersion,
};
use si_cea::component::prelude::*;
use std::fmt;
use std::str::FromStr;

use tracing::{self, debug, span, Level};
use tracing_futures::Instrument as _;

pub use crate::protobuf::KubernetesDeploymentComponent;

type Constraints = KubernetesDeploymentComponentConstraints;
type KubernetesVersion = KubernetesDeploymentComponentConstraintsKubernetesVersion;

impl KubernetesDeploymentComponent {
    pub async fn pick(
        db: &Db,
        raw_constraints: Option<Constraints>,
    ) -> CeaResult<(Constraints, Self)> {
        let span = span!(
            Level::DEBUG,
            "kubernetes_deployment_pick",
            pick.raw_constraints = tracing::field::debug(&raw_constraints),
            pick.implicit_constraints = tracing::field::Empty,
            pick.constraints = tracing::field::Empty,
            pick.component_name = tracing::field::display("kubernetes_deployment_component"),
            pick.component = tracing::field::Empty,
        );
        async {
            let span = tracing::Span::current();
            if raw_constraints.is_none() {
                let mut implicit_constraints = Constraints::default();
                implicit_constraints.set_kubernetes_version(KubernetesVersion::V115);

                span.record(
                    "pick.implict_constraints",
                    &tracing::field::debug(&implicit_constraints),
                );

                let mut query_items = Vec::new();
                query_items.push(si_data::DataQueryItems::generate_expression_for_int(
                    "constraints.kubernetesVersion",
                    si_data::DataQueryItemsExpressionComparison::Equals,
                    (KubernetesVersion::V115 as i32).to_string(),
                ));

                let component =
                    Self::pick_by_expressions(db, query_items, si_data::DataQueryBooleanTerm::And)
                        .await?;

                span.record("pick.component", &tracing::field::debug(&component));

                return Ok((implicit_constraints, component));
            }
            let constraints = raw_constraints.unwrap();
            span.record("pick.constraints", &tracing::field::debug(&constraints));

            if let Some(found) = Self::pick_by_component_name(db, &constraints).await? {
                span.record("pick.component", &tracing::field::debug(&found));
                return Ok(found);
            }
            if let Some(found) = Self::pick_by_component_display_name(db, &constraints).await? {
                span.record("pick.component", &tracing::field::debug(&found));
                return Ok(found);
            }

            let mut implicit_constraints = Constraints::default();
            let mut query_items = Vec::new();

            let kubernetes_version = match constraints.kubernetes_version() {
                KubernetesVersion::Unknown => {
                    let default = KubernetesVersion::default();
                    implicit_constraints.set_kubernetes_version(default);
                    span.record(
                        "pick.implicit_constraints",
                        &tracing::field::debug(&implicit_constraints),
                    );
                    default
                }
                value => value,
            };
            query_items.push(si_data::DataQueryItems::generate_expression_for_int(
                "constraints.kubernetesVersion",
                si_data::DataQueryItemsExpressionComparison::Equals,
                (kubernetes_version as i32).to_string(),
            ));

            let component =
                Self::pick_by_expressions(db, query_items, si_data::DataQueryBooleanTerm::And)
                    .await?;
            span.record("pick.component", &tracing::field::debug(&component));

            Ok((implicit_constraints, component))
        }
        .instrument(span)
        .await
    }

    pub async fn migrate(db: &Db) -> DataResult<()> {
        debug!("aws_integration");
        // Should these be internal model calls? Pretty sure they def should.
        let aws_integration: Integration =
            db.lookup_by_natural_key("global:integration:aws").await?;

        let aws_eks_integration_service_id = format!(
            "{}:integration_service:eks_kubernetes",
            aws_integration.id()?
        );
        debug!(
            ?aws_eks_integration_service_id,
            "aws integration service eks_kubernetes",
        );

        let aws_eks_integration_service: IntegrationService = db
            .lookup_by_natural_key(aws_eks_integration_service_id)
            .await?;

        debug!("poop");
        debug!(
            ?aws_eks_integration_service,
            "aws integration service eks_kubernetes lookup passed",
        );

        for kubernetes_version in KubernetesVersion::iterator() {
            let name = format!("AWS EKS Kubernetes {} Deployment", kubernetes_version);
            let mut c = Self {
                name: Some(name.clone()),
                display_name: Some(name.clone()),
                description: Some(name.clone()),
                constraints: Some(KubernetesDeploymentComponentConstraints {
                    kubernetes_version: kubernetes_version as i32,
                    ..Default::default()
                }),
                si_properties: Some(si_cea::protobuf::ComponentSiProperties {
                    version: Some(1),
                    integration_id: aws_integration.id.clone(),
                    integration_service_id: aws_eks_integration_service.id.clone(),
                }),
                ..Default::default()
            };
            c.add_to_tenant_ids("global".to_string());
            c.add_to_tenant_ids(aws_integration.id()?.to_string());
            c.add_to_tenant_ids(aws_eks_integration_service.id()?.to_string());
            debug!(?c, "component migration",);

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
