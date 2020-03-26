use si_data::{error::DataError, Reference, Storable};
use uuid::Uuid;

pub use crate::error::{AccountError, Result};
pub use crate::protobuf::{Capability, Group};

impl Storable for Group {
    fn get_id(&self) -> &str {
        &self.id
    }

    fn set_id(&mut self, id: impl Into<String>) {
        self.id = id.into();
    }

    fn type_name() -> &'static str {
        "group"
    }

    fn set_type_name(&mut self) {
        self.type_name = Group::type_name().to_string();
    }

    fn get_natural_key(&self) -> Option<&str> {
        Some(&self.natural_key)
    }

    fn set_natural_key(&mut self) {
        self.natural_key = format!(
            "{}:{}:{}",
            self.get_tenant_ids()[0],
            Group::type_name(),
            self.name
        );
    }

    fn generate_id(&mut self) {
        let uuid = Uuid::new_v4();
        self.id = format!("{}:{}", Group::type_name(), uuid);
    }

    fn validate(&self) -> si_data::error::Result<()> {
        if self.name == "" {
            return Err(DataError::ValidationError(
                AccountError::InvalidMissingName.to_string(),
            ));
        }
        if self.display_name == "" {
            return Err(DataError::ValidationError(
                AccountError::InvalidMissingDisplayName.to_string(),
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
        vec![Reference::HasMany("user_ids", self.user_ids.clone())]
    }

    fn order_by_fields() -> Vec<&'static str> {
        vec!["id", "naturalKey", "typeName", "name", "displayName"]
    }
}

impl Group {
    pub fn new(name: String, display_name: String) -> Group {
        Group {
            name: name,
            display_name: display_name,
            ..Default::default()
        }
    }

    pub fn add_user(&mut self, id: String) {
        if !self.user_ids.contains(&id) {
            self.user_ids.push(id);
        }
    }

    pub fn add_capability(&mut self, subject: String, actions: Vec<String>) {
        let cap = Capability {
            subject: subject,
            actions: actions,
        };
        self.capabilities.push(cap);
    }
}
