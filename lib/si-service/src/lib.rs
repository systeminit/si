//! System Initiative common service/server support.

#![warn(
    clippy::unwrap_in_result,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    clippy::unwrap_used,
    clippy::panic,
    clippy::missing_panics_doc,
    clippy::panic_in_result_fn,
    missing_docs
)]
#![allow(
    clippy::missing_errors_doc,
    clippy::module_inception,
    clippy::module_name_repetitions
)]

pub mod rt;
pub mod shutdown;
pub mod startup;

pub use color_eyre::{
    self,
    Result,
    eyre::Error,
};
pub use telemetry_application;
pub use tokio_util::{
    sync::CancellationToken,
    task::TaskTracker,
};

/// A "prelude" for crates implementing services/server binaries.
pub mod prelude {
    pub use std::future::IntoFuture as _;

    pub use color_eyre::{
        self,
        Result,
    };
    pub use si_runtime::{
        CoreId,
        CoreIds,
        get_cpu_cores,
        get_cpu_cores_from_range_expr,
    };
    pub use si_std::SensitiveString;
    pub use telemetry_application::{
        self,
        prelude::*,
    };
    pub use tokio_util::{
        sync::CancellationToken,
        task::TaskTracker,
    };

    pub use super::{
        rt,
        shutdown,
        startup,
    };

    // NOTE(nick): if we decide to restore/reuse tokio watchdog, we should provide it through the
    // service prelude again.
    // pub use tokio_watchdog;
}
