use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::data::Db;
use crate::models::{generate_id, UpdateClock, UpdateClockError};

#[derive(Error, Debug)]
pub enum SiStorableError {
    #[error("update count error: {0}")]
    UpdateCount(#[from] UpdateClockError),
}

pub type SiStorableResult<T> = Result<T, SiStorableError>;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SiStorable {
    pub type_name: String,
    pub object_id: String,
    pub billing_account_id: String,
    pub organization_id: String,
    pub workspace_id: String,
    pub tenant_ids: Vec<String>,
    pub created_by_user_id: Option<String>,
    pub update_clock: UpdateClock,
    pub deleted: bool,
}

impl SiStorable {
    pub async fn new(
        db: &Db,
        type_name: impl Into<String>,
        billing_account_id: impl Into<String>,
        organization_id: impl Into<String>,
        workspace_id: impl Into<String>,
        created_by_user_id: Option<impl Into<String>>,
    ) -> SiStorableResult<SiStorable> {
        let type_name = type_name.into();
        let billing_account_id = billing_account_id.into();
        let organization_id = organization_id.into();
        let workspace_id = workspace_id.into();
        let created_by_user_id = created_by_user_id.map(|u| u.into());
        let object_id = generate_id(&type_name);
        let tenant_ids = vec![
            billing_account_id.clone(),
            organization_id.clone(),
            workspace_id.clone(),
            object_id.clone(),
        ];
        let update_clock = UpdateClock::create_or_update(db, &workspace_id, 0).await?;
        Ok(SiStorable {
            type_name,
            object_id,
            billing_account_id,
            workspace_id,
            organization_id,
            tenant_ids,
            created_by_user_id,
            update_clock,
            deleted: false,
        })
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SimpleStorable {
    pub type_name: String,
    pub object_id: String,
    pub billing_account_id: String,
    pub tenant_ids: Vec<String>,
    pub deleted: bool,
}

impl SimpleStorable {
    pub fn new(
        object_id: impl Into<String>,
        type_name: impl Into<String>,
        billing_account_id: impl Into<String>,
    ) -> SimpleStorable {
        let object_id = object_id.into();
        let type_name = type_name.into();
        let billing_account_id = billing_account_id.into();

        let tenant_ids = if object_id == billing_account_id {
            vec![object_id.clone()]
        } else {
            vec![billing_account_id.clone(), object_id.clone()]
        };
        let deleted = false;
        SimpleStorable {
            type_name,
            object_id,
            billing_account_id,
            tenant_ids,
            deleted,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MinimalStorable {
    pub type_name: String,
    pub object_id: String,
    pub deleted: bool,
}

impl MinimalStorable {
    pub fn new(object_id: impl Into<String>, type_name: impl Into<String>) -> MinimalStorable {
        let object_id = object_id.into();
        let type_name = type_name.into();
        let deleted = false;
        MinimalStorable {
            type_name,
            object_id,
            deleted,
        }
    }
}
