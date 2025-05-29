use serde::{
    Deserialize,
    Serialize,
};

use crate::{
    checksum::FrontendChecksum,
    object::FrontendObject,
    reference::{
        IndexReference,
        ReferenceKind,
    },
};

#[derive(
    Clone,
    Debug,
    Eq,
    PartialEq,
    Deserialize,
    Serialize,
    si_frontend_mv_types_macros::FrontendChecksum,
)]
#[serde(rename_all = "camelCase")]
pub struct MvIndex {
    pub snapshot_address: String,
    pub mv_list: Vec<IndexReference>,
}

impl MvIndex {
    pub fn new(snapshot_address: String, mv_list: Vec<IndexReference>) -> Self {
        MvIndex {
            snapshot_address,
            mv_list,
        }
    }
}

impl TryFrom<MvIndex> for FrontendObject {
    type Error = serde_json::Error;

    fn try_from(value: MvIndex) -> Result<Self, Self::Error> {
        Ok(Self {
            kind: ReferenceKind::MvIndex.to_string(),
            id: value.snapshot_address.to_string(),
            checksum: FrontendChecksum::checksum(&value).to_string(),
            data: serde_json::to_value(&value)?,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexPointerValue {
    pub index_object_key: String,
    pub snapshot_address: String,
}
