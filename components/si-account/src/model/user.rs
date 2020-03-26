use si_data::{error::DataError, Reference, Storable};
use sodiumoxide::crypto::pwhash::argon2id13;
use tracing::trace;
use uuid::Uuid;

pub use crate::error::{AccountError, Result};
pub use crate::protobuf::{Capability, CreateUserReply, CreateUserRequest, User};

impl Storable for User {
    fn get_id(&self) -> &str {
        &self.id
    }

    fn set_id(&mut self, id: impl Into<String>) {
        self.id = id.into();
    }

    fn type_name() -> &'static str {
        "user"
    }

    fn set_type_name(&mut self) {
        self.type_name = User::type_name().to_string();
    }

    fn generate_id(&mut self) {
        let uuid = Uuid::new_v4();
        self.id = format!("{}:{}", User::type_name(), uuid);
    }

    fn validate(&self) -> si_data::error::Result<()> {
        if self.email == "" {
            return Err(DataError::ValidationError(
                AccountError::InvalidMissingEmail.to_string(),
            ));
        }
        if self.domain == "" {
            return Err(DataError::ValidationError(
                AccountError::InvalidMissingDomain.to_string(),
            ));
        }
        if self.display_name == "" {
            return Err(DataError::ValidationError(
                AccountError::InvalidMissingDisplayName.to_string(),
            ));
        }
        if self.given_name == "" {
            return Err(DataError::ValidationError(
                AccountError::InvalidMissingGivenName.to_string(),
            ));
        }
        if self.family_name == "" {
            return Err(DataError::ValidationError(
                AccountError::InvalidMissingFamilyName.to_string(),
            ));
        }
        if self.billing_account_id == "" {
            return Err(DataError::ValidationError(
                AccountError::InvalidMissingBillingAccountId.to_string(),
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

    fn get_natural_key(&self) -> Option<&str> {
        Some(&self.natural_key)
    }

    fn set_natural_key(&mut self) {
        self.natural_key = format!(
            "{}:{}:{}",
            self.get_tenant_ids()[0],
            User::type_name(),
            self.email
        );
    }

    fn order_by_fields() -> Vec<&'static str> {
        vec!["id", "naturalKey", "typeName", "email", "domain"]
    }
}

impl User {
    pub fn new_from_create_request(req: &CreateUserRequest) -> Result<User> {
        if req.given_name == "" {
            return Err(AccountError::InvalidMissingGivenName);
        }
        if req.family_name == "" {
            return Err(AccountError::InvalidMissingFamilyName);
        }
        let display_name = if req.display_name == "" {
            req.given_name.clone()
        } else {
            req.display_name.clone()
        };
        if req.email == "" {
            return Err(AccountError::InvalidMissingEmail);
        }
        let domain = match req.email.rsplit("@").nth(0) {
            Some(domain) => domain,
            None => return Err(AccountError::InvalidEmail),
        };
        if req.password == "" {
            return Err(AccountError::InvalidMissingPassword);
        }

        // TODO: THIS SHOULD BE IN A BLOCKING CODE BLOCK; WHEN TONIC HITS
        // TOKIO 0.2.2, WE CAN FIX IT.
        let password_hash = argon2id13::pwhash(
            req.password.as_bytes(),
            argon2id13::OPSLIMIT_INTERACTIVE,
            argon2id13::MEMLIMIT_INTERACTIVE,
        )
        .map_err(|_| AccountError::PasswordHash)?;
        let password_hash_str = std::str::from_utf8(password_hash.as_ref())?;

        Ok(User {
            email: req.email.clone(),
            domain: domain.to_string(),
            given_name: req.given_name.clone(),
            family_name: req.family_name.clone(),
            display_name: display_name,
            password_hash: password_hash_str.to_string(),
            ..Default::default()
        })
    }

    #[tracing::instrument]
    pub fn verify_password(&self, password: &str) -> bool {
        let password_bytes = password.as_bytes();
        if let Some(argon_password) =
            argon2id13::HashedPassword::from_slice(self.password_hash.as_bytes())
        {
            if argon2id13::pwhash_verify(&argon_password, password_bytes) {
                trace!("correct password");
                true
            } else {
                trace!("incorrect password");
                false
            }
        } else {
            trace!("corrupt password hash in database");
            false
        }
    }
}
