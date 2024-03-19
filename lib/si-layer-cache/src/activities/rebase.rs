use serde::{Deserialize, Serialize};
use serde_json::Value;
use ulid::Ulid;

/// The message that the server receives to perform a rebase.
#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq)]
pub struct RebaseRequest {
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

impl RebaseRequest {
    pub fn new(
        to_rebase_change_set_id: Ulid,
        onto_workspace_snapshot_id: Ulid,
        onto_vector_clock_id: Ulid,
    ) -> RebaseRequest {
        RebaseRequest {
            to_rebase_change_set_id,
            onto_workspace_snapshot_id,
            onto_vector_clock_id,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct RebaseFinished {
    status: RebaseStatus,
    to_rebase_change_set_id: Ulid,
    onto_workspace_snapshot_id: Ulid,
}

impl RebaseFinished {
    pub fn new(
        status: RebaseStatus,
        to_rebase_change_set_id: Ulid,
        onto_workspace_snapshot_id: Ulid,
    ) -> RebaseFinished {
        RebaseFinished {
            status,
            to_rebase_change_set_id,
            onto_workspace_snapshot_id,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum RebaseStatus {
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
