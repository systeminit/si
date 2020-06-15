use crate::protobuf::{
    KubernetesServiceComponentConstraints, KubernetesServiceComponentConstraintsKubernetesVersion,
};
use si_cea::component::prelude::*;

type Constraints = KubernetesServiceComponentConstraints;
type KubernetesVersion = KubernetesServiceComponentConstraintsKubernetesVersion;

pub use crate::protobuf::KubernetesServiceComponent;

impl KubernetesServiceComponent {
    pub async fn pick(
        db: &Db,
        raw_constraints: Option<Constraints>,
    ) -> CeaResult<(Constraints, Self)> {
        let span = tracing_pick_span!("kubernetes_service", &raw_constraints);
        async {
            let span = Span::current();
            match raw_constraints {
                None => {
                    let mut implicit_constraints = Constraints::default();
                    implicit_constraints.set_kubernetes_version(KubernetesVersion::default_value());

                    span.record(
                        "pick.implicit_constraints",
                        &field::debug(&implicit_constraints),
                    );

                    let mut query_items = Vec::new();
                    query_items.push(DataQueryItems::generate_expression_for_int(
                        "constraints.kubernetesVersion",
                        DataQueryItemsExpressionComparison::Equals,
                        KubernetesVersion::default_value().to_i32_string(),
                    ));

                    let component =
                        Self::pick_by_expressions(db, query_items, DataQueryBooleanTerm::And)
                            .await?;

                    span.record("pick.component", &field::debug(&component));

                    Ok((implicit_constraints, component))
                }
                Some(constraints) => {
                    span.record("pick.constraints", &field::debug(&constraints));

                    if let Some(found) = Self::pick_by_component_name(db, &constraints).await? {
                        span.record("pick.component", &field::debug(&found));
                        return Ok(found);
                    }
                    if let Some(found) =
                        Self::pick_by_component_display_name(db, &constraints).await?
                    {
                        span.record("pick.component", &field::debug(&found));
                        return Ok(found);
                    }

                    let mut implicit_constraints = Constraints::default();
                    let mut query_items = Vec::new();

                    let kubernetes_version = match constraints.kubernetes_version() {
                        KubernetesVersion::Unknown => {
                            let default = KubernetesVersion::default_value();
                            implicit_constraints.set_kubernetes_version(default);
                            span.record(
                                "pick.implicit_constraints",
                                &field::debug(&implicit_constraints),
                            );
                            default
                        }
                        value => value,
                    };
                    query_items.push(DataQueryItems::generate_expression_for_int(
                        "constraints.kubernetesVersion",
                        DataQueryItemsExpressionComparison::Equals,
                        kubernetes_version.to_i32_string(),
                    ));

                    let component =
                        Self::pick_by_expressions(db, query_items, DataQueryBooleanTerm::And)
                            .await?;
                    span.record("pick.component", &field::debug(&component));

                    Ok((implicit_constraints, component))
                }
            }
        }
        .instrument(span)
        .await
    }

    pub async fn migrate(db: &Db) -> DataResult<()> {
        // Should these be internal model calls? Pretty sure they def should.
        let aws_integration: Integration =
            db.lookup_by_natural_key("global:integration:aws").await?;

        let aws_eks_integration_service_id = format!(
            "{}:integration_service:eks_kubernetes",
            aws_integration.id()?
        );

        let aws_eks_integration_service: IntegrationService = db
            .lookup_by_natural_key(aws_eks_integration_service_id)
            .await?;

        for kubernetes_version in KubernetesVersion::iterator() {
            let name = format!("AWS EKS Kubernetes {} Service", kubernetes_version);
            let mut c = Self {
                name: Some(name.clone()),
                display_name: Some(name.clone()),
                description: Some(name.clone()),
                constraints: Some(KubernetesServiceComponentConstraints {
                    kubernetes_version: kubernetes_version.into(),
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

            db.migrate(&mut c).await?;
        }

        Ok(())
    }
}

// TODO(fnichol) Code gen this
impl KubernetesServiceComponentConstraintsKubernetesVersion {
    pub fn iterator() -> impl Iterator<Item = Self> {
        [Self::V112, Self::V113, Self::V114, Self::V115]
            .iter()
            .copied()
    }

    fn default_value() -> Self {
        Self::V115
    }

    fn to_i32_string(&self) -> String {
        (*self as i32).to_string()
    }
}

// TODO(fnichol) Code gen this
impl std::fmt::Display for KubernetesServiceComponentConstraintsKubernetesVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
