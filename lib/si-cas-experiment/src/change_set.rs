use ulid::Ulid;

use crate::workspace::WorkspacePk;

pub type ChangeSetPk = Ulid;

#[derive(Debug, Clone)]
pub struct ChangeSet {
    pub pk: ChangeSetPk,
    pub name: String,
    pub base_workspace_pk: WorkspacePk,
    pub target_change_set_name: String,
}

impl ChangeSet {
    pub fn new(
        name: impl Into<String>,
        target_change_set_name: impl Into<String>,
        base_workspace_pk: WorkspacePk,
    ) -> ChangeSet {
        let name = name.into();
        let target_change_set_name = target_change_set_name.into();
        ChangeSet {
            pk: ChangeSetPk::new(),
            name,
            base_workspace_pk,
            target_change_set_name,
        }
    }

    pub fn pk(&self) -> ChangeSetPk {
        self.pk
    }
}
