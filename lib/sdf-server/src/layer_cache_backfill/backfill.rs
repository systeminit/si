use std::{
    collections::HashMap,
    time::Instant,
};

use dal::ServicesContext;
use telemetry::prelude::*;
use telemetry_utils::monotonic;
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;

use super::{
    BackfillResult,
    config::BackfillConfig,
    helpers::{
        KeyInfo,
        check_s3_exists,
        fetch_key_batch,
        fetch_pg_value,
        upload_to_s3,
    },
};

#[derive(Debug)]
struct UploadStats {
    key_info: KeyInfo,
    skipped: bool,
    bytes_uploaded: Option<usize>,
}

async fn upload_task(
    ctx: ServicesContext,
    cache_type: String,
    key_info: KeyInfo,
) -> BackfillResult<UploadStats> {
    let exists = check_s3_exists(&ctx, &cache_type, &key_info.key).await?;

    if exists {
        trace!(
            cache_type = cache_type.as_str(),
            key = %key_info.key,
            "skipping key already in S3"
        );
        return Ok(UploadStats {
            key_info,
            skipped: true,
            bytes_uploaded: None,
        });
    }

    let value = fetch_pg_value(&ctx, &cache_type, &key_info.key).await?;
    let value_size = value.len();

    upload_to_s3(&ctx, &cache_type, &key_info.key, &value).await?;

    trace!(
        cache_type = cache_type.as_str(),
        key = %key_info.key,
        size = value_size,
        "uploaded to S3"
    );

    Ok(UploadStats {
        key_info,
        skipped: false,
        bytes_uploaded: Some(value_size),
    })
}

pub async fn backfill_cache(
    ctx: ServicesContext,
    cache_type: String,
    config: BackfillConfig,
    shutdown_token: CancellationToken,
) -> BackfillResult<()> {
    let mut key_batch = Vec::new();
    let mut current_cutoff_timestamp = config.cutoff_timestamp;
    let mut last_checkpoint = Instant::now();
    let mut joinset: JoinSet<BackfillResult<UploadStats>> = JoinSet::new();
    let mut in_flight_timestamps = HashMap::new();
    let mut checkpoint_timestamp = config.cutoff_timestamp;

    info!(
        cache_type = cache_type.as_str(),
        cutoff_timestamp = %config.cutoff_timestamp,
        max_concurrent = config.max_concurrent_uploads,
        "starting backfill"
    );

    loop {
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

            if key_batch.is_empty() && joinset.is_empty() {
                // No more keys and no in-flight tasks
                break;
            }
        }

        // Fill joinset up to max_concurrent
        while joinset.len() < config.max_concurrent_uploads {
            if let Some(key_info) = key_batch.pop() {
                current_cutoff_timestamp = key_info.created_at;
                in_flight_timestamps.insert(key_info.key.clone(), key_info.created_at);

                let ctx_clone = ctx.clone();
                let cache_type_clone = cache_type.clone();
                joinset.spawn(upload_task(ctx_clone, cache_type_clone, key_info));
            } else {
                break; // Batch exhausted
            }
        }

        // Wait for either a task completion or shutdown signal
        tokio::select! {
            Some(result) = joinset.join_next() => {
                let stats = result??;

                // Remove completed key from in-flight tracking
                in_flight_timestamps.remove(&stats.key_info.key);

                // Update checkpoint to maximum in-flight timestamp
                checkpoint_timestamp = in_flight_timestamps
                    .values()
                    .max()
                    .copied()
                    .unwrap_or(current_cutoff_timestamp);

                // Update telemetry
                if stats.skipped {
                    monotonic!(
                        layer_cache.backfill.items_skipped = 1,
                        cache_type = cache_type.as_str()
                    );
                } else {
                    monotonic!(
                        layer_cache.backfill.items_uploaded = 1,
                        cache_type = cache_type.as_str()
                    );
                    if let Some(bytes) = stats.bytes_uploaded {
                        monotonic!(
                            layer_cache.backfill.bytes_uploaded = bytes as u64,
                            cache_type = cache_type.as_str()
                        );
                    }
                }

                monotonic!(
                    layer_cache.backfill.items_processed = 1,
                    cache_type = cache_type.as_str()
                );

                // Time-based checkpoint for progress visibility
                if last_checkpoint.elapsed() > config.checkpoint_interval {
                    info!(
                        cache_type = cache_type.as_str(),
                        checkpoint_timestamp = %checkpoint_timestamp,
                        active_uploads = joinset.len(),
                        "backfill checkpoint"
                    );
                    last_checkpoint = Instant::now();
                }
            }
            _ = shutdown_token.cancelled() => {
                info!(
                    cache_type = cache_type.as_str(),
                    checkpoint_timestamp = %checkpoint_timestamp,
                    in_flight_uploads = joinset.len(),
                    "backfill shutting down gracefully"
                );
                break;
            }
        }
    }

    // Final log when cache complete
    info!(cache_type = cache_type.as_str(), "completed backfill");

    Ok(())
}
