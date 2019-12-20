use crate::collection::{Collection, SharedCollection};
use crate::error::CouchbaseError;
use crate::instance::{Instance, SharedInstance};
use crate::options::{AnalyticsOptions, QueryOptions};
use crate::result::{AnalyticsResult, QueryResult};
use std::rc::Rc;
use std::sync::Arc;

/// Provides access to `Bucket` level operations and `Collections`.
#[derive(Debug)]
pub struct Bucket {
    instance: Rc<Instance>,
}

impl Bucket {
    /// Internal method to create a new bucket, which in turn creates the lcb instance
    /// attached to this bucket.
    pub(crate) fn new(cs: &str, user: &str, pw: &str) -> Result<Self, CouchbaseError> {
        let instance = Instance::new(cs, user, pw)?;
        Ok(Self {
            instance: Rc::new(instance),
        })
    }

    /// Opens the default `Collection`.
    ///
    /// This method provides access to the default collection, which is present if you do
    /// not have any collections (upgrading from an older cluster) or if you are on a
    /// Couchbase Server version which does not support collections yet.
    pub fn default_collection(&self) -> Collection {
        Collection::new(self.instance.clone())
    }

    /// Internal proxy method that gets called from the cluster so we can send it into the
    /// instance.
    pub(crate) async fn query<S>(
        &self,
        statement: S,
        options: Option<QueryOptions>,
    ) -> Result<QueryResult, CouchbaseError>
    where
        S: Into<String>,
    {
        self.instance.query(statement.into(), options).await
    }

    /// Internal proxy method that gets called from the cluster so we can send it into the
    /// instance.
    pub(crate) async fn analytics_query<S>(
        &self,
        statement: S,
        options: Option<AnalyticsOptions>,
    ) -> Result<AnalyticsResult, CouchbaseError>
    where
        S: Into<String>,
    {
        self.instance.analytics_query(statement.into(), options).await
    }

    /// Internal proxy method that gets called from the cluster so we can send it into the
    /// instance.
    pub(crate) fn close(&self) -> Result<(), CouchbaseError> {
        self.instance.shutdown()
    }
}

/// Provides access to `SharedBucket` level operations and `SharedCollections`.
#[derive(Debug)]
pub struct SharedBucket {
    instance: Arc<SharedInstance>,
}

impl SharedBucket {
    /// Internal method to create a new bucket, which in turn creates the lcb instance
    /// attached to this bucket.
    pub(crate) fn new(cs: &str, user: &str, pw: &str) -> Result<Self, CouchbaseError> {
        let instance = SharedInstance::new(cs, user, pw)?;
        Ok(Self {
            instance: Arc::new(instance),
        })
    }

    /// Opens the default `SharedCollection`.
    ///
    /// This method provides access to the default collection, which is present if you do
    /// not have any collections (upgrading from an older cluster) or if you are on a
    /// Couchbase Server version which does not support collections yet.
    pub fn default_collection(&self) -> SharedCollection {
        SharedCollection::new(self.instance.clone())
    }

    /// Internal proxy method that gets called from the cluster so we can send it into the
    /// instance.
    pub(crate) async fn query<S>(
        &self,
        statement: S,
        options: Option<QueryOptions>,
    ) -> Result<QueryResult, CouchbaseError>
    where
        S: Into<String>,
    {
        self.instance.query(statement.into(), options).await
    }

    /// Internal proxy method that gets called from the cluster so we can send it into the
    /// instance.
    pub(crate) async fn analytics_query<S>(
        &self,
        statement: S,
        options: Option<AnalyticsOptions>,
    ) -> Result<AnalyticsResult, CouchbaseError>
    where
        S: Into<String>,
    {
        self.instance.analytics_query(statement.into(), options).await
    }

    /// Internal proxy method that gets called from the cluster so we can send it into the
    /// instance.
    pub(crate) async fn close(&self) -> Result<(), CouchbaseError> {
        self.instance.shutdown().await
    }
}
