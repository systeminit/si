use serde::{Deserialize, Serialize};
use si_events::ChangeSetId;

use crate::change_set::ChangeSet;

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceMetadata {
    pub name: String,
    pub id: String,
    pub default_change_set_id: ChangeSetId,
    pub change_sets: Vec<ChangeSet>,
    /// list of user ids that are approvers for this workspace
    pub approvers: Vec<String>,
}
