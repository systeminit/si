use serde::{Deserialize, Serialize};
use si_data_nats::NatsError;
use si_data_pg::PgError;
use si_events::ViewId;
use telemetry::prelude::*;
use thiserror::Error;
use tokio::task::JoinError;

use crate::ws_event::{WsEvent, WsEventResult, WsPayload};
use crate::{
    standard_model_accessor_ro, ChangeSetId, DalContext, HistoryEvent, HistoryEventError, Tenancy,
    Timestamp, TransactionsError, WorkspacePk,
};

const USER_GET_BY_PK: &str = include_str!("queries/user/get_by_pk.sql");
const USER_LIST_FOR_WORKSPACE: &str = include_str!("queries/user/list_members_for_workspace.sql");

#[remain::sorted]
#[derive(Error, Debug)]
pub enum UserError {
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("failed to join long lived async task; bug!")]
    Join(#[from] JoinError),
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
    #[error("user not found in tenancy: {0} {1:?}")]
    NotFoundInTenancy(UserPk, Tenancy),
    #[error("no workspace in tenancy")]
    NoWorkspaceInTenancy,
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    Transactions(#[from] TransactionsError),
}

pub type UserResult<T> = Result<T, UserError>;

pub use si_id::UserPk;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct User {
    pk: UserPk,
    name: String,
    email: String,
    // TODO: should be serialized in api as camelCase
    picture_url: Option<String>,
    #[serde(flatten)]
    timestamp: Timestamp,
}

impl User {
    pub fn pk(&self) -> UserPk {
        self.pk
    }

    standard_model_accessor_ro!(name, String);
    standard_model_accessor_ro!(email, String);

    pub async fn new(
        ctx: &DalContext,
        pk: UserPk,
        name: impl AsRef<str>,
        email: impl AsRef<str>,
        picture_url: Option<impl AsRef<str>>,
    ) -> UserResult<Self> {
        let name = name.as_ref();
        let email = email.as_ref();

        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "SELECT object FROM user_create_v1($1, $2, $3, $4)",
                &[
                    &pk,
                    &name,
                    &email,
                    &picture_url.as_ref().map(|p| p.as_ref()),
                ],
            )
            .await?;

        // Inlined `finish_create_from_row`

        let json: serde_json::Value = row.try_get("object")?;
        let object: Self = serde_json::from_value(json)?;

        // HistoryEvent won't be accessible by any tenancy (null tenancy_workspace_pk)
        let _history_event = HistoryEvent::new(
            ctx,
            "user.create".to_owned(),
            "User created".to_owned(),
            &serde_json::json![{ "visibility": ctx.visibility() }],
        )
        .await?;

        Ok(object)
    }

    pub async fn get_by_pk_opt(ctx: &DalContext, pk: UserPk) -> UserResult<Option<Self>> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_opt(USER_GET_BY_PK, &[&pk])
            .await?;
        if let Some(row) = row {
            let json: serde_json::Value = row.try_get("object")?;
            Ok(serde_json::from_value(json)?)
        } else {
            Ok(None)
        }
    }
    pub async fn get_by_pk(ctx: &DalContext, pk: UserPk) -> UserResult<Self> {
        Self::get_by_pk_opt(ctx, pk)
            .await?
            .ok_or_else(|| UserError::NotFoundInTenancy(pk, *ctx.tenancy()))
    }

    pub async fn associate_workspace(
        &self,
        ctx: &DalContext,
        workspace_pk: WorkspacePk,
    ) -> UserResult<()> {
        ctx.txns()
            .await?
            .pg()
            .execute(
                "SELECT user_associate_workspace_v1($1, $2)",
                &[&self.pk, &workspace_pk],
            )
            .await?;
        Ok(())
    }

    pub async fn is_first_user(&self, ctx: &DalContext) -> UserResult<bool> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_opt("SELECT pk FROM users ORDER BY created_at ASC LIMIT 1", &[])
            .await?;

        match row {
            Some(row) => {
                let oldest_user_pk: UserPk = row.get("pk");
                Ok(oldest_user_pk == self.pk)
            }
            None => Ok(false),
        }
    }

    pub async fn delete_user_from_workspace(
        ctx: &DalContext,
        user_pk: UserPk,
        workspace_pkg: String,
    ) -> UserResult<()> {
        ctx.txns()
            .await?
            .pg()
            .execute(
                "DELETE from user_belongs_to_workspaces WHERE user_pk = $1 AND workspace_pk = $2",
                &[&user_pk, &workspace_pkg],
            )
            .await?;
        Ok(())
    }

    pub async fn list_members_for_workspace(
        ctx: &DalContext,
        workspace_pk: String,
    ) -> UserResult<Vec<Self>> {
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(USER_LIST_FOR_WORKSPACE, &[&workspace_pk])
            .await?;

        let mut users: Vec<User> = Vec::new();
        for row in rows.into_iter() {
            let json: serde_json::Value = row.try_get("object")?;
            let object = serde_json::from_value(json)?;
            users.push(object);
        }

        Ok(users)
    }

    pub async fn list_member_pks_for_workspace(
        ctx: &DalContext,
        workspace_pk: String,
    ) -> UserResult<Vec<UserPk>> {
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(
                "SELECT users.pk FROM users INNER JOIN user_belongs_to_workspaces ON user_belongs_to_workspaces.user_pk = users.pk WHERE user_belongs_to_workspaces.workspace_pk = $1 ORDER BY users.created_at ASC",
                &[&workspace_pk]
            )
            .await?;

        let mut user_pks: Vec<UserPk> = Vec::new();
        for row in rows.into_iter() {
            user_pks.push(row.try_get("pk")?);
        }
        Ok(user_pks)
    }
}

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
}
