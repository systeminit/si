use std::str::FromStr;

use chrono::{
    DateTime,
    Utc,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_data_pg::PgRow;
use si_events::Timestamp;
use si_id::{
    ChangeSetId,
    ComponentId,
    FuncRunId,
    ManagementFuncJobStateId,
    ManagementPrototypeId,
    UserPk,
    WorkspacePk,
};
use strum::EnumString;

use crate::{
    SiDbContext,
    SiDbTransactions,
    getter,
    getter_copy,
};

#[remain::sorted]
#[derive(thiserror::Error, Debug)]
pub enum ManagementFuncExecutionError {
    #[error("management function execution could not be created")]
    CreationFailed,
    #[error("cannot transition from {0} to {1}")]
    InvalidTransition(ManagementState, ManagementState),
    #[error("no execution found with id: {0}")]
    NotFound(ManagementFuncJobStateId),
    #[error(
        "no in progress execution found for workspace {0}, change set {1}, component {2}, management prototype {3}"
    )]
    NotFoundInProgress(WorkspacePk, ChangeSetId, ComponentId, ManagementPrototypeId),
    #[error("pg error: {0}")]
    Pg(#[from] si_data_pg::PgError),
    #[error("pg pool error: {0}")]
    PgPool(#[from] si_data_pg::PgPoolError),
    #[error("si db error: {0}")]
    SiDb(#[from] crate::Error),
    #[error("si db transactions error: {0}")]
    SiDbTransactions(#[from] crate::transactions::SiDbTransactionsError),
    #[error("strum parse error: {0}")]
    StrumParse(#[from] strum::ParseError),
}

pub type ManagementFuncExecutionResult<T> = std::result::Result<T, ManagementFuncExecutionError>;

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, EnumString, strum::Display)]
pub enum ManagementState {
    /// Waiting to be executed
    #[strum(serialize = "pending")]
    Pending,
    /// Executing the management function in veritech/cyclone
    #[strum(serialize = "executing")]
    Executing,
    /// Operating on the return value of the management function
    #[strum(serialize = "operating")]
    Operating,
    /// Success
    #[strum(serialize = "success")]
    Success,
    /// Failure
    #[strum(serialize = "failure")]
    Failure,
}

impl ManagementState {
    pub fn is_valid_transition(&self, next: Self) -> bool {
        matches!(
            (self, next),
            (Self::Pending, Self::Executing)
                | (Self::Executing, Self::Operating)
                | (Self::Operating, Self::Success)
                | (
                    Self::Pending | Self::Executing | Self::Operating,
                    Self::Failure
                )
        )
    }
}

impl TryFrom<PgRow> for ManagementFuncJobState {
    type Error = ManagementFuncExecutionError;

    fn try_from(row: PgRow) -> std::result::Result<Self, Self::Error> {
        let id: ManagementFuncJobStateId = row.try_get("id")?;
        let workspace_id: WorkspacePk = row.try_get("workspace_id")?;
        let change_set_id: ChangeSetId = row.try_get("change_set_id")?;
        let component_id: ComponentId = row.try_get("component_id")?;
        let prototype_id: ManagementPrototypeId = row.try_get("prototype_id")?;
        let func_run_id: Option<FuncRunId> = row.try_get("func_run_id")?;
        let user_id: Option<UserPk> = row.try_get("user_id")?;
        let state_string: String = row.try_get("state")?;
        let state = ManagementState::from_str(&state_string)?;
        let created_at: DateTime<Utc> = row.try_get("created_at")?;
        let updated_at: DateTime<Utc> = row.try_get("updated_at")?;
        let message: Option<String> = row.try_get("message")?;

        Ok(Self {
            id,
            workspace_id,
            change_set_id,
            component_id,
            prototype_id,
            func_run_id,
            user_id,
            state,
            timestamp: Timestamp::new(created_at, updated_at),
            message,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagementFuncJobState {
    id: ManagementFuncJobStateId,
    workspace_id: WorkspacePk,
    change_set_id: ChangeSetId,
    component_id: ComponentId,
    prototype_id: ManagementPrototypeId,
    user_id: Option<UserPk>,
    func_run_id: Option<FuncRunId>,
    state: ManagementState,
    timestamp: Timestamp,
    message: Option<String>,
}

impl ManagementFuncJobState {
    getter_copy!(id, ManagementFuncJobStateId);
    getter_copy!(workspace_id, WorkspacePk);
    getter_copy!(change_set_id, ChangeSetId);
    getter_copy!(component_id, ComponentId);
    getter_copy!(prototype_id, ManagementPrototypeId);
    getter_copy!(user_id, Option<UserPk>);
    getter_copy!(func_run_id, Option<FuncRunId>);
    getter_copy!(state, ManagementState);
    getter_copy!(timestamp, Timestamp);
    getter!(message, Option<String>);

    pub async fn new_pending(
        ctx: &impl SiDbContext,
        component_id: ComponentId,
        prototype_id: ManagementPrototypeId,
    ) -> ManagementFuncExecutionResult<Self> {
        let state = ManagementState::Pending;
        let user_pk = ctx.history_actor().user_pk();
        let workspace_id = ctx.tenancy().workspace_pk()?;
        let change_set_id = ctx.change_set_id();

        let row = ctx.txns().await?.pg().query_opt(
            r#"INSERT INTO management_func_job_states (workspace_id, change_set_id, component_id, prototype_id, user_id, state) VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT DO NOTHING RETURNING *"#,
            &[&workspace_id, &change_set_id, &component_id, &prototype_id, &user_pk, &state.to_string()]
        ).await?;

        Self::try_from(row.ok_or(ManagementFuncExecutionError::CreationFailed)?)
    }

    pub async fn get_pending(
        ctx: &impl SiDbContext,
        component_id: ComponentId,
        prototype_id: ManagementPrototypeId,
    ) -> ManagementFuncExecutionResult<Option<Self>> {
        let workspace_id = ctx.tenancy().workspace_pk()?;
        let change_set_id = ctx.change_set_id();

        let row = ctx.txns().await?.pg().query_opt(
            r#"SELECT * FROM management_func_job_states WHERE workspace_id = $1 AND change_set_id = $2 AND component_id = $3 AND prototype_id = $4 AND state = 'pending' LIMIT 1"#,
            &[&workspace_id, &change_set_id, &component_id, &prototype_id]
        ).await?;

        Ok(match row {
            Some(row) => Some(Self::try_from(row)?),
            None => None,
        })
    }

    pub async fn get_by_id(
        ctx: &impl SiDbContext,
        management_func_job_state_id: ManagementFuncJobStateId,
    ) -> ManagementFuncExecutionResult<Self> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_opt(
                r#"SELECT * FROM management_func_job_states WHERE id = $1"#,
                &[&management_func_job_state_id],
            )
            .await?;

        Self::try_from(row.ok_or(ManagementFuncExecutionError::NotFound(
            management_func_job_state_id,
        ))?)
    }

    pub async fn get_latest_by_keys(
        ctx: &impl SiDbContext,
        component_id: ComponentId,
        prototype_id: ManagementPrototypeId,
    ) -> ManagementFuncExecutionResult<Option<Self>> {
        let workspace_id = ctx.tenancy().workspace_pk()?;
        let change_set_id = ctx.change_set_id();

        let row = ctx.txns().await?.pg().query_opt(
            r#"SELECT * FROM management_func_job_states WHERE workspace_id = $1 AND change_set_id = $2 AND component_id = $3 AND prototype_id = $4 ORDER BY created_at DESC LIMIT 1"#,
            &[&workspace_id, &change_set_id, &component_id, &prototype_id]
        ).await?;

        Ok(match row {
            Some(row) => Some(Self::try_from(row)?),
            None => None,
        })
    }

    /// Returns the latest [`ManagementFuncJobState`] for a given [`FuncRunId`](dal::FuncRun).
    /// There should be exactly zero or one results from the inner query, so the "latest" qualifier
    /// is unnecessary. However, there is neither a unique index nor any form of blocking logic to
    /// disallow that to be the case, so grab the latest to be certain.
    pub async fn get_latest_by_func_run_id(
        ctx: &impl SiDbContext,
        func_run_id: FuncRunId,
    ) -> ManagementFuncExecutionResult<Option<Self>> {
        let workspace_id = ctx.tenancy().workspace_pk()?;
        let change_set_id = ctx.change_set_id();

        let row = ctx.txns().await?.pg().query_opt(
            r#"SELECT * FROM management_func_job_states WHERE workspace_id = $1 AND change_set_id = $2 AND func_run_id = $3 ORDER BY created_at DESC LIMIT 1"#,
            &[&workspace_id, &change_set_id, &func_run_id]
        ).await?;

        Ok(match row {
            Some(row) => Some(Self::try_from(row)?),
            None => None,
        })
    }

    pub async fn transition_state(
        ctx: &impl SiDbContext,
        id: ManagementFuncJobStateId,
        next_state: ManagementState,
        func_run_id: Option<FuncRunId>,
        // TODO(nick): allow messages on non-failure states. This is purely logic guarded and you
        // can technically do this for any state transition. Why is this guarded? Testing and using
        // messages only happens upon failure and the product does not need messages for any other
        // state at the time of writing. An "UPDATE" query for setting the message for another
        // would be entirely untested and unused.
        failure_message: Option<String>,
    ) -> ManagementFuncExecutionResult<Self> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_opt(
                r#"SELECT * FROM management_func_job_states WHERE id = $1 FOR UPDATE"#,
                &[&id],
            )
            .await?;

        let current = Self::try_from(row.ok_or(ManagementFuncExecutionError::NotFound(id))?)?;

        if !current.state().is_valid_transition(next_state) {
            return Err(ManagementFuncExecutionError::InvalidTransition(
                current.state(),
                next_state,
            ));
        }

        let updated_row = match (next_state, failure_message) {
            (ManagementState::Executing, _) => {
                ctx.txns().await?.pg().query_one(r#"UPDATE management_func_job_states SET state = $1, func_run_id = $2, updated_at = now() WHERE id = $3 RETURNING *"#,
                    &[&next_state.to_string(), &func_run_id, &current.id()]
                ).await?
            }
            (ManagementState::Failure, Some(message))=> {
                ctx.txns()
                    .await?
                    .pg()
                    .query_one(
                        r#"UPDATE management_func_job_states SET state = $1, message = $2, updated_at = now() WHERE id = $3 RETURNING *"#,
                        &[&next_state.to_string(), &message, &current.id()],
                    )
                    .await?
            }
            _ => {
                ctx.txns()
                    .await?
                    .pg()
                    .query_one(
                        r#"UPDATE management_func_job_states SET state = $1, updated_at = now() WHERE id = $2 RETURNING *"#,
                        &[&next_state.to_string(), &current.id()],
                    )
                    .await?
            }
        };

        Self::try_from(updated_row)
    }
}
