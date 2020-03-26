use si_data::{error::DataError, Reference, Storable};
use uuid::Uuid;

pub use crate::error::{AccountError, Result};
pub use crate::protobuf::{
    CreateIntegrationInstanceRequest, IntegrationInstance, IntegrationOptionType,
    IntegrationOptionValue,
};

impl Storable for IntegrationInstance {
    fn get_id(&self) -> &str {
        &self.id
    }

    fn set_id(&mut self, id: impl Into<String>) {
        self.id = id.into();
    }

    fn type_name() -> &'static str {
        "integration_instance"
    }

    fn set_type_name(&mut self) {
        self.type_name = IntegrationInstance::type_name().to_string();
    }

    fn generate_id(&mut self) {
        let uuid = Uuid::new_v4();
        self.id = format!("{}:{}", IntegrationInstance::type_name(), uuid);
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

    fn add_to_tenant_ids(&mut self, id: impl Into<String>) {
        self.tenant_ids.push(id.into());
    }

    fn referential_fields(&self) -> Vec<Reference> {
        vec![
            Reference::HasOne("billing_account_id", &self.billing_account_id),
            Reference::HasOne("integration_id", &self.integration_id),
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
            IntegrationInstance::type_name(),
            self.name
        );
    }

    fn order_by_fields() -> Vec<&'static str> {
        vec!["id", "naturalKey", "typeName", "displayName", "name"]
    }
}

impl IntegrationInstance {
    pub fn new_from_create_request(
        req: &CreateIntegrationInstanceRequest,
    ) -> Result<IntegrationInstance> {
        if req.integration_id == "" {
            return Err(AccountError::InvalidMissingIntegrationId);
        }
        if req.name == "" {
            return Err(AccountError::InvalidMissingName);
        }
        if req.display_name == "" {
            return Err(AccountError::InvalidMissingDisplayName);
        }
        if req.integration_option_values.len() == 0 {
            return Err(AccountError::InvalidMissingIntegrationOptionValues);
        }
        let mut integration_option_values = vec![];
        for iov in req.integration_option_values.iter() {
            integration_option_values.push(IntegrationOptionValue {
                // I suppose we don't have to allocate here - but, whatever. Penny wise
                // pound foolish.
                name: iov.name.to_string(),
                value: iov.value.to_string(),
                option_type: iov.option_type,
            });
        }
        Ok(IntegrationInstance {
            name: req.name.to_string(),
            display_name: req.name.to_string(),
            integration_id: req.integration_id.to_string(),
            integration_option_values,
            ..Default::default()
        })
    }
}
