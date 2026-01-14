use dal::ServicesContext;
use futures::future::try_join_all;
use telemetry::prelude::*;
use tokio_util::sync::CancellationToken;

use super::{
    BackfillResult,
    backfill::backfill_cache,
    config::BackfillConfig,
};
use crate::init;

pub struct LayerCacheBackfiller {
    services_context: ServicesContext,
    config: BackfillConfig,
}

impl LayerCacheBackfiller {
    #[instrument(name = "sdf.layer_cache_backfiller.new", level = "info", skip_all)]
    pub async fn new(
        config: crate::Config,
        backfill_config: BackfillConfig,
        task_tracker: &tokio_util::task::TaskTracker,
        task_token: CancellationToken,
    ) -> BackfillResult<Self> {
        let (services_context, layer_db_graceful_shutdown) =
            init::services_context_from_config(&config, task_token).await?;

        task_tracker.spawn(layer_db_graceful_shutdown.into_future());

        Ok(Self {
            services_context,
            config: backfill_config,
        })
    }

    #[instrument(
        name = "sdf.layer_cache_backfiller.backfill_all_caches",
        level = "info",
        skip_all
    )]
    pub async fn backfill_all_caches(
        self,
        shutdown_token: CancellationToken,
    ) -> BackfillResult<()> {
        let cache_types = &self.config.cache_types;

        info!(
            cache_count = cache_types.len(),
            cache_types = ?cache_types,
            "starting backfill for all cache types"
        );

        // Spawn one task per cache type
        let handles: Vec<_> = cache_types
            .iter()
            .map(|cache_type| {
                let ctx = self.services_context.clone();
                let config = self.config.clone();
                let token = shutdown_token.clone();
                let cache_type = cache_type.clone();

                tokio::spawn(async move { backfill_cache(ctx, cache_type, config, token).await })
            })
            .collect();

        // Wait for all caches to complete (each task checks shutdown_token internally)
        let results = try_join_all(handles).await?;

        for result in results {
            result?;
        }

        info!("completed backfill for all cache types");

        Ok(())
    }
}
