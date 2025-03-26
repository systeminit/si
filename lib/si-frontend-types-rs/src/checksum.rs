use chrono::{DateTime, Utc};
use si_events::{
    workspace_snapshot::{Checksum, ChecksumHasher},
    ChangeSetStatus, Timestamp,
};
use si_id::{ChangeSetId, ViewId, WorkspaceId};

pub trait FrontendChecksum {
    fn checksum(&self) -> Checksum;
}

// Would be nice to do this automatically as part of the macros. As an impl for a trait
// seems difficult to work around "conflicting implementations for trait" errors with
// the other trait impls for the more basic types.
impl FrontendChecksum for ChangeSetId {
    fn checksum(&self) -> Checksum {
        FrontendChecksum::checksum(&self.to_string())
    }
}

impl FrontendChecksum for ChangeSetStatus {
    fn checksum(&self) -> Checksum {
        FrontendChecksum::checksum(&self.to_string())
    }
}

impl FrontendChecksum for WorkspaceId {
    fn checksum(&self) -> Checksum {
        FrontendChecksum::checksum(&self.to_string())
    }
}

impl FrontendChecksum for ViewId {
    fn checksum(&self) -> Checksum {
        FrontendChecksum::checksum(&self.to_string())
    }
}

impl FrontendChecksum for Timestamp {
    fn checksum(&self) -> Checksum {
        let mut hasher = ChecksumHasher::new();
        hasher.update(FrontendChecksum::checksum(&self.created_at).as_bytes());
        hasher.update(FrontendChecksum::checksum(&self.updated_at).as_bytes());
        hasher.finalize()
    }
}

// Generic impl for a basic type.
impl FrontendChecksum for String {
    fn checksum(&self) -> Checksum {
        let mut hasher = ChecksumHasher::new();
        hasher.update(self.as_bytes());
        hasher.finalize()
    }
}

impl FrontendChecksum for bool {
    fn checksum(&self) -> Checksum {
        let mut hasher = ChecksumHasher::new();
        hasher.update(if *self { &[1] } else { &[0] });
        hasher.finalize()
    }
}

impl<T> FrontendChecksum for Option<T>
where
    T: FrontendChecksum,
{
    fn checksum(&self) -> Checksum {
        if let Some(inner) = self {
            inner.checksum()
        } else {
            Checksum::default()
        }
    }
}

impl<T> FrontendChecksum for Vec<T>
where
    T: FrontendChecksum,
{
    fn checksum(&self) -> Checksum {
        let mut hasher = ChecksumHasher::new();
        for item in self {
            hasher.update(item.checksum().as_bytes());
        }
        hasher.finalize()
    }
}

impl FrontendChecksum for DateTime<Utc> {
    fn checksum(&self) -> Checksum {
        FrontendChecksum::checksum(&self.to_rfc3339())
    }
}
