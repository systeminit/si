use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::data::REQWEST;
use crate::data::{NatsTxn, NatsTxnError, PgTxn};

use crate::models::{
    next_update_clock, EdgeError, EntityError, EventError, ModelError, NodeError, ResourceError,
    SiChangeSet, SiChangeSetError, SiChangeSetEvent, SiStorable, UpdateClockError,
};
use crate::veritech::VeritechError;

pub mod entity_delete;
pub use self::entity_delete::{OpEntityDelete, OpEntityDeleteRequest};
pub mod entity_action;
pub use self::entity_action::{OpEntityAction, OpEntityActionRequest};

#[derive(Error, Debug)]
pub enum OpError {
    #[error("si_change_set error: {0}")]
    SiChangeSet(#[from] SiChangeSetError),
    #[error("error in core model functions: {0}")]
    Model(#[from] ModelError),
    #[error("entity error: {0}")]
    Entity(#[from] EntityError),
    #[error("cannot set value: path({0}) value({1})")]
    Failed(String, serde_json::Value),
    #[error("malformed target")]
    MalformedTarget,
    #[error("error making http call: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("edge error: {0}")]
    Edge(#[from] EdgeError),
    #[error("node error: {0}")]
    Node(#[from] NodeError),
    #[error("missing: {0}")]
    Missing(&'static str),
    #[error("resource error: {0}")]
    Resource(#[from] ResourceError),
    #[error("event error: {0}")]
    Event(#[from] EventError),
    #[error("veritech error: {0}")]
    Veritech(#[from] VeritechError),
    #[error("pg error: {0}")]
    TokioPg(#[from] tokio_postgres::Error),
    #[error("nats txn error: {0}")]
    NatsTxn(#[from] NatsTxnError),
    #[error("update clock: {0}")]
    UpdateClock(#[from] UpdateClockError),
    #[error("json serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum OpRequest {
    EntitySet(OpEntitySetRequest),
    NameSet(OpSetNameRequest),
    EntityDelete(OpEntityDeleteRequest),
    EntityAction(OpEntityActionRequest),
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OpReply {
    pub item_ids: Vec<String>,
}

pub type OpResult<T> = Result<T, OpError>;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SiOp {
    pub skip: bool,
    pub override_system: Option<String>,
}

impl SiOp {
    fn new(override_system: Option<String>) -> Self {
        SiOp {
            skip: false,
            override_system,
        }
    }

    fn skip(&self) -> bool {
        self.skip
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OpEntitySetRequest {
    pub path: Vec<String>,
    pub value: serde_json::Value,
    pub override_system: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OpEntitySet {
    pub id: String,
    pub to_id: String,
    pub path: Vec<String>,
    pub value: serde_json::Value,
    pub si_op: SiOp,
    pub si_storable: SiStorable,
    pub si_change_set: SiChangeSet,
}

impl OpEntitySet {
    pub async fn new(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        to_id: impl AsRef<str>,
        path: Vec<String>,
        value: impl Into<serde_json::Value>,
        override_system: Option<String>,
        workspace_id: impl AsRef<str>,
        change_set_id: impl AsRef<str>,
        edit_session_id: impl AsRef<str>,
    ) -> OpResult<Self> {
        let workspace_id = workspace_id.as_ref();
        let change_set_id = change_set_id.as_ref();
        let edit_session_id = edit_session_id.as_ref();
        let to_id = to_id.as_ref();
        let value = value.into();

        let workspace_update_clock = next_update_clock(workspace_id).await?;
        let change_set_update_clock = next_update_clock(change_set_id).await?;

        let row = txn
            .query_one(
                "SELECT object FROM op_create_v1($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
                &[
                    &"opEntitySet",
                    &to_id,
                    &serde_json::json![{"path": path, "value": value }],
                    &override_system,
                    &change_set_id,
                    &edit_session_id,
                    &SiChangeSetEvent::Operation.to_string(),
                    &workspace_id,
                    &workspace_update_clock.epoch,
                    &workspace_update_clock.update_count,
                    &change_set_update_clock.epoch,
                    &change_set_update_clock.update_count,
                ],
            )
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        nats.publish(&json).await?;
        let object: Self = serde_json::from_value(json)?;
        Ok(object)
    }

    pub fn skip(&self) -> bool {
        self.si_op.skip()
    }

    pub async fn apply(&self, to: &mut serde_json::Value) -> OpResult<()> {
        if self.skip() {
            return Ok(());
        }
        let override_system = self
            .si_op
            .override_system
            .as_deref()
            .unwrap_or("__baseline");

        let mut op_path = self.path.clone();
        let mut full_path: Vec<String> = vec!["manualProperties".into(), override_system.into()];
        full_path.append(&mut op_path);

        let apply_req = ApplyOpRequest::new(
            ApplyOperation::Set,
            &self.to_id,
            full_path,
            Some(serde_json::json![self.value]),
            to,
        );
        let result = apply_op(apply_req).await?;
        *to = result;
        Ok(())
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OpSetNameRequest {
    pub value: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OpSetName {
    pub id: String,
    pub to_id: String,
    pub value: String,
    pub si_op: SiOp,
    pub si_storable: SiStorable,
    pub si_change_set: SiChangeSet,
}

impl OpSetName {
    pub async fn new(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        to_id: impl AsRef<str>,
        value: impl AsRef<str>,
        workspace_id: impl AsRef<str>,
        change_set_id: impl AsRef<str>,
        edit_session_id: impl AsRef<str>,
    ) -> OpResult<Self> {
        let workspace_id = workspace_id.as_ref();
        let change_set_id = change_set_id.as_ref();
        let edit_session_id = edit_session_id.as_ref();
        let to_id = to_id.as_ref();
        let value = value.as_ref();

        let workspace_update_clock = next_update_clock(workspace_id).await?;
        let change_set_update_clock = next_update_clock(change_set_id).await?;

        let override_system: Option<String> = None;

        let row = txn
            .query_one(
                "SELECT object FROM op_create_v1($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
                &[
                    &"opSetName",
                    &to_id,
                    &serde_json::json![{"value": value }],
                    &override_system,
                    &change_set_id,
                    &edit_session_id,
                    &SiChangeSetEvent::Operation.to_string(),
                    &workspace_id,
                    &workspace_update_clock.epoch,
                    &workspace_update_clock.update_count,
                    &change_set_update_clock.epoch,
                    &change_set_update_clock.update_count,
                ],
            )
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        nats.publish(&json).await?;
        let object: Self = serde_json::from_value(json)?;
        Ok(object)
    }

    pub fn skip(&self) -> bool {
        self.si_op.skip()
    }

    pub async fn apply<'a>(&'a self, to: &'a mut serde_json::Value) -> OpResult<()> {
        if self.skip() {
            return Ok(());
        }
        match to.get_mut("name") {
            Some(name) => *name = serde_json::Value::String(self.value.clone()),
            None => return Err(OpError::Failed("name field missing".into(), to.clone())),
        }
        Ok(())
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum ApplyOperation {
    Set,
    Unset,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ApplyOpRequest<'a> {
    operation: ApplyOperation,
    to_id: String,
    path: Vec<String>,
    value: Option<serde_json::Value>,
    object: &'a serde_json::Value,
}

impl<'a> ApplyOpRequest<'a> {
    pub fn new(
        operation: ApplyOperation,
        to_id: impl Into<String>,
        path: Vec<String>,
        value: Option<serde_json::Value>,
        object: &'a serde_json::Value,
    ) -> ApplyOpRequest {
        let to_id = to_id.into();
        ApplyOpRequest {
            operation,
            to_id,
            path,
            value,
            object,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ApplyOpReply {
    object: serde_json::Value,
}

pub async fn apply_op<'a>(apply_op: ApplyOpRequest<'a>) -> OpResult<serde_json::Value> {
    let res = REQWEST
        .post("http://localhost:5157/applyOp")
        .json(&apply_op)
        .send()
        .await?;
    let apply_op_reply: ApplyOpReply = res.json().await?;
    Ok(apply_op_reply.object)
}
