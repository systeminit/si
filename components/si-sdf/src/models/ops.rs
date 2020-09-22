use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::data::Db;
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
    pub pointer: String,
    pub value: String,
    pub override_system: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OpEntitySetString {
    pub id: String,
    pub entity_id: String,
    pub pointer: String,
    pub value: String,
    pub si_op: SiOp,
    pub si_storable: SiStorable,
    pub si_change_set: SiChangeSet,
}

impl OpEntitySetString {
    pub async fn new(
        db: &Db,
        entity_id: impl Into<String>,
        pointer: impl Into<String>,
        value: impl Into<String>,
        override_system: Option<String>,
        billing_account_id: String,
        organization_id: String,
        workspace_id: String,
        change_set_id: String,
        edit_session_id: String,
        created_by_user_id: String,
    ) -> OpResult<Self> {
        let entity_id = entity_id.into();
        let pointer = pointer.into();
        let value = value.into();
        let si_storable = SiStorable::new(
            db,
            "opEntitySetString",
            billing_account_id,
            organization_id,
            workspace_id,
            Some(created_by_user_id),
        )
        .await?;

        let id = si_storable.object_id.clone();

        let si_change_set = SiChangeSet::new(
            db,
            change_set_id,
            edit_session_id,
            SiChangeSetEvent::Operation,
        )
        .await?;

        let si_op = SiOp::new(override_system);

        let op = OpEntitySetString {
            id,
            entity_id,
            pointer,
            value,
            si_op,
            si_storable,
            si_change_set,
        };
        insert_model(db, &op.id, &op).await?;
        Ok(op)
    }

    pub fn skip(&self) -> bool {
        self.si_op.skip()
    }

    pub fn apply(&self, entity: &mut Entity) -> OpResult<()> {
        let fallback_system = String::from("__baseline");
        let override_system = match self.si_op.override_system {
            Some(ref override_system) => override_system,
            None => &fallback_system,
        };
        let properties = entity.manual_properties.get_or_create_mut(&override_system);

        match properties.pointer_mut(&self.pointer) {
            Some(property) => {
                let replaced =
                    std::mem::replace(property, serde_json::Value::String(self.value.clone()));
                tracing::debug!(?replaced, ?self.value, "replaced string");
            }
            None => {
                let part_number = self.pointer.rfind('/');
                // TODO: You know this is buggy. I *think* its okay, because essentially we won't
                //       allow you to call it if it parents don't exist (because of the nature of
                //       our structured editing - you would have to add the object before the
                //       string!)
                //
                //       We make an exception for top level strings, but we make no attempt to
                //       safely do that. so it's pretty sketchy, but moves us forward.
                if part_number.is_some() && part_number.unwrap() == 0 {
                    let mut new_pointer = self.pointer.clone();
                    new_pointer.remove(0);
                    if let Some(mprop) = properties.as_object_mut() {
                        mprop.insert(new_pointer, serde_json::Value::String(self.value.clone()));
                    } else {
                        return Err(OpError::Failed(
                            self.pointer.clone(),
                            serde_json::json![self.value],
                        ));
                    }
                } else {
                    return Err(OpError::Failed(
                        self.pointer.clone(),
                        serde_json::json![self.value],
                    ));
                }
            }
        }
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
            billing_account_id,
            organization_id,
            workspace_id,
            Some(created_by_user_id),
        )
        .await?;

        let id = si_storable.object_id.clone();

        let si_change_set = SiChangeSet::new(
            db,
            change_set_id,
            edit_session_id,
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
        insert_model(db, &op.id, &op).await?;
        Ok(op)
    }

    pub fn skip(&self) -> bool {
        self.si_op.skip()
    }

    pub fn apply(&self, to: &mut serde_json::Value) -> OpResult<()> {
        if let Some(name) = to.pointer_mut("/name") {
            if let Some(name_string) = name.as_str() {
                if name_string != self.value {
                    *name = serde_json::json!(self.value);
                }
            }
        }
        Ok(())
    }
}
