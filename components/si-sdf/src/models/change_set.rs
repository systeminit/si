use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::data::Db;
use crate::models::{insert_model, ModelError, SiStorable, SiStorableError};

#[derive(Error, Debug)]
pub enum ChangeSetError {
    #[error("si_storable error: {0}")]
    SiStorable(#[from] SiStorableError),
    #[error("error in core model functions: {0}")]
    Model(#[from] ModelError),
}

pub type ChangeSetResult<T> = Result<T, ChangeSetError>;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateRequest {
    pub name: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateReply {
    pub item: ChangeSet,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum ChangeSetStatus {
    Open,
    Closed,
    Abandoned,
    Executing,
    Failed,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ChangeSet {
    pub id: String,
    pub name: String,
    pub note: String,
    pub status: ChangeSetStatus,
    pub si_storable: SiStorable,
}

impl ChangeSet {
    pub async fn new(
        db: &Db,
        name: Option<String>,
        billing_account_id: String,
        organization_id: String,
        workspace_id: String,
        created_by_user_id: String,
    ) -> ChangeSetResult<ChangeSet> {
        let name = crate::models::generate_name(name);
        let si_storable = SiStorable::new(
            db,
            "changeSet",
            billing_account_id,
            organization_id,
            workspace_id,
            created_by_user_id,
        )
        .await?;
        let id = si_storable.object_id.clone();
        let change_set = ChangeSet {
            id,
            name,
            note: "".to_string(),
            status: ChangeSetStatus::Open,
            si_storable,
        };
        insert_model(db, &change_set.id, &change_set).await?;
        Ok(change_set)
    }
}
