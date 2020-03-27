pub use crate::protobuf::{
    Component, ImplicitConstraint, KeyFormat, KeyType, ListComponentsReply, ListComponentsRequest,
    PickComponentReply, PickComponentRequest,
};
use si_cea::component::prelude::*;
use si_data::Db;

impl std::fmt::Display for KeyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            &KeyType::Rsa => "RSA".to_string(),
            &KeyType::Dsa => "DSA".to_string(),
            &KeyType::Ecdsa => "ECDSA".to_string(),
            &KeyType::Ed25519 => "ED25519".to_string(),
        };
        write!(f, "{}", msg)
    }
}

impl std::fmt::Display for KeyFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            &KeyFormat::Rfc4716 => "RFC4716".to_string(),
            &KeyFormat::Pkcs8 => "PKCS8".to_string(),
            &KeyFormat::Pem => "PEM".to_string(),
        };
        write!(f, "{}", msg)
    }
}

gen_component!(
    type_name: "ssh_key",
    order_by_fields: [
        "bits",
        "keyType",
        "keyFormat"
    ],
    validate_fn: |self| {
        if self.display_name == "" {
            return Err(DataError::ValidationError("missing display name".to_string()));
        }
        if self.name == "" {
            return Err(DataError::ValidationError("missing short name".to_string()));
        }
        Ok(())
    }
);

impl Component {
    pub async fn pick(
        db: &Db,
        req: &PickComponentRequest,
    ) -> CeaResult<(ImplicitConstraints, Component)> {
        let mut implicit_constraints = ImplicitConstraints::new();

        if let Some(result) =
            Component::pick_by_string_field(db, "name".to_string(), req.name.clone()).await?
        {
            return Ok((implicit_constraints, result));
        }
        if let Some(result) =
            Component::pick_by_string_field(db, "displayName".to_string(), req.display_name.clone())
                .await?
        {
            return Ok((implicit_constraints, result));
        }

        let key_type: KeyType; // = KeyType::Rsa;
        let key_format: KeyFormat; // = KeyFormat::Rfc4716;
        let bits: u32; // = 2048;

        // Means you have some kind of a type provided as a constraint
        if req.key_type == 0 {
            key_type = KeyType::Rsa;
            implicit_constraints.add("keyType", key_type.to_string());
        } else {
            key_type = match req.key_type {
                0 => unreachable!("You cannot get here"),
                1 => KeyType::Rsa,
                2 => KeyType::Dsa,
                3 => KeyType::Ecdsa,
                4 => KeyType::Ed25519,
                _ => return Err(CeaError::PickComponent("key type is invalid".to_string())),
            };
        }

        // THEN SOLVE FOR BITS

        // If you didn't supply bits, we pick the right number of bits
        // on your behalf
        if req.bits == 0 {
            bits = match key_type {
                KeyType::Rsa => 2048,
                KeyType::Dsa => 1024,
                // No idea if this is right, but lets go for bigger. Because,
                // you know... better.
                KeyType::Ecdsa => 521,
                KeyType::Ed25519 => 256,
            };
            implicit_constraints.add("bits", bits.to_string());
        } else {
            // You provided me bits, and I need to check that the bits are valid
            // for your key_type.
            bits = match key_type {
                KeyType::Rsa => match req.bits {
                    1024 | 2048 | 3072 | 4096 => req.bits,
                    value => {
                        return Err(CeaError::PickComponent(format!(
                            "invalid bits ({}) for keyType: {}",
                            value, key_type
                        )));
                    }
                },
                KeyType::Dsa => match req.bits {
                    1024 => req.bits,
                    value => {
                        return Err(CeaError::PickComponent(format!(
                            "invalid bits ({}) for keyType: {}",
                            value, key_type
                        )));
                    }
                },
                KeyType::Ecdsa => match req.bits {
                    256 | 384 | 521 => req.bits,
                    value => {
                        return Err(CeaError::PickComponent(format!(
                            "invalid bits ({}) for keyType: {}",
                            value, key_type
                        )));
                    }
                },
                KeyType::Ed25519 => match req.bits {
                    256 => req.bits,
                    value => {
                        return Err(CeaError::PickComponent(format!(
                            "invalid bits ({}) for keyType: {}",
                            value, key_type
                        )));
                    }
                },
            };
        }

        // SOLVE FOR FORMAT
        if req.key_format == 0 {
            key_format = KeyFormat::Rfc4716;
            implicit_constraints.add("keyFormat", key_format.to_string());
        } else {
            key_format = match req.key_format {
                0 => unreachable!("You cannot get here"),
                1 => KeyFormat::Rfc4716,
                2 => KeyFormat::Pkcs8,
                3 => KeyFormat::Pem,
                _ => {
                    return Err(CeaError::PickComponent(
                        "keyFormat is not valid".to_string(),
                    ));
                }
            };
        }

        let query_expressions = vec![
            si_data::Query::generate_expression_for_string(
                "keyType",
                si_data::QueryComparison::Equals,
                key_type.to_string(),
            ),
            si_data::Query::generate_expression_for_string(
                "keyFormat",
                si_data::QueryComparison::Equals,
                key_format.to_string(),
            ),
            si_data::Query::generate_expression_for_int(
                "bits",
                si_data::QueryComparison::Equals,
                bits.to_string(),
            ),
        ];
        let component =
            Component::pick_by_expressions(db, query_expressions, si_data::QueryBooleanLogic::And)
                .await?;
        Ok((implicit_constraints, component))
    }
}

#[async_trait::async_trait]
impl MigrateComponent for Component {
    async fn migrate(db: &Db) -> CeaResult<()> {
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
                        name,
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
        let aws_integration: Integration =
            db.lookup_by_natural_key("global:integration:aws").await?;

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
                name,
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
}
