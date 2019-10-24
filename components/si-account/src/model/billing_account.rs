use si_data::{error::DataError, Reference, Storable};
use uuid::Uuid;

pub use crate::error::{AccountError, Result};
pub use crate::protobuf::{BillingAccount, CreateBillingAccountReply, CreateBillingAccountRequest};

impl Storable for BillingAccount {
    fn get_id(&self) -> &str {
        &self.id
    }

    fn set_id<S: Into<String>>(&mut self, id: S) {
        self.id = id.into();
    }

    fn type_name() -> &'static str {
        "billing_account"
    }

    fn set_type_name(&mut self) {
        self.type_name = BillingAccount::type_name().to_string();
    }

    fn generate_id(&mut self) {
        let uuid = Uuid::new_v4();
        self.id = format!("{}:{}", BillingAccount::type_name(), uuid);
    }

    fn validate(&self) -> si_data::error::Result<()> {
        if self.display_name == "" {
            return Err(DataError::ValidationError(
                AccountError::InvalidMissingDisplayName.to_string(),
            ));
        }
        if self.short_name == "" {
            return Err(DataError::ValidationError(
                AccountError::InvalidMissingShortName.to_string(),
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
            BillingAccount::type_name(),
            self.short_name
        );
    }

    fn order_by_fields() -> Vec<&'static str> {
        vec!["id", "naturalKey", "typeName", "displayName", "shortName"]
    }
}

impl BillingAccount {
    pub fn new_from_create_request(req: &CreateBillingAccountRequest) -> Result<BillingAccount> {
        if req.short_name == "" {
            return Err(AccountError::InvalidMissingShortName);
        }
        let display_name = if req.display_name == "" {
            req.short_name.clone()
        } else {
            req.display_name.clone()
        };

        Ok(BillingAccount {
            short_name: req.short_name.clone(),
            display_name: display_name,
            ..Default::default()
        })
    }
}
