mod config;
pub mod server;

pub use crate::{
    config::{
        detect_and_configure_development, Config, ConfigBuilder, ConfigError, ConfigFile,
        StandardConfig, StandardConfigFile,
    },
    server::{Server, ServerError},
};

const NATS_JOBS_DEFAULT_SUBJECT: &str = "pinga-jobs";
const NATS_JOBS_DEFAULT_QUEUE: &str = "pinga";

pub fn nats_jobs_subject(prefix: Option<&str>) -> String {
    nats_subject(prefix, NATS_JOBS_DEFAULT_SUBJECT)
}

fn nats_subject(prefix: Option<&str>, suffix: impl AsRef<str>) -> String {
    let suffix = suffix.as_ref();
    match prefix {
        Some(prefix) => format!("{prefix}.{suffix}"),
        None => suffix.to_string(),
    }
}
