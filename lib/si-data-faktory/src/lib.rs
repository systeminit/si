#![warn(
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    clippy::panic,
    clippy::missing_panics_doc,
    clippy::panic_in_result_fn
)]
#![allow(clippy::missing_errors_doc)]

use serde::{Deserialize, Serialize};

pub use faktory_async::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct FaktoryConfig {
    pub url: String,
}

impl Default for FaktoryConfig {
    fn default() -> Self {
        FaktoryConfig {
            url: "localhost:7419".to_string(),
        }
    }
}
