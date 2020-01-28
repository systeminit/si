use si_data::{error::DataError, Migrateable, Reference, Storable};
use uuid::Uuid;

use crate::error::SshKeyError;
pub use crate::protobuf::{Component, KeyFormat, KeyType};
use crate::protobuf::{PickComponentReply, PickComponentRequest};

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

impl Storable for Component {
    fn get_id(&self) -> &str {
        &self.id
    }

    fn set_id<S: Into<String>>(&mut self, id: S) {
        self.id = id.into();
    }

    fn type_name() -> &'static str {
        "ssh_key"
    }

    fn set_type_name(&mut self) {
        self.type_name = Component::type_name().to_string();
    }

    fn generate_id(&mut self) {
        let uuid = Uuid::new_v4();
        self.id = format!("{}:{}", Component::type_name(), uuid);
    }

    fn validate(&self) -> si_data::error::Result<()> {
        if self.display_name == "" {
            return Err(DataError::ValidationError(
                SshKeyError::InvalidMissingDisplayName.to_string(),
            ));
        }
        if self.name == "" {
            return Err(DataError::ValidationError(
                SshKeyError::InvalidMissingName.to_string(),
            ));
        }
        Ok(())
    }

    fn get_tenant_ids(&self) -> &[String] {
        &self.tenant_ids
    }

    fn add_to_tenant_ids(&mut self, id: String) {
        self.tenant_ids.push(id);
    }

    fn referential_fields(&self) -> Vec<Reference> {
        vec![
            Reference::HasOne("integration_id", &self.integration_id),
            Reference::HasOne("integration_service_id", &self.integration_service_id),
        ]
    }

    fn get_natural_key(&self) -> Option<&str> {
        Some(&self.natural_key)
    }

    fn set_natural_key(&mut self) {
        self.natural_key = format!(
            "{}:{}:{}",
            // This is safe *only* after the object has been created.
            self.get_tenant_ids()[0],
            Component::type_name(),
            self.name
        );
    }

    fn order_by_fields() -> Vec<&'static str> {
        vec![
            "id",
            "naturalKey",
            "typeName",
            "displayName",
            "name",
            "description",
            "displayTypeName",
            "bits",
            "keyType",
            "keyFormat",
        ]
    }
}

impl Migrateable for Component {
    fn get_version(&self) -> i32 {
        self.version
    }
}
