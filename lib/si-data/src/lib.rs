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
pub use pg::{PgError, PgPool, PgPoolConfig, PgPoolError, PgRow, PgTxn};

mod sensitive_string;
pub use sensitive_string::SensitiveString;

mod result_ext;
pub use result_ext::ResultExt;

mod option_ext;
pub use option_ext::OptionExt;
