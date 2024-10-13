use std::sync::Arc;

use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use telemetry::prelude::*;
use tokio::select;
use tokio_util::sync::CancellationToken;

use crate::error::LayerDbResult;
use crate::event::LayeredEvent;

const DEFAULT_CACHE_TTL_SECONDS: u64 = 60 * 60 * 24; // 24 hours
const DEFAULT_CHECK_CACHE_TTL_SECONDS: u64 = 60 * 60; // check every hour

#[derive(Clone, Debug)]
pub struct DiskCache {
    ttl: Duration,
    ttl_check_interval: Duration,
    write_path: Arc<PathBuf>,
}

impl DiskCache {
    pub fn new(config: DiskCacheConfig) -> LayerDbResult<Self> {
        let cache = Self {
            ttl: config.ttl,
            ttl_check_interval: config.ttl_check_interval,
            write_path: config.path,
        };
        Ok(cache)
    }

    pub fn write_path(&self) -> PathBuf {
        self.write_path.as_ref().clone()
    }

    pub async fn get(&self, key: Arc<str>) -> LayerDbResult<Vec<u8>> {
        let data = cacache::read(self.write_path.as_ref(), key.clone()).await?;

        // we need to ensure that recently-accessed items have up to date metadata so the TTL does
        // not clean them up inappropriately
        self.update_cache_entry(key.clone(), data.clone());

        Ok(data)
    }

    pub async fn contains_key(&self, key: Arc<str>) -> LayerDbResult<bool> {
        let result = cacache::metadata(self.write_path.as_ref(), key).await?;
        Ok(result.is_some())
    }

    pub async fn insert(&self, key: Arc<str>, value: Vec<u8>) -> LayerDbResult<()> {
        cacache::write(self.write_path.as_ref(), key, value).await?;
        Ok(())
    }

    pub async fn remove(&self, key: Arc<str>) -> LayerDbResult<()> {
        let maybe_metadata = cacache::metadata(self.write_path.as_ref(), key).await?;
        if let Some(metadata) = maybe_metadata {
            cacache::remove_hash(self.write_path.as_ref(), &metadata.integrity).await?;
        }
        Ok(())
    }

    pub async fn write_to_disk(&self, event: Arc<LayeredEvent>) -> LayerDbResult<()> {
        self.insert(event.payload.key.clone(), event.payload.value.to_vec())
            .await?;
        Ok(())
    }

    pub async fn remove_from_disk(&self, event: Arc<LayeredEvent>) -> LayerDbResult<()> {
        self.remove(event.payload.key.clone()).await?;
        Ok(())
    }

    async fn cleanup(&self) {
        for md in cacache::list_sync(self.write_path.as_ref()).flatten() {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect(
                    "unable to get the current time, what does this mean? How could this happen?",
                )
                .as_millis();
            if now - md.time > self.ttl.as_millis() {
                if let Err(err) = self.remove(md.key.into()).await {
                    warn!("unable to remove item from disk cache: {}", err);
                };
            }
        }
    }

    pub fn start_cleanup_task(&self, cancellation_token: CancellationToken) {
        let self_clone = self.clone();

        info!(
            "starting disk cache cleanup task for {}",
            self.write_path.display()
        );

        tokio::spawn(async move {
            loop {
                select! {
                    _ = tokio::time::sleep(self_clone.ttl_check_interval) => {
                        self_clone.cleanup().await;
                    }
                    _ = cancellation_token.cancelled() => {
                        break;
                    }
                }
            }
        });
    }

    fn update_cache_entry(&self, key: Arc<str>, value: Vec<u8>) {
        let me = self.clone();
        tokio::spawn(async move { me.insert(key, value).await });
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DiskCacheConfig {
    pub path: Arc<PathBuf>,
    pub ttl: Duration,
    pub ttl_check_interval: Duration,
}

impl DiskCacheConfig {
    pub fn new(
        dir: impl Into<PathBuf>,
        table_name: impl Into<String>,
        ttl: Duration,
        ttl_check_interval: Duration,
    ) -> Self {
        let dir = dir.into();
        let table_name_string = table_name.into();
        let write_path = dir.join(table_name_string);
        Self {
            path: write_path.into(),
            ttl,
            ttl_check_interval,
        }
    }

    pub fn default_for_service(service: &str) -> Self {
        let prefix = format!("{}-cache-", service);
        let dir = tempfile::TempDir::with_prefix_in(prefix, "/tmp")
            .expect("unable to create tmp dir for layerdb")
            .into_path();
        Self::new(
            dir,
            service,
            Duration::from_secs(DEFAULT_CACHE_TTL_SECONDS),
            Duration::from_secs(DEFAULT_CHECK_CACHE_TTL_SECONDS),
        )
    }
}

impl Default for DiskCacheConfig {
    fn default() -> Self {
        let path = tempfile::TempDir::with_prefix_in("default-cache-", "/tmp")
            .expect("unable to create tmp dir for layerdb")
            .into_path();
        Self {
            path: Arc::new(path),
            ttl: Duration::from_secs(DEFAULT_CACHE_TTL_SECONDS),
            ttl_check_interval: Duration::from_secs(DEFAULT_CHECK_CACHE_TTL_SECONDS),
        }
    }
}
