use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct S3Config {
    pub access_key_id: Option<String>,
    pub secret_access_key: Option<String>,
    pub region: String,
    pub bucket: String,
    pub path_prefix: String,
}

impl Default for S3Config {
    fn default() -> Self {
        S3Config {
            access_key_id: None,
            secret_access_key: None,
            region: String::from("us-east-2"),
            bucket: String::from("modules-index-sandbox"),
            // TODO? is this right?
            path_prefix: String::from("dev"),
        }
    }
}
