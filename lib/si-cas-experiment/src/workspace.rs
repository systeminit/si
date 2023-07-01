//use std::ops::Deref;

use ulid::Ulid;

use crate::{OriginId, ContentHash};

//#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Copy, Hash)]
//pub struct WorkspacePk(Ulid);
//
//impl WorkspacePk {
//    pub fn new() -> WorkspacePk {
//        WorkspacePk(Ulid::new())
//    }
//
//    pub fn inner(self) -> Ulid {
//        self.0
//    }
//}
//
//impl Deref for WorkspacePk {
//    type Target = Ulid;
//    fn deref(&self) -> &Ulid {
//        &self.0
//    }
//}
//
//impl std::fmt::Display for WorkspacePk {
//    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//       write!(f, "{}", self.0)
//    }
//}

pub type WorkspacePk = Ulid;
pub type WorkspaceId = Ulid;

#[derive(Debug, Clone, PartialEq)]
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

    pub fn id(&self) -> WorkspaceId {
        self.id
    }

    pub fn pk(&self) -> WorkspacePk {
        self.pk
    }
}
