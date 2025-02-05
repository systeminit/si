use serde::{Deserialize, Serialize};
use si_events::workspace_snapshot::{Checksum, ChecksumHasher};
use si_id::ChangeSetId;

use crate::{
    checksum::FrontendChecksum,
    object::FrontendObject,
    reference::{IndexReference, ReferenceKind},
};

#[derive(
    Clone, Debug, Eq, PartialEq, Deserialize, Serialize, si_frontend_types_macros::FrontendChecksum,
)]
#[serde(rename_all = "camelCase")]
pub struct MvIndex {
    pub change_set_id: ChangeSetId,
    pub mv_list: Vec<IndexReference>,
}

impl MvIndex {
    pub fn new(change_set_id: ChangeSetId, mv_list: Vec<IndexReference>) -> Self {
        MvIndex {
            change_set_id,
            mv_list,
        }
    }
}

impl TryFrom<MvIndex> for FrontendObject {
    type Error = serde_json::Error;

    fn try_from(value: MvIndex) -> Result<Self, Self::Error> {
        Ok(Self {
            kind: ReferenceKind::MvIndex.to_string(),
            id: value.change_set_id.to_string(),
            checksum: FrontendChecksum::checksum(&value).to_string(),
            data: serde_json::to_value(&value)?,
        })
    }
}
