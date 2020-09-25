use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::data::{Connection, Db, REQWEST};
use crate::models::{
    insert_model, Entity, ModelError, SiChangeSet, SiChangeSetError, SiChangeSetEvent, SiStorable,
    SiStorableError,
};

#[derive(Error, Debug)]
pub enum OpError {
    #[error("si_storable error: {0}")]
    SiStorable(#[from] SiStorableError),
    #[error("si_change_set error: {0}")]
    SiChangeSet(#[from] SiChangeSetError),
    #[error("error in core model functions: {0}")]
    Model(#[from] ModelError),
    #[error("cannot set value: path({0}) value({1})")]
    Failed(String, serde_json::Value),
    #[error("malformed target")]
    MalformedTarget,
    #[error("error making http call: {0}")]
    Reqwest(#[from] reqwest::Error),
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum OpRequest {
    EntitySetString(OpEntitySetStringRequest),
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OpReply {
    pub item_ids: Vec<String>,
}

pub type OpResult<T> = Result<T, OpError>;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SiOp {
    skip: bool,
    override_system: Option<String>,
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
pub struct OpEntitySetStringRequest {
    pub path: String,
    pub value: String,
    pub override_system: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OpEntitySetString {
    pub id: String,
    pub to_id: String,
    pub path: String,
    pub value: String,
    pub si_op: SiOp,
    pub si_storable: SiStorable,
    pub si_change_set: SiChangeSet,
}

impl OpEntitySetString {
    pub async fn new(
        db: &Db,
        nats: &Connection,
        to_id: impl Into<String>,
        path: impl Into<String>,
        value: impl Into<String>,
        override_system: Option<String>,
        billing_account_id: String,
        organization_id: String,
        workspace_id: String,
        change_set_id: String,
        edit_session_id: String,
        created_by_user_id: String,
    ) -> OpResult<Self> {
        let to_id = to_id.into();
        let path = path.into();
        let value = value.into();
        let si_storable = SiStorable::new(
            db,
            "opEntitySetString",
            billing_account_id.clone(),
            organization_id,
            workspace_id,
            Some(created_by_user_id),
        )
        .await?;

        let id = si_storable.object_id.clone();

        let si_change_set = SiChangeSet::new(
            db,
            nats,
            change_set_id,
            edit_session_id,
            &id,
            billing_account_id,
            SiChangeSetEvent::Operation,
        )
        .await?;

        let si_op = SiOp::new(override_system);

        let op = OpEntitySetString {
            id,
            to_id,
            path,
            value,
            si_op,
            si_storable,
            si_change_set,
        };
        insert_model(db, nats, &op.id, &op).await?;
        Ok(op)
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

        let apply_req = ApplyOpRequest::new(
            ApplyOperation::Set,
            &self.to_id,
            format!("manualProperties.{}.{}", override_system, self.path),
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
        db: &Db,
        nats: &Connection,
        to_id: impl Into<String>,
        value: impl Into<String>,
        billing_account_id: String,
        organization_id: String,
        workspace_id: String,
        change_set_id: String,
        edit_session_id: String,
        created_by_user_id: String,
    ) -> OpResult<Self> {
        let to_id = to_id.into();
        let value = value.into();
        let si_storable = SiStorable::new(
            db,
            "opSetName",
            billing_account_id.clone(),
            organization_id,
            workspace_id,
            Some(created_by_user_id),
        )
        .await?;

        let id = si_storable.object_id.clone();

        let si_change_set = SiChangeSet::new(
            db,
            nats,
            change_set_id,
            edit_session_id,
            &id,
            billing_account_id,
            SiChangeSetEvent::Operation,
        )
        .await?;

        let si_op = SiOp::new(None);

        let op = OpSetName {
            id,
            to_id,
            value,
            si_op,
            si_storable,
            si_change_set,
        };
        insert_model(db, nats, &op.id, &op).await?;
        Ok(op)
    }

    pub fn skip(&self) -> bool {
        self.si_op.skip()
    }

    pub async fn apply<'a>(&'a self, to: &'a mut serde_json::Value) -> OpResult<()> {
        if self.skip() {
            return Ok(());
        }
        let apply_req = ApplyOpRequest::new(
            ApplyOperation::Set,
            &self.to_id,
            "name",
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
pub enum ApplyOperation {
    Set,
    Unset,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ApplyOpRequest<'a> {
    operation: ApplyOperation,
    to_id: String,
    path: String,
    value: Option<serde_json::Value>,
    object: &'a serde_json::Value,
}

impl<'a> ApplyOpRequest<'a> {
    pub fn new(
        operation: ApplyOperation,
        to_id: impl Into<String>,
        path: impl Into<String>,
        value: Option<serde_json::Value>,
        object: &'a serde_json::Value,
    ) -> ApplyOpRequest {
        let to_id = to_id.into();
        let path = path.into();
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
