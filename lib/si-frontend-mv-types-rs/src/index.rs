use lazy_static::lazy_static;
use serde::{
    Deserialize,
    Serialize,
};
use si_events::workspace_snapshot::Checksum;
use si_id::ulid::{
    CoreUlid,
    Ulid,
};

use crate::{
    checksum::FrontendChecksum,
    materialized_view::materialized_view_definitions_checksum,
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
pub struct ChangeSetMvIndex {
    pub snapshot_address: String,
    pub mv_list: Vec<IndexReference>,
    pub definition_checksum: Checksum,
}

impl ChangeSetMvIndex {
    pub fn new(snapshot_address: String, mv_list: Vec<IndexReference>) -> Self {
        ChangeSetMvIndex {
            snapshot_address,
            mv_list,
            definition_checksum: materialized_view_definitions_checksum(),
        }
    }
}

impl TryFrom<ChangeSetMvIndex> for FrontendObject {
    type Error = serde_json::Error;

    fn try_from(value: ChangeSetMvIndex) -> Result<Self, Self::Error> {
        Ok(Self {
            kind: ReferenceKind::ChangeSetMvIndex.to_string(),
            id: value.snapshot_address.to_string(),
            checksum: FrontendChecksum::checksum(&value).to_string(),
            data: serde_json::to_value(&value)?,
        })
    }
}

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
pub struct DeploymentMvIndex {
    pub mv_list: Vec<IndexReference>,
    pub definition_checksum: Checksum,
}

lazy_static! {
    static ref DEPLOYMENT_INDEX_STATIC_ULID: Ulid =
        CoreUlid::from_parts(1754943219, 167302281320132304586783).into();
}

impl DeploymentMvIndex {
    pub fn new(mv_list: Vec<IndexReference>) -> Self {
        Self {
            mv_list,
            definition_checksum: materialized_view_definitions_checksum(),
        }
    }
}

impl TryFrom<DeploymentMvIndex> for FrontendObject {
    type Error = serde_json::Error;

    fn try_from(value: DeploymentMvIndex) -> Result<Self, Self::Error> {
        Ok(Self {
            kind: ReferenceKind::DeploymentMvIndex.to_string(),
            id: DEPLOYMENT_INDEX_STATIC_ULID.to_string(),
            checksum: FrontendChecksum::checksum(&value).to_string(),
            data: serde_json::to_value(&value)?,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeploymentIndexPointerValue {
    pub index_object_key: String,
    pub definition_checksum: Checksum,
    pub index_checksum: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeSetIndexPointerValue {
    pub index_object_key: String,
    pub snapshot_address: String,
    pub definition_checksum: Checksum,
    pub index_checksum: String,
}
