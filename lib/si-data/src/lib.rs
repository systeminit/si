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

#[cfg(feature = "event-log-fs")]
pub mod event_log_fs;
#[cfg(feature = "event-log-fs")]
pub use event_log_fs::{EventLogFS, EventLogFSError, OutputLineStream};

pub mod faktory;
pub use faktory::FaktoryConfig;

#[cfg(feature = "pg")]
pub mod pg;
#[cfg(feature = "pg")]
pub use pg::{Error as PgError, PgPool, PgPoolConfig, PgPoolError, PgTxn};

#[cfg(feature = "nats")]
pub mod nats;
#[cfg(feature = "nats")]
pub use nats::{Client as NatsClient, Error as NatsError, NatsConfig, NatsTxn};

mod sensitive_string;
pub use sensitive_string::SensitiveString;
