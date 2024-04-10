use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::diagram::DiagramResult;
use crate::{ActorView, DalContext, HistoryActorTimestamp};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEventMetadata {
    pub actor: ActorView,
    pub timestamp: DateTime<Utc>,
}

impl HistoryEventMetadata {
    pub async fn from_history_actor_timestamp(
        ctx: &DalContext,
        value: HistoryActorTimestamp,
    ) -> DiagramResult<Self> {
        let actor = ActorView::from_history_actor(ctx, value.actor).await?;

        Ok(Self {
            actor,
            timestamp: value.timestamp,
        })
    }
}
