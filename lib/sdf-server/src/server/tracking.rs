use dal::DalContext;
use hyper::Uri;
use telemetry::prelude::*;

use super::state::PosthogClient;

pub fn track(
    posthog_client: &PosthogClient,
    ctx: &DalContext,
    original_uri: &Uri,
    event_name: impl AsRef<str>,
    mut properties: serde_json::Value,
) {
    if !properties.is_object() {
        error!(
            "tracking call without a json object as properties: {:?}",
            &properties
        );
        return;
    }
    let distinct_id = ctx.history_actor().distinct_id();
    let workspace_id = ctx
        .tenancy()
        .workspace_pk()
        .map(|workspace_pk| workspace_pk.to_string())
        .unwrap_or_else(|| "unknown".to_string());
    let changeset_id = ctx.visibility().change_set_pk.to_string();
    let current_url = original_uri.to_string();
    if let Some(prop_map) = properties.as_object_mut() {
        prop_map.insert("workspace_id".to_string(), serde_json::json!(workspace_id));
        prop_map.insert("$current_url".to_string(), serde_json::json!(current_url));
        prop_map.insert("changeset_id".to_string(), serde_json::json!(changeset_id));

        posthog_client
            .capture(
                format!("sdf-{event_name}", event_name = event_name.as_ref()),
                distinct_id,
                properties,
            )
            .unwrap_or_else(|e| warn!("cannot send event to posthog: {:?}", e));
    } else {
        debug!(
            properties = %properties,
            "properties is not an object but was expected to be an object"
        );
    }
}
