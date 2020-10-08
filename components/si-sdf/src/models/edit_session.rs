use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::data::{Connection, DataError, Db};
use crate::models::{
    insert_model, publish_model, ChangeSet, ChangeSetError, ModelError, SiStorable, SiStorableError,
};

use std::collections::HashMap;

#[derive(Error, Debug)]
pub enum EditSessionError {
    #[error("si_storable error: {0}")]
    SiStorable(#[from] SiStorableError),
    #[error("error in core model functions: {0}")]
    Model(#[from] ModelError),
    #[error("data error: {0}")]
    Data(#[from] DataError),
    #[error("changeSet error: {0}")]
    ChangeSet(#[from] ChangeSetError),
}

pub type EditSessionResult<T> = Result<T, EditSessionError>;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum PatchRequest {
    Cancel(bool),
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum PatchReply {
    Cancel(EditSession),
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateRequest {
    pub name: Option<String>,
    pub workspace_id: String,
    pub organization_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateReply {
    pub item: EditSession,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EditSession {
    pub id: String,
    pub name: String,
    pub note: String,
    pub reverted: bool,
    pub change_set_id: String,
    pub si_storable: SiStorable,
}

impl EditSession {
    pub async fn new(
        db: &Db,
        nats: &Connection,
        name: Option<String>,
        change_set_id: String,
        billing_account_id: String,
        organization_id: String,
        workspace_id: String,
        created_by_user_id: String,
    ) -> EditSessionResult<EditSession> {
        let name = crate::models::generate_name(name);
        let si_storable = SiStorable::new(
            db,
            "editSession",
            billing_account_id,
            organization_id,
            workspace_id,
            Some(created_by_user_id),
        )
        .await?;
        let id = si_storable.object_id.clone();
        let edit_session = EditSession {
            id,
            name,
            change_set_id,
            note: "".to_string(),
            reverted: false,
            si_storable,
        };
        insert_model(db, nats, &edit_session.id, &edit_session).await?;
        Ok(edit_session)
    }

    pub async fn cancel(&self, db: &Db, nats: &Connection) -> EditSessionResult<()> {
        let query = format!(
            "UPDATE `{bucket}`
                SET siOp.skip = true
                WHERE siChangeSet.changeSetId = $change_set_id 
                  AND siChangeSet.editSessionId = $edit_session_id
            RETURNING `{bucket}`.*",
            bucket = db.bucket_name,
        );
        let mut named_params: HashMap<String, serde_json::Value> = HashMap::new();
        named_params.insert(
            "change_set_id".into(),
            serde_json::json![&self.change_set_id],
        );
        named_params.insert("edit_session_id".into(), serde_json::json![&self.id]);
        let query_results: Vec<serde_json::Value> = db.query(query, Some(named_params)).await?;
        for item in query_results.iter() {
            publish_model(nats, item).await?;
        }
        let query = format!(
            "UPDATE `{bucket}`
                SET siStorable.deleted = true
                WHERE siChangeSet.changeSetId = $change_set_id 
                  AND siChangeSet.editSessionId = $edit_session_id
                  AND base = true
            RETURNING `{bucket}`.*",
            bucket = db.bucket_name,
        );
        let mut named_params: HashMap<String, serde_json::Value> = HashMap::new();
        named_params.insert(
            "change_set_id".into(),
            serde_json::json![&self.change_set_id],
        );
        named_params.insert("edit_session_id".into(), serde_json::json![&self.id]);
        let query_results: Vec<serde_json::Value> = db.query(query, Some(named_params)).await?;
        for item in query_results.iter() {
            publish_model(nats, item).await?;
        }
        let mut change_set = ChangeSet::get(
            db,
            &self.change_set_id,
            &self.si_storable.billing_account_id,
        )
        .await?;
        change_set.execute(db, nats, true).await?;

        tracing::info!(?query_results, "cancel edit session");
        Ok(())
    }
}
