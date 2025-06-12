use serde::Serialize;
use si_id::{
    ChangeSetId,
    WorkspacePk,
};

const PATCH_BATCH_KIND: &str = "PatchMessage";
const INDEX_UPDATE_KIND: &str = "IndexUpdate";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMeta {
    /// The workspace this patch batch is targeting.
    pub workspace_id: WorkspacePk,
    /// The change set this patch batch is targeting.
    pub change_set_id: Option<ChangeSetId>,
    /// The index checksum the patches will result in data for.
    pub to_index_checksum: String,
    /// The index checksum the patches the patches are being applied to.
    /// Or in the case of rebuild or a brand new change set, will match the [`to_index_checksum`]
    pub from_index_checksum: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchBatch {
    /// Metadata about the patch batch.
    pub meta: UpdateMeta,
    /// The message kind for the front end.
    kind: &'static str,
    /// The list of patches to apply.
    pub patches: Vec<ObjectPatch>,
}

impl PatchBatch {
    pub fn new(meta: UpdateMeta, patches: Vec<ObjectPatch>) -> Self {
        Self {
            meta,
            kind: PATCH_BATCH_KIND,
            patches,
        }
    }

    pub fn kind(&self) -> &'static str {
        self.kind
    }
}

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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexUpdate {
    /// Metadata about the patch batch.
    pub meta: UpdateMeta,
    /// The message kind for the front end.
    kind: &'static str,
    /// Checksum
    pub index_checksum: String,
}

impl IndexUpdate {
    pub fn new(meta: UpdateMeta, index_checksum: String) -> Self {
        Self {
            meta,
            kind: INDEX_UPDATE_KIND,
            index_checksum,
        }
    }

    pub fn kind(&self) -> &'static str {
        self.kind
    }
}
