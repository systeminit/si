use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::{from_config, PosthogClient, PosthogError, PosthogResult, PosthogSender};

const DEFAULT_API_ENDPOINT: &str = "https://e.systeminit.com";
const DEFAULT_API_KEY: &str = "phc_SoQak5PP054RdTumd69bOz7JhM0ekkxxTXEQsbn3Zg9";
const DEFAULT_REQUEST_TIMEOUT_MS: u64 = 800;

#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
#[builder(default, build_fn(error = "PosthogError", name = "_build"))]
pub struct PosthogConfig {
    #[builder(setter(into))]
    api_endpoint: String,
    #[builder(setter(into))]
    api_key: String,
    request_timeout_ms: u64,
    #[builder(setter(into))]
    enabled: bool,
}

impl PosthogConfig {
    pub fn api_endpoint(&self) -> &str {
        self.api_endpoint.as_ref()
    }

    pub fn api_key(&self) -> &str {
        self.api_key.as_ref()
    }

    pub fn request_timeout_ms(&self) -> u64 {
        self.request_timeout_ms
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }
}

impl Default for PosthogConfig {
    fn default() -> Self {
        Self {
            api_endpoint: DEFAULT_API_ENDPOINT.to_string(),
            api_key: DEFAULT_API_KEY.to_string(),
            request_timeout_ms: DEFAULT_REQUEST_TIMEOUT_MS,
            enabled: true,
        }
    }
}

impl PosthogConfigBuilder {
    pub fn build(&self) -> PosthogResult<(PosthogClient, PosthogSender)> {
        let config = self._build()?;
        from_config(&config)
    }
}
