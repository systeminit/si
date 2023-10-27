use serde::{Deserialize, Serialize};
use si_data_nats::NatsError;
use si_data_pg::PgError;
use telemetry::prelude::*;
use thiserror::Error;
use tokio::task::JoinError;

use crate::{
    jwt_key::JwtKeyError, pk, standard_model_accessor_ro, DalContext, HistoryEvent,
    HistoryEventError, JwtPublicSigningKey, Tenancy, Timestamp, TransactionsError, WorkspacePk,
};

const USER_GET_BY_PK: &str = include_str!("queries/user/get_by_pk.sql");
const USER_GET_BY_EMAIL_RAW: &str = include_str!("queries/user/get_by_email_raw.sql");

#[remain::sorted]
#[derive(Error, Debug)]
pub enum UserError {
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("failed to join long lived async task; bug!")]
    Join(#[from] JoinError),
    #[error(transparent)]
    JwtKey(#[from] JwtKeyError),
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

pk!(UserPk);

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

    #[allow(clippy::too_many_arguments)]
    #[instrument(skip_all)]
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

    pub async fn get_by_email_raw(ctx: &DalContext, email: &str) -> UserResult<Option<Self>> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_opt(USER_GET_BY_EMAIL_RAW, &[&email])
            .await?;
        if let Some(row) = row {
            let json: serde_json::Value = row.try_get("object")?;
            Ok(serde_json::from_value(json)?)
        } else {
            Ok(None)
        }
    }

    pub async fn get_by_pk(ctx: &DalContext, pk: UserPk) -> UserResult<Option<Self>> {
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

    pub async fn authorize(_ctx: &DalContext, _user_pk: &UserPk) -> UserResult<bool> {
        // TODO(paulo,theo): implement capabilities through auth0
        Ok(true)
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
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
pub struct UserClaim {
    pub user_pk: UserPk,
    pub workspace_pk: WorkspacePk,
}

impl UserClaim {
    pub fn new(user_pk: UserPk, workspace_pk: WorkspacePk) -> Self {
        UserClaim {
            user_pk,
            workspace_pk,
        }
    }

    pub async fn from_bearer_token(
        workspace_pk: Option<WorkspacePk>,
        public_key: JwtPublicSigningKey,
        token: impl AsRef<str>,
    ) -> UserResult<UserClaim> {
        let claims = crate::jwt_key::validate_bearer_token(public_key, &token).await?;
        let mut custom = claims.custom;
        if let Some(workspace_pk) = workspace_pk {
            custom.workspace_pk = workspace_pk;
        }
        Ok(custom)
    }
}
