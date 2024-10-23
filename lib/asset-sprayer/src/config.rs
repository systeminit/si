use async_openai::config::OpenAIConfig;
use serde::{Deserialize, Serialize};
use si_std::SensitiveString;
use std::path::PathBuf;

/// OpenAI configuration. Mirrors OpenAI configuration, but with Deserialize support.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SIOpenAIConfig {
    api_key: Option<SensitiveString>,
    api_base: Option<String>,
    org_id: Option<String>,
    project_id: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct AssetSprayerConfig {
    pub prompts_dir: Option<PathBuf>,
}

impl SIOpenAIConfig {
    pub fn into_openai_config_opt(self) -> Option<OpenAIConfig> {
        if self.api_key.is_none()
            && self.api_base.is_none()
            && self.org_id.is_none()
            && self.project_id.is_none()
        {
            return None;
        }

        let mut config = OpenAIConfig::new();
        if let Some(api_key) = self.api_key {
            config = config.with_api_key(api_key);
        }
        if let Some(api_base) = self.api_base {
            config = config.with_api_base(api_base);
        }
        if let Some(org_id) = self.org_id {
            config = config.with_org_id(org_id);
        }
        if let Some(project_id) = self.project_id {
            config = config.with_project_id(project_id);
        }
        Some(config)
    }
}
