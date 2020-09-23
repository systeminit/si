use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::data::{Connection, Db};
use crate::models::{insert_model, ModelError, SiStorable, SiStorableError};

#[derive(Error, Debug)]
pub enum EditSessionError {
    #[error("si_storable error: {0}")]
    SiStorable(#[from] SiStorableError),
    #[error("error in core model functions: {0}")]
    Model(#[from] ModelError),
}

pub type EditSessionResult<T> = Result<T, EditSessionError>;

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
}
