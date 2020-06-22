use crate::protobuf::{
    KubernetesDeploymentComponentConstraints,
    KubernetesDeploymentComponentConstraintsKubernetesVersion,
};
use si_cea::component::prelude::*;

type Constraints = KubernetesDeploymentComponentConstraints;
type KubernetesVersion = KubernetesDeploymentComponentConstraintsKubernetesVersion;

pub use crate::protobuf::KubernetesDeploymentComponent;

impl KubernetesDeploymentComponent {
    pub async fn pick(
        db: &Db,
        raw_constraints: Option<Constraints>,
    ) -> CeaResult<(Constraints, Self)> {
        let span = tracing_pick_span!("kubernetes_deployment", &raw_constraints);
        async {
            let span = tracing::Span::current();
            match raw_constraints {
                None => {
                    let mut implicit_constraints = Constraints::default();
                    implicit_constraints.set_kubernetes_version(KubernetesVersion::default_value());

                    span.record(
                        "pick.implict_constraints",
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
                    span.record("pick.constraints", &tracing::field::debug(&constraints));

                    if let Some(found) = Self::pick_by_component_name(db, &constraints).await? {
                        span.record("pick.component", &tracing::field::debug(&found));
                        return Ok(found);
                    }
                    if let Some(found) =
                        Self::pick_by_component_display_name(db, &constraints).await?
                    {
                        span.record("pick.component", &tracing::field::debug(&found));
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
                                &tracing::field::debug(&implicit_constraints),
                            );
                            default
                        }
                        value => value,
                    };
                    query_items.push(si_data::DataQueryItems::generate_expression_for_int(
                        "constraints.kubernetesVersion",
                        DataQueryItemsExpressionComparison::Equals,
                        kubernetes_version.to_i32_string(),
                    ));

                    let component =
                        Self::pick_by_expressions(db, query_items, DataQueryBooleanTerm::And)
                            .await?;
                    span.record("pick.component", &tracing::field::debug(&component));

                    Ok((implicit_constraints, component))
                }
            }
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
