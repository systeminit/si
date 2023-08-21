use serde::{Deserialize, Serialize};
use si_data_nats::NatsError;
use si_data_pg::PgError;
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    pk, standard_model, standard_model_accessor_ro, DalContext, HistoryActor, HistoryEvent,
    HistoryEventError, KeyPair, KeyPairError, StandardModelError, Tenancy, Timestamp,
    TransactionsError, User, UserError, UserPk,
};

const WORKSPACE_GET_BY_PK: &str = include_str!("queries/workspace/get_by_pk.sql");
const WORKSPACE_FIND_BY_NAME: &str = include_str!("queries/workspace/find_by_name.sql");

#[remain::sorted]
#[derive(Error, Debug)]
pub enum WorkspaceError {
    #[error(transparent)]
    HistoryEvent(#[from] HistoryEventError),
    #[error(transparent)]
    KeyPair(#[from] KeyPairError),
    #[error(transparent)]
    Nats(#[from] NatsError),
    #[error(transparent)]
    Pg(#[from] PgError),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error(transparent)]
    Transactions(#[from] TransactionsError),
    #[error(transparent)]
    User(#[from] UserError),
}

pub type WorkspaceResult<T> = Result<T, WorkspaceError>;

pk!(WorkspacePk);
pk!(WorkspaceId);

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceSignup {
    pub key_pair: KeyPair,
    pub user: User,
    pub workspace: Workspace,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Workspace {
    pk: WorkspacePk,
    name: String,
    #[serde(flatten)]
    timestamp: Timestamp,
}

impl Workspace {
    pub fn pk(&self) -> &WorkspacePk {
        &self.pk
    }

    #[instrument(skip_all)]
    pub async fn builtin(ctx: &DalContext) -> WorkspaceResult<Self> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "SELECT object FROM workspace_find_or_create_builtin_v1()",
                &[],
            )
            .await?;

        let object = standard_model::object_from_row(row)?;
        Ok(object)
    }

    pub async fn find_first_user_workspace(ctx: &DalContext) -> WorkspaceResult<Option<Self>> {
        let row = ctx.txns().await?.pg().query_opt(
            "SELECT row_to_json(w.*) AS object FROM workspaces AS w WHERE pk != $1 ORDER BY created_at ASC LIMIT 1", &[&WorkspacePk::NONE],
        ).await?;

        Ok(standard_model::option_object_from_row(row)?)
    }

    #[instrument(skip_all)]
    pub async fn new(
        ctx: &mut DalContext,
        pk: WorkspacePk,
        name: impl AsRef<str>,
    ) -> WorkspaceResult<Self> {
        let name = name.as_ref();
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "SELECT object FROM workspace_create_v1($1, $2)",
                &[&pk, &name],
            )
            .await?;

        // Inlined `finish_create_from_row`

        let json: serde_json::Value = row.try_get("object")?;
        let object: Self = serde_json::from_value(json)?;

        ctx.update_tenancy(Tenancy::new(object.pk));

        let _history_event = HistoryEvent::new(
            ctx,
            "workspace.create".to_owned(),
            "Workspace created".to_owned(),
            &serde_json::json![{ "visibility": ctx.visibility() }],
        )
        .await?;
        Ok(object)
    }

    pub async fn signup(
        ctx: &mut DalContext,
        workspace_name: impl AsRef<str>,
        user_name: impl AsRef<str>,
        user_email: impl AsRef<str>,
    ) -> WorkspaceResult<WorkspaceSignup> {
        let workspace = Workspace::new(ctx, WorkspacePk::generate(), workspace_name).await?;
        let key_pair = KeyPair::new(ctx, "default").await?;

        let user = User::new(
            ctx,
            UserPk::generate(),
            &user_name,
            &user_email,
            None::<&str>,
        )
        .await?;
        ctx.update_history_actor(HistoryActor::User(user.pk()));

        ctx.import_builtins().await?;

        Ok(WorkspaceSignup {
            key_pair,
            user,
            workspace,
        })
    }

    pub async fn find_by_name(ctx: &DalContext, name: &str) -> WorkspaceResult<Option<Workspace>> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_opt(WORKSPACE_FIND_BY_NAME, &[&name])
            .await?;
        let result = standard_model::option_object_from_row(row)?;
        Ok(result)
    }

    pub async fn get_by_pk(
        ctx: &DalContext,
        pk: &WorkspacePk,
    ) -> WorkspaceResult<Option<Workspace>> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_opt(WORKSPACE_GET_BY_PK, &[&pk])
            .await?;
        if let Some(row) = row {
            let json: serde_json::Value = row.try_get("object")?;
            Ok(serde_json::from_value(json)?)
        } else {
            Ok(None)
        }
    }

    standard_model_accessor_ro!(name, String);
}
