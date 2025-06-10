use async_trait::async_trait;
use pinga_core::api_types::job_execution_request::JobArgsVCurrent;
use si_db::{
    ManagementFuncExecutionError,
    ManagementFuncJobState,
    ManagementState,
};
use si_events::audit_log::AuditLogKind;
use si_id::{
    ChangeSetId,
    ComponentId,
    FuncRunId,
    ManagementFuncExecutionId,
    ManagementPrototypeId,
    ViewId,
    WorkspacePk,
};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::{
    task::JoinError,
    time::Duration,
};
use veritech_client::ManagementFuncStatus;

use crate::{
    DalContext,
    Func,
    FuncError,
    TransactionsError,
    WorkspaceSnapshotError,
    WsEvent,
    WsEventError,
    job::consumer::{
        DalJob,
        JobCompletionState,
        JobConsumer,
        JobConsumerResult,
    },
    management::{
        ManagementError,
        ManagementFuncReturn,
        ManagementOperator,
        prototype::{
            ManagementPrototype,
            ManagementPrototypeError,
            ManagementPrototypeExecution,
        },
    },
    workspace_snapshot::{
        DependentValueRoot,
        dependent_value_root::DependentValueRootError,
    },
};

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ManagementFuncJobError {
    #[error("dependent value root error: {0}")]
    DependentValueRoot(#[from] DependentValueRootError),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("management prototype {0} is not valid for component {1}")]
    InvalidPrototypeForComponent(ManagementPrototypeId, ComponentId),
    #[error("management error: {0}")]
    Management(#[from] ManagementError),
    #[error("management execution state error: {0}")]
    ManagementFuncExecutionState(#[from] ManagementFuncExecutionError),
    #[error("management func js execution failed")]
    ManagementFuncJsExecutionFailed,
    #[error("management prototype error: {0}")]
    ManagementPrototype(#[from] ManagementPrototypeError),
    #[error(
        "no pending execution for component {0} and management prototype {1} in change set {2}"
    )]
    NoPendingExecution(ComponentId, ManagementPrototypeId, ChangeSetId),
    #[error("serde json error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error(transparent)]
    TokioTask(#[from] JoinError),
    #[error(transparent)]
    Transactions(#[from] TransactionsError),
    #[error(
        "management func {0} for component {1} waited too long for dependent values to be calculated"
    )]
    WaitedTooLongForDependentValueRoots(ManagementPrototypeId, ComponentId),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
    #[error("ws event error: {0}")]
    WsEvent(#[from] WsEventError),
}

pub type ManagementFuncJobResult<T> = Result<T, ManagementFuncJobError>;

const WAIT_MS: u64 = 50;
const MAX_WAITS: u64 = 5_000;

#[derive(Clone, Debug)]
pub struct ManagementFuncJob {
    workspace_id: WorkspacePk,
    change_set_id: ChangeSetId,
    component_id: ComponentId,
    prototype_id: ManagementPrototypeId,
    view_id: ViewId,
    request_ulid: Option<ulid::Ulid>,
}

impl ManagementFuncJob {
    pub fn new(
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
        prototype_id: ManagementPrototypeId,
        component_id: ComponentId,
        view_id: ViewId,
        request_ulid: Option<ulid::Ulid>,
    ) -> Box<Self> {
        Self {
            workspace_id,
            change_set_id,
            component_id,
            prototype_id,
            view_id,
            request_ulid,
        }
        .into()
    }

    async fn spin_until_ready(
        &self,
        ctx: &mut DalContext,
        max: u64,
    ) -> Result<(), ManagementFuncJobError> {
        let mut count = 0;
        loop {
            if count >= max {
                return Err(ManagementFuncJobError::WaitedTooLongForDependentValueRoots(
                    self.prototype_id,
                    self.component_id,
                ));
            }

            ctx.update_snapshot_to_visibility().await?;
            if !DependentValueRoot::roots_exist(ctx).await? {
                break;
            }
            tokio::time::sleep(Duration::from_millis(WAIT_MS)).await;
            count += 1;
        }
        Ok(())
    }

    async fn run_inner(
        &self,
        ctx: &DalContext,
        execution_state_id: ManagementFuncExecutionId,
    ) -> ManagementFuncJobResult<JobCompletionState> {
        if !ManagementPrototype::is_valid_prototype_for_component(
            ctx,
            self.prototype_id,
            self.component_id,
        )
        .await?
        {
            return Err(ManagementFuncJobError::InvalidPrototypeForComponent(
                self.prototype_id,
                self.component_id,
            ));
        }

        let mut ctx_clone = ctx.clone();
        // Loop for 5_000 * 50 ms (= 250 seconds max) and then mark job as failed if not ready
        self.spin_until_ready(&mut ctx_clone, MAX_WAITS).await?;
        // We want to operate on the snapshot that is "ready"
        let ctx = &ctx_clone;

        let (geometries, placeholders, run_channel, func_run_id) =
            ManagementPrototype::start_execution(
                ctx,
                self.prototype_id,
                self.component_id,
                self.view_id.into(),
            )
            .await?;

        Self::executing_state(ctx, execution_state_id, func_run_id).await?;

        let execution_result = ManagementPrototype::finalize_execution(
            ctx,
            self.component_id,
            self.prototype_id,
            geometries,
            placeholders,
            run_channel,
        )
        .await?;

        self.operate(ctx, execution_state_id, execution_result)
            .await?;

        Self::success_state(ctx, execution_state_id).await?;

        ctx.commit().await?;

        Ok(JobCompletionState::Done)
    }

    async fn operate(
        &self,
        ctx: &DalContext,
        execution_state_id: ManagementFuncExecutionId,
        mut execution_result: ManagementPrototypeExecution,
    ) -> ManagementFuncJobResult<()> {
        let result = execution_result
            .result
            .take()
            .ok_or(ManagementFuncJobError::ManagementFuncJsExecutionFailed)?;

        Self::operating_state(ctx, execution_state_id).await?;

        let result: ManagementFuncReturn = result.try_into()?;
        let mut created_component_ids = None;
        if result.status == ManagementFuncStatus::Ok {
            if let Some(operations) = result.operations {
                created_component_ids = ManagementOperator::new(
                    ctx,
                    self.component_id,
                    operations,
                    execution_result,
                    self.view_id.into(),
                )
                .await?
                .operate()
                .await?;
            }
        }

        let func_id = ManagementPrototype::func_id(ctx, self.prototype_id).await?;
        let func = Func::get_by_id(ctx, func_id).await?;

        WsEvent::management_operations_complete(
            ctx,
            self.request_ulid,
            func.name.clone(),
            result.message.clone(),
            result.status,
            created_component_ids,
        )
        .await?
        .publish_on_commit(ctx)
        .await?;

        ctx.write_audit_log(
            AuditLogKind::ManagementOperationsComplete {
                component_id: self.component_id,
                prototype_id: self.prototype_id,
                func_id,
                func_name: func.name.clone(),
                status: match result.status {
                    ManagementFuncStatus::Ok => "ok",
                    ManagementFuncStatus::Error => "error",
                }
                .to_string(),
                message: result.message.clone(),
            },
            func.name,
        )
        .await?;

        Ok(())
    }

    async fn run_prototype(
        &self,
        ctx: &mut DalContext,
    ) -> ManagementFuncJobResult<JobCompletionState> {
        let pending_execution =
            ManagementFuncJobState::get_pending(ctx, self.component_id, self.prototype_id)
                .await?
                .ok_or(ManagementFuncJobError::NoPendingExecution(
                    self.component_id,
                    self.prototype_id,
                    self.change_set_id,
                ))?;
        let execution_state_id = pending_execution.id();

        match self.run_inner(ctx, execution_state_id).await {
            Ok(completion_state) => Ok(completion_state),
            Err(err) => {
                Self::fail_state(ctx, execution_state_id).await?;
                Err(err)
            }
        }
    }

    async fn transition_state(
        ctx: &DalContext,
        execution_state_id: ManagementFuncExecutionId,
        new_state: si_db::ManagementState,
        func_run_id: Option<FuncRunId>,
    ) -> ManagementFuncJobResult<()> {
        let ctx_clone = ctx.clone();
        if new_state == ManagementState::Failure {
            if let Ok(snap) = ctx.workspace_snapshot() {
                snap.revert().await;
            }
        }
        ManagementFuncJobState::transition_state(ctx, execution_state_id, new_state, func_run_id)
            .await?;
        ctx_clone.commit_no_rebase().await?;

        Ok(())
    }

    async fn executing_state(
        ctx: &DalContext,
        execution_state_id: ManagementFuncExecutionId,
        func_run_id: FuncRunId,
    ) -> ManagementFuncJobResult<()> {
        Self::transition_state(
            ctx,
            execution_state_id,
            ManagementState::Executing,
            Some(func_run_id),
        )
        .await
    }

    async fn operating_state(
        ctx: &DalContext,
        execution_state_id: ManagementFuncExecutionId,
    ) -> ManagementFuncJobResult<()> {
        Self::transition_state(ctx, execution_state_id, ManagementState::Operating, None).await
    }

    async fn success_state(
        ctx: &DalContext,
        execution_state_id: ManagementFuncExecutionId,
    ) -> ManagementFuncJobResult<()> {
        Self::transition_state(ctx, execution_state_id, ManagementState::Success, None).await
    }

    async fn fail_state(
        ctx: &DalContext,
        execution_state_id: ManagementFuncExecutionId,
    ) -> ManagementFuncJobResult<()> {
        Self::transition_state(ctx, execution_state_id, ManagementState::Failure, None).await
    }
}

impl DalJob for ManagementFuncJob {
    fn args(&self) -> JobArgsVCurrent {
        JobArgsVCurrent::ManagementFunc {
            component_id: self.component_id,
            prototype_id: self.prototype_id,
            view_id: self.view_id,
            request_ulid: self.request_ulid,
        }
    }

    fn workspace_id(&self) -> WorkspacePk {
        self.workspace_id
    }

    fn change_set_id(&self) -> ChangeSetId {
        self.change_set_id
    }
}

#[async_trait]
impl JobConsumer for ManagementFuncJob {
    #[instrument(name = "management_func.run", level = "info", skip_all)]
    async fn run(&self, ctx: &mut DalContext) -> JobConsumerResult<JobCompletionState> {
        Ok(self.run_prototype(ctx).await?)
    }
}
