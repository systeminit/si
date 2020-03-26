use si_data::{Db, Storable};

use crate::error::Result;
use crate::model::integration::{
    Integration, IntegrationOption, IntegrationOptionType, IntegrationService,
};

pub async fn migrate(db: &Db) -> Result<()> {
    let mut integrations = vec![
        (
            Integration {
                name: "global".to_string(),
                display_name: "Always On".to_string(),
                tenant_ids: vec!["global".to_string()],
                version: 1,
                ..Default::default()
            },
            vec![IntegrationService {
                name: "ssh_key".to_string(),
                display_name: "SSH Key".to_string(),
                tenant_ids: vec!["global".to_string()],
                version: 1,
                ..Default::default()
            }],
        ),
        (
            Integration {
                name: "aws".to_string(),
                display_name: "Amazon Web Services".to_string(),
                tenant_ids: vec!["global".to_string()],
                version: 4,
                integration_options: vec![
                    IntegrationOption {
                        name: "access_key".to_string(),
                        display_name: "Access Key".to_string(),
                        option_type: IntegrationOptionType::String as i32,
                    },
                    IntegrationOption {
                        name: "secret_key".to_string(),
                        display_name: "Secret Key".to_string(),
                        option_type: IntegrationOptionType::Secret as i32,
                    },
                    IntegrationOption {
                        name: "region".to_string(),
                        display_name: "region".to_string(),
                        option_type: IntegrationOptionType::String as i32,
                    },
                ],
                ..Default::default()
            },
            vec![
                IntegrationService {
                    name: "ec2".to_string(),
                    display_name: "EC2".to_string(),
                    tenant_ids: vec!["global".to_string()],
                    version: 1,
                    ..Default::default()
                },
                IntegrationService {
                    name: "eks".to_string(),
                    display_name: "EKS".to_string(),
                    tenant_ids: vec!["global".to_string()],
                    version: 1,
                    ..Default::default()
                },
            ],
        ),
    ];

    for (item, services) in integrations.iter_mut() {
        db.migrate(item).await?;
        for service in services.iter_mut() {
            service.add_to_tenant_ids(item.get_id().to_string());
            service.integration_id = item.get_id().to_string();
            db.migrate(service).await?;
        }
    }
    Ok(())
}
