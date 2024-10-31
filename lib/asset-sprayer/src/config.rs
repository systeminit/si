use async_openai::config::OpenAIConfig;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, NoneAsEmptyString};
use si_std::SensitiveString;

/// OpenAI configuration. Mirrors OpenAI configuration, but with Deserialize support.
#[serde_as]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SIOpenAIConfig {
    #[serde_as(as = "NoneAsEmptyString")]
    api_key: Option<SensitiveString>,
    #[serde_as(as = "NoneAsEmptyString")]
    api_base: Option<String>,
    #[serde_as(as = "NoneAsEmptyString")]
    org_id: Option<String>,
    #[serde_as(as = "NoneAsEmptyString")]
    project_id: Option<String>,
}

#[serde_as]
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct AssetSprayerConfig {
    #[serde_as(as = "NoneAsEmptyString")]
    pub prompts_dir: Option<String>,
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
