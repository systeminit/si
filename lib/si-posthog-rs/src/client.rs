use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::time::Duration;
use strum::Display;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::Mutex;
use tokio::time::Instant;

use crate::api::PosthogApiClient;
use crate::{
    api::{PosthogApiEvent, PosthogMessage},
    error::PosthogResult,
    PosthogConfig,
};

#[derive(Debug, Display)]
#[strum(serialize_all = "snake_case")]
pub enum FeatureFlag {
    Secrets,
}

#[derive(Debug)]
struct FlagsCacheEntry {
    retrieved_at: Instant,
    flags: HashMap<String, bool>,
}

static FLAGS_CACHE: Lazy<Mutex<HashMap<String, FlagsCacheEntry>>> = Lazy::new(Mutex::default);

#[derive(Debug, Clone)]
pub struct PosthogClient {
    tx: UnboundedSender<PosthogMessage>,
    api_client: PosthogApiClient,
}

impl PosthogClient {
    pub fn new(
        tx: UnboundedSender<PosthogMessage>,
        config: &PosthogConfig,
    ) -> PosthogResult<PosthogClient> {
        let api_client = PosthogApiClient::new(config)?;
        Ok(PosthogClient { tx, api_client })
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

    pub async fn check_feature_flag(
        &self,
        flag: FeatureFlag,
        user_id: String,
    ) -> PosthogResult<bool> {
        let mut cache = FLAGS_CACHE.lock().await;
        let maybe_flags_cache = cache.get(&user_id);

        let flags = if maybe_flags_cache
            .is_some_and(|e| e.retrieved_at.elapsed() < Duration::from_secs(10))
        {
            maybe_flags_cache.unwrap().flags.clone()
        } else {
            let flags = self.api_client.load_feature_flags(&user_id).await?;
            cache.insert(
                user_id,
                FlagsCacheEntry {
                    retrieved_at: Instant::now(),
                    flags: flags.clone(),
                },
            );

            flags
        };

        Ok(*flags.get(&flag.to_string()).unwrap_or(&false))
    }
}
