use ulid::Ulid;

use crate::{OriginId, ContentHash};

pub type WorkspacePk = Ulid;
pub type WorkspaceId = Ulid;

#[derive(Debug, Clone)]
pub struct Workspace {
    pub id: WorkspaceId,
    pub pk: WorkspacePk,
    pub name: String,
    pub origin_id: OriginId,
    pub content_hash: ContentHash,
    pub previous_workspace: Option<WorkspacePk>,
}

impl Workspace {
    pub fn new(name: impl Into<String>) -> Workspace {
        let name = name.into();
        let id = WorkspaceId::new();
        let pk = WorkspacePk::new();
        let origin_id = OriginId::new();
        let mut hasher = blake3::Hasher::new();
        hasher.update(origin_id.to_string().as_bytes());
        let content_hash = hasher.finalize();

        Workspace {
            name,
            id,
            pk,
            origin_id,
            content_hash,
            previous_workspace: None,
        }
    }

    pub fn id(&self) -> WorkspacePk {
        self.id
    }

    pub fn pk(&self) -> WorkspacePk {
        self.pk
    }
}
