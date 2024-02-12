//! This library exists to ensure that crate "rebaser-client" crate does not depend on the "rebaser-server" crate and
//! vice versa. Keeping the dependency chain intact is important because "rebaser-server" depends on the
//! dal. The dal, and any crate other than "rebaser-server" and this crate, must be able to use the "rebaser-client".

#![warn(
    bad_style,
    clippy::missing_panics_doc,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    dead_code,
    improper_ctypes,
    missing_debug_implementations,
    missing_docs,
    no_mangle_generic_items,
    non_shorthand_field_patterns,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    rust_2018_idioms,
    unconditional_recursion,
    unreachable_pub,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true
)]

use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use ulid::Ulid;

mod messaging_config;
mod subject;

pub use messaging_config::RebaserMessagingConfig;
pub use subject::SubjectGenerator;

/// The message that the server receives to perform a rebase.
#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub struct RequestRebaseMessage {
    /// Corresponds to the change set whose pointer is to be updated.
    pub to_rebase_change_set_id: Ulid,
    /// Corresponds to the workspace snapshot that will be the "onto" workspace snapshot when
    /// rebasing the "to rebase" workspace snapshot.
    pub onto_workspace_snapshot_id: Ulid,
    /// Derived from the ephemeral or persisted change set that's either the base change set, the
    /// last change set before edits were made, or the change set that you are trying to rebase
    /// onto base.
    pub onto_vector_clock_id: Ulid,
}

/// The message that the server sends back to the requester.
#[derive(Debug, Serialize, Deserialize)]
pub enum ReplyRebaseMessage {
    /// Processing the request and performing updates were both successful. Additionally, no conflicts were found.
    Success {
        /// The serialized updates performed when rebasing.
        updates_performed: Value,
    },
    /// Conflicts found when processing the request.
    ConflictsFound {
        /// A serialized list of the conflicts found during detection.
        conflicts_found: Value,
        /// A serialized list of the updates found during detection and skipped because at least
        /// once conflict was found.
        updates_found_and_skipped: Value,
    },
    /// Error encountered when processing the request.
    Error {
        /// The error message.
        message: String,
    },
}
