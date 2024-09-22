use std::{
    fmt,
    ops::{Deref, DerefMut},
};

use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumDiscriminants, EnumIs, EnumString, VariantNames};

use crate::{ApiVersionsWrapper, ApiWrapper, RequestId, UpgradeError};

pub mod v1;

pub use self::v1::EnqueueUpdatesResponseV1;

pub type EnqueueUpdatesResponseVCurrent = EnqueueUpdatesResponseV1;

#[derive(Clone, Eq, Serialize, PartialEq, VariantNames)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum EnqueueUpdatesResponse {
    V1(EnqueueUpdatesResponseV1),
}

impl ApiWrapper for EnqueueUpdatesResponse {
    type VersionsTarget = EnqueueUpdatesResponseVersions;
    type Current = EnqueueUpdatesResponseVCurrent;

    const MESSAGE_TYPE: &'static str = "EnqueueUpdatesResponse";

    fn id(&self) -> RequestId {
        match self {
            Self::V1(EnqueueUpdatesResponseVCurrent { id, .. }) => *id,
        }
    }

    fn new_current(current: Self::Current) -> Self {
        Self::V1(current)
    }
}

impl fmt::Debug for EnqueueUpdatesResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::V1(inner) => inner.fmt(f),
        }
    }
}

impl Deref for EnqueueUpdatesResponse {
    type Target = EnqueueUpdatesResponseVCurrent;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::V1(inner) => inner,
        }
    }
}

impl DerefMut for EnqueueUpdatesResponse {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            EnqueueUpdatesResponse::V1(inner) => inner,
        }
    }
}

#[remain::sorted]
#[derive(Clone, Debug, Deserialize, EnumDiscriminants, EnumIs, Eq, PartialEq, VariantNames)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
#[strum_discriminants(strum(serialize_all = "camelCase"), derive(AsRefStr, EnumString))]
pub enum EnqueueUpdatesResponseVersions {
    V1(EnqueueUpdatesResponseV1),
}

impl ApiVersionsWrapper for EnqueueUpdatesResponseVersions {
    type Target = EnqueueUpdatesResponse;

    fn id(&self) -> RequestId {
        match self {
            Self::V1(EnqueueUpdatesResponseV1 { id, .. }) => *id,
        }
    }

    fn into_current_version(self) -> Result<Self::Target, UpgradeError> {
        match self {
            Self::V1(inner) => Ok(Self::Target::V1(inner)),
        }
    }
}
