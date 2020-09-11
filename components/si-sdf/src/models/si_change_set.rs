use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::data::Db;
use crate::models::{UpdateClock, UpdateClockError};

#[derive(Error, Debug)]
pub enum SiChangeSetError {
    #[error("update count error: {0}")]
    UpdateCount(#[from] UpdateClockError),
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
        change_set_id: impl Into<String>,
        edit_session_id: impl Into<String>,
        event: SiChangeSetEvent,
    ) -> SiChangeSetResult<SiChangeSet> {
        let change_set_id = change_set_id.into();
        let edit_session_id = edit_session_id.into();
        let order_clock = UpdateClock::create_or_update(db, &change_set_id, 0).await?;
        Ok(SiChangeSet {
            change_set_id,
            edit_session_id,
            event,
            order_clock,
        })
    }
}
