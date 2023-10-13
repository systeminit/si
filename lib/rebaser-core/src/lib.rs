//! This library exists to ensure that rebaser-client does not depend on rebaser-server and vice
//! versa. Keeping the dependency chain intact is important because rebaser-server depends on the
//! dal and the dal (really anyone) must be able to use the rebaser-client.
//!
//! This library also contains tests for rebaser-client and rebaser-server interaction.

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
    unconditional_recursion,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true,
    clippy::missing_panics_doc
)]

use serde::Deserialize;
use serde::Serialize;
use ulid::Ulid;

/// Stream to manage rebaser consumer loops.
pub const REBASER_MANAGEMENT_STREAM: &str = "rebaser-management";

/// The action for the rebaser management loop.
#[derive(Debug, Serialize, Deserialize)]
pub enum ManagementMessageAction {
    /// Close the inner rebaser loop for a change set. If it has already been closed, this is a
    /// no-op.
    CloseChangeSet,
    /// Open the inner rebaser loop for a change set. If one already exists, it is a no-op.
    OpenChangeSet,
}

/// The message that the rebaser management consumer expects in the server.
#[derive(Debug, Serialize, Deserialize)]
pub struct ManagementMessage {
    /// The ID of the change set wishing to be operated on.
    pub change_set_id: Ulid,
    /// The action to instruct the management loop to perform.
    pub action: ManagementMessageAction,
}

/// The message that the rebaser change set consumer expects in the server.
#[derive(Debug, Serialize, Deserialize)]
pub struct ChangeSetMessage {
    /// Corresponds to the change set whose pointer is to be updated.
    pub change_set_to_update: Ulid,
    /// Corresponds to the workspace snapshot that will be rebased on top of the snapshot that the
    /// change set is currently pointing at.
    pub workspace_snapshot_to_rebase_on_top_of_current_snapshot_being_pointed_at: Ulid,
    /// Corresponds to the change set that's either the base change set, the last change set before
    /// edits were made, or the change set that you are trying to “merge” into the base.
    pub change_set_that_dictates_changes: Ulid,
}

/// The message shape that the rebaser change set loop will use for replying to the client.
#[derive(Debug, Serialize, Deserialize)]
pub enum ChangeSetReplyMessage {
    /// Processing the delivery was a success.
    Success {
        /// The results of processing the delivery.
        results: String,
    },
    /// Processing the delivery was a failure.
    Failure {
        /// The error encountered when processing the delivery.
        error: String,
    },
}
