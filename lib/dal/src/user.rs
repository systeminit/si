use serde::{
    Deserialize,
    Serialize,
};
pub use si_id::{
    ChangeSetId,
    UserPk,
    ViewId,
    WorkspacePk,
};

use crate::ws_event::{
    WsEvent,
    WsEventResult,
    WsPayload,
};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CursorPayload {
    pub x: Option<String>,
    pub y: Option<String>,
    pub container: Option<String>,
    pub container_key: Option<String>,
    pub user_pk: UserPk,
    pub user_name: String,
    pub change_set_id: Option<ChangeSetId>,
    pub view_id: Option<ViewId>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OnlinePayload {
    pub user_pk: UserPk,
    pub name: String,
    pub picture_url: Option<String>,
    pub change_set_id: Option<ChangeSetId>,
    pub view_id: Option<ViewId>,
    pub idle: bool,
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UserWorkspaceFlagsPayload {
    user_pk: UserPk,
    flags: serde_json::Value,
}

impl WsEvent {
    pub async fn cursor(
        workspace_pk: WorkspacePk,
        change_set_id: Option<ChangeSetId>,
        cursor: CursorPayload,
    ) -> WsEventResult<Self> {
        WsEvent::new_raw(
            workspace_pk,
            change_set_id,
            None,
            None,
            WsPayload::Cursor(cursor),
        )
        .await
    }

    pub async fn online(workspace_pk: WorkspacePk, online: OnlinePayload) -> WsEventResult<Self> {
        WsEvent::new_raw(workspace_pk, None, None, None, WsPayload::Online(online)).await
    }

    pub async fn user_workspace_flags_update(
        workspace_pk: WorkspacePk,
        user_pk: UserPk,
        flags: serde_json::Value,
    ) -> WsEventResult<Self> {
        let payload = UserWorkspaceFlagsPayload { user_pk, flags };

        WsEvent::new_raw(
            workspace_pk,
            None,
            None,
            None,
            WsPayload::UserWorkspaceFlagsUpdated(payload),
        )
        .await
    }
}
