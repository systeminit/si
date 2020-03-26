use si_account::{Integration, IntegrationService};
use si_data::{Db, Storable};

use crate::error::Result;
use crate::model::component::Component;

pub async fn migrate(db: &Db) -> Result<()> {
    // AWS EC2 SSH Keys
    //  Should these be internal model calls? Pretty sure they def should.
    let aws_integration: Integration = db.lookup_by_natural_key("global:integration:aws").await?;

    let aws_eks_integration_service_id = format!(
        "global:{}:integration_service:eks",
        aws_integration.get_id()
    );

    let aws_eks_integration_service: IntegrationService = db
        .lookup_by_natural_key(aws_eks_integration_service_id)
        .await?;

    let kubernetes_versions = vec!["1.12", "1.13", "1.14", "1.15"];
    for kubernetes_version in kubernetes_versions {
        let name = format!("AWS EKS Cluster {} Runtime", kubernetes_version);
        let mut c = Component {
            display_name: name.clone(),
            description: name.clone(),
            integration_id: aws_integration.get_id().to_string(),
            integration_service_id: aws_eks_integration_service.get_id().to_string(),
            display_type_name: "AWS EKS ".to_string(),
            name: name,
            version: 1,
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
