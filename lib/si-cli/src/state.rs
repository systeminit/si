use axum::extract::FromRef;
use std::env;
use std::ops::Deref;
use std::sync::Arc;
use telemetry::tracing;

pub struct AppState {
    posthog_client: PosthogClient,
    version: Arc<str>,
    mode: Arc<str>,
    is_preview: bool,
    bind_host: String,
    bind_port: u32,
}

impl AppState {
    pub fn new(
        posthog_client: impl Into<PosthogClient>,
        version: Arc<str>,
        mode: Arc<str>,
        is_preview: bool,
        bind_host: String,
        bind_port: u32,
    ) -> Self {
        Self {
            posthog_client: posthog_client.into(),
            version,
            mode,
            is_preview,
            bind_host,
            bind_port,
        }
    }

    pub fn version(&self) -> &str {
        self.version.deref()
    }

    pub fn mode(&self) -> &str {
        self.mode.deref()
    }

    pub fn is_preview(&self) -> bool {
        self.is_preview
    }

    pub fn bind_host(&self) -> String {
        self.bind_host.clone()
    }

    pub fn bind_port(&self) -> u32 {
        self.bind_port
    }

    pub fn posthog_client(&self) -> &PosthogClient {
        &self.posthog_client
    }

    pub fn track(&self, distinct_id: String, mut properties: serde_json::Value) {
        if !properties.is_object() {
            tracing::error!(
                "tracking call without a json object as properties: {:?}",
                &properties
            );
            return;
        }

        let ph_client = self.posthog_client();

        let prop_map = properties
            .as_object_mut()
            .expect("properties is not an object; should be impossible, checked above");
        prop_map.insert("si-version".to_string(), serde_json::json!(self.version()));
        prop_map.insert("mode".to_string(), serde_json::json!(self.mode()));
        prop_map.insert("os".to_string(), serde_json::json!(env::consts::OS));
        prop_map.insert("arch".to_string(), serde_json::json!(env::consts::ARCH));

        ph_client
            .capture("si-command", distinct_id, properties)
            .unwrap_or_else(|e| tracing::warn!("cannot send event to posthog: {:?}", e));
    }
}

#[derive(Clone, Debug, FromRef)]
pub struct PosthogClient(si_posthog::PosthogClient);

impl PosthogClient {
    pub fn into_inner(self) -> si_posthog::PosthogClient {
        self.into()
    }
}

impl Deref for PosthogClient {
    type Target = si_posthog::PosthogClient;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<si_posthog::PosthogClient> for PosthogClient {
    fn from(value: si_posthog::PosthogClient) -> Self {
        Self(value)
    }
}

impl From<PosthogClient> for si_posthog::PosthogClient {
    fn from(value: PosthogClient) -> Self {
        value.0
    }
}
