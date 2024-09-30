use std::collections::HashMap;
use std::sync::Arc;

use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;
use aws_sdk_s3::{config::Builder, error::SdkError};

use serde::{Deserialize, Serialize};
use telemetry::tracing::info;

use crate::{error::LayerDbResult, event::LayeredEvent};

#[derive(Clone, Debug)]
pub struct ObjectCache {
    bucket: String,
    client: Client,
}

impl ObjectCache {
    pub async fn new(cache_config: ObjectCacheConfig) -> LayerDbResult<Self> {
        let config = aws_config::load_from_env().await;
        let builder = Builder::from(&config)
            .endpoint_url(cache_config.endpoint)
            .build();

        let client = aws_sdk_s3::Client::from_conf(builder);

        let new = Self {
            bucket: cache_config.bucket,
            client,
        };

        new.ensure_bucket_exists().await?;

        Ok(new)
    }

    pub async fn get(&self, key: Arc<str>) -> LayerDbResult<Option<Vec<u8>>> {
        let obj = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key.as_ref())
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
        for key in keys {
            if let Some(result) = self.get(key.clone()).await? {
                results.insert(key.to_string(), result);
            }
        }
        if results.is_empty() {
            return Ok(None);
        }

        Ok(Some(results))
    }

    pub async fn contains_key(&self, key: Arc<str>) -> LayerDbResult<bool> {
        let result = self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(key.as_ref())
            .send()
            .await;

        Ok(result.is_ok())
    }

    pub async fn insert(&self, key: Arc<str>, value: Vec<u8>) -> LayerDbResult<()> {
        let byte_stream = ByteStream::from(value);

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key.as_ref())
            .body(byte_stream)
            .send()
            .await?;

        Ok(())
    }

    pub async fn remove(&self, key: Arc<str>) -> LayerDbResult<()> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key.as_ref())
            .send()
            .await?;

        Ok(())
    }

    pub async fn write_to_cache(&self, event: Arc<LayeredEvent>) -> LayerDbResult<()> {
        self.insert(event.payload.key.clone(), event.payload.value.to_vec())
            .await?;
        Ok(())
    }

    pub async fn remove_from_cache(&self, event: Arc<LayeredEvent>) -> LayerDbResult<()> {
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
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ObjectCacheConfig {
    bucket: String,
    endpoint: String,
}

impl Default for ObjectCacheConfig {
    fn default() -> Self {
        Self {
            bucket: "si-local".to_string(),
            endpoint: "http://0.0.0.0:4566".to_string(),
        }
    }
}
