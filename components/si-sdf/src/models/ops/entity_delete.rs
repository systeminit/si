use crate::data::{Connection, Db};
use crate::models::{
    insert_model, OpError, OpResult, SiChangeSet, SiChangeSetEvent, SiOp, SiStorable,
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
        db: &Db,
        nats: &Connection,
        to_id: impl Into<String>,
        billing_account_id: String,
        organization_id: String,
        workspace_id: String,
        change_set_id: String,
        edit_session_id: String,
        created_by_user_id: String,
    ) -> OpResult<Self> {
        let to_id = to_id.into();
        let si_storable = SiStorable::new(
            db,
            "opEntityDelete",
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

        let op = OpEntityDelete {
            id,
            to_id,
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
        match to["siStorable"].as_object_mut() {
            Some(si_storable) => {
                si_storable.insert("deleted".into(), serde_json::Value::Bool(true));
            }
            None => return Err(OpError::MalformedTarget),
        }
        Ok(())
    }
}
