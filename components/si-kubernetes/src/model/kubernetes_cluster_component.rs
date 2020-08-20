use crate::protobuf::KubernetesClusterComponentConstraints;
use si_cea::component::prelude::*;

type Constraints = KubernetesClusterComponentConstraints;

pub use crate::protobuf::KubernetesClusterComponent;

impl KubernetesClusterComponent {
    pub async fn pick(db: &Db, constraints: Option<Constraints>) -> CeaResult<(Constraints, Self)> {
        let span = tracing_pick_span!("kubernetesCluster", &constraints);
        async {
            let span = Span::current();
            match constraints {
                None => {
                    let implicit_constraints = Constraints::default();

                    span.record(
                        "pick.implicit_constraints",
                        &field::debug(&implicit_constraints),
                    );

                    let query_items = Vec::new();

                    let component =
                        Self::pick_by_expressions(db, query_items, DataQueryBooleanTerm::And)
                            .await?;

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

                    let implicit_constraints = Constraints::default();
                    let query_items = Vec::new();

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
        let integration: Integration = db.lookup_by_natural_key("global:integration:aws").await?;

        let integration_service_id =
            format!("{}:integration_service:eks_kubernetes", integration.id()?);

        let integration_service: IntegrationService =
            db.lookup_by_natural_key(integration_service_id).await?;

        let name = "Kubernetes Cluster Component".to_string();
        let mut c = Self {
            name: Some(name.clone()),
            display_name: Some(name.clone()),
            description: Some(name.clone()),
            constraints: Some(Constraints::default()),
            si_properties: Some(si_cea::protobuf::ComponentSiProperties {
                version: Some(1),
                integration_id: integration.id.clone(),
                integration_service_id: integration_service.id.clone(),
            }),
            ..Default::default()
        };
        c.add_to_tenant_ids("global".to_string());
        c.add_to_tenant_ids(integration.id()?.to_string());
        c.add_to_tenant_ids(integration_service.id()?.to_string());

        db.migrate(&mut c).await?;

        Ok(())
    }
}
