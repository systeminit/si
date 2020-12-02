use serde::{Deserialize, Serialize};
use thiserror::Error;

use std::collections::HashMap;

use crate::data::{Connection, DataError, Db};
use crate::models::{
    check_secondary_key, generate_id, get_model, insert_model, ModelError, SiStorableError,
    SimpleStorable,
};

#[derive(Error, Debug)]
pub enum GroupError {
    #[error("a group with this name already exists")]
    NameExists,
    #[error("si_storable error: {0}")]
    SiStorable(#[from] SiStorableError),
    #[error("error in core model functions: {0}")]
    Model(#[from] ModelError),
    #[error("invalid uft-8 string: {0}")]
    Utf8(#[from] std::str::Utf8Error),
    #[error("error generating password hash")]
    PasswordHash,
    #[error("group not found")]
    NotFound,
    #[error("data: {0}")]
    Data(#[from] DataError),
}

pub type GroupResult<T> = Result<T, GroupError>;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Capability {
    pub subject: String,
    pub action: String,
}

impl Capability {
    pub fn new(subject: impl Into<String>, action: impl Into<String>) -> Capability {
        let subject = subject.into();
        let action = action.into();
        Capability { subject, action }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    pub id: String,
    pub name: String,
    pub user_ids: Vec<String>,
    pub api_client_ids: Vec<String>,
    pub capabilities: Vec<Capability>,
    pub si_storable: SimpleStorable,
}

impl Group {
    pub async fn new(
        db: &Db,
        nats: &Connection,
        name: impl Into<String>,
        user_ids: Vec<String>,
        api_client_ids: Vec<String>,
        capabilities: Vec<Capability>,
        billing_account_id: impl Into<String>,
    ) -> GroupResult<Group> {
        let name = name.into();
        let billing_account_id = billing_account_id.into();

        if check_secondary_key(db, &billing_account_id, "group", "name", &name).await? {
            return Err(GroupError::NameExists);
        }

        let id = generate_id("group");
        let si_storable = SimpleStorable::new(&id, "group", &billing_account_id);
        let object = Group {
            id,
            name,
            user_ids,
            api_client_ids,
            capabilities,
            si_storable,
        };
        insert_model(db, nats, &object.id, &object).await?;
        Ok(object)
    }

    pub async fn get(
        db: &Db,
        group_id: impl AsRef<str>,
        billing_account_id: impl AsRef<str>,
    ) -> GroupResult<Group> {
        let id = group_id.as_ref();
        let billing_account_id = billing_account_id.as_ref();
        let object: Group = get_model(db, id, billing_account_id).await?;
        Ok(object)
    }

    pub async fn get_administrators_group(
        db: &Db,
        billing_account_id: impl AsRef<str>,
    ) -> GroupResult<Group> {
        let billing_account_id = billing_account_id.as_ref();
        let query = format!(
            "SELECT a.*
               FROM `{bucket}` AS a
               WHERE a.siStorable.typeName = \"group\"
                 AND a.siStorable.billingAccountId = $billing_account_id
                 AND a.name = \"administrators\" 
               LIMIT 1",
            bucket = db.bucket_name,
        );
        let mut named_params: HashMap<String, serde_json::Value> = HashMap::new();
        named_params.insert(
            "billing_account_id".into(),
            serde_json::json![billing_account_id],
        );
        let mut results: Vec<Group> = db.query(query, Some(named_params)).await?;
        if let Some(group) = results.pop() {
            Ok(group)
        } else {
            Err(GroupError::NotFound)
        }
    }
}
