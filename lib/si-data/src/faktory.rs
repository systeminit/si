use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct FaktoryConfig {
    pub url: String,
}

impl Default for FaktoryConfig {
    fn default() -> Self {
        FaktoryConfig {
            url: "tcp://localhost:7419".to_string(),
        }
    }
}
