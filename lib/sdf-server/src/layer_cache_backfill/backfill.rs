use std::time::Instant;

use dal::ServicesContext;
use telemetry::prelude::*;
use telemetry_utils::monotonic;
use tokio_util::sync::CancellationToken;

use super::{
    BackfillResult,
    config::BackfillConfig,
    helpers::{
        check_s3_exists,
        fetch_key_batch,
        fetch_pg_value,
        upload_to_s3,
    },
};

pub async fn backfill_cache(
    ctx: ServicesContext,
    cache_type: String,
    config: BackfillConfig,
    shutdown_token: CancellationToken,
) -> BackfillResult<()> {
    let mut key_batch = Vec::new();
    let mut current_cutoff_timestamp = config.cutoff_timestamp;
    let mut last_checkpoint = Instant::now();

    info!(
        cache_type = cache_type.as_str(),
        cutoff_timestamp = %config.cutoff_timestamp,
        "starting backfill"
    );

    loop {
        if shutdown_token.is_cancelled() {
            info!(
                cache_type = cache_type.as_str(),
                current_timestamp = %current_cutoff_timestamp,
                "backfill shutting down gracefully"
            );
            break;
        }

        // Refill key batch if empty
        if key_batch.is_empty() {
            key_batch = fetch_key_batch(
                &ctx,
                &cache_type,
                current_cutoff_timestamp,
                config.key_batch_size,
            )
            .await?;

            // Reverse so pop() gives us newest-first (query returns DESC order)
            key_batch.reverse();

            trace!(
                cache_type = cache_type.as_str(),
                cutoff = %current_cutoff_timestamp,
                batch_size = key_batch.len(),
                "fetched key batch"
            );

            if key_batch.is_empty() {
                break;
            }
        }

        // Pop is guaranteed to succeed since we just verified batch is non-empty
        let key_info = key_batch.pop().expect("batch verified non-empty");
        current_cutoff_timestamp = key_info.created_at;

        let exists = check_s3_exists(&ctx, &cache_type, &key_info.key).await?;

        if exists {
            trace!(
                cache_type = cache_type.as_str(),
                key = %key_info.key,
                "skipping key already in S3"
            );
            monotonic!(
                layer_cache.backfill.items_skipped = 1,
                cache_type = cache_type.as_str()
            );
        } else {
            let value = fetch_pg_value(&ctx, &cache_type, &key_info.key).await?;
            let value_size = value.len();

            upload_to_s3(&ctx, &cache_type, &key_info.key, &value).await?;

            trace!(
                cache_type = cache_type.as_str(),
                key = %key_info.key,
                size = value_size,
                "uploaded to S3"
            );

            monotonic!(
                layer_cache.backfill.items_uploaded = 1,
                cache_type = cache_type.as_str()
            );
            monotonic!(
                layer_cache.backfill.bytes_uploaded = value_size as u64,
                cache_type = cache_type.as_str()
            );
        }

        monotonic!(
            layer_cache.backfill.items_processed = 1,
            cache_type = cache_type.as_str()
        );

        // Time-based checkpoint for progress visibility
        if last_checkpoint.elapsed() > config.checkpoint_interval {
            info!(
                cache_type = cache_type.as_str(),
                current_timestamp = %current_cutoff_timestamp,
                "backfill checkpoint"
            );
            last_checkpoint = Instant::now();
        }
    }

    // Final log when cache complete
    info!(cache_type = cache_type.as_str(), "completed backfill");

    Ok(())
}
