use std::{
    sync::Arc,
    time::{
        Duration,
        Instant,
        SystemTime,
    },
};

use si_data_pg::PgPool;
use si_events::{
    Actor,
    ChangeSetId,
    Tenancy,
    WorkspacePk,
    WorkspaceSnapshotAddress,
};
use si_layer_cache::{
    db::workspace_snapshot::CACHE_NAME,
    event::{
        LayeredEvent,
        LayeredEventClient,
        LayeredEventKind,
    },
    pg::PgLayer,
};
use telemetry::prelude::*;
use telemetry_utils::{
    histogram,
    monotonic,
};
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

use crate::{
    SnapshotEvictionConfig,
    error::SnapshotEvictionResult,
};

pub struct SnapshotEvictor {
    si_db_pool: PgPool,
    layer_cache_pool: PgPool,
    layered_event_client: LayeredEventClient,
    config: SnapshotEvictionConfig,
}

impl SnapshotEvictor {
    pub fn new(
        si_db_pool: PgPool,
        layer_cache_pool: PgPool,
        layered_event_client: LayeredEventClient,
        config: SnapshotEvictionConfig,
    ) -> Self {
        Self {
            si_db_pool,
            layer_cache_pool,
            layered_event_client,
            config,
        }
    }

    /// Main eviction loop - polls for candidates until shutdown
    #[instrument(name = "snapshot_evictor.run", skip(self, shutdown))]
    pub async fn run(&self, shutdown: CancellationToken) -> SnapshotEvictionResult<()> {
        info!(
            grace_period_seconds = self.config.grace_period_seconds,
            poll_interval_seconds = self.config.poll_interval_seconds,
            "Snapshot eviction task starting"
        );

        loop {
            tokio::select! {
                biased;

                _ = shutdown.cancelled() => {
                    info!("Shutdown requested, stopping eviction task");
                    break;
                }

                _ = sleep(self.config.poll_interval()) => {
                    let cycle_start = Instant::now();
                    match self.process_all_candidates().await {
                        Ok(count) if count > 0 => {
                            let duration_secs = cycle_start.elapsed().as_secs_f64();
                            histogram!(snapshot_eviction_cycle_duration_seconds = duration_secs);
                            info!(evicted_count = count, duration_secs, "Eviction cycle complete");
                        }
                        Ok(_) => {
                            let duration_secs = cycle_start.elapsed().as_secs_f64();
                            histogram!(snapshot_eviction_cycle_duration_seconds = duration_secs);
                            debug!(duration_secs, "Eviction cycle complete, no candidates");
                        }
                        Err(e) => {
                            let duration_secs = cycle_start.elapsed().as_secs_f64();
                            histogram!(snapshot_eviction_cycle_duration_seconds = duration_secs);
                            error!(error = ?e, duration_secs, "Eviction cycle failed");
                            // Continue to next cycle despite errors
                        }
                    }
                }
            }
        }

        info!("Snapshot eviction task stopped");
        Ok(())
    }

    /// Process all candidates until none remain
    async fn process_all_candidates(&self) -> SnapshotEvictionResult<usize> {
        let mut total_evicted = 0;
        let mut cycle_successes = 0;

        loop {
            let candidates = self.find_candidates().await?;

            if candidates.is_empty() {
                break;
            }

            for (address, last_used_at) in candidates {
                match self.evict_snapshot(&address, last_used_at).await {
                    Ok(()) => {
                        cycle_successes += 1;
                        total_evicted += 1;
                        debug!(
                            snapshot_address = %address,
                            "Successfully evicted snapshot"
                        );
                    }
                    Err(e) => {
                        // Note: failures are now tracked per-eviction in evict_snapshot()
                        error!(
                            snapshot_address = %address,
                            error = ?e,
                            "Failed to evict snapshot, will retry next cycle"
                        );
                    }
                }
            }
        }

        // Record metrics once per cycle
        monotonic!(snapshot_eviction_successes = cycle_successes as u64);

        Ok(total_evicted)
    }

