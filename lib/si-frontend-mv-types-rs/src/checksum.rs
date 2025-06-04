use std::collections::HashMap;

use chrono::{
    DateTime,
    Utc,
};
use si_events::{
    ActionKind,
    ActionState,
    ChangeSetStatus,
    Timestamp,
    workspace_snapshot::{
        Checksum,
        ChecksumHasher,
    },
};
use si_id::{
    ActionId,
    ActionPrototypeId,
    AttributeValueId,
    ChangeSetId,
    ComponentId,
    FuncId,
    FuncRunId,
    InputSocketId,
    ManagementPrototypeId,
    OutputSocketId,
    PropId,
    SchemaId,
    SchemaVariantId,
    SecretId,
    ViewId,
    WorkspaceId,
    WorkspacePk,
};

use crate::{
    component::attribute_tree::ValidationStatus,
    schema_variant::{
        SchemaVariantsByCategory,
        prop_tree::{
            Prop,
            PropKind,
            PropWidgetKind,
        },
    },
};

pub trait FrontendChecksum {
    fn checksum(&self) -> Checksum;
}

// TODO(Wendy) - Would be nice to have a default checksum once specialization is enabled in Rust
// impl<T: ToString> FrontendChecksum for T {
//     fn checksum(&self) -> Checksum {
//         FrontendChecksum::checksum(&self.to_string())
//     }
// }

// Would be nice to do this automatically as part of the macros. As an impl for a trait
// seems difficult to work around "conflicting implementations for trait" errors with
// the other trait impls for the more basic types.
impl FrontendChecksum for ChangeSetId {
    fn checksum(&self) -> Checksum {
        FrontendChecksum::checksum(&self.to_string())
    }
}

impl FrontendChecksum for WorkspacePk {
    fn checksum(&self) -> Checksum {
        FrontendChecksum::checksum(&self.to_string())
    }
}

impl FrontendChecksum for ChangeSetStatus {
    fn checksum(&self) -> Checksum {
        FrontendChecksum::checksum(&self.to_string())
    }
}

impl FrontendChecksum for Checksum {
    fn checksum(&self) -> Checksum {
        *self
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

impl FrontendChecksum for ManagementPrototypeId {
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

impl FrontendChecksum for SecretId {
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
        hasher.update(FrontendChecksum::checksum(&self.eligible_for_connection).as_bytes());
        hasher.update(FrontendChecksum::checksum(&self.create_only).as_bytes());
        hasher.update(FrontendChecksum::checksum(&self.doc_link).as_bytes());
        hasher.update(FrontendChecksum::checksum(&self.documentation).as_bytes());
        hasher.update(FrontendChecksum::checksum(&self.default_can_be_set_by_socket).as_bytes());
        hasher.update(FrontendChecksum::checksum(&self.is_origin_secret).as_bytes());
        hasher.update(FrontendChecksum::checksum(&self.widget_kind).as_bytes());
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

impl<K, V> FrontendChecksum for HashMap<K, V>
where
    K: FrontendChecksum + Ord + std::hash::Hash,
    V: FrontendChecksum,
{
    fn checksum(&self) -> Checksum {
        let mut hasher = ChecksumHasher::new();
        let mut keys: Vec<&K> = self.keys().collect();
        keys.sort();
        for key in keys {
            hasher.update(key.checksum().as_bytes());
            hasher.update(
                match self.get(key) {
                    Some(val) => val.checksum(),
                    None => Checksum::default(),
                }
                .as_bytes(),
            );
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

impl FrontendChecksum for usize {
    fn checksum(&self) -> Checksum {
        FrontendChecksum::checksum(&self.to_string())
    }
}

impl FrontendChecksum for i64 {
    fn checksum(&self) -> Checksum {
        FrontendChecksum::checksum(&self.to_string())
    }
}

impl FrontendChecksum for AttributeValueId {
    fn checksum(&self) -> Checksum {
        FrontendChecksum::checksum(&self.to_string())
    }
}

impl FrontendChecksum for PropWidgetKind {
    fn checksum(&self) -> Checksum {
        FrontendChecksum::checksum(&self.to_string())
    }
}

impl FrontendChecksum for serde_json::Value {
    fn checksum(&self) -> Checksum {
        FrontendChecksum::checksum(&self.to_string())
    }
}

impl FrontendChecksum for ValidationStatus {
    fn checksum(&self) -> Checksum {
        FrontendChecksum::checksum(&self.to_string())
    }
}

impl FrontendChecksum for u64 {
    fn checksum(&self) -> Checksum {
        FrontendChecksum::checksum(&self.to_string())
    }
}

impl FrontendChecksum for Vec<u8> {
    fn checksum(&self) -> Checksum {
        let mut hasher = ChecksumHasher::new();
        hasher.update(self.as_slice());
        hasher.finalize()
    }
}

impl FrontendChecksum for &[u8] {
    fn checksum(&self) -> Checksum {
        let mut hasher = ChecksumHasher::new();
        hasher.update(self);
        hasher.finalize()
    }
}
