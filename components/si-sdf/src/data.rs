use couchbase::{options::QueryOptions, options::ScanConsistency, SharedBucket, SharedCluster};
use futures::stream::StreamExt;
use reqwest;
use serde::de::DeserializeOwned;
use si_settings::Settings;
use sodiumoxide::crypto::secretbox;
use thiserror::Error;

use std::collections::HashMap;
use std::sync::Arc;

lazy_static::lazy_static! {
    pub static ref REQWEST: reqwest::Client = reqwest::Client::new();
}

#[derive(Error, Debug)]
pub enum DataError {
    #[error("couchbase error: {0}")]
    Couchbase(#[from] couchbase::CouchbaseError),
}

pub type DataResult<T> = Result<T, DataError>;

pub use nats::asynk::Connection;

#[derive(Debug, Clone)]
pub struct Db {
    pub cluster: SharedCluster,
    pub bucket: Arc<SharedBucket>,
    pub bucket_name: Arc<String>,
    pub scan_consistency: ScanConsistency,
    pub page_secret_key: secretbox::Key,
}

impl Db {
    #[tracing::instrument(level = "trace")]
    pub fn new(settings: &Settings) -> DataResult<Self> {
        let mut cluster = SharedCluster::connect(
            &settings.db.cluster_url,
            &settings.db.cluster_user,
            &settings.db.cluster_password,
        )?;

        // Buckets are cached forever, so you are supposed to connect once, early.
        // Later, when you ask for a bucket, you can get it back.
        let bucket = cluster.bucket(&settings.db.bucket_name)?;

        tracing::info!("couchbase cluster connected");

        let scan_consistency = match settings.db.scan_consistency.as_ref() {
            "NotBounded" => ScanConsistency::NotBounded,
            "RequestPlus" => ScanConsistency::RequestPlus,
            _ => ScanConsistency::NotBounded,
        };

        Ok(Db {
            cluster,
            bucket,
            bucket_name: Arc::new(settings.db.bucket_name.clone()),
            scan_consistency,
            page_secret_key: settings.paging.key.clone(),
        })
    }

    #[tracing::instrument(level = "trace")]
    pub async fn query<I>(
        &self,
        query: String,
        named_params: Option<HashMap<String, serde_json::Value>>,
    ) -> DataResult<Vec<I>>
    where
        I: DeserializeOwned + std::fmt::Debug,
    {
        let query_options = QueryOptions::new().set_scan_consistency(self.scan_consistency);
        let named_options = match named_params {
            Some(hashmap) => Some(query_options.set_named_parameters(hashmap)),
            None => Some(query_options),
        };
        tracing::trace!("calling query");
        let mut result = self.cluster.query(query, named_options).await?;
        let mut result_stream = result.rows_as::<I>()?;
        let mut final_vec: Vec<I> = Vec::new();
        while let Some(r) = result_stream.next().await {
            match r {
                Ok(v) => final_vec.push(v),
                Err(e) => return Err(DataError::Couchbase(e)),
            }
        }
        Ok(final_vec)
    }

    #[tracing::instrument(level = "trace")]
    pub async fn get<S, T>(&self, id: S) -> DataResult<T>
    where
        S: Into<String> + std::fmt::Debug,
        T: DeserializeOwned + std::fmt::Debug,
    {
        let id_string = id.into();
        let item = {
            let bucket = self.bucket.clone();
            let collection = bucket.default_collection();
            collection.get(id_string, None).await?
        };
        Ok(item.content_as::<T>()?)
    }
}

pub async fn create_index(db: &Db, index: impl AsRef<str>) -> DataResult<()> {
    let index = index.as_ref();
    let mut result = db.cluster.query(index, None).await?;
    let meta = result.meta().await?;
    match meta.errors {
        Some(error) => tracing::debug!(?error, "index already exists"),
        None => tracing::debug!("created index"),
    }
    Ok(())
}

pub async fn create_indexes(db: &Db) -> DataResult<()> {
    create_index(
        &db,
        format!(
            "CREATE INDEX `idx_si_storable_typename` on `{bucket}`(siStorable.typeName)",
            bucket = db.bucket_name
        ),
    )
    .await?;
    create_index(
        &db,
        format!(
            "CREATE INDEX `idx_si_changeset_changesetid` on `{bucket}`(siChangeSet.changeSetId)",
            bucket = db.bucket_name
        ),
    )
    .await?;
    create_index(
        &db,
        format!(
            "CREATE INDEX `idx_id` on `{bucket}`(id)",
            bucket = db.bucket_name
        ),
    )
    .await?;

    Ok(())
}

pub async fn delete_data(db: &Db) -> DataResult<()> {
    let delete_query = format!(
        "DELETE FROM `{bucket}` WHERE id IS VALUED",
        bucket = db.bucket_name
    );
    let mut result = db.cluster.query(delete_query, None).await?;
    let meta = result.meta().await?;
    match meta.errors {
        Some(error) => tracing::error!("issue deleting: {}", error),
        None => (),
    }
    Ok(())
}
