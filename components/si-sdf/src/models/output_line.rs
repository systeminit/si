use crate::data::PgTxn;
use crate::models::{Event, EventResult, SiStorable};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq, Display, EnumString)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum OutputLineStream {
    Stdout,
    Stderr,
    All,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OutputLine {
    pub line: String,
    pub stream: OutputLineStream,
    pub event_id: String,
    pub event_log_id: String,
    pub closed: bool,
    pub si_storable: SiStorable,
}

impl OutputLine {
    pub fn new(
        line: impl Into<String>,
        stream: OutputLineStream,
        event_id: impl Into<String>,
        event_log_id: impl Into<String>,
        closed: bool,
        si_storable: SiStorable,
    ) -> Self {
        let line = line.into();
        let event_id = event_id.into();
        let event_log_id = event_log_id.into();

        let output_line = OutputLine {
            line,
            stream,
            event_id,
            event_log_id,
            closed,
            si_storable,
        };

        output_line
    }

    pub async fn has_parent(
        &self,
        txn: &PgTxn<'_>,
        parent_id: impl AsRef<str>,
    ) -> EventResult<bool> {
        let parent_id = parent_id.as_ref();

        if self.event_id == parent_id {
            return Ok(true);
        }

        let event: Event = Event::get(txn, &self.event_id).await?;
        let has_parent = event.has_parent(txn, parent_id).await?;

        Ok(has_parent)
    }
}
