use std::time::Instant;

use dal::{DalContext, ServicesContext};
use telemetry::prelude::*;
use telemetry_utils::monotonic;
use tokio_util::sync::CancellationToken;
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
    pub async fn upload_all_func_runs(
        self,
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
            let id_batch = func_run_db
                .read_batch_of_ids(batch_size, cutoff_id)
                .await?;

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
}
