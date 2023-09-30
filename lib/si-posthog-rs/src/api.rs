use std::collections::HashMap;
use std::time::Duration;

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::error::{PosthogError, PosthogResult};
use crate::PosthogConfig;

#[remain::sorted]
#[derive(Debug, Serialize)]
pub enum PosthogMessage {
    Disable,
    Event(PosthogApiEvent),
}

#[derive(Debug, Serialize)]
pub struct PosthogApiEvent {
    event: String,
    properties: serde_json::Value,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeatureFlagsResponse {
    feature_flags: HashMap<String, bool>,
}

impl PosthogApiEvent {
    pub fn new(
        event: String,
        distinct_id: String,
        mut properties: serde_json::Value,
    ) -> PosthogResult<PosthogApiEvent> {
        if !properties.is_object() {
            return Err(PosthogError::PropertiesType);
        } else {
            // This is safe, because we just checked to see if its an object above.
            let prop_map = properties.as_object_mut().unwrap();
            prop_map.insert("distinct_id".to_string(), serde_json::json!(distinct_id));
            prop_map.insert("$lib".to_string(), serde_json::json!("si-posthog-rs"));
            prop_map.insert("$lib_version".to_string(), serde_json::json!("0.1"));
        }
        Ok(PosthogApiEvent { event, properties })
    }
}

#[derive(Debug, Clone)]
pub struct PosthogApiClient {
    reqwest: reqwest::Client,
    api_endpoint: String,
    api_key: String,
}

impl PosthogApiClient {
    pub(crate) fn new(config: &PosthogConfig) -> PosthogResult<PosthogApiClient> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(config.request_timeout_ms()))
            .build()?;

        Ok(PosthogApiClient {
            api_endpoint: config.api_endpoint().to_string(),
            api_key: config.api_key().to_string(),
            reqwest: client,
        })
    }

    pub async fn send_event(&self, event: PosthogApiEvent) -> PosthogResult<()> {
        let mut event_json = serde_json::to_value(event)?;

        event_json
            .as_object_mut()
            .expect("event was explicitly serialized from rust type as is therefore an object")
            .insert("api_key".to_string(), serde_json::json!(self.api_key));

        let response = self
            .reqwest
            .post(format!(
                "{api_endpoint}/capture",
                api_endpoint = self.api_endpoint
            ))
            .json(&event_json)
            .send()
            .await?;
        match response.status() {
            StatusCode::OK => Ok(()),
            error => Err(PosthogError::PosthogApi(error, response.text().await?)),
        }
    }

    pub async fn load_feature_flags(&self, user_id: &str) -> PosthogResult<HashMap<String, bool>> {
        let payload = HashMap::from([
            ("api_key".to_string(), self.api_key.clone()),
            ("distinct_id".to_string(), user_id.to_string()),
        ]);

        let response = self
            .reqwest
            .post(format!(
                "{api_endpoint}/decide?v=3",
                api_endpoint = self.api_endpoint
            ))
            .json(&serde_json::json!(payload))
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => Ok(response
                .json::<FeatureFlagsResponse>()
                .await
                .map(|r| r.feature_flags)?),
            error => Err(PosthogError::PosthogApi(error, response.text().await?)),
        }
    }
}
