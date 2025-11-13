use std::{
    fmt::Debug,
    str::FromStr,
    sync::Arc,
};

use chrono::{
    DateTime,
    Utc,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_data_pg::{
    PgError,
    PgRow,
};
use si_events::Timestamp;
use si_id::{
    ChangeSetId,
    ComponentId,
    DebugFuncJobStateId,
    FuncId,
    FuncRunId,
    UserPk,
    WorkspacePk,
};
use si_layer_cache::LayerDbError;
use strum::EnumString;
use thiserror::Error;
use veritech_client::ComponentKind;

use super::{
    Func,
    FuncError,
    FuncKind,
    runner::{
        FuncRunner,
        FuncRunnerError,
    },
};
use crate::{
    Component,
    ComponentError,
    DalContext,
    TransactionsError,
    workspace_snapshot::dependent_value_root::DependentValueRootError,
};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum DebugFuncError {
    #[error("component error: {0}")]
    Component(#[from] Box<ComponentError>),
    #[error("dependent roots error: {0}")]
    DependentValueRoot(#[from] Box<DependentValueRootError>),
    #[error("func error: {0}")]
    Func(#[from] Box<FuncError>),
    #[error("debug func already running or finished with job state id: {0}")]
    FuncAlreadyRunning(DebugFuncJobStateId),
    #[error("func runner error: {0}")]
    FuncRunner(#[from] Box<FuncRunnerError>),
    #[error("layer db error: {0}")]
    LayerDb(#[from] LayerDbError),
    #[error("func {0} is not a debug function")]
    NotADebugFunc(FuncId),
    #[error("debug func job state not found with id: {0}")]
    NotFound(DebugFuncJobStateId),
    #[error("oneshot recv error: {0}")]
    OneshotRecv(#[from] tokio::sync::oneshot::error::RecvError),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("si-db error: {0}")]
    SiDb(#[from] si_db::Error),
    #[error("strum parse error: {0}")]
    StrumParse(#[from] strum::ParseError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
}

pub type DebugFuncResult<T> = Result<T, DebugFuncError>;

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize, EnumString, strum::Display)]
pub enum DebugFuncJobState {
    #[strum(serialize = "pending")]
    Pending,
    #[strum(serialize = "running")]
    Running,
    #[strum(serialize = "failure")]
    Failure,
    #[strum(serialize = "success")]
    Success,
}

#[derive(Debug, Clone)]
pub struct DebugFuncJobStateRow {
    pub id: DebugFuncJobStateId,
    pub func_run_id: Option<FuncRunId>,
    pub component_id: ComponentId,
    pub workspace_id: WorkspacePk,
    pub change_set_id: ChangeSetId,
    pub user_id: Option<UserPk>,
    pub debug_input: Option<serde_json::Value>,
    pub state: DebugFuncJobState,
    pub failure: Option<String>,
    pub result: Option<serde_json::Value>,
    pub code: String,
    pub handler: String,
    pub func_name: String,
    pub timestamp: Timestamp,
}

impl TryFrom<PgRow> for DebugFuncJobStateRow {
    type Error = DebugFuncError;

    fn try_from(row: PgRow) -> std::result::Result<Self, Self::Error> {
        let id: DebugFuncJobStateId = row.try_get("id")?;
        let workspace_id: WorkspacePk = row.try_get("workspace_id")?;
        let change_set_id: ChangeSetId = row.try_get("change_set_id")?;
        let func_run_id: Option<FuncRunId> = row.try_get("func_run_id")?;
        let component_id: ComponentId = row.try_get("component_id")?;
        let user_id: Option<UserPk> = row.try_get("user_id")?;
        let state_string: String = row.try_get("state")?;
        let state = DebugFuncJobState::from_str(&state_string)?;
        let created_at: DateTime<Utc> = row.try_get("created_at")?;
        let updated_at: DateTime<Utc> = row.try_get("updated_at")?;
        let failure: Option<String> = row.try_get("failure")?;
        let result: Option<serde_json::Value> = row.try_get("result")?;
        let debug_input: Option<serde_json::Value> = row.try_get("debug_input")?;
        let code: String = row.try_get("code")?;
        let func_name: String = row.try_get("func_name")?;
        let handler: String = row.try_get("handler")?;

        Ok(Self {
            id,
            func_run_id,
            component_id,
            workspace_id,
            change_set_id,
            user_id,
            state,
            timestamp: Timestamp::new(created_at, updated_at),
            result,
            debug_input,
            failure,
            code,
            func_name,
            handler,
        })
    }
}

impl DebugFuncJobStateRow {
    pub async fn get_by_id(
        ctx: &DalContext,
        debug_func_job_state_id: DebugFuncJobStateId,
    ) -> DebugFuncResult<Self> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_opt(
                r#"SELECT * FROM debug_func_job_states WHERE id = $1"#,
                &[&debug_func_job_state_id],
            )
            .await?;

        Self::try_from(row.ok_or(DebugFuncError::NotFound(debug_func_job_state_id))?)
    }

    pub async fn new_pending(
        ctx: &DalContext,
        component_id: ComponentId,
        code: &str,
        handler: &str,
        name: &str,
        debug_input: Option<serde_json::Value>,
    ) -> DebugFuncResult<DebugFuncJobStateId> {
        let mut ctx_clone = ctx.clone();
        ctx_clone.restart_connections().await?;

        let user_pk = ctx_clone.history_actor().user_pk();
        let workspace_id = ctx_clone.tenancy().workspace_pk()?;
        let change_set_id = ctx_clone.change_set_id();

        let row = ctx_clone
            .txns()
            .await?
            .pg()
            .query_one(
                r#"
            INSERT INTO debug_func_job_states (
                workspace_id,
                change_set_id,
                user_id,
                component_id,
                code,
                handler,
                func_name,
                debug_input,
                state
            ) VALUES (
                $1,
                $2,
                $3,
                $4,
                $5,
                $6,
                $7,
                $8,
                $9
            ) RETURNING id;
        "#,
                &[
                    &workspace_id,
                    &change_set_id,
                    &user_pk,
                    &component_id,
                    &code,
                    &handler,
                    &name,
                    &debug_input,
                    &DebugFuncJobState::Pending.to_string(),
                ],
            )
            .await?;

        ctx_clone.commit_no_rebase().await?;

        let id = row.try_get("id")?;

        Ok(id)
    }

    pub async fn set_running(
        ctx: &DalContext,
        id: DebugFuncJobStateId,
        func_run_id: FuncRunId,
    ) -> DebugFuncResult<()> {
        let mut ctx_clone = ctx.clone();
        ctx_clone.restart_connections().await?;

        let workspace_id = ctx_clone.tenancy().workspace_pk()?;
        let change_set_id = ctx_clone.change_set_id();

        ctx_clone
            .txns()
            .await?
            .pg()
            .query(
                r#"
            UPDATE debug_func_job_states
                SET state = $1,
                    func_run_id = $2,
                    updated_at = NOW()
            WHERE id = $3
                AND workspace_id = $4
                AND change_set_id = $5;
        "#,
                &[
                    &DebugFuncJobState::Running.to_string(),
                    &func_run_id,
                    &id,
                    &workspace_id,
                    &change_set_id,
                ],
            )
            .await?;

        ctx_clone.commit_no_rebase().await?;

        Ok(())
    }

    pub async fn set_failed(
        ctx: &DalContext,
        id: DebugFuncJobStateId,
        func_run_id: Option<FuncRunId>,
        failure: String,
    ) -> DebugFuncResult<()> {
        if let Some(func_run_id) = func_run_id {
            FuncRunner::update_run(ctx, func_run_id, |func_run| {
                func_run.set_action_result_state(Some(si_events::ActionResultState::Failure));
            })
            .await
            .map_err(Box::new)?;
        }

        let mut ctx_clone = ctx.clone();
        ctx_clone.restart_connections().await?;

        let workspace_id = ctx_clone.tenancy().workspace_pk()?;
        let change_set_id = ctx_clone.change_set_id();

        ctx_clone
            .txns()
            .await?
            .pg()
            .query(
                r#"
            UPDATE debug_func_job_states
                SET state = $1,
                    failure = $2,
                    updated_at = NOW()
            WHERE id = $3
                AND workspace_id = $4
                AND change_set_id = $5;
        "#,
                &[
                    &DebugFuncJobState::Failure.to_string(),
                    &failure,
                    &id,
                    &workspace_id,
                    &change_set_id,
                ],
            )
            .await?;

        ctx_clone.commit_no_rebase().await?;

        Ok(())
    }

    pub async fn set_success(
        ctx: &DalContext,
        id: DebugFuncJobStateId,
        func_run_id: FuncRunId,
        result: Option<serde_json::Value>,
    ) -> DebugFuncResult<()> {
        let maybe_value: Option<si_events::CasValue> = result.clone().map(|value| value.into());
        let maybe_result_address = match maybe_value {
            Some(value) => Some(
                ctx.layer_db()
                    .cas()
                    .write(
                        Arc::new(value.into()),
                        None,
                        ctx.events_tenancy(),
                        ctx.events_actor(),
                    )?
                    .0,
            ),
            None => None,
        };

        FuncRunner::update_run(ctx, func_run_id, |func_run| {
            func_run.set_success(None, maybe_result_address);
            func_run.set_action_result_state(Some(si_events::ActionResultState::Success));
        })
        .await
        .map_err(Box::new)?;

        let mut ctx_clone = ctx.clone();
        ctx_clone.restart_connections().await?;

        let workspace_id = ctx_clone.tenancy().workspace_pk()?;
        let change_set_id = ctx_clone.change_set_id();

        ctx_clone
            .txns()
            .await?
            .pg()
            .query(
                r#"
            UPDATE debug_func_job_states
                SET state = $1,
                    result = $2,
                    updated_at = NOW()
            WHERE id = $3
                AND workspace_id = $4
                AND change_set_id = $5;
        "#,
                &[
                    &DebugFuncJobState::Success.to_string(),
                    &result,
                    &id,
                    &workspace_id,
                    &change_set_id,
                ],
            )
            .await?;

        ctx_clone.commit_no_rebase().await?;

        Ok(())
    }
}

pub async fn dispatch_debug_func<Input: Serialize>(
    ctx: &DalContext,
    debug_component_id: ComponentId,
    debug_func: Func,
    debug_input: Option<Input>,
) -> DebugFuncResult<DebugFuncJobStateId> {
    if debug_func.kind != FuncKind::Debug {
        return Err(DebugFuncError::NotADebugFunc(debug_func.id));
    }

    let code: String = debug_func
        .code_plaintext()
        .map_err(Box::new)?
        .unwrap_or("".into());

    let debug_input = match debug_input {
        Some(debug_input) => Some(serde_json::to_value(&debug_input)?),
        None => None,
    };

    let handler = debug_func.handler.as_deref().unwrap_or("main");
    let name = debug_func.name.as_str();

    let id = DebugFuncJobStateRow::new_pending(
        ctx,
        debug_component_id,
        &code,
        handler,
        name,
        debug_input,
    )
    .await?;

    ctx.enqueue_debug_func(id).await?;

    Ok(id)
}

pub async fn prepare_debug_func_args<Input: Serialize>(
    ctx: &DalContext,
    component_id: ComponentId,
    debug_input: Option<Input>,
) -> DebugFuncResult<serde_json::Value> {
    let debug_input = match debug_input {
        Some(debug_input) => Some(serde_json::to_value(&debug_input)?),
        None => None,
    };

    let view = Component::view_by_id(ctx, component_id)
        .await
        .map_err(Box::new)?;

    let args = serde_json::json!({
        "component": {
            "kind": ComponentKind::Standard,
            "properties": view,
        },
        "debug_input": debug_input,
    });

    Ok(args)
}
