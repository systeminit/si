use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use aws_config::retry::RetryConfig;
use aws_sdk_s3::client::Waiters;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;
use aws_sdk_s3::{config::Builder, error::SdkError};

use serde::{Deserialize, Serialize};
use telemetry::tracing::info;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use crate::{error::LayerDbResult, event::LayeredEvent};

#[derive(Clone, Debug)]
pub struct ObjectCache {
    bucket: String,
    client: Client,
    prefix: String,
    semaphore: Arc<Semaphore>,
}

impl ObjectCache {
    pub async fn new(cache_config: ObjectCacheConfig) -> LayerDbResult<Self> {
        let config = aws_config::load_from_env().await;

        let mut builder = Builder::from(&config);
        builder.set_force_path_style(Some(true));
        builder.set_retry_config(Some(RetryConfig::adaptive()));

        if cache_config.endpoint.is_some() {
            builder.set_endpoint_url(cache_config.endpoint);
        }

        let config = builder.build();
        let client = aws_sdk_s3::Client::from_conf(config);

        let new = Self {
            bucket: cache_config.bucket,
            client,
            prefix: cache_config.prefix,
            semaphore: Semaphore::new(cache_config.concurrency_limit).into(),
        };

        new.ensure_bucket_exists().await?;

        Ok(new)
    }

    pub async fn get(&self, key: Arc<str>) -> LayerDbResult<Option<Vec<u8>>> {
        let obj = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(self.key_with_prefix(key.as_ref()))
            .response_content_type("application/octet-stream")
            .send()
            .await;

        match obj {
            Ok(obj) => {
                let data = obj.body.collect().await?.into_bytes().to_vec();
                if data.is_empty() {
                    return Ok(None);
                }
                Ok(Some(data))
            }
            Err(SdkError::ServiceError(err)) if err.err().is_no_such_key() => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn get_many(
        &self,
        keys: &[Arc<str>],
    ) -> LayerDbResult<Option<HashMap<String, Vec<u8>>>> {
        let mut results = HashMap::new();
        let mut tasks = JoinSet::new();

        for key in keys {
            let self_clone = self.clone();
            let key_clone = key.clone();
            let semaphore = self.semaphore.clone();

            tasks.spawn(async move {
                let _permit = semaphore.acquire().await;
                let result = self_clone.get(key_clone.clone()).await;
                result.map(|v| v.map(|val| (key_clone.to_string(), val)))
            });
        }

        tasks.join_all().await.into_iter().for_each(|response| {
            if let Ok(Some((key, value))) = response {
                results.insert(key, value);
            }
        });

        if results.is_empty() {
            return Ok(None);
        }

        Ok(Some(results))
    }

    pub async fn contains_key(&self, key: Arc<str>) -> LayerDbResult<bool> {
        let result = self
            .client
            .wait_until_object_exists()
            .bucket(&self.bucket)
            .key(self.key_with_prefix(key.as_ref()))
            .wait(Duration::from_millis(100))
            .await?
            .into_result();

        Ok(result.is_ok())
    }

    pub async fn insert(&self, key: Arc<str>, value: Vec<u8>) -> LayerDbResult<()> {
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(self.key_with_prefix(key.as_ref()))
            .body(ByteStream::from(value))
            .send()
            .await?;

        Ok(())
    }

    pub async fn remove(&self, key: Arc<str>) -> LayerDbResult<()> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(self.key_with_prefix(key.as_ref()))
            .send()
            .await?;

        Ok(())
    }

    pub async fn write_to_cache(mut self, event: Arc<LayeredEvent>) -> LayerDbResult<()> {
        self = self.with_prefix(event.payload.db_name.to_string());
        self.insert(event.payload.key.clone(), event.payload.value.to_vec())
            .await?;
        Ok(())
    }

    pub async fn remove_from_cache(mut self, event: Arc<LayeredEvent>) -> LayerDbResult<()> {
        self = self.with_prefix(event.payload.db_name.to_string());
        self.remove(event.payload.key.clone()).await?;
        Ok(())
    }

    async fn ensure_bucket_exists(&self) -> LayerDbResult<()> {
        let client = self.client.clone();
        let bucket = self.bucket.to_owned();
        match client.head_bucket().bucket(&bucket).send().await {
            Ok(_) => Ok(()),
            Err(SdkError::ServiceError(err)) if err.err().is_not_found() => {
                info!("Bucket '{}' does not exist. Creating it...", &bucket);
                client.create_bucket().bucket(bucket).send().await?;
                Ok(())
            }
            Err(e) => Err(e.into()),
        }
    }

    fn key_with_prefix(&self, key: &str) -> String {
        format!("{}/{}", self.prefix, key)
    }

    pub fn with_prefix(mut self, prefix: String) -> Self {
        self.prefix = prefix;
        self
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ObjectCacheConfig {
    pub bucket: String,
    pub concurrency_limit: usize,
    pub endpoint: Option<String>,
    pub prefix: String,
}

impl ObjectCacheConfig {
    pub fn with_prefix(mut self, prefix: String) -> Self {
        self.prefix = prefix;
        self
    }

    pub fn with_endpoint(mut self, endpoint: String) -> Self {
        self.endpoint = Some(endpoint);
        self
    }
}

impl Default for ObjectCacheConfig {
    fn default() -> Self {
        Self {
            bucket: "si-local".to_string(),
            concurrency_limit: 5500,
            endpoint: None,
            prefix: "dummy".to_string(),
        }
    }
}
