#![recursion_limit = "256"]
#![warn(
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    clippy::panic,
    clippy::missing_panics_doc,
    clippy::panic_in_result_fn
)]
#![allow(
    clippy::missing_errors_doc,
    clippy::module_inception,
    clippy::module_name_repetitions
)]

mod server;
pub use server::{
    build_service, build_service_for_tests, detect_and_configure_development,
    job_processor::JobProcessorClientCloser, job_processor::JobProcessorConnector, service, Config,
    ConfigError, ConfigFile, IncomingStream, JobQueueProcessor, MigrationMode, NatsProcessor,
    Server, ServicesContext, StandardConfig, StandardConfigFile,
};
