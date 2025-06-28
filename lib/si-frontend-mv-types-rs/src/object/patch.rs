use serde::Serialize;
use si_id::{
    ChangeSetId,
    WorkspacePk,
};

use crate::reference::ReferenceKind;

const INDEX_UPDATE_KIND: &str = "IndexUpdate";
const PATCH_BATCH_KIND: &str = "PatchMessage";
const STREAMING_PATCH_MESSAGE_KIND: &str = "StreamingPatch";

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
pub struct StreamingPatch {
    /// The workspace this patch is targeting.
    pub workspace_id: WorkspacePk,
    /// The change set this patch is targeting.
    pub change_set_id: ChangeSetId,
    /// The MV kind this patch is targeting.
    pub kind: String,
    /// The ID of the object this patch is targeting.
    pub id: String,
    /// The original checksum of the object before this patch.
    ///
    /// Checksum of `"0"` means this is a new object that must be created.
    pub from_checksum: String,
    /// The new checksum of the object after this patch.
    ///
    /// Checksum of `"0"` means this is an existing object that must be removed
    pub to_checksum: String,
    /// The JSON patch to apply to the object.
    ///
    /// If neither of `from_checksum`, and `to_checksum` are all `0`, this field
    /// contains the JSON Patch document to apply to the version of the object with
    /// a checksum matching `from_checksum` that will result in a version with the
    /// checksum matching `to_checksum`.
    pub patch: json_patch::Patch,
    /// The message kind for the front end.
    message_kind: &'static str,
}

impl StreamingPatch {
    pub fn new(
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
        kind: ReferenceKind,
        id: String,
        from_checksum: String,
        to_checksum: String,
        patch: json_patch::Patch,
    ) -> Self {
        Self {
            workspace_id,
            change_set_id,
            kind: kind.to_string(),
            id,
            from_checksum,
            to_checksum,
            patch,
            message_kind: STREAMING_PATCH_MESSAGE_KIND,
        }
    }

    pub fn message_kind(&self) -> &'static str {
        self.message_kind
    }
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
