use serde::{Deserialize, Serialize};
use strum_macros::Display;
use thiserror::Error;

use si_data::NatsTxnError;

use crate::{entity::diff::Diffs, Resource, SiStorable};

#[derive(Error, Debug)]
pub enum ActionError {
    #[error("json serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("nats txn error: {0}")]
    NatsTxn(#[from] NatsTxnError),
    #[error("pg error: {0}")]
    TokioPg(#[from] tokio_postgres::Error),
}

pub type ActionResult<T> = Result<T, ActionError>;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Display, Clone)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum ActionState {
    Running,
    Success,
    Failure,
    Unknown,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Action {
    pub id: String,
    pub name: String,
    pub dry_run: bool,
    pub state: ActionState,
    pub resource: Option<Resource>,
    pub resource_diff: Option<Diffs>,
    pub start_unix_timestamp: i64,
    pub start_timestamp: String,
    pub end_unix_timestamp: Option<i64>,
    pub end_timestamp: Option<String>,
    pub output: Option<String>,
    pub error: Option<String>,
    pub entity_id: String,
    pub system_id: String,
    pub workflow_run_id: String,
    pub si_storable: SiStorable,
}
