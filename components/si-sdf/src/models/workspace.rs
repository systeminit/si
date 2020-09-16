use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::data::Db;
use crate::models::{
    check_secondary_key, generate_id, get_model, insert_model, ModelError, SiStorableError,
    SimpleStorable,
};

#[derive(Error, Debug)]
pub enum WorkspaceError {
    #[error("a workspace with this name already exists")]
    NameExists,
    #[error("si_storable error: {0}")]
    SiStorable(#[from] SiStorableError),
    #[error("error in core model functions: {0}")]
    Model(#[from] ModelError),
}

pub type WorkspaceResult<T> = Result<T, WorkspaceError>;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub si_storable: SimpleStorable,
}

impl Workspace {
    pub async fn new(
        db: &Db,
        name: impl Into<String>,
        billing_account_id: impl Into<String>,
    ) -> WorkspaceResult<Workspace> {
        let name = name.into();
        let billing_account_id = billing_account_id.into();

        if check_secondary_key(db, &billing_account_id, "workspace", "name", &name).await? {
            return Err(WorkspaceError::NameExists);
        }

        let id = generate_id("workspace");
        let si_storable = SimpleStorable::new(&id, "workspace", &billing_account_id);
        let object = Workspace {
            id,
            name,
            si_storable,
        };
        insert_model(db, &object.id, &object).await?;
        Ok(object)
    }

    pub async fn get(
        db: &Db,
        workspace_id: impl AsRef<str>,
        billing_account_id: impl AsRef<str>,
    ) -> WorkspaceResult<Workspace> {
        let id = workspace_id.as_ref();
        let billing_account_id = billing_account_id.as_ref();
        let object: Workspace = get_model(db, id, billing_account_id).await?;
        Ok(object)
    }
}
