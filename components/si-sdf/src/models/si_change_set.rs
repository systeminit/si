use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::data::{Connection, Db};
use crate::models::{
    ChangeSetParticipant, Edge, EdgeError, EdgeKind, UpdateClock, UpdateClockError,
};

#[derive(Error, Debug)]
pub enum SiChangeSetError {
    #[error("update count error: {0}")]
    UpdateCount(#[from] UpdateClockError),
    #[error("change set participation error: {0}")]
    ChangeSetParticipant(String),
    #[error("edge error: {0}")]
    Edge(#[from] EdgeError),
}

pub type SiChangeSetResult<T> = Result<T, SiChangeSetError>;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum SiChangeSetEvent {
    Create,
    Delete,
    Operation,
    Action,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SiChangeSet {
    pub change_set_id: String,
    pub edit_session_id: String,
    pub event: SiChangeSetEvent,
    pub order_clock: UpdateClock,
}

impl SiChangeSet {
    pub async fn new(
        db: &Db,
        nats: &Connection,
        change_set_id: impl Into<String>,
        edit_session_id: impl Into<String>,
        object_id: impl Into<String>,
        billing_account_id: impl Into<String>,
        event: SiChangeSetEvent,
    ) -> SiChangeSetResult<SiChangeSet> {
        let change_set_id = change_set_id.into();
        let edit_session_id = edit_session_id.into();
        let object_id = object_id.into();
        let billing_account_id = billing_account_id.into();
        let (_, inserted) = ChangeSetParticipant::new(
            &db,
            &nats,
            &change_set_id,
            &object_id,
            billing_account_id.clone(),
        )
        .await
        .map_err(|e| SiChangeSetError::ChangeSetParticipant(e.to_string()))?;

        if inserted {
            let edges =
                Edge::all_predecessor_edges_by_object_id(&db, EdgeKind::Configures, &object_id)
                    .await?;
            for edge in edges.iter() {
                ChangeSetParticipant::new(
                    &db,
                    &nats,
                    &change_set_id,
                    &edge.tail_vertex.object_id,
                    billing_account_id.clone(),
                )
                .await
                .map_err(|e| SiChangeSetError::ChangeSetParticipant(e.to_string()))?;
            }
        }

        let order_clock = UpdateClock::create_or_update(db, &change_set_id, 0).await?;

        Ok(SiChangeSet {
            change_set_id,
            edit_session_id,
            event,
            order_clock,
        })
    }
}
