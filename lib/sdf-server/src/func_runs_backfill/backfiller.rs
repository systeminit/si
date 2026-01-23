use std::time::Instant;

use dal::{
    DalContext,
    ServicesContext,
};
use si_db::{
    FuncRunDb,
    FuncRunLogDb,
    HistoryActor,
};
use si_events::authentication_method::AuthenticationMethodV1;
use si_id::{
    FuncRunId,
    FuncRunLogId,
};
use telemetry::prelude::*;
use telemetry_utils::monotonic;
use tokio::join;
use tokio_util::sync::CancellationToken;

use super::FuncRunsBackfillResult;
use crate::init;

pub struct FuncRunsBackfiller {
    services_context: ServicesContext,
}

impl FuncRunsBackfiller {
    #[instrument(name = "sdf.func_runs_backfiller.new", level = "info", skip_all)]
    pub async fn new(
        config: crate::Config,
        task_tracker: tokio_util::task::TaskTracker,
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
    pub async fn upload_all_func_runs(
        &self,
        shutdown_token: CancellationToken,
        cutoff_id: Option<FuncRunId>,
        batch_size: i64,
    ) -> FuncRunsBackfillResult<()> {
        info!(
            cutoff_id = ?cutoff_id,
            batch_size = batch_size,
            "starting func runs backfill"
        );

        let func_run_db = self.services_context.layer_db().func_run();

        let mut cutoff_id = cutoff_id;
        let mut last_checkpoint = Instant::now();
        let mut total_processed = 0u64;
        let mut total_uploaded = 0u64;
        let mut total_skipped = 0u64;

        let ctx = DalContext::builder(self.services_context.clone(), false)
            .build_without_workspace(
                HistoryActor::SystemInit,
                None,
                AuthenticationMethodV1::System,
            )
            .await?;

        loop {
            // Check for shutdown signal
            if shutdown_token.is_cancelled() {
                info!(
                    last_cutoff_id = ?cutoff_id,
                    total_processed = total_processed,
                    total_uploaded = total_uploaded,
                    total_skipped = total_skipped,
                    "backfill shutting down gracefully"
                );
                break;
            }

            // Fetch next batch
            let id_batch = func_run_db.read_batch_of_ids(batch_size, cutoff_id).await?;

            trace!(
                cutoff_id = ?cutoff_id,
                batch_size = id_batch.len(),
                "fetched func run id batch"
            );

            if id_batch.is_empty() {
                // No more ids to process
                break;
            }

            // Update cutoff_id to the last ID in the batch
            if let Some(last_id) = id_batch.last() {
                cutoff_id = Some(*last_id);
            }

            // Find which IDs from the batch are missing in si-db
            let missing_ids = FuncRunDb::find_missing_ids(&ctx, &id_batch).await?;

            let batch_skipped = id_batch.len() - missing_ids.len();
            total_skipped += batch_skipped as u64;
            total_processed += id_batch.len() as u64;

            trace!(
                batch_total = id_batch.len(),
                batch_missing = missing_ids.len(),
                batch_skipped = batch_skipped,
                "processed batch, found missing IDs"
            );

            if !missing_ids.is_empty() {
                // Read the missing func runs from layer-db
                let mut func_runs_to_insert = Vec::with_capacity(missing_ids.len());
                for id in &missing_ids {
                    let layer_func_run = ctx
                        .services_context()
                        .layer_db()
                        .func_run()
                        .try_read(*id)
                        .await?;
                    func_runs_to_insert.push((*layer_func_run).clone());
                }

                // Batch upsert all missing func runs
                FuncRunDb::upsert_batch(&ctx, func_runs_to_insert).await?;
                ctx.commit().await?;

                total_uploaded += missing_ids.len() as u64;
            }

            // Update telemetry
            monotonic!(func_runs.backfill_runs.items_uploaded = missing_ids.len() as u64);
            monotonic!(func_runs.backfill_runs.items_skipped = batch_skipped as u64);
            monotonic!(func_runs.backfill_runs.items_processed = id_batch.len() as u64);

            // Time-based checkpoint for progress visibility
            if last_checkpoint.elapsed() > std::time::Duration::from_secs(10) {
                info!(
                    cutoff_id = ?cutoff_id,
                    total_processed = total_processed,
                    total_uploaded = total_uploaded,
                    total_skipped = total_skipped,
                    "backfill checkpoint"
                );
                last_checkpoint = Instant::now();
            }
        }

        // Final log when backfill complete
        info!(
            total_processed = total_processed,
            total_uploaded = total_uploaded,
            total_skipped = total_skipped,
            "completed func runs backfill"
        );

        Ok(())
    }

    #[instrument(
        name = "sdf.func_runs_backfiller.upload_all_func_run_logs",
        level = "info",
        skip_all
    )]
    pub async fn upload_all_func_run_logs(
        &self,
        shutdown_token: CancellationToken,
        cutoff_id: Option<FuncRunLogId>,
        batch_size: i64,
    ) -> FuncRunsBackfillResult<()> {
        info!(
            cutoff_id = ?cutoff_id,
            batch_size = batch_size,
            "starting func run logs backfill"
        );

        let func_run_log_db = self.services_context.layer_db().func_run_log();

        let mut cutoff_id = cutoff_id;
        let mut last_checkpoint = Instant::now();
        let mut total_processed = 0u64;
        let mut total_uploaded = 0u64;
        let mut total_skipped = 0u64;

        let ctx = DalContext::builder(self.services_context.clone(), false)
            .build_without_workspace(
                HistoryActor::SystemInit,
                None,
                AuthenticationMethodV1::System,
            )
            .await?;

        loop {
            // Check for shutdown signal
            if shutdown_token.is_cancelled() {
                info!(
                    last_cutoff_id = ?cutoff_id,
                    total_processed = total_processed,
                    total_uploaded = total_uploaded,
                    total_skipped = total_skipped,
                    "backfill shutting down gracefully"
                );
                break;
            }

            // Fetch next batch
            let id_batch = func_run_log_db
                .read_batch_of_ids(batch_size, cutoff_id)
                .await?;

            trace!(
                cutoff_id = ?cutoff_id,
                batch_size = id_batch.len(),
                "fetched func run log id batch"
            );

            if id_batch.is_empty() {
                // No more ids to process
                break;
            }

            // Update cutoff_id to the last ID in the batch
            if let Some(last_id) = id_batch.last() {
                cutoff_id = Some(*last_id);
            }

            // Find which IDs from the batch are missing in si-db
            let missing_ids = FuncRunLogDb::find_missing_ids(&ctx, &id_batch).await?;

            let batch_skipped = id_batch.len() - missing_ids.len();
            total_skipped += batch_skipped as u64;
            total_processed += id_batch.len() as u64;

            trace!(
                batch_total = id_batch.len(),
                batch_missing = missing_ids.len(),
                batch_skipped = batch_skipped,
                "processed batch, found missing IDs"
            );

            if !missing_ids.is_empty() {
                // Read the missing func run logs from layer-db
                let mut func_run_logs_to_insert = Vec::with_capacity(missing_ids.len());
                for id in &missing_ids {
                    let layer_func_run_log = ctx
                        .services_context()
                        .layer_db()
                        .func_run_log()
                        .try_read(*id)
                        .await?;
                    func_run_logs_to_insert.push((*layer_func_run_log).clone());
                }

                // Batch upsert all missing func run logs
                FuncRunLogDb::upsert_batch(&ctx, func_run_logs_to_insert).await?;
                ctx.commit().await?;

                total_uploaded += missing_ids.len() as u64;
            }

            // Update telemetry
            monotonic!(func_run_logs.backfill_logs.items_uploaded = missing_ids.len() as u64);
            monotonic!(func_run_logs.backfill_logs.items_skipped = batch_skipped as u64);
            monotonic!(func_run_logs.backfill_logs.items_processed = id_batch.len() as u64);

            // Time-based checkpoint for progress visibility
            if last_checkpoint.elapsed() > std::time::Duration::from_secs(10) {
                info!(
                    cutoff_id = ?cutoff_id,
                    total_processed = total_processed,
                    total_uploaded = total_uploaded,
                    total_skipped = total_skipped,
                    "backfill checkpoint"
                );
                last_checkpoint = Instant::now();
            }
        }

        // Final log when backfill complete
        info!(
            total_processed = total_processed,
            total_uploaded = total_uploaded,
            total_skipped = total_skipped,
            "completed func run logs backfill"
        );

        Ok(())
    }

    #[instrument(
        name = "sdf.func_runs_backfiller.upload_all_func_runs_and_logs",
        level = "info",
        skip_all
    )]
    pub async fn upload_all_func_runs_and_logs_concurrently(
        config: crate::Config,
        task_tracker: tokio_util::task::TaskTracker,
        task_token: CancellationToken,
        shutdown_token: CancellationToken,
        func_run_cutoff_id: Option<FuncRunId>,
        func_run_log_cutoff_id: Option<FuncRunLogId>,
        batch_size: i64,
    ) -> FuncRunsBackfillResult<()> {
        // Create one backfiller for each (each with their own services context)
        // so they don't bleed transactions between them
        let backfiller_runs =
            FuncRunsBackfiller::new(config.clone(), task_tracker.clone(), task_token.clone())
                .await?;

        let runs_token = shutdown_token.clone();

        let backfiller_logs =
            FuncRunsBackfiller::new(config, task_tracker.clone(), task_token.clone()).await?;

        let logs_token = shutdown_token.clone();

        match join!(
            tokio::spawn(async move {
                backfiller_runs
                    .upload_all_func_runs(runs_token, func_run_cutoff_id, batch_size)
                    .await
            }),
            tokio::spawn(async move {
                backfiller_logs
                    .upload_all_func_run_logs(logs_token, func_run_log_cutoff_id, batch_size)
                    .await
            }),
        ) {
            (Ok(_), Ok(_)) => Ok(()),
            (Ok(_), err @ Err(_)) => err?,
            (err @ Err(_), Ok(_)) => err?,
            (err @ Err(_), Err(e)) => {
                error!(
                    logs_error = e.to_string(),
                    "got error for both tasks. exiting on func runs error. logging the func run logs error here."
                );
                err?
            }
        }
    }
}
