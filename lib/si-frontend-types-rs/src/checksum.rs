use chrono::{DateTime, Utc};
use si_events::{
    workspace_snapshot::{Checksum, ChecksumHasher},
    ActionKind, ActionState, ChangeSetStatus, Timestamp,
};
use si_id::{
    ActionId, ActionPrototypeId, ChangeSetId, ComponentId, FuncId, FuncRunId, InputSocketId,
    OutputSocketId, PropId, SchemaId, SchemaVariantId, ViewId, WorkspaceId,
};

use crate::schema_variant::SchemaVariantsByCategory;
use crate::{InputSocket, OutputSocket, Prop, PropKind};

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

impl FrontendChecksum for SchemaId {
    fn checksum(&self) -> Checksum {
        FrontendChecksum::checksum(&self.to_string())
    }
}

impl FrontendChecksum for FuncId {
    fn checksum(&self) -> Checksum {
        FrontendChecksum::checksum(&self.to_string())
    }
}

impl FrontendChecksum for InputSocketId {
    fn checksum(&self) -> Checksum {
        FrontendChecksum::checksum(&self.to_string())
    }
}

impl FrontendChecksum for OutputSocketId {
    fn checksum(&self) -> Checksum {
        FrontendChecksum::checksum(&self.to_string())
    }
}

impl FrontendChecksum for PropId {
    fn checksum(&self) -> Checksum {
        FrontendChecksum::checksum(&self.to_string())
    }
}

impl FrontendChecksum for PropKind {
    fn checksum(&self) -> Checksum {
        FrontendChecksum::checksum(&self.to_string())
    }
}

impl FrontendChecksum for Prop {
    fn checksum(&self) -> Checksum {
        let mut hasher = ChecksumHasher::new();
        hasher.update(FrontendChecksum::checksum(&self.id).as_bytes());
        hasher.update(FrontendChecksum::checksum(&self.name).as_bytes());
        hasher.update(FrontendChecksum::checksum(&self.kind).as_bytes());
        hasher.update(FrontendChecksum::checksum(&self.path).as_bytes());
        hasher.update(FrontendChecksum::checksum(&self.hidden).as_bytes());
        hasher.update(FrontendChecksum::checksum(&self.eligible_to_receive_data).as_bytes());
        hasher.update(FrontendChecksum::checksum(&self.eligible_to_send_data).as_bytes());
        hasher.finalize()
    }
}

impl FrontendChecksum for InputSocket {
    fn checksum(&self) -> Checksum {
        let mut hasher = ChecksumHasher::new();
        hasher.update(FrontendChecksum::checksum(&self.id).as_bytes());
        hasher.update(FrontendChecksum::checksum(&self.name).as_bytes());
        hasher.update(FrontendChecksum::checksum(&self.eligible_to_send_data).as_bytes());
        hasher.finalize()
    }
}

impl FrontendChecksum for OutputSocket {
    fn checksum(&self) -> Checksum {
        let mut hasher = ChecksumHasher::new();
        hasher.update(FrontendChecksum::checksum(&self.id).as_bytes());
        hasher.update(FrontendChecksum::checksum(&self.name).as_bytes());
        hasher.update(FrontendChecksum::checksum(&self.eligible_to_receive_data).as_bytes());
        hasher.finalize()
    }
}

impl FrontendChecksum for SchemaVariantId {
    fn checksum(&self) -> Checksum {
        FrontendChecksum::checksum(&self.to_string())
    }
}

impl FrontendChecksum for SchemaVariantsByCategory {
    fn checksum(&self) -> Checksum {
        let mut hasher = ChecksumHasher::new();
        hasher.update(FrontendChecksum::checksum(&self.display_name).as_bytes());
        hasher.update(FrontendChecksum::checksum(&self.schema_variants).as_bytes());
        hasher.finalize()
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

impl FrontendChecksum for ActionId {
    fn checksum(&self) -> Checksum {
        FrontendChecksum::checksum(&self.to_string())
    }
}

impl FrontendChecksum for ActionPrototypeId {
    fn checksum(&self) -> Checksum {
        FrontendChecksum::checksum(&self.to_string())
    }
}

impl FrontendChecksum for ComponentId {
    fn checksum(&self) -> Checksum {
        FrontendChecksum::checksum(&self.to_string())
    }
}

impl FrontendChecksum for ActionKind {
    fn checksum(&self) -> Checksum {
        FrontendChecksum::checksum(&self.to_string())
    }
}

impl FrontendChecksum for ActionState {
    fn checksum(&self) -> Checksum {
        FrontendChecksum::checksum(&self.to_string())
    }
}

impl FrontendChecksum for FuncRunId {
    fn checksum(&self) -> Checksum {
        FrontendChecksum::checksum(&self.to_string())
    }
}
