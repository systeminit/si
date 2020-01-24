use si_data::{error::DataError, Migrateable, Reference, Storable};
use uuid::Uuid;

pub use crate::error::{AccountError, Result};
pub use crate::protobuf::{
    Integration, IntegrationOption, IntegrationOptionType, IntegrationService,
};

impl Storable for Integration {
    fn get_id(&self) -> &str {
        &self.id
    }

    fn set_id<S: Into<String>>(&mut self, id: S) {
        self.id = id.into();
    }

    fn type_name() -> &'static str {
        "integration"
    }

    fn set_type_name(&mut self) {
        self.type_name = Integration::type_name().to_string();
    }

    fn generate_id(&mut self) {
        let uuid = Uuid::new_v4();
        self.id = format!("{}:{}", Integration::type_name(), uuid);
    }

    fn validate(&self) -> si_data::error::Result<()> {
        if self.display_name == "" {
            return Err(DataError::ValidationError(
                AccountError::InvalidMissingDisplayName.to_string(),
            ));
        }
        if self.name == "" {
            return Err(DataError::ValidationError(
                AccountError::InvalidMissingName.to_string(),
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
        Vec::new()
    }

    fn get_natural_key(&self) -> Option<&str> {
        Some(&self.natural_key)
    }

    fn set_natural_key(&mut self) {
        self.natural_key = format!(
            "{}:{}:{}",
            // This is safe *only* after the object has been created.
            self.get_tenant_ids()[0],
            Integration::type_name(),
            self.name
        );
    }

    fn order_by_fields() -> Vec<&'static str> {
        vec!["id", "naturalKey", "typeName", "displayName", "name"]
    }
}

impl Migrateable for Integration {
    fn get_version(&self) -> i32 {
        self.version
    }
}

impl Storable for IntegrationService {
    fn get_id(&self) -> &str {
        &self.id
    }

    fn set_id<S: Into<String>>(&mut self, id: S) {
        self.id = id.into();
    }

    fn type_name() -> &'static str {
        "integration_service"
    }

    fn set_type_name(&mut self) {
        self.type_name = IntegrationService::type_name().to_string();
    }

    fn generate_id(&mut self) {
        let uuid = Uuid::new_v4();
        self.id = format!("{}:{}", IntegrationService::type_name(), uuid);
    }

    fn validate(&self) -> si_data::error::Result<()> {
        if self.display_name == "" {
            return Err(DataError::ValidationError(
                AccountError::InvalidMissingDisplayName.to_string(),
            ));
        }
        if self.name == "" {
            return Err(DataError::ValidationError(
                AccountError::InvalidMissingName.to_string(),
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
        Vec::new()
    }

    fn get_natural_key(&self) -> Option<&str> {
        Some(&self.natural_key)
    }

    fn set_natural_key(&mut self) {
        self.natural_key = format!(
            "{}:{}:{}:{}",
            // This is safe *only* after the object has been created.
            self.get_tenant_ids()[0],
            self.get_tenant_ids()[1],
            IntegrationService::type_name(),
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
            "integration_id",
        ]
    }
}

impl Migrateable for IntegrationService {
    fn get_version(&self) -> i32 {
        self.version
    }
}
