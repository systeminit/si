use std::time::Duration;

use async_trait::async_trait;
use pinga_core::api_types::job_execution_request::JobArgsVCurrent;
use si_id::{
    ChangeSetId,
    DebugFuncJobStateId,
    FuncRunId,
    WorkspacePk,
};
use telemetry::prelude::*;

use crate::{
    DalContext,
    Func,
    func::{
        debug::{
            DebugFuncError,
            DebugFuncJobState,
            DebugFuncJobStateRow,
            DebugFuncResult,
            prepare_debug_func_args,
        },
        runner::{
            FuncRunner,
            FuncRunnerValueChannel,
        },
    },
    job::consumer::{
        DalJob,
        JobCompletionState,
        JobConsumer,
        JobConsumerResult,
    },
    workspace_snapshot::DependentValueRoot,
};

const WAIT_MS: u64 = 50;
const MAX_WAITS: u64 = 10_000;

#[derive(Clone, Debug)]
pub struct DebugFuncJob {
    workspace_id: WorkspacePk,
    change_set_id: ChangeSetId,
    job_state_id: DebugFuncJobStateId,
}

impl DebugFuncJob {
    pub fn new(
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
        job_state_id: DebugFuncJobStateId,
    ) -> Box<Self> {
        Self {
            workspace_id,
            change_set_id,
            job_state_id,
        }
        .into()
    }

    async fn attempt_dispatch(
        &self,
        ctx: &DalContext,
        job_state_row: &DebugFuncJobStateRow,
    ) -> DebugFuncResult<(FuncRunId, FuncRunnerValueChannel)> {
        let args = prepare_debug_func_args(
            ctx,
            job_state_row.component_id,
            job_state_row.debug_input.as_ref(),
        )
        .await?;
        let func = Func::new_debug(
            &job_state_row.func_name,
            &job_state_row.code,
            &job_state_row.handler,
        );

        let (func_run_id, result_channel) =
            FuncRunner::run_debug(ctx, func, job_state_row.component_id, args)
                .await
                .map_err(Box::new)?;

        Ok((func_run_id, result_channel))
    }

    async fn fail_run(&self, ctx: &DalContext, func_run_id: Option<FuncRunId>, error: String) {
        if let Err(err) =
            DebugFuncJobStateRow::set_failed(ctx, self.job_state_id, func_run_id, error).await
        {
            error!(
                error=%err,
                err=%err,
                "Failed to set job state to failed for func run job state id: {}",
                self.job_state_id
            );
        }
    }

    pub async fn run_debug_func(&self, ctx: &DalContext) -> DebugFuncResult<JobCompletionState> {
        let job_state_row = DebugFuncJobStateRow::get_by_id(ctx, self.job_state_id).await?;

        if job_state_row.state != DebugFuncJobState::Pending {
            return Err(DebugFuncError::FuncAlreadyRunning(self.job_state_id));
        }

        let mut ctx_clone = ctx.clone();
        match self.spin_until_ready(&mut ctx_clone, MAX_WAITS).await {
            Ok(true) => {}
            Ok(false) => {
                self.fail_run(
                    ctx,
                    None,
                    "Waited too long for dependent functions to finish".to_string(),
                )
                .await;
            }
            Err(err) => {
                self.fail_run(ctx, None, err.to_string()).await;
            }
        }
        let ctx = &ctx_clone;

        let (func_run_id, result_channel) = match self.attempt_dispatch(ctx, &job_state_row).await {
            Ok((func_run_id, result_channel)) => (func_run_id, result_channel),
            Err(err) => {
                error!(error=%err, err=%err, "failed to dispatch debug function job state id: {}", self.job_state_id);
                self.fail_run(ctx, None, err.to_string()).await;
                return Err(err);
            }
        };

        if let Err(err) =
            DebugFuncJobStateRow::set_running(ctx, self.job_state_id, func_run_id).await
        {
            error!(error=%err, err=%err, "failed to set debug function job state to running with id: {}", self.job_state_id);
            self.fail_run(ctx, Some(func_run_id), err.to_string()).await;
            return Err(err);
        }

        match result_channel.await {
            Ok(value_result) => match value_result {
                Ok(mut value) => {
                    let result = value.take_value();
                    if let Err(err) = DebugFuncJobStateRow::set_success(
                        ctx,
                        self.job_state_id,
                        func_run_id,
                        result,
                    )
                    .await
                    {
                        error!(error=%err, err=%err, "failed to set debug function job state to success with id: {}", self.job_state_id);
                        self.fail_run(ctx, Some(func_run_id), err.to_string()).await;
                        Err(err)
                    } else {
                        Ok(JobCompletionState::Done)
                    }
                }
                Err(err) => {
                    error!(error=%err, err=%err, "failed to execute debug function with id: {}", self.job_state_id);
                    self.fail_run(ctx, Some(func_run_id), err.to_string()).await;
                    Err(Box::new(err))?
                }
            },
            Err(err) => {
                error!(error=%err, err=%err, "failed to receive result from debug function job state id: {}", self.job_state_id);
                self.fail_run(ctx, Some(func_run_id), err.to_string()).await;
                Err(err)?
            }
        }
    }

    async fn spin_until_ready(&self, ctx: &mut DalContext, max: u64) -> DebugFuncResult<bool> {
        let mut count = 0;
        loop {
            if count >= max {
                return Ok(false);
            }

            ctx.update_snapshot_to_visibility().await?;
            if !DependentValueRoot::roots_exist(ctx)
                .await
                .map_err(Box::new)?
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(WAIT_MS)).await;
            count += 1;
        }

        Ok(true)
    }
}

impl DalJob for DebugFuncJob {
    fn args(&self) -> JobArgsVCurrent {
        JobArgsVCurrent::DebugFunc {
            debug_func_job_state_id: self.job_state_id,
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
impl JobConsumer for DebugFuncJob {
    #[instrument(name = "debug_func.run", level = "info", skip_all)]
    async fn run(&self, ctx: &mut DalContext) -> JobConsumerResult<JobCompletionState> {
        Ok(self.run_debug_func(ctx).await.map_err(Box::new)?)
    }
}
