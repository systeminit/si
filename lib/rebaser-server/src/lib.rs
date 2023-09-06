//! This crate provides the rebaser [`Server`].

#![warn(
    missing_debug_implementations,
    missing_docs,
    unreachable_pub,
    bad_style,
    dead_code,
    improper_ctypes,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    private_in_public,
    unconditional_recursion,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true,
    clippy::missing_panics_doc
)]

mod config;
mod server;

pub use config::Config;
pub use config::ConfigBuilder;
pub use config::ConfigError;
pub use config::ConfigFile;
pub use server::Server;
pub use si_settings::StandardConfig;
pub use si_settings::StandardConfigFile;

use serde::{Deserialize, Serialize};
use ulid::Ulid;

/// Stream to manage rebaser consumer loops.
pub const REBASER_MANAGEMENT_STREAM: &str = "rebaser-management";

/// Stream prefix for rebaser consumer loops.
pub const REBASER_STREAM_PREFIX: &str = "rebaser";

/// The action for the rebaser management loop.
#[derive(Debug, Serialize, Deserialize)]
pub enum ManagementMessageAction {
    /// Close the inner rebaser loop for a change set. If it has already been closed, this is a
    /// no-op.
    Close,
    /// Open the inner rebaser loop for a change set. If one already exists, it is a no-op.
    Open,
}

/// The message that the rebaser management consumer expects in the server.
#[derive(Debug, Serialize, Deserialize)]
pub struct ManagementMessage {
    /// The ID of the change set wishing to be operated on.
    pub change_set_id: Ulid,
    /// The action to instruct the management loop to perform.
    pub action: ManagementMessageAction,
}
