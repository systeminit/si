use std::{
    fmt,
    ops::{Deref, DerefMut},
};

use naxum_api_types::{ApiVersionsWrapper, ApiWrapper, RequestId, UpgradeError};
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumDiscriminants, EnumIs, EnumString, VariantNames};

mod v1;
mod v2;

pub use self::v1::EnqueueUpdatesRequestV1;
pub use self::v2::EnqueueUpdatesRequestV2;

pub type EnqueueUpdatesRequestVCurrent = EnqueueUpdatesRequestV2;

#[derive(Clone, Eq, Serialize, PartialEq, VariantNames)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum EnqueueUpdatesRequest {
    V2(EnqueueUpdatesRequestV2),
}

impl ApiWrapper for EnqueueUpdatesRequest {
    type VersionsTarget = EnqueueUpdatesRequestVersions;
    type Current = EnqueueUpdatesRequestVCurrent;

    const MESSAGE_TYPE: &'static str = "EnqueueUpdatesRequest";

    fn id(&self) -> RequestId {
        match self {
            Self::V2(EnqueueUpdatesRequestVCurrent { id, .. }) => *id,
        }
    }

    fn new_current(current: Self::Current) -> Self {
        Self::V2(current)
    }
}

impl fmt::Debug for EnqueueUpdatesRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::V2(inner) => inner.fmt(f),
        }
    }
}

impl Deref for EnqueueUpdatesRequest {
    type Target = EnqueueUpdatesRequestVCurrent;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::V2(inner) => inner,
        }
    }
}

impl DerefMut for EnqueueUpdatesRequest {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::V2(inner) => inner,
        }
    }
}

#[remain::sorted]
#[derive(Clone, Debug, Deserialize, EnumDiscriminants, EnumIs, Eq, PartialEq, VariantNames)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
#[strum_discriminants(strum(serialize_all = "camelCase"), derive(AsRefStr, EnumString))]
pub enum EnqueueUpdatesRequestVersions {
    V1(EnqueueUpdatesRequestV1),
    V2(EnqueueUpdatesRequestV2),
}

impl ApiVersionsWrapper for EnqueueUpdatesRequestVersions {
    type Target = EnqueueUpdatesRequest;

    fn id(&self) -> RequestId {
        match self {
            Self::V1(EnqueueUpdatesRequestV1 { id, .. }) => *id,
            Self::V2(EnqueueUpdatesRequestV2 { id, .. }) => *id,
        }
    }

    fn into_current_version(self) -> Result<Self::Target, UpgradeError> {
        match self {
            Self::V1(inner) => Ok(Self::Target::V2(EnqueueUpdatesRequestVCurrent {
                id: inner.id,
                workspace_id: inner.workspace_id,
                change_set_id: inner.change_set_id,
                updates_address: inner.updates_address,
                from_change_set_id: inner.from_change_set_id,
                event_session_id: None,
            })),
            Self::V2(inner) => Ok(Self::Target::V2(inner)),
        }
    }
}
