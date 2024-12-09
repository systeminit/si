use serde::{Deserialize, Serialize};

pub use si_id::ChangeSetId;
pub use si_id::WorkspacePk;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Tenancy {
    pub change_set_id: ChangeSetId,
    pub workspace_pk: WorkspacePk,
}

impl Tenancy {
    pub fn new(workspace_pk: WorkspacePk, change_set_id: ChangeSetId) -> Self {
        Tenancy {
            change_set_id,
            workspace_pk,
        }
    }
}
