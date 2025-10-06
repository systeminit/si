use dal::DalContext;
use hyper::Uri;
use si_id::{
    ChangeSetId,
    WorkspacePk,
};
use telemetry::prelude::*;

use crate::app_state::PosthogClient;

/// Send a tracking event to PostHog with properties derived from the current DalContext
pub fn track(
    posthog_client: &PosthogClient,
    ctx: &DalContext,
    original_uri: &Uri,
    host_name: &String,
    event_name: impl AsRef<str>,
    properties: serde_json::Value,
) {
    let distinct_id = ctx.history_actor().distinct_id();
    let workspace_id = ctx
        .tenancy()
        .workspace_pk_opt()
        .map(|workspace_pk| workspace_pk.to_string())
        .unwrap_or_else(|| "unknown".to_string());
    let changeset_id = ctx.change_set_id().to_string();
    _track(
        posthog_client,
        original_uri,
        host_name,
        distinct_id,
        Some(workspace_id),
        Some(changeset_id),
        event_name,
        properties,
    )
}

/// Send tracking events to PostHog when you either don't have a DalContext
/// or when the DalContext does not have the correct workspace and change_set id
/// (e.g., for admin routes that operate across workspaces)
#[allow(clippy::too_many_arguments)]
pub fn track_no_ctx(
    posthog_client: &PosthogClient,
    original_uri: &Uri,
    host_name: &String,
    distinct_id: String,
    workspace_id: WorkspacePk,
    changeset_id: ChangeSetId,
    event_name: impl AsRef<str>,
    properties: serde_json::Value,
) {
    _track(
        posthog_client,
        original_uri,
        host_name,
        distinct_id,
        Some(workspace_id.to_string()),
        Some(changeset_id.to_string()),
        event_name,
        properties,
    )
}

/// Send tracking events to PostHog when you either don't have a DalContext
/// or when the DalContext does not have the correct workspace and change_set id
/// (e.g., for admin routes that operate across workspaces)
#[allow(clippy::too_many_arguments)]
pub fn track_no_ctx_workspace(
    posthog_client: &PosthogClient,
    original_uri: &Uri,
    host_name: &String,
    distinct_id: String,
    workspace_id: WorkspacePk,
    event_name: impl AsRef<str>,
    properties: serde_json::Value,
) {
    _track(
        posthog_client,
        original_uri,
        host_name,
        distinct_id,
        Some(workspace_id.to_string()),
        None,
        event_name,
        properties,
    )
}

/// Send tracking events to PostHog when you either don't have a DalContext
/// or when the DalContext does not have the correct workspace and change_set id
/// (e.g., for admin routes that operate across workspaces)
#[allow(clippy::too_many_arguments)]
pub fn _track(
    posthog_client: &PosthogClient,
    original_uri: &Uri,
    host_name: &String,
    distinct_id: String,
    workspace_id: Option<String>,
    changeset_id: Option<String>,
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

    let current_url = original_uri.to_string();

    if let Some(prop_map) = properties.as_object_mut() {
        if let Some(workspace_id) = workspace_id {
            prop_map.insert("workspace_id".to_string(), serde_json::json!(workspace_id));
        }
        if let Some(changeset_id) = changeset_id {
            prop_map.insert("changeset_id".to_string(), serde_json::json!(changeset_id));
        }
        prop_map.insert("$current_url".to_string(), serde_json::json!(current_url));
        prop_map.insert("host".to_string(), serde_json::json!(host_name));

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
