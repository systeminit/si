use async_trait::async_trait;
use itertools::Itertools;
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
    ManagementFuncJobStateId,
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
    Component,
    ComponentError,
    DalContext,
    Func,
    FuncError,
    TransactionsError,
    WorkspaceSnapshotError,
    WsEvent,
    WsEventError,
    action::{
        Action,
        ActionError,
    },
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
            ManagementFuncKind,
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
    #[error("action error: {0}")]
    Action(#[from] ActionError),
    #[error("component error: {0}")]
    Component(#[from] Box<ComponentError>),
    #[error("dependent value root error: {0}")]
    DependentValueRoot(#[from] Box<DependentValueRootError>),
    #[error("func error: {0}")]
    Func(#[from] Box<FuncError>),
    #[error("management prototype {0} is not valid for component {1}")]
    InvalidPrototypeForComponent(ManagementPrototypeId, ComponentId),
    #[error("management error: {0}")]
    Management(#[from] Box<ManagementError>),
    #[error("management execution state error: {0}")]
    ManagementFuncExecutionState(#[from] ManagementFuncExecutionError),
    #[error("management func js execution failed")]
    ManagementFuncJsExecutionFailed,
    #[error("management prototype error: {0}")]
    ManagementPrototype(#[from] Box<ManagementPrototypeError>),
    #[error(
        "no pending execution for component {0} and management prototype {1} in change set {2}"
    )]
    NoPendingExecution(ComponentId, ManagementPrototypeId, ChangeSetId),
    #[error("serde json error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error(transparent)]
    TokioTask(#[from] JoinError),
    #[error(transparent)]
    Transactions(#[from] Box<TransactionsError>),
    #[error(
        "management func {0} for component {1} waited too long for dependent values to be calculated"
    )]
    WaitedTooLongForDependentValueRoots(ManagementPrototypeId, ComponentId),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] Box<WorkspaceSnapshotError>),
    #[error("ws event error: {0}")]
    WsEvent(#[from] Box<WsEventError>),
}

impl From<ComponentError> for ManagementFuncJobError {
    fn from(value: ComponentError) -> Self {
        Box::new(value).into()
    }
}

impl From<DependentValueRootError> for ManagementFuncJobError {
    fn from(value: DependentValueRootError) -> Self {
        Box::new(value).into()
    }
}

impl From<FuncError> for ManagementFuncJobError {
    fn from(value: FuncError) -> Self {
        Box::new(value).into()
    }
}

impl From<ManagementError> for ManagementFuncJobError {
    fn from(value: ManagementError) -> Self {
        Box::new(value).into()
    }
}

impl From<ManagementPrototypeError> for ManagementFuncJobError {
    fn from(value: ManagementPrototypeError) -> Self {
        Box::new(value).into()
    }
}

impl From<TransactionsError> for ManagementFuncJobError {
    fn from(value: TransactionsError) -> Self {
        Box::new(value).into()
    }
}

impl From<WorkspaceSnapshotError> for ManagementFuncJobError {
    fn from(value: WorkspaceSnapshotError) -> Self {
        Box::new(value).into()
    }
}

impl From<WsEventError> for ManagementFuncJobError {
    fn from(value: WsEventError) -> Self {
        Box::new(value).into()
    }
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
    request_ulid: ulid::Ulid,
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
        // NOTE(nick): this may seem hacky, but it is acceptable to create this later down the
        // chain. That's because this ID is threaded through all management operations and its
        // lifecycle. None of that tracking requires you create the ID at the outermost point. You
        // just may miss early event(s).
        let request_ulid = request_ulid.unwrap_or_default();
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

            // Signal progress when spinning until ready.
            WsEvent::management_operations_in_progress(ctx, self.request_ulid)
                .await?
                .publish_immediately(ctx)
                .await?;
        }
        Ok(())
    }

    async fn run_inner(
        &self,
        ctx: &DalContext,
        execution_state_id: ManagementFuncJobStateId,
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

        // Signal progress after execution is started.
        WsEvent::management_operations_in_progress(ctx, self.request_ulid)
            .await?
            .publish_immediately(ctx)
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

        // Signal progress after execution is finalized, but before operation.
        WsEvent::management_operations_in_progress(ctx, self.request_ulid)
            .await?
            .publish_immediately(ctx)
            .await?;

        self.operate(ctx, execution_state_id, execution_result)
            .await?;

        Self::success_state(ctx, execution_state_id).await?;
        let kind = ManagementPrototype::kind_by_id(ctx, self.prototype_id).await?;

        // if the management prototype is for an Import func and we're not on head, and the component doesn't already have a resource, let's dispatch it!
        // note: this is to future proof for when our import functions set the resource directly
        // as if the resource has already been set, we don't need to run the refresh func in this change set
        if kind == ManagementFuncKind::Import
            && ctx.get_workspace_default_change_set_id().await? != ctx.change_set_id()
            && Component::exists_on_head_by_ids(ctx, &[self.component_id])
                .await?
                .is_empty()
            && Component::resource_by_id(ctx, self.component_id)
                .await?
                .is_none()
        {
            let actions = Action::find_for_kind_and_component_id(
                ctx,
                self.component_id,
                crate::action::prototype::ActionKind::Refresh,
            )
            .await?;
            if let Ok(action_id) = actions.iter().exactly_one() {
                // Note: this is intentionally not waiting for DVU
                // or worrying about the state of other actions and
                // whether this action is dependent on something else
                // we don't care, because the user just ran import,
                // and post-import, the component already has everything it
                // needs to successfully run the refresh function
                Action::dispatch_action(ctx, *action_id).await?;
            }
        }
        ctx.commit().await?;

        Ok(JobCompletionState::Done)
    }

    async fn operate(
        &self,
        ctx: &DalContext,
        execution_state_id: ManagementFuncJobStateId,
        mut execution_result: ManagementPrototypeExecution,
    ) -> ManagementFuncJobResult<Option<Vec<ComponentId>>> {
        let result = execution_result
            .result
            .take()
            .ok_or(ManagementFuncJobError::ManagementFuncJsExecutionFailed)?;

        Self::operating_state(ctx, execution_state_id).await?;

        let result: ManagementFuncReturn = result.try_into()?;
        let mut created_component_ids = None;
        if result.status == ManagementFuncStatus::Ok {
            if let Some(operations) = result.operations {
                // Signal progress before performing operations.
                WsEvent::management_operations_in_progress(ctx, self.request_ulid)
                    .await?
                    .publish_immediately(ctx)
                    .await?;
                created_component_ids = ManagementOperator::new(
                    ctx,
                    self.component_id,
                    operations,
                    execution_result,
                    self.view_id.into(),
                    self.request_ulid,
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
            // TODO(nick): make this required.
            Some(self.request_ulid),
            func.name.clone(),
            result.message.clone(),
            result.status,
            created_component_ids.clone(),
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

        Ok(created_component_ids)
    }

    async fn run_prototype(
        &self,
        ctx: &mut DalContext,
    ) -> ManagementFuncJobResult<JobCompletionState> {
        // Signal progress when running the prototype.
        WsEvent::management_operations_in_progress(ctx, self.request_ulid)
            .await?
            .publish_immediately(ctx)
            .await?;

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
                let failure_message = err.to_string();
                Self::fail_state(ctx, execution_state_id, failure_message).await?;

                // Signal failure because we never reach a "complete" state.
                WsEvent::management_operations_failed(ctx, self.request_ulid)
                    .await?
                    .publish_immediately(ctx)
                    .await?;
                Err(err)
            }
        }
    }

    async fn transition_state(
        ctx: &DalContext,
        execution_state_id: ManagementFuncJobStateId,
        new_state: si_db::ManagementState,
        func_run_id: Option<FuncRunId>,
        failure_message: Option<String>,
    ) -> ManagementFuncJobResult<()> {
        let mut ctx_clone = ctx.clone();
        // Restart connections to ensure we don't flush jobs, and because we
        // want to commit the changes to the database independently of any other
        // queries made in the "parent" contexts.
        ctx_clone.restart_connections().await?;

        if new_state == ManagementState::Failure {
            if let Ok(snap) = ctx.workspace_snapshot() {
                snap.revert().await;
            }
        }
        ManagementFuncJobState::transition_state(
            ctx,
            execution_state_id,
            new_state,
            func_run_id,
            failure_message,
        )
        .await?;

        ctx_clone.commit_no_rebase().await?;

        Ok(())
    }

    async fn executing_state(
        ctx: &DalContext,
        execution_state_id: ManagementFuncJobStateId,
        func_run_id: FuncRunId,
    ) -> ManagementFuncJobResult<()> {
        Self::transition_state(
            ctx,
            execution_state_id,
            ManagementState::Executing,
            Some(func_run_id),
            None,
        )
        .await
    }

    async fn operating_state(
        ctx: &DalContext,
        execution_state_id: ManagementFuncJobStateId,
    ) -> ManagementFuncJobResult<()> {
        Self::transition_state(
            ctx,
            execution_state_id,
            ManagementState::Operating,
            None,
            None,
        )
        .await
    }

    async fn success_state(
        ctx: &DalContext,
        execution_state_id: ManagementFuncJobStateId,
    ) -> ManagementFuncJobResult<()> {
        Self::transition_state(
            ctx,
            execution_state_id,
            ManagementState::Success,
            None,
            None,
        )
        .await
    }

    async fn fail_state(
        ctx: &DalContext,
        execution_state_id: ManagementFuncJobStateId,
        message: String,
    ) -> ManagementFuncJobResult<()> {
        Self::transition_state(
            ctx,
            execution_state_id,
            ManagementState::Failure,
            None,
            Some(message),
        )
        .await
    }
}

impl DalJob for ManagementFuncJob {
    fn args(&self) -> JobArgsVCurrent {
        JobArgsVCurrent::ManagementFunc {
            component_id: self.component_id,
            prototype_id: self.prototype_id,
            view_id: self.view_id,
            // TODO(nick): make this required.
            request_ulid: Some(self.request_ulid),
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
