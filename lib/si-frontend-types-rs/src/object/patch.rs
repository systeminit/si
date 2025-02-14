use serde::Serialize;
use si_events::workspace_snapshot::Checksum;
use si_id::EntityId;

use crate::reference::ReferenceKind;

#[derive(Debug, Clone, Serialize)]
pub struct ObjectPatch {
    pub kind: ReferenceKind,
    pub id: EntityId,
    /// Checksum of all `0` means this is a new object that must be created.
    pub from_checksum: Checksum,
    /// Checksum of all `0` means this is an existing object that must be removed
    pub to_checksum: Checksum,
    /// If neither of `from_checksum`, and `to_checksum` are all `0`, this field
    /// contains the JSON Patch document to apply to the version of the object with
    /// a checksum matching `from_checksum` that will result in a version with the
    /// checksum matching `to_checksum`.
    pub patch: json_patch::Patch,
}
