use std::{
    collections::HashSet,
    future::IntoFuture as _,
};

use dal::{
    ChangeSetStatus,
    ServicesContext,
    TransactionsError,
    WorkspaceSnapshotAddress,
};
use si_data_pg::{
    PgError,
    PgPoolError,
};
use si_layer_cache::LayerDbError;
use strum::IntoEnumIterator;
use telemetry::prelude::*;
use thiserror::Error;
use tokio_util::{
    sync::CancellationToken,
    task::TaskTracker,
};

use crate::{
    Config,
    init,
};

#[remain::sorted]
#[derive(Debug, Error)]
pub enum GarbageCollectorError {
    #[error("error while initializing: {0}")]
    Init(#[from] init::InitError),
    #[error("Layer DB error: {0}")]
    LayerDb(#[from] LayerDbError),
    #[error("Pg error: {0}")]
    Pg(#[from] PgError),
    #[error("Pg Pool error: {0}")]
    PgPool(#[from] PgPoolError),
    #[error("Transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("Unable to query workspace snapshots")]
    UnableToQuerySnapshots,
}

type Result<T> = std::result::Result<T, GarbageCollectorError>;

pub struct SnapshotGarbageCollector {
    services_context: ServicesContext,
}

impl SnapshotGarbageCollector {
    #[instrument(name = "sdf.snapshot_garbage_collector.new", level = "info", skip_all)]
    pub async fn new(
        config: Config,
        task_tracker: &TaskTracker,
        task_token: CancellationToken,
    ) -> Result<Self> {
        let (services_context, layer_db_graceful_shutdown) =
            init::services_context_from_config(&config, task_token).await?;

        task_tracker.spawn(layer_db_graceful_shutdown.into_future());

        Ok(Self { services_context })
    }

    #[instrument(
        name = "sdf.snapshot_garbage_collector.garbage_collect_snapshots",
        level = "info",
        skip_all
    )]
    pub async fn garbage_collect_snapshots(self) -> Result<()> {
        let span = current_span_for_instrument_at!("info");

        let dal_context = self.services_context.clone().into_builder(true);
        let ctx = dal_context
            .build_default(None)
            .await
            .map_err(|err| span.record_err(err))?;
        let ctx = &ctx;

        let mut open_change_set_snapshot_ids = HashSet::new();
        let mut all_snapshot_ids = HashSet::new();

        // Gather the WorkspaceSnapshotAddress of all open change sets.
        let open_statuses: Vec<String> = ChangeSetStatus::iter()
            .filter_map(|status| {
                if status.is_active_or_applying() {
                    Some(status.to_string())
                } else {
                    None
                }
            })
            .collect();
        info!(
            "Change set status(es) to consider 'active': {:?}",
            &open_statuses
        );
        let change_set_snapshot_rows = ctx.txns().await?.pg().query(
            "SELECT workspace_snapshot_address AS snapshot_id FROM change_set_pointers WHERE status = ANY($1::text[]) GROUP BY workspace_snapshot_address",
            &[&open_statuses],
        ).await?;
        for row in change_set_snapshot_rows {
            let snapshot_id: WorkspaceSnapshotAddress = row.try_get("snapshot_id")?;
            open_change_set_snapshot_ids.insert(snapshot_id);
        }
        info!(
            "Found {} distinct snapshot address(es) for open change sets.",
            open_change_set_snapshot_ids.len()
        );

        // Gather the WorkspaceSnapshotAddress of all existing snapshots that are
        // at least an hour old.
        //
        // By only considering the snapshots that are at least an hour old, we avoid
        // race conditions where a change set is created, or modified between when we
        // queried the change_set_pointers table and the workspace_snapshots table.
        // We can't rely on transactional integrity to avoid race conditions as the
        // tables are in completely separate databases.
        let snapshot_id_rows = ctx
            .layer_db()
            .workspace_snapshot()
            .cache
            .pg()
            .query(
                "SELECT key AS snapshot_id
                 FROM workspace_snapshots
                 WHERE created_at < NOW() - '1 hour'::interval
                 GROUP BY key",
                &[],
            )
            .await?
            .ok_or_else(|| GarbageCollectorError::UnableToQuerySnapshots)?;
        for row in snapshot_id_rows {
            let snapshot_id: WorkspaceSnapshotAddress = row.try_get("snapshot_id")?;
            all_snapshot_ids.insert(snapshot_id);
        }
        info!(
            "Found {} distinct snapshot address(es) older than cutoff.",
            all_snapshot_ids.len()
        );

        // Any WorkspaceSnapshotAddress not in both open_change_set_snapshot_ids
        // and all_snapshot_ids is for a closed/applied/abandoned change set and
        // can be deleted, as we do not allow re-opening change sets.
        let snapshot_ids_to_delete: HashSet<_> = all_snapshot_ids
            .difference(&open_change_set_snapshot_ids)
            .collect();
        info!(
            "Found {} snapshot address(es) only used by inactive change sets.",
            snapshot_ids_to_delete.len()
        );

        let mut counter = 0;
        for key in snapshot_ids_to_delete.iter().take(10_000) {
            ctx.layer_db()
                .workspace_snapshot()
                .cache
                .pg()
                .delete(&key.to_string())
                .await?;

            counter += 1;
            if counter % 100 == 0 {
                info!("Deleted {} snapshot addresses.", counter);
            }
        }
        info!("Deleted {} snapshot address(es).", counter);

        ctx.commit().await?;

        span.record_ok();
        Ok(())
    }
}
