use si_data::{error::DataError, Reference, Storable};
use uuid::Uuid;

pub use crate::error::{AccountError, Result};
pub use crate::protobuf::Workspace;

impl Storable for Workspace {
    fn get_id(&self) -> &str {
        &self.id
    }

    fn set_id<S: Into<String>>(&mut self, id: S) {
        self.id = id.into();
    }

    fn type_name() -> &'static str {
        "workspace"
    }

    fn set_type_name(&mut self) {
        self.type_name = Workspace::type_name().to_string();
    }

    fn get_natural_key(&self) -> Option<&str> {
        Some(&self.natural_key)
    }

    fn set_natural_key(&mut self) {
        self.natural_key = format!(
            "{}:{}:{}:{}",
            self.get_tenant_ids()[0],
            self.get_tenant_ids()[1],
            Workspace::type_name(),
            self.name
        );
    }

    fn generate_id(&mut self) {
        let uuid = Uuid::new_v4();
        self.id = format!("{}:{}", Workspace::type_name(), uuid);
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

    fn add_to_tenant_ids(&mut self, id: String) {
        self.tenant_ids.push(id);
    }

    fn referential_fields(&self) -> Vec<Reference> {
        vec![
            Reference::HasOne("billing_account_id", &self.billing_account_id),
            Reference::HasOne("organization_id", &self.organization_id),
        ]
    }

    fn order_by_fields() -> Vec<&'static str> {
        vec!["id", "naturalKey", "typeName", "name"]
    }
}

impl Workspace {
    pub fn new(name: String, billing_account_id: String, organization_id: String) -> Workspace {
        Workspace {
            name,
            billing_account_id,
            organization_id,
            ..Default::default()
        }
    }
}
