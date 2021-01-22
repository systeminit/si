use crate::data::{NatsTxn, PgTxn};

use crate::models::{
    next_update_clock, OpError, OpResult, SiChangeSet, SiChangeSetEvent, SiOp, SiStorable,
};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OpEntityDeleteRequest {
    pub cascade: bool,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OpEntityDelete {
    pub id: String,
    pub to_id: String,
    pub si_op: SiOp,
    pub si_storable: SiStorable,
    pub si_change_set: SiChangeSet,
}

impl OpEntityDelete {
    pub async fn new(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        to_id: impl AsRef<str>,
        workspace_id: impl AsRef<str>,
        change_set_id: impl AsRef<str>,
        edit_session_id: impl AsRef<str>,
    ) -> OpResult<Self> {
        let to_id = to_id.as_ref();
        let workspace_id = workspace_id.as_ref();
        let change_set_id = change_set_id.as_ref();
        let edit_session_id = edit_session_id.as_ref();

        let workspace_update_clock = next_update_clock(workspace_id).await?;
        let change_set_update_clock = next_update_clock(change_set_id).await?;

        let override_system: Option<String> = None;

        let row = txn
            .query_one(
                "SELECT object FROM op_create_v1($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
                &[
                    &"opEntityDelete",
                    &to_id,
                    &serde_json::json![{}],
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
        match to["siStorable"].as_object_mut() {
            Some(si_storable) => {
                si_storable.insert("deleted".into(), serde_json::Value::Bool(true));
            }
            None => return Err(OpError::MalformedTarget),
        }
        Ok(())
    }
}
