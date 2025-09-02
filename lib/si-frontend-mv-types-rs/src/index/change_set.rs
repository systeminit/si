use std::collections::HashMap;

use serde::{
    Deserialize,
    Serialize,
};
use si_events::workspace_snapshot::Checksum;

use crate::{
    checksum::FrontendChecksum,
    definition_checksum::materialized_view_definition_checksums,
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
    si_frontend_mv_types_macros::DefinitionChecksum,
    si_frontend_mv_types_macros::FrontendChecksum,
)]
#[serde(untagged)]
pub enum ChangeSetMvIndexVersion {
    V1(ChangeSetMvIndex),
    V2(ChangeSetMvIndexV2),
}

#[derive(
    Clone,
    Debug,
    Eq,
    PartialEq,
    Deserialize,
    Serialize,
    si_frontend_mv_types_macros::DefinitionChecksum,
    si_frontend_mv_types_macros::FrontendChecksum,
)]
#[serde(rename_all = "camelCase")]
pub struct ChangeSetMvIndex {
    pub snapshot_address: String,
    pub mv_list: Vec<IndexReference>,
    pub definition_checksum: Checksum,
}

#[derive(
    Clone,
    Debug,
    Eq,
    PartialEq,
    Deserialize,
    Serialize,
    si_frontend_mv_types_macros::DefinitionChecksum,
    si_frontend_mv_types_macros::FrontendChecksum,
)]
#[serde(rename_all = "camelCase")]
pub struct ChangeSetMvIndexV2 {
    pub snapshot_address: String,
    pub mv_list: Vec<IndexReference>,
    pub definition_checksums: HashMap<String, Checksum>,
}

impl ChangeSetMvIndexV2 {
    pub fn new(snapshot_address: String, mv_list: Vec<IndexReference>) -> Self {
        Self {
            snapshot_address,
            mv_list,
            definition_checksums: materialized_view_definition_checksums().clone(),
        }
    }
}

impl TryFrom<ChangeSetMvIndexV2> for FrontendObject {
    type Error = serde_json::Error;

    fn try_from(value: ChangeSetMvIndexV2) -> Result<Self, Self::Error> {
        Ok(Self {
            kind: ReferenceKind::ChangeSetMvIndex.to_string(),
            id: value.snapshot_address.to_string(),
            checksum: FrontendChecksum::checksum(&value).to_string(),
            data: serde_json::to_value(&value)?,
        })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ChangeSetIndexPointerVersion {
    V1(ChangeSetIndexPointerValueV1),
    V2(ChangeSetIndexPointerValueV2),
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeSetIndexPointerValueV1 {
    pub index_object_key: String,
    pub snapshot_address: String,
    pub definition_checksum: Checksum,
    pub index_checksum: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeSetIndexPointerValueV2 {
    pub index_object_key: String,
    pub snapshot_address: String,
    pub definition_checksums: HashMap<String, Checksum>,
    pub index_checksum: String,
}
