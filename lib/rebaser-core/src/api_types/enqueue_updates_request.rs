use acceptable::{
    AllVersions,
    Container,
    CurrentContainer,
    IntoContainer,
    UpgradeError,
};
use serde::Deserialize;
use si_events::RebaseBatchAddressKind;

mod v1;
mod v2;
mod v3;
mod v4;

pub use acceptable::RequestId;

pub use self::{
    v1::EnqueueUpdatesRequestV1,
    v2::EnqueueUpdatesRequestV2,
    v3::EnqueueUpdatesRequestV3,
    v4::{
        ApplyToHeadRequestV4,
        BeginApplyMode,
        BeginApplyToHeadRequestV4,
        ChangeSetStatusUpdateRequestV4,
        CreateChangeSetRequestV4,
        EnqueueUpdatesRequestV4,
        RebaseRequestV4,
    },
};

#[remain::sorted]
#[derive(AllVersions, CurrentContainer, Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum EnqueueUpdatesRequestAllVersions {
    V1(EnqueueUpdatesRequestV1),
    V2(EnqueueUpdatesRequestV2),
    V3(EnqueueUpdatesRequestV3),
    #[acceptable(current)]
    V4(EnqueueUpdatesRequestV4),
}

impl IntoContainer for EnqueueUpdatesRequestAllVersions {
    type Container = EnqueueUpdatesRequest;

    fn into_container(self) -> Result<Self::Container, UpgradeError> {
        Ok(match self {
            Self::V1(inner) => Self::Container::new(EnqueueUpdatesRequestVCurrent::Rebase {
                id: inner.id,
                request: RebaseRequestV4 {
                    workspace_id: inner.workspace_id,
                    change_set_id: inner.change_set_id,
                    updates_address: RebaseBatchAddressKind::Legacy(inner.updates_address),
                    from_change_set_id: inner.from_change_set_id,
                    event_session_id: None,
                },
            }),
            Self::V2(inner) => Self::Container::new(EnqueueUpdatesRequestVCurrent::Rebase {
                id: inner.id,
                request: RebaseRequestV4 {
                    workspace_id: inner.workspace_id,
                    change_set_id: inner.change_set_id,
                    updates_address: RebaseBatchAddressKind::Legacy(inner.updates_address),
                    from_change_set_id: inner.from_change_set_id,
                    event_session_id: inner.event_session_id,
                },
            }),
            Self::V3(inner) => Self::Container::new(EnqueueUpdatesRequestVCurrent::Rebase {
                id: inner.id,
                request: RebaseRequestV4 {
                    workspace_id: inner.workspace_id,
                    change_set_id: inner.change_set_id,
                    updates_address: inner.updates_address,
                    from_change_set_id: inner.from_change_set_id,
                    event_session_id: inner.event_session_id,
                },
            }),
            Self::V4(inner) => Self::Container::new(inner),
        })
    }
}
