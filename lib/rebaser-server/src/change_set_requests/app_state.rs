//! Application state for change set request tasks.

use dal::DalContextBuilder;
use si_events::{ChangeSetId, WorkspacePk};

/// Application state.
#[derive(Clone, Debug)]
pub struct AppState {
    /// Workspace ID for the task
    pub workspace_id: WorkspacePk,
    /// Change set ID for the task
    pub change_set_id: ChangeSetId,
    /// DAL context builder for each processing request
    pub ctx_builder: DalContextBuilder,
}

impl AppState {
    /// Creates a new [`AppState`].
    pub fn new(
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
        ctx_builder: DalContextBuilder,
    ) -> Self {
        Self {
            workspace_id,
            change_set_id,
            ctx_builder,
        }
    }
}
