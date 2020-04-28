use si_data::{Db, Storable};

use crate::error::Result;
use crate::protobuf::{
    Integration, IntegrationOptions, IntegrationOptionsOptionType, IntegrationService,
    IntegrationServiceSiProperties,
};

pub async fn migrate(db: &Db) -> Result<()> {
    let mut integrations = vec![
        (
            Integration {
                name: Some("global".to_string()),
                display_name: Some("Always On".to_string()),
                version: Some(1),
                ..Default::default()
            },
            vec![IntegrationService {
                name: Some("ssh_key".to_string()),
                display_name: Some("SSH Key".to_string()),
                version: Some(1),
                ..Default::default()
            }],
        ),
        (
            Integration {
                name: Some("aws".to_string()),
                display_name: Some("Amazon Web Services".to_string()),
                version: Some(4),
                options: vec![
                    IntegrationOptions {
                        name: Some("access_key".to_string()),
                        display_name: Some("Access Key".to_string()),
                        option_type: IntegrationOptionsOptionType::String as i32,
                    },
                    IntegrationOptions {
                        name: Some("secret_key".to_string()),
                        display_name: Some("Secret Key".to_string()),
                        option_type: IntegrationOptionsOptionType::Secret as i32,
                    },
                    IntegrationOptions {
                        name: Some("region".to_string()),
                        display_name: Some("region".to_string()),
                        option_type: IntegrationOptionsOptionType::String as i32,
                    },
                ],
                ..Default::default()
            },
            vec![
                IntegrationService {
                    name: Some("ec2".to_string()),
                    display_name: Some("EC2".to_string()),
                    version: Some(1),
                    ..Default::default()
                },
                IntegrationService {
                    name: Some("eks".to_string()),
                    display_name: Some("EKS".to_string()),
                    version: Some(1),
                    ..Default::default()
                },
                IntegrationService {
                    name: Some("eks_kubernetes".to_string()),
                    display_name: Some("EKS Kubernetes".to_string()),
                    version: Some(1),
                    ..Default::default()
                },
            ],
        ),
    ];

    for (item, services) in integrations.iter_mut() {
        item.add_to_tenant_ids("global".to_string());
        db.migrate(item).await?;
        for service in services.iter_mut() {
            service.add_to_tenant_ids(item.get_id().to_string());
            service
                .si_properties
                .replace(IntegrationServiceSiProperties {
                    integration_id: Some(item.get_id().to_string()),
                });
            db.migrate(service).await?;
        }
    }
    Ok(())
}
