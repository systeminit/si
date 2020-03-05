use si_data::{error::DataError, Reference, Storable};
use uuid::Uuid;

pub use crate::error::{AccountError, Result};
pub use crate::protobuf::Organization;

impl Storable for Organization {
    fn get_id(&self) -> &str {
        &self.id
    }

    fn set_id(&mut self, id: impl Into<String>) {
        self.id = id.into();
    }

    fn type_name() -> &'static str {
        "organization"
    }

    fn set_type_name(&mut self) {
        self.type_name = Organization::type_name().to_string();
    }

    fn get_natural_key(&self) -> Option<&str> {
        Some(&self.natural_key)
    }

    fn set_natural_key(&mut self) {
        self.natural_key = format!(
            "{}:{}:{}",
            self.get_tenant_ids()[0],
            Organization::type_name(),
            self.name
        );
    }

    fn generate_id(&mut self) {
        let uuid = Uuid::new_v4();
        self.id = format!("{}:{}", Organization::type_name(), uuid);
    }

    fn validate(&self) -> si_data::error::Result<()> {
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
        vec![Reference::HasOne(
            "billing_account_id",
            &self.billing_account_id,
        )]
    }

    fn order_by_fields() -> Vec<&'static str> {
        vec!["id", "naturalKey", "typeName", "name"]
    }
}

impl Organization {
    pub fn new(name: String, billing_account_id: String) -> Organization {
        Organization {
            name,
            billing_account_id,
            ..Default::default()
        }
    }
}
