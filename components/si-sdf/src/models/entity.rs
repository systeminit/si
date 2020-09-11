use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::data::Db;
use crate::models::{
    insert_model, ModelError, SiChangeSet, SiChangeSetError, SiChangeSetEvent, SiStorable,
    SiStorableError,
};

#[derive(Error, Debug)]
pub enum EntityError {
    #[error("si_storable error: {0}")]
    SiStorable(#[from] SiStorableError),
    #[error("si_change_set error: {0}")]
    SiChangeSet(#[from] SiChangeSetError),
    #[error("error in core model functions: {0}")]
    Model(#[from] ModelError),
}

pub type EntityResult<T> = Result<T, EntityError>;

#[derive(Deserialize, Serialize, Debug)]
pub struct CreateRequest {
    pub name: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CreateReply {
    pub item: Entity,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EntityProperties {
    baseline: serde_json::Value,
}

impl EntityProperties {
    pub fn new() -> Self {
        EntityProperties {
            baseline: serde_json::json!["{}"],
        }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Entity {
    pub id: String,
    pub name: String,
    pub object_type: String,
    pub description: String,
    pub properties: EntityProperties,
    pub node_id: String,
    pub si_storable: SiStorable,
    pub si_change_set: Option<SiChangeSet>,
}

impl Entity {
    #[tracing::instrument(level = "trace")]
    pub async fn new(
        db: &Db,
        name: Option<String>,
        description: Option<String>,
        node_id: String,
        object_type: String,
        billing_account_id: String,
        organization_id: String,
        workspace_id: String,
        change_set_id: String,
        edit_session_id: String,
        created_by_user_id: String,
    ) -> EntityResult<Entity> {
        let name = crate::models::generate_name(name);
        let description = if description.is_some() {
            description.unwrap()
        } else {
            name.clone()
        };
        let si_storable = SiStorable::new(
            db,
            "entity",
            billing_account_id,
            organization_id,
            workspace_id,
            created_by_user_id,
        )
        .await?;
        let properties = EntityProperties::new();
        let id = format!("{}:{}", si_storable.object_id, change_set_id);
        let si_change_set =
            SiChangeSet::new(db, change_set_id, edit_session_id, SiChangeSetEvent::Create).await?;
        let entity = Entity {
            id,
            name,
            object_type,
            description,
            properties,
            node_id,
            si_storable,
            si_change_set: Some(si_change_set),
        };
        insert_model(db, &entity.id, &entity).await?;

        Ok(entity)
    }
}
