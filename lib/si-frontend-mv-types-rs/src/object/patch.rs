use serde::Serialize;
use si_events::WorkspaceSnapshotAddress;
use si_id::{
    ChangeSetId,
    WorkspacePk,
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectPatch {
    pub kind: String,
    pub id: String,
    /// Checksum of `"0"` means this is a new object that must be created.
    pub from_checksum: String,
    /// Checksum of `"0"` means this is an existing object that must be removed
    pub to_checksum: String,
    /// If neither of `from_checksum`, and `to_checksum` are all `0`, this field
    /// contains the JSON Patch document to apply to the version of the object with
    /// a checksum matching `from_checksum` that will result in a version with the
    /// checksum matching `to_checksum`.
    pub patch: json_patch::Patch,
}

pub const PATCH_BATCH_KIND: &str = "PatchMessage";
pub const PATCH_BATCH_POST_FIX: &str = "patch_batch";
pub const INDEX_UPDATE_KIND: &str = "IndexUpdate";
pub const INDEX_UPDATE_POST_FIX: &str = "index_update";
pub const DATA_CACHE_SUBJECT_PREFIX: &str = "data_cache";

#[derive(Debug, Clone, Serialize, Copy)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMeta {
    /// The workspace this patch batch is targeting.
    pub workspace_id: WorkspacePk,
    /// The change set this patch batch is targeting.
    pub change_set_id: Option<ChangeSetId>,
    /// The snapshot address the patches are being applied to.
    pub snapshot_from_address: Option<WorkspaceSnapshotAddress>,
    /// The snapshot address the patches will result in data for.
    pub snapshot_to_address: Option<WorkspaceSnapshotAddress>,
}

impl UpdateMeta {
    pub fn publish_subject(&self, post_fix: &'static str) -> String {
        format!(
            "{}.workspace_id.{}.{}",
            DATA_CACHE_SUBJECT_PREFIX, self.workspace_id, post_fix
        )
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchBatch {
    /// Metadata about the patch batch.
    pub meta: UpdateMeta,
    /// The message kind for the front end.
    pub kind: &'static str,
    /// The list of patches to apply.
    pub patches: Vec<ObjectPatch>,
}

impl PatchBatch {
    pub fn publish_subject(&self) -> String {
        self.meta.publish_subject(PATCH_BATCH_POST_FIX)
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexUpdate {
    /// Metadata about the patch batch.
    pub meta: UpdateMeta,
    /// The message kind for the front end.
    pub kind: &'static str,
}

impl IndexUpdate {
    pub fn publish_subject(&self) -> String {
        self.meta.publish_subject(INDEX_UPDATE_POST_FIX)
    }
}

pub enum MessageKind {
    // do this!
}

pub struct FullObject {
    // from webworker.ts
//     export interface AtomMessage {
//   kind: MessageKind.MJOLNIR;
//   atom: Atom;
//   data: object;
// }

// const msg: AtomMessage = {
//     kind: MessageKind.MJOLNIR,
//     atom: {
//       id: req.data.frontEndObject.id,
//       kind: req.data.frontEndObject.kind,
//       toChecksum: req.data.frontEndObject.checksum,
//       workspaceId,
//       changeSetId,
//       toIndexChecksum: indexChecksum,
//       fromIndexChecksum: indexChecksum,
//     },
//     data: req.data.frontEndObject.data,
//   };
}
