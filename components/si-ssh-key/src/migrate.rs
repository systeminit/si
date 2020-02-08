use si_account::{Integration, IntegrationService};
use si_data::{Db, Storable};

use crate::error::Result;
use crate::model::component::{Component, KeyFormat, KeyType};

pub async fn migrate(db: &Db) -> Result<()> {
    let key_types = [KeyType::Rsa, KeyType::Dsa, KeyType::Ecdsa, KeyType::Ed25519];
    let key_formats = [KeyFormat::Rfc4716, KeyFormat::Pkcs8, KeyFormat::Pem];

    // Global Integration SSH Keys
    let global_integration: Integration = db
        .lookup_by_natural_key("global:integration:global")
        .await?;

    let global_service_integration_id = format!(
        "global:{}:integration_service:ssh_key",
        global_integration.get_id()
    );

    let global_service_integration: IntegrationService = db
        .lookup_by_natural_key(global_service_integration_id)
        .await?;

    for key_type in key_types.iter() {
        let valid_bits: Vec<u32> = match key_type {
            KeyType::Rsa => vec![1024, 2048, 3072, 4096],
            KeyType::Dsa => vec![1024],
            KeyType::Ecdsa => vec![256, 384, 521],
            KeyType::Ed25519 => vec![256],
        };

        for key_format in key_formats.iter() {
            for bits in valid_bits.iter() {
                let mut name = "Global ".to_string();
                match key_type {
                    KeyType::Rsa => name.push_str("RSA "),
                    KeyType::Dsa => name.push_str("DSA "),
                    KeyType::Ecdsa => name.push_str("ECDSA "),
                    KeyType::Ed25519 => name.push_str("ED25519 "),
                };
                name.push_str(&format!("{}", bits));
                match key_format {
                    KeyFormat::Rfc4716 => name.push_str(" RFC4716"),
                    KeyFormat::Pkcs8 => name.push_str(" PKCS8"),
                    KeyFormat::Pem => name.push_str(" PEM"),
                };

                let mut c = Component {
                    display_name: name.clone(),
                    description: name.clone(),
                    key_type: *key_type as i32,
                    key_format: *key_format as i32,
                    bits: bits.clone(),
                    integration_id: global_integration.get_id().to_string(),
                    integration_service_id: global_service_integration.get_id().to_string(),
                    display_type_name: "SSH Key".to_string(),
                    name: name,
                    version: 3,
                    ..Default::default()
                };
                c.add_to_tenant_ids("global".to_string());
                c.add_to_tenant_ids(global_integration.get_id().to_string());
                c.add_to_tenant_ids(global_service_integration.get_id().to_string());

                db.migrate(&mut c).await?;
            }
        }
    }

    // AWS EC2 SSH Keys
    //  Should these be internal model calls? Pretty sure they def should.
    let aws_integration: Integration = db.lookup_by_natural_key("global:integration:aws").await?;

    let aws_ec2_integration_service_id = format!(
        "global:{}:integration_service:ec2",
        aws_integration.get_id()
    );

    let aws_ec2_integration_service: IntegrationService = db
        .lookup_by_natural_key(aws_ec2_integration_service_id)
        .await?;
    {
        let name = String::from("AWS RSA 2048 PEM");
        let mut c = Component {
            display_name: name.clone(),
            description: name.clone(),
            key_type: KeyType::Rsa as i32,
            key_format: KeyFormat::Pem as i32,
            bits: 2048,
            integration_id: aws_integration.get_id().to_string(),
            integration_service_id: aws_ec2_integration_service.get_id().to_string(),
            display_type_name: "SSH Key".to_string(),
            name: name,
            version: 1,
            ..Default::default()
        };
        c.add_to_tenant_ids("global".to_string());
        c.add_to_tenant_ids(aws_integration.get_id().to_string());
        c.add_to_tenant_ids(aws_ec2_integration_service.get_id().to_string());
        db.migrate(&mut c).await?;
    }

    Ok(())
}
