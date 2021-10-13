use crate::{generate_name, LabelList, LabelListItem, SiStorable};
use lazy_static::lazy_static;
use regex::Regex;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use si_data::{NatsTxn, NatsTxnError, PgTxn};
use strum_macros::{Display, EnumString};
use thiserror::Error;

const CHANGE_SET_OPEN_LIST_AS_LABLES: &str =
    include_str!("./queries/change_set_open_list_as_labels.sql");
const CHANGE_SET_REVISION_LIST_AS_LABLES: &str =
    include_str!("./queries/change_set_revision_list_as_labels.sql");
const OBJECT_FOR_EDIT_SESSION: &str = include_str!("./queries/object_for_edit_session.sql");
const OBJECT_FOR_CHANGE_SET: &str = include_str!("./queries/object_for_change_set.sql");
const OBJECT_FOR_HEAD: &str = include_str!("./queries/object_for_head.sql");

lazy_static! {
    static ref ROOT_TABLE_NAME_RE: Regex = Regex::new("(?P<root_table_name>\\{root_table_name\\})")
        .expect("root table name regex does not compile; bug!");
}

#[derive(Error, Debug)]
pub enum ChangeSetError {
    #[error("missing change set event field")]
    EventMissing,
    #[error("malformed change set entry; id is missing")]
    IdMissing,
    #[error("missing head value in object")]
    MissingHead,
    #[error("nats txn error: {0}")]
    NatsTxn(#[from] NatsTxnError),
    #[error("pg error: {0}")]
    Pg(#[from] si_data::PgError),
    #[error("pg pool error: {0}")]
    PgPool(#[from] si_data::PgPoolError),
    #[error("error creating our object from json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("malformed change set entry; to_id is missing")]
    ToIdMissing,
    #[error("malformed change set entry; type is missing")]
    TypeMissing,
    #[error("schema {0} not found for edit session {1}")]
    NotFoundForEditSession(String, String),
    #[error("schema {0} not found for change set {1}")]
    NotFoundForChangeSet(String, String),
    #[error("schema {0} not found for head")]
    NotFoundForHead(String),
    #[error("schema {0} not found for head or change set {1:?}")]
    NotFoundForHeadOrChangeSet(String, Option<String>),
    #[error("schema {0} not found for head, or change set {1:?}, or edit session {2:?}")]
    NotFoundForHeadOrChangeSetOrEditSession(String, Option<String>, Option<String>),
    #[error("this query requires a change set, and one was not provided")]
    NoChangeSet,
}

pub type ChangeSetResult<T> = Result<T, ChangeSetError>;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SiChangeSet {
    change_set_id: String,
    edit_session_id: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Display, EnumString, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum ChangeSetStatus {
    Open,
    Closed,
    Abandoned,
    Applied,
    Failed,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
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
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        name: Option<String>,
        workspace_id: String,
    ) -> ChangeSetResult<ChangeSet> {
        let name = generate_name(name);
        let row = txn
            .query_one(
                "SELECT object FROM change_set_create_v1($1, $2, $3, $4)",
                &[
                    &name,
                    &String::new(),
                    &ChangeSetStatus::Open.to_string(),
                    &workspace_id,
                ],
            )
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        nats.publish(&json).await?;
        let object: ChangeSet = serde_json::from_value(json)?;
        Ok(object)
    }

    pub async fn get(
        txn: &PgTxn<'_>,
        change_set_id: impl AsRef<str>,
    ) -> ChangeSetResult<ChangeSet> {
        let id = change_set_id.as_ref();
        let row = txn
            .query_one("SELECT object FROM change_set_get_v1($1)", &[&id])
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        let object = serde_json::from_value(json)?;
        Ok(object)
    }

    pub async fn save(&mut self, txn: &PgTxn<'_>, nats: &NatsTxn) -> ChangeSetResult<()> {
        let json = serde_json::to_value(&self)?;
        let row = txn
            .query_one("SELECT object FROM change_set_save_v1($1)", &[&json])
            .await?;
        let updated_result: serde_json::Value = row.try_get("object")?;
        nats.publish(&updated_result).await?;
        let mut updated: ChangeSet = serde_json::from_value(updated_result)?;
        std::mem::swap(self, &mut updated);
        Ok(())
    }

    pub async fn open_list_as_labels(
        txn: &PgTxn<'_>,
        workspace_id: impl AsRef<str>,
    ) -> ChangeSetResult<LabelList> {
        let workspace_id = workspace_id.as_ref();
        let mut results = Vec::new();
        let rows = txn
            .query(CHANGE_SET_OPEN_LIST_AS_LABLES, &[&workspace_id])
            .await?;
        for row in rows.into_iter() {
            let json: serde_json::Value = row.try_get("item")?;
            let object: LabelListItem = serde_json::from_value(json)?;
            results.push(object);
        }

        return Ok(results);
    }

    pub async fn revision_list_as_labels(
        txn: &PgTxn<'_>,
        workspace_id: impl AsRef<str>,
    ) -> ChangeSetResult<LabelList> {
        let workspace_id = workspace_id.as_ref();
        let mut results = Vec::new();
        let rows = txn
            .query(CHANGE_SET_REVISION_LIST_AS_LABLES, &[&workspace_id])
            .await?;
        for row in rows.into_iter() {
            let json: serde_json::Value = row.try_get("item")?;
            let object: LabelListItem = serde_json::from_value(json)?;
            results.push(object);
        }

        return Ok(results);
    }

    pub async fn apply(&mut self, txn: &PgTxn<'_>) -> ChangeSetResult<()> {
        let row = txn
            .query_one("SELECT object FROM change_set_apply_v1($1)", &[&self.id])
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        let change_set: ChangeSet = serde_json::from_value(json)?;
        *self = change_set;
        Ok(())
    }
}

pub async fn for_edit_session<Object: DeserializeOwned>(
    txn: &PgTxn<'_>,
    root_table_name: impl AsRef<str>,
    object_id: impl AsRef<str>,
    change_set_id: impl AsRef<str>,
    edit_session_id: impl AsRef<str>,
) -> ChangeSetResult<Object> {
    let root_table_name = root_table_name.as_ref();
    let object_id = object_id.as_ref();
    let change_set_id = change_set_id.as_ref();
    let edit_session_id = edit_session_id.as_ref();

    let query = ROOT_TABLE_NAME_RE.replace_all(OBJECT_FOR_EDIT_SESSION, root_table_name);

    let row = txn
        .query_opt(&query, &[&object_id, &change_set_id, &edit_session_id])
        .await?
        .ok_or_else(|| {
            ChangeSetError::NotFoundForEditSession(
                object_id.to_string(),
                edit_session_id.to_string(),
            )
        })?;
    let json: serde_json::Value = row.try_get("object")?;
    let object: Object = serde_json::from_value(json)?;
    Ok(object)
}

pub async fn for_change_set<Object: DeserializeOwned>(
    txn: &PgTxn<'_>,
    root_table_name: impl AsRef<str>,
    object_id: impl AsRef<str>,
    change_set_id: impl AsRef<str>,
) -> ChangeSetResult<Object> {
    let root_table_name = root_table_name.as_ref();
    let object_id = object_id.as_ref();
    let change_set_id = change_set_id.as_ref();

    let query = ROOT_TABLE_NAME_RE.replace_all(OBJECT_FOR_CHANGE_SET, root_table_name);

    let row = txn
        .query_opt(&query, &[&object_id, &change_set_id])
        .await?
        .ok_or_else(|| {
            ChangeSetError::NotFoundForChangeSet(object_id.to_string(), change_set_id.to_string())
        })?;
    let json: serde_json::Value = row.try_get("object")?;
    let object: Object = serde_json::from_value(json)?;
    Ok(object)
}

pub async fn for_head<Object: DeserializeOwned>(
    txn: &PgTxn<'_>,
    root_table_name: impl AsRef<str>,
    object_id: impl AsRef<str>,
) -> ChangeSetResult<Object> {
    let root_table_name = root_table_name.as_ref();
    let object_id = object_id.as_ref();

    let query = ROOT_TABLE_NAME_RE.replace_all(OBJECT_FOR_HEAD, root_table_name);

    let row = txn
        .query_opt(&query, &[&object_id])
        .await?
        .ok_or_else(|| ChangeSetError::NotFoundForHead(object_id.to_string()))?;
    let json: serde_json::Value = row.try_get("object")?;
    let object: Object = serde_json::from_value(json)?;
    Ok(object)
}

pub async fn for_head_or_change_set<Object: DeserializeOwned>(
    txn: &PgTxn<'_>,
    root_table_name: impl AsRef<str>,
    object_id: impl AsRef<str>,
    change_set_id: Option<&String>,
) -> ChangeSetResult<Object> {
    if let Some(change_set_id) = change_set_id {
        for_change_set(&txn, root_table_name, object_id, change_set_id)
            .await
            .map_err(|err| match err {
                ChangeSetError::NotFoundForChangeSet(entity_id, change_set_id) => {
                    ChangeSetError::NotFoundForHeadOrChangeSet(entity_id, Some(change_set_id))
                }
                err => err,
            })
    } else {
        for_head(&txn, root_table_name, object_id)
            .await
            .map_err(|err| match err {
                ChangeSetError::NotFoundForHead(entity_id) => {
                    ChangeSetError::NotFoundForHeadOrChangeSet(entity_id, None)
                }
                err => err,
            })
    }
}

pub async fn for_head_or_change_set_or_edit_session<Object: DeserializeOwned>(
    txn: &PgTxn<'_>,
    root_table_name: impl AsRef<str>,
    object_id: impl AsRef<str>,
    change_set_id: Option<&String>,
    edit_session_id: Option<&String>,
) -> ChangeSetResult<Object> {
    if let Some(edit_session_id) = edit_session_id {
        if let Some(change_set_id) = change_set_id {
            for_edit_session(
                &txn,
                root_table_name,
                &object_id,
                change_set_id,
                edit_session_id,
            )
            .await
            .map_err(|err| match err {
                ChangeSetError::NotFoundForEditSession(entity_id, edit_session_id) => {
                    ChangeSetError::NotFoundForHeadOrChangeSetOrEditSession(
                        entity_id,
                        Some(change_set_id.to_string()),
                        Some(edit_session_id),
                    )
                }
                err => err,
            })
        } else {
            return Err(ChangeSetError::NoChangeSet);
        }
    } else if let Some(change_set_id) = change_set_id {
        for_change_set(&txn, root_table_name, object_id, change_set_id)
            .await
            .map_err(|err| match err {
                ChangeSetError::NotFoundForChangeSet(entity_id, change_set_id) => {
                    ChangeSetError::NotFoundForHeadOrChangeSetOrEditSession(
                        entity_id,
                        Some(change_set_id),
                        None,
                    )
                }
                err => err,
            })
    } else {
        for_head(&txn, root_table_name, object_id)
            .await
            .map_err(|err| match err {
                ChangeSetError::NotFoundForHead(entity_id) => {
                    ChangeSetError::NotFoundForHeadOrChangeSetOrEditSession(entity_id, None, None)
                }
                err => err,
            })
    }
}

#[macro_export]
macro_rules! change_set_methods {
    ($root_table_name:expr, $result_type:ty) => {
        pub async fn for_edit_session(
            txn: &si_data::pg::PgTxn<'_>,
            object_id: impl AsRef<str>,
            change_set_id: impl AsRef<str>,
            edit_session_id: impl AsRef<str>,
        ) -> $result_type {
            let o = crate::change_set::for_edit_session(
                txn,
                $root_table_name,
                object_id,
                change_set_id,
                edit_session_id,
            )
            .await?;
            Ok(o)
        }

        pub async fn for_change_set(
            txn: &si_data::pg::PgTxn<'_>,
            object_id: impl AsRef<str>,
            change_set_id: impl AsRef<str>,
        ) -> $result_type {
            let o = crate::change_set::for_change_set(
                txn,
                $root_table_name,
                object_id,
                change_set_id,
            )
            .await?;
            Ok(o)
        }

        pub async fn for_head(
            txn: &si_data::pg::PgTxn<'_>,
            object_id: impl AsRef<str>,
        ) -> $result_type {
            let o = crate::change_set::for_head(txn, $root_table_name, object_id).await?;
            Ok(o)
        }

        pub async fn for_head_or_change_set(
            txn: &si_data::pg::PgTxn<'_>,
            object_id: impl AsRef<str>,
            change_set_id: Option<&String>,
        ) -> $result_type {
            let o = crate::change_set::for_head_or_change_set(
                txn,
                $root_table_name,
                object_id,
                change_set_id,
            )
            .await?;
            Ok(o)
        }

        pub async fn for_head_or_change_set_or_edit_session(
            txn: &si_data::pg::PgTxn<'_>,
            object_id: impl AsRef<str>,
            change_set_id: Option<&String>,
            edit_session_id: Option<&String>,
        ) -> $result_type {
            let o = crate::change_set::for_head_or_change_set_or_edit_session(
                txn,
                $root_table_name,
                object_id,
                change_set_id,
                edit_session_id,
            )
            .await?;
            Ok(o)
        }
    };
}
