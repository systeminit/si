use std::collections::HashMap;

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
pub enum DeploymentMvIndexVersion {
    V1(DeploymentMvIndex),
    V2(DeploymentMvIndexV2),
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
pub struct DeploymentMvIndex {
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
pub struct DeploymentMvIndexV2 {
    pub mv_list: Vec<IndexReference>,
    pub definition_checksums: HashMap<String, Checksum>,
}

lazy_static! {
    static ref DEPLOYMENT_INDEX_STATIC_ULID: Ulid =
        CoreUlid::from_parts(1754943219, 167302281320132304586783).into();
}

impl DeploymentMvIndexV2 {
    pub fn new(mv_list: Vec<IndexReference>) -> Self {
        Self {
            mv_list,
            definition_checksums: materialized_view_definition_checksums().clone(),
        }
    }
}

impl TryFrom<DeploymentMvIndexV2> for FrontendObject {
    type Error = serde_json::Error;

    fn try_from(value: DeploymentMvIndexV2) -> Result<Self, Self::Error> {
        Ok(Self {
            kind: ReferenceKind::DeploymentMvIndex.to_string(),
            id: DEPLOYMENT_INDEX_STATIC_ULID.to_string(),
            checksum: FrontendChecksum::checksum(&value).to_string(),
            data: serde_json::to_value(&value)?,
        })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum DeploymentIndexPointerVersion {
    V1(DeploymentIndexPointerValueV1),
    V2(DeploymentIndexPointerValueV2),
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeploymentIndexPointerValueV1 {
    pub index_object_key: String,
    pub definition_checksum: Checksum,
    pub index_checksum: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeploymentIndexPointerValueV2 {
    pub index_object_key: String,
    pub definition_checksums: HashMap<String, Checksum>,
    pub index_checksum: String,
}
