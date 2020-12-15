use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

use crate::data::{Connection, Db};
use crate::models::{
    insert_model, ModelError, SiChangeSet, SiChangeSetError, SiChangeSetEvent, SiStorable,
    SiStorableError,
};

#[derive(Error, Debug)]
pub enum SystemError {
    #[error("si_storable error: {0}")]
    SiStorable(#[from] SiStorableError),
    #[error("si_change_set error: {0}")]
    SiChangeSet(#[from] SiChangeSetError),
    #[error("error in core model functions: {0}")]
    Model(#[from] ModelError),
    #[error("no head entity found; logic error")]
    NoHead,
    #[error("data layer error: {0}")]
    Data(#[from] crate::data::DataError),
}

pub type SystemResult<T> = Result<T, SystemError>;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateRequest {
    pub name: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateReply {
    pub item: System,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct System {
    pub id: String,
    pub name: String,
    pub description: String,
    pub node_id: String,
    pub head: bool,
    pub base: bool,
    pub si_storable: SiStorable,
    pub si_change_set: Option<SiChangeSet>,
}

impl System {
    
    pub async fn new(
        db: &Db,
        nats: &Connection,
        name: Option<String>,
        description: Option<String>,
        node_id: String,
        head: bool,
        billing_account_id: String,
        organization_id: String,
        workspace_id: String,
        change_set_id: String,
        edit_session_id: String,
        created_by_user_id: Option<String>,
    ) -> SystemResult<System> {
        let name = crate::models::generate_name(name);
        let description = if description.is_some() {
            description.unwrap()
        } else {
            name.clone()
        };
        let si_storable = SiStorable::new(
            db,
            "system",
            billing_account_id.clone(),
            organization_id,
            workspace_id,
            created_by_user_id,
        )
        .await?;
        let id = si_storable.object_id.clone();
        let key = format!("{}:{}", si_storable.object_id, &change_set_id);
        let base_key = format!("{}:{}:base", &si_storable.object_id, &change_set_id);
        let si_change_set = SiChangeSet::new(
            db,
            nats,
            change_set_id,
            edit_session_id,
            &id,
            billing_account_id,
            SiChangeSetEvent::Create,
        )
        .await?;
        let mut system = System {
            id,
            name,
            head,
            description,
            node_id,
            base: false,
            si_storable,
            si_change_set: Some(si_change_set),
        };
        insert_model(db, nats, &key, &system).await?;
        system.base = true;
        insert_model(db, nats, &base_key, &system).await?;

        Ok(system)
    }

    pub async fn get_any(db: &Db, id: impl AsRef<str>) -> SystemResult<System> {
        let id = id.as_ref();
        let query = format!(
            "SELECT a.*
          FROM `{bucket}` AS a
          WHERE a.siStorable.typeName = \"system\"
            AND a.siStorable.objectId = $id 
          LIMIT 1
        ",
            bucket = db.bucket_name
        );
        let mut named_params: HashMap<String, serde_json::Value> = HashMap::new();
        named_params.insert("id".into(), serde_json::json![id]);
        let mut query_results: Vec<System> = db.query(query, Some(named_params)).await?;
        if query_results.len() == 0 {
            Err(SystemError::NoHead)
        } else {
            let result = query_results.pop().unwrap();
            Ok(result)
        }
    }

    pub async fn get_head(db: &Db, id: impl AsRef<str>) -> SystemResult<System> {
        let id = id.as_ref();
        let query = format!(
            "SELECT a.*
          FROM `{bucket}` AS a
          WHERE a.siStorable.typeName = \"system\"
            AND a.siStorable.objectId = $id 
            AND a.head = true
          LIMIT 1
        ",
            bucket = db.bucket_name
        );
        let mut named_params: HashMap<String, serde_json::Value> = HashMap::new();
        named_params.insert("id".into(), serde_json::json![id]);
        let mut query_results: Vec<System> = db.query(query, Some(named_params)).await?;
        if query_results.len() == 0 {
            Err(SystemError::NoHead)
        } else {
            let result = query_results.pop().unwrap();
            Ok(result)
        }
    }
}
