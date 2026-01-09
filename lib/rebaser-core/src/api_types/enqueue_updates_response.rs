use acceptable::{
    AllVersions,
    Container,
    CurrentContainer,
    IntoContainer,
    UpgradeError,
};
use serde::Deserialize;

mod v1;
mod v2;

pub use self::{
    v1::{
        EnqueueUpdatesResponseV1,
        RebaseStatus,
    },
    v2::{
        ApplyToHeadStatus,
        BeginApplyStatus,
        CreateChangeSetStatus,
        EnqueueUpdatesResponseV2,
        UpdateChangeSetStatusStatus,
    },
};

#[remain::sorted]
#[derive(AllVersions, CurrentContainer, Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum EnqueueUpdatesResponseAllVersions {
    V1(EnqueueUpdatesResponseV1),
    #[acceptable(current)]
    V2(EnqueueUpdatesResponseV2),
}

impl IntoContainer for EnqueueUpdatesResponseAllVersions {
    type Container = EnqueueUpdatesResponse;

    fn into_container(self) -> Result<Self::Container, UpgradeError> {
        match self {
            Self::V1(inner) => Ok(Self::Container::new(EnqueueUpdatesResponseV2::Rebase {
                id: inner.id,
                workspace_id: inner.workspace_id,
                change_set_id: inner.change_set_id,
                status: inner.status,
            })),
            Self::V2(inner) => Ok(Self::Container::new(inner)),
        }
    }
}
