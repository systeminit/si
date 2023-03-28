use tokio::sync::mpsc::UnboundedSender;

use crate::{
    api::{PosthogApi, PosthogApiEvent},
    error::PosthogResult,
};

#[derive(Debug, Clone)]
pub struct PosthogClient {
    tx: UnboundedSender<PosthogApi>,
}

impl PosthogClient {
    pub fn new(tx: UnboundedSender<PosthogApi>) -> PosthogClient {
        PosthogClient { tx }
    }

    pub fn capture(
        &self,
        event_name: impl Into<String>,
        distinct_id: impl Into<String>,
        properties: impl Into<serde_json::Value>,
    ) -> PosthogResult<()> {
        let event = PosthogApiEvent::new(event_name.into(), distinct_id.into(), properties.into())?;
        self.tx.send(PosthogApi::Event(event))?;
        Ok(())
    }

    pub fn disable(&self) -> PosthogResult<()> {
        self.tx.send(PosthogApi::Disable)?;
        Ok(())
    }
}
