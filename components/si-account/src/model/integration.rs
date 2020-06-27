use crate::protobuf::{
    Integration, IntegrationOptions, IntegrationOptionsOptionType, IntegrationService,
    IntegrationServiceSiProperties, IntegrationSiProperties,
};
use si_data::{Db, Storable};

impl Integration {
    pub async fn migrate(db: &Db) -> si_data::Result<()> {
        let mut integrations = vec![
            (
                Integration {
                    name: Some("global".to_string()),
                    display_name: Some("Always On".to_string()),
                    si_properties: Some(IntegrationSiProperties { version: Some(1) }),
                    ..Default::default()
                },
                vec![
                    IntegrationService {
                        name: Some("core".to_string()),
                        display_name: Some("Core".to_string()),
                        si_properties: Some(IntegrationServiceSiProperties {
                            version: Some(1),
                            ..Default::default()
                        }),
                        ..Default::default()
                    },
                    IntegrationService {
                        name: Some("ssh_key".to_string()),
                        display_name: Some("SSH Key".to_string()),
                        si_properties: Some(IntegrationServiceSiProperties {
                            version: Some(1),
                            ..Default::default()
                        }),
                        ..Default::default()
                    },
                ],
            ),
            (
                Integration {
                    name: Some("aws".to_string()),
                    display_name: Some("Amazon Web Services".to_string()),
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
                    si_properties: Some(IntegrationSiProperties { version: Some(1) }),
                    ..Default::default()
                },
                vec![
                    IntegrationService {
                        name: Some("ec2".to_string()),
                        display_name: Some("EC2".to_string()),
                        si_properties: Some(IntegrationServiceSiProperties {
                            version: Some(1),
                            ..Default::default()
                        }),
                        ..Default::default()
                    },
                    IntegrationService {
                        name: Some("eks".to_string()),
                        display_name: Some("EKS".to_string()),
                        si_properties: Some(IntegrationServiceSiProperties {
                            version: Some(1),
                            ..Default::default()
                        }),
                        ..Default::default()
                    },
                    IntegrationService {
                        name: Some("eks_kubernetes".to_string()),
                        display_name: Some("EKS Kubernetes".to_string()),
                        si_properties: Some(IntegrationServiceSiProperties {
                            version: Some(1),
                            ..Default::default()
                        }),
                        ..Default::default()
                    },
                ],
            ),
        ];

        for (item, services) in integrations.iter_mut() {
            item.add_to_tenant_ids("global".to_string());
            db.migrate(item).await?;
            for service in services.iter_mut() {
                let item_id = item.id()?;
                service.add_to_tenant_ids(item_id.to_string());
                service
                    .si_properties
                    .as_mut()
                    .map(|p| p.integration_id = Some(item_id.to_string()));
                db.migrate(service).await?;
            }
        }
        Ok(())
    }
}