    /// Query for eviction candidates
    #[instrument(name = "snapshot_evictor.find_candidates", skip(self))]
    async fn find_candidates(
        &self,
    ) -> SnapshotEvictionResult<Vec<(WorkspaceSnapshotAddress, SystemTime)>> {
        let client = self.si_db_pool.get().await?;

        let rows = client
            .query(
                "SELECT s.snapshot_id, s.last_used_at
                 FROM snapshot_last_used s
                 LEFT JOIN change_set_pointers cs ON cs.workspace_snapshot_address = s.snapshot_id
                 WHERE s.last_used_at < (NOW() - $1 * INTERVAL '1 second')
                   AND cs.id IS NULL
                 ORDER BY s.last_used_at ASC
                 LIMIT $2
                 FOR UPDATE OF s SKIP LOCKED",
                &[
                    &(self.config.grace_period_seconds as f64),
                    &(self.config.batch_size as i64),
                ],
            )
            .await?;

        let mut candidates = Vec::with_capacity(rows.len());
        for row in rows {
            let address: WorkspaceSnapshotAddress = row.try_get("snapshot_id")?;
            let last_used_at: SystemTime = row.try_get("last_used_at")?;
            candidates.push((address, last_used_at));
        }

        // Record metric - histogram to track distribution of workload characteristics
        histogram!(snapshot_eviction_candidates_found = candidates.len() as f64);

        Ok(candidates)
    }

    /// Evict a single snapshot with metrics tracking
    async fn evict_snapshot(
        &self,
        address: &WorkspaceSnapshotAddress,
        last_used_at: SystemTime,
    ) -> SnapshotEvictionResult<()> {
        // Calculate candidate queue latency
        let grace_period = Duration::from_secs(self.config.grace_period_seconds as u64);
        let eligible_at = last_used_at + grace_period;
        let now = SystemTime::now();

        if let Ok(queue_time) = now.duration_since(eligible_at) {
            histogram!(snapshot_candidate_queue_latency_seconds = queue_time.as_secs_f64());
        }

        let start = Instant::now();

        match self.evict_snapshot_inner(address).await {
            Ok(()) => {
                // Record success duration (existing metric, already in evict_snapshot_inner)
                Ok(())
            }
            Err(e) => {
                // Extract error type for labeling
                let error_type = match &e {
                    crate::SnapshotEvictionError::Database(_) => "database",
                    crate::SnapshotEvictionError::PgPool(_) => "pg_pool",
                    crate::SnapshotEvictionError::LayerCache(_) => "layer_cache",
                    crate::SnapshotEvictionError::Nats(_) => "nats",
                };

                // Record failure counter with error type
                monotonic!(snapshot_eviction_failures = 1, error_type = error_type);

                // Record failure duration
                let duration_secs = start.elapsed().as_secs_f64();
                histogram!(
                    snapshot_eviction_duration_seconds = duration_secs,
                    status = "failure",
                    error_type = error_type
                );

                Err(e)
            }
        }
    }

    /// Evict a single snapshot - inner implementation
    #[instrument(name = "snapshot_evictor.evict_snapshot", skip(self))]
    async fn evict_snapshot_inner(
        &self,
        address: &WorkspaceSnapshotAddress,
    ) -> SnapshotEvictionResult<()> {
        let snapshot_id = address.to_string();
        let start = Instant::now();

        // Step 1: Delete from workspace_snapshots table (layer-cache database)
        let pg_layer = PgLayer::new(self.layer_cache_pool.clone(), CACHE_NAME);
        pg_layer.delete(&snapshot_id).await?;

        // Step 2: Publish NATS eviction event for cache invalidation
        let event = LayeredEvent::new(
            LayeredEventKind::SnapshotEvict,
            Arc::new(CACHE_NAME.to_string()),
            Arc::from(snapshot_id.as_str()),
            Arc::new(Vec::new()),
            Arc::new(CACHE_NAME.to_string()),
            None,                                                 // No web events for GC
            Tenancy::new(WorkspacePk::new(), ChangeSetId::new()), // System eviction
            Actor::System,
        );
        self.layered_event_client.publish(Arc::new(event)).await?;

        // Step 3: Delete metadata from si-db
        let client = self.si_db_pool.get().await?;
        client
            .execute(
                "DELETE FROM snapshot_last_used WHERE snapshot_id = $1",
                &[&snapshot_id],
            )
            .await?;

        // Record metrics
        let duration_secs = start.elapsed().as_secs_f64();
        histogram!(
            snapshot_eviction_duration_seconds = duration_secs,
            status = "success"
        );

        Ok(())
    }
}
