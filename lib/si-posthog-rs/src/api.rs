use serde::Serialize;

use crate::error::{PosthogError, PosthogResult};

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
