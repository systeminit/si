use serde::{Deserialize, Serialize};
use strum_macros::Display;
use thiserror::Error;

use crate::data::{NatsTxn, NatsTxnError, PgTxn};
use crate::models::{
    ChangeSetParticipant, Edge, EdgeError, EdgeKind, UpdateClock, UpdateClockError,
};

//use crate::data::{Connection, Db};
//use crate::models::{
//    ChangeSetParticipant, Edge, EdgeError, EdgeKind, UpdateClock, UpdateClockError,
//};

#[derive(Error, Debug)]
pub enum SiChangeSetError {
    #[error("change set participation error: {0}")]
    ChangeSetParticipant(String),
    #[error("edge error: {0}")]
    Edge(#[from] EdgeError),
    #[error("pg error: {0}")]
    TokioPg(#[from] tokio_postgres::Error),
    #[error("nats txn error: {0}")]
    NatsTxn(#[from] NatsTxnError),
    #[error("update clock: {0}")]
    UpdateClock(#[from] UpdateClockError),
    #[error("error creating our object from json: {0}")]
    SerdeJson(#[from] serde_json::Error),
}

pub type SiChangeSetResult<T> = Result<T, SiChangeSetError>;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Display)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum SiChangeSetEvent {
    Create,
    Delete,
    Operation,
    Action,
    Projection,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SiChangeSet {
    pub change_set_id: String,
    pub edit_session_id: String,
    pub event: SiChangeSetEvent,
    pub order_clock: UpdateClock,
}

impl SiChangeSet {
    pub async fn create_change_set_participants(
        &self,
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        object_id: impl AsRef<str>,
        workspace_id: impl AsRef<str>,
    ) -> SiChangeSetResult<()> {
        let change_set_id: &str = self.change_set_id.as_ref();
        let object_id = object_id.as_ref();
        let workspace_id = workspace_id.as_ref();

        let inserted = ChangeSetParticipant::new_if_not_exists(
            &txn,
            &nats,
            &change_set_id,
            &object_id,
            &workspace_id,
        )
        .await
        .map_err(|e| SiChangeSetError::ChangeSetParticipant(e.to_string()))?;

        if inserted.is_some() {
            let edges =
                Edge::all_predecessor_edges_by_object_id(&txn, &EdgeKind::Configures, object_id)
                    .await?;
            for edge in edges.iter() {
                ChangeSetParticipant::new_if_not_exists(
                    &txn,
                    &nats,
                    &change_set_id,
                    &edge.tail_vertex.object_id,
                    &workspace_id,
                )
                .await
                .map_err(|e| SiChangeSetError::ChangeSetParticipant(e.to_string()))?;
            }
        }
        Ok(())
    }
}
