use crate::storable::Storable;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LookupObject {
    id: String,
    object_id: String,
    type_name: String,
    si_storable: Option<LookupSiStorable>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LookupSiStorable {
    tenant_ids: Vec<String>,
}

impl Storable for LookupObject {
    fn get_id(&self) -> &str {
        &self.id
    }

    fn set_id<S: Into<String>>(&mut self, id: S) {
        self.id = id.into();
    }

    fn type_name() -> &'static str {
        "lookup_object"
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

    fn get_tenant_ids(&self) -> Vec<String> {
        self.si_storable.unwrap().tenant_ids.clone()
    }

    fn add_to_tenant_ids(&mut self, id: String) {
        self.si_storable.unwrap().tenant_ids.push(id);
    }

    fn referential_fields(&self) -> Vec<Reference> {
        vec![Reference::HasOne(
            "billing_account_id",
            &self.billing_account_id,
        )]
    }

    fn order_by_fields() -> Vec<&'static str> {
        vec!["id", "naturalKey", "typeName", "email", "domain"]
    }
}
