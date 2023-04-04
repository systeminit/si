use tokio::sync::mpsc::UnboundedSender;

use crate::{
    api::{PosthogApiEvent, PosthogMessage},
    error::PosthogResult,
};

#[derive(Debug, Clone)]
pub struct PosthogClient {
    tx: UnboundedSender<PosthogMessage>,
}

impl PosthogClient {
    pub fn new(tx: UnboundedSender<PosthogMessage>) -> PosthogClient {
        PosthogClient { tx }
    }

    pub fn capture(
        &self,
        event_name: impl Into<String>,
        distinct_id: impl Into<String>,
        properties: impl Into<serde_json::Value>,
    ) -> PosthogResult<()> {
        let event = PosthogApiEvent::new(event_name.into(), distinct_id.into(), properties.into())?;
        self.tx.send(PosthogMessage::Event(event))?;
        Ok(())
    }

    pub fn disable(&self) -> PosthogResult<()> {
        self.tx.send(PosthogMessage::Disable)?;
        Ok(())
    }
}
