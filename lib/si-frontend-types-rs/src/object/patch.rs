use serde::Serialize;
use si_events::workspace_snapshot::Checksum;
use si_id::EntityId;

use crate::reference::ReferenceKind;

#[derive(Debug, Clone, Serialize)]
pub struct ObjectPatch {
    pub kind: ReferenceKind,
    pub id: EntityId,
    pub from_checksum: Checksum,
    pub to_checksum: Checksum,
    pub patch: json_patch::Patch,
}
