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
pub const DATA_CACHE_SUBJECT_PREFIX: &str = "data_cache";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchBatchMeta {
    /// The workspace this patch batch is targeting.
    pub workspace_id: WorkspacePk,
    /// The change set this patch batch is targeting.
    pub change_set_id: Option<ChangeSetId>,
    /// The snapshot address the patches are being applied to.
    pub snapshot_from_address: Option<WorkspaceSnapshotAddress>,
    /// The snapshot address the patches will result in data for.
    pub snapshot_to_address: Option<WorkspaceSnapshotAddress>,
}

impl PatchBatchMeta {
    pub fn publish_subject(&self) -> String {
        format!(
            "{}.workspace_id.{}.patch_batch",
            DATA_CACHE_SUBJECT_PREFIX, self.workspace_id
        )
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchBatch {
    /// Metadata about the patch batch.
    pub meta: PatchBatchMeta,
    /// The message kind for the front end.
    pub kind: &'static str,
    /// The list of patches to apply.
    pub patches: Vec<ObjectPatch>,
}

impl PatchBatch {
    pub fn publish_subject(&self) -> String {
        self.meta.publish_subject()
    }
}
