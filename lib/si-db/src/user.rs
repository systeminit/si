use serde::{
    Deserialize,
    Serialize,
};
use si_events::Timestamp;
use si_id::{
    UserPk,
    WorkspacePk,
};

use crate::{
    Error,
    Result,
    context::SiDbContext,
    getter,
    history_event::HistoryEvent,
    transactions::SiDbTransactions as _,
};

pub const USER_GET_BY_PK: &str = include_str!("queries/user/get_by_pk.sql");
pub const USER_LIST_FOR_WORKSPACE: &str =
    include_str!("queries/user/list_members_for_workspace.sql");
pub const FLAGS_GET_BY_PK_ON_WORKSPACE: &str =
    include_str!("queries/user/get_flags_by_pk_and_workspace.sql");
pub const FLAGS_SET_BY_PK_ON_WORKSPACE: &str =
    include_str!("queries/user/set_flags_by_pk_and_workspace.sql");

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

    getter!(name, String);
    getter!(email, String);

    pub async fn new(
        ctx: &impl SiDbContext,
        pk: UserPk,
        name: impl AsRef<str>,
        email: impl AsRef<str>,
        picture_url: Option<impl AsRef<str>>,
    ) -> Result<Self> {
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

    pub async fn get_by_pk_opt(ctx: &impl SiDbContext, pk: UserPk) -> Result<Option<Self>> {
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
    pub async fn get_by_pk(ctx: &impl SiDbContext, pk: UserPk) -> Result<Self> {
        Self::get_by_pk_opt(ctx, pk)
            .await?
            .ok_or(Error::UserNotFound(pk))
    }

    pub async fn associate_workspace(
        &self,
        ctx: &impl SiDbContext,
        workspace_pk: WorkspacePk,
    ) -> Result<()> {
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

    pub async fn is_first_user(&self, ctx: &impl SiDbContext) -> Result<bool> {
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
        ctx: &impl SiDbContext,
        user_pk: UserPk,
        workspace_pkg: String,
    ) -> Result<()> {
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
        ctx: &impl SiDbContext,
        workspace_pk: String,
    ) -> Result<Vec<Self>> {
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
        ctx: &impl SiDbContext,
        workspace_pk: String,
    ) -> Result<Vec<UserPk>> {
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

    pub async fn get_flags_for_user_on_workspace(
        ctx: &impl SiDbContext,
        user_pk: UserPk,
        workspace_pk: WorkspacePk,
    ) -> Result<serde_json::Value> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(FLAGS_GET_BY_PK_ON_WORKSPACE, &[&user_pk, &workspace_pk])
            .await?;

        let map: serde_json::Value = row.try_get("object")?;

        Ok(map)
    }

    pub async fn set_flag_for_user_on_workspace(
        ctx: &impl SiDbContext,
        user_pk: UserPk,
        workspace_pk: WorkspacePk,
        key: impl AsRef<str>,
        value: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let formatted_key = Vec::from([key.as_ref()]);

        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                FLAGS_SET_BY_PK_ON_WORKSPACE,
                &[&user_pk, &workspace_pk, &formatted_key, &value],
            )
            .await?;

        let map: serde_json::Value = row.try_get("object")?;

        Ok(map)
    }
}
