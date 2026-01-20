use dal::{DalContext, ServicesContext};
use telemetry::prelude::*;
use tokio_util::sync::CancellationToken;
use ulid::Ulid;
use si_db::{FuncRunDb, HistoryActor};
use si_events::authentication_method::AuthenticationMethodV1;
use si_id::FuncRunId;
use super::FuncRunsBackfillResult;
use crate::init;

pub struct FuncRunsBackfiller {
    services_context: ServicesContext,
}

impl FuncRunsBackfiller {
    #[instrument(name = "sdf.func_runs_backfiller.new", level = "info", skip_all)]
    pub async fn new(
        config: crate::Config,
        task_tracker: &tokio_util::task::TaskTracker,
        task_token: CancellationToken,
    ) -> FuncRunsBackfillResult<Self> {
        let (services_context, layer_db_graceful_shutdown) =
            init::services_context_from_config(&config, task_token).await?;

        task_tracker.spawn(layer_db_graceful_shutdown.into_future());

        Ok(Self { services_context })
    }

    #[instrument(
        name = "sdf.func_runs_backfiller.log_all_func_runs",
        level = "info",
        skip_all
    )]
    pub async fn log_all_func_runs(self) -> FuncRunsBackfillResult<()> {
        info!("starting func runs backfill - logging all data from func_runs table");

        let func_run_db = self.services_context.layer_db().func_run();

        let cutoff_id: FuncRunId = Ulid::from_string("01KF1JK675EPZTVXG0J88S35BG").unwrap().into();

        let func_run_ids = func_run_db
            .read_batch_of_ids(
                10,
                Some(cutoff_id)
            ).await?;

        let ctx = DalContext::builder(self.services_context.clone(), false)
            .build_without_workspace(
                HistoryActor::SystemInit,
                None,
                AuthenticationMethodV1::System
            ).await?;
        // Process each func run
        for id in func_run_ids {


            let existing = FuncRunDb::read(&ctx, id).await?;

            if existing.is_some() {
                info!("func run {} - skipping, already exists", id);
                continue;
            }

            let layer_func_run = func_run_db.try_read(id).await?;

            info!("func run {id} - will insert}");


        }

        Ok(())
    }
}
