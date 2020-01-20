use si_data::db::Db;

use crate::error::Result;
use crate::model::integration::{Integration, IntegrationOption, IntegrationOptionType};

pub async fn migrate(db: &Db) -> Result<()> {
    let mut integrations = vec![
        Integration {
            name: "global".to_string(),
            display_name: "Always On".to_string(),
            tenant_ids: vec!["global".to_string()],
            version: 1,
            ..Default::default()
        },
        Integration {
            name: "aws".to_string(),
            display_name: "Amazon Web Services".to_string(),
            tenant_ids: vec!["global".to_string()],
            version: 2,
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
            ],
            ..Default::default()
        },
    ];
    for item in integrations.iter_mut() {
        db.migrate(item).await?;
    }
    Ok(())
}
