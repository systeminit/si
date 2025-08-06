use serde::{
    Deserialize,
    Serialize,
};
use si_events::workspace_snapshot::Checksum;
use si_id::ulid::Ulid;
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

const DEPLOYMENT_INDEX_STATIC_ID_STR: &str = "01K2097S214WWYXEZ571XBV5MT";

impl DeploymentMvIndex {
    const STATIC_ID: Ulid = match Ulid::from_string(DEPLOYMENT_INDEX_STATIC_ID_STR) {
        Ok(id) => id,
        Err(_) => panic!("Invalid static id for cached schemas")
    };

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
            id: DeploymentMvIndex::STATIC_ID.to_string(),
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
