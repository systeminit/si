use acceptable::{
    AllVersions,
    Container,
    CurrentContainer,
    IntoContainer,
    UpgradeError,
};
use serde::Deserialize;

mod v1;

pub use self::v1::{
    EnqueueUpdatesResponseV1,
    RebaseStatus,
};

#[remain::sorted]
#[derive(AllVersions, CurrentContainer, Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum EnqueueUpdatesResponseAllVersions {
    #[acceptable(current)]
    V1(EnqueueUpdatesResponseV1),
}

impl IntoContainer for EnqueueUpdatesResponseAllVersions {
    type Container = EnqueueUpdatesResponse;

    fn into_container(self) -> Result<Self::Container, UpgradeError> {
        match self {
            Self::V1(inner) => Ok(Self::Container::new(inner)),
        }
    }
}
