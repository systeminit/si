use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::data::{Connection, Db};
use crate::models::{
    check_secondary_key, generate_id, get_model, insert_model, ModelError, SiStorableError,
    SimpleStorable,
};

#[derive(Error, Debug)]
pub enum OrganizationError {
    #[error("an organization already exists with this name")]
    NameExists,
    #[error("si_storable error: {0}")]
    SiStorable(#[from] SiStorableError),
    #[error("error in core model functions: {0}")]
    Model(#[from] ModelError),
}

pub type OrganizationResult<T> = Result<T, OrganizationError>;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Organization {
    pub id: String,
    pub name: String,
    pub si_storable: SimpleStorable,
}

impl Organization {
    pub async fn new(
        db: &Db,
        nats: &Connection,
        name: impl Into<String>,
        billing_account_id: impl Into<String>,
    ) -> OrganizationResult<Organization> {
        let name = name.into();
        let billing_account_id = billing_account_id.into();

        if check_secondary_key(db, &billing_account_id, "organization", "name", &name).await? {
            return Err(OrganizationError::NameExists);
        }

        let id = generate_id("organization");
        let si_storable = SimpleStorable::new(&id, "organization", &billing_account_id);
        let object = Organization {
            id,
            name,
            si_storable,
        };
        insert_model(db, nats, &object.id, &object).await?;
        Ok(object)
    }

    pub async fn get(
        db: &Db,
        organization_id: impl AsRef<str>,
        billing_account_id: impl AsRef<str>,
    ) -> OrganizationResult<Organization> {
        let id = organization_id.as_ref();
        let billing_account_id = billing_account_id.as_ref();
        let object: Organization = get_model(db, id, billing_account_id).await?;
        Ok(object)
    }
}
