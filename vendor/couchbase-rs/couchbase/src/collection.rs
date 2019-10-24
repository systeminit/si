use crate::error::CouchbaseError;
use crate::instance::{Instance, SharedInstance};
use crate::options::*;
use crate::result::*;
use crate::subdoc::*;
use crate::util::JSON_COMMON_FLAG;
use serde::Serialize;
use serde_json::to_vec;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

/// `Collection` level access to operations.
pub struct Collection {
    instance: Rc<Instance>,
}

impl Collection {
    /// Creates a new `Collection`.
    ///
    /// This function is not intended to be called directly, but rather a new
    /// `Collection` should be retrieved through the `Bucket`.
    ///
    pub(crate) fn new(instance: Rc<Instance>) -> Self {
        Self { instance }
    }

    /// Fetches a document from the collection.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the document.
    /// * `options` - Options to customize the default behavior.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use couchbase::Cluster;
    /// use serde_json::Value;
    /// use futures::Future;
    /// # let mut cluster = Cluster::connect("couchbase://127.0.0.1", "Administrator", "password")
    /// #   .expect("Could not create Cluster reference!");
    /// # let bucket = cluster
    /// #   .bucket("travel-sample")
    /// #   .expect("Could not open bucket");
    /// # let collection = bucket.default_collection();
    /// #
    /// # async {
    /// let found_doc = collection
    ///     .get("airport_1297", None)
    ///     .await
    ///     .expect("Error while loading doc");
    ///
    ///     println!(
    ///         "Content Decoded {:?}",
    ///         found_doc.content_as::<Value>()
    ///     );
    /// # };
    /// ```
    pub async fn get<S>(
        &self,
        id: S,
        options: Option<GetOptions>,
    ) -> Result<GetResult, CouchbaseError>
    where
        S: Into<String>,
    {
        self.instance.get(id.into(), options).await
    }

    /// Fetches a document from the collection and write locks it.
    ///
    /// Note that the `lock` time can be overridden in the options struct. If none is set explicitly,
    /// the default duration of 30 seconds is used.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the document.
    /// * `options` - Options to customize the default behavior.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use couchbase::Cluster;
    /// use futures::Future;
    /// use serde_json::Value;
    /// # let mut cluster = Cluster::connect("couchbase://127.0.0.1", "Administrator", "password")
    /// #   .expect("Could not create Cluster reference!");
    /// # let bucket = cluster
    /// #   .bucket("travel-sample")
    /// #   .expect("Could not open bucket");
    /// # let collection = bucket.default_collection();
    /// #
    /// # async {
    /// let found_doc = collection
    ///     .get_and_lock("airport_1297", None)
    ///     .await
    ///     .expect("Error while loading and locking doc");
    ///
    ///     println!(
    ///         "Content Decoded {:?}",
    ///         found_doc.content_as::<Value>()
    ///    );
    /// # };
    /// ```
    pub async fn get_and_lock<S>(
        &self,
        id: S,
        options: Option<GetAndLockOptions>,
    ) -> Result<GetResult, CouchbaseError>
    where
        S: Into<String>,
    {
        self.instance.get_and_lock(id.into(), options).await
    }

    /// Fetches a document from the collection and modifies its expiry.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the document.
    /// * `expiration` - The new expiration of the document.
    /// * `options` - Options to customize the default behavior.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use couchbase::Cluster;
    /// use std::time::Duration;
    /// use serde_json::Value;
    /// use futures::Future;
    /// # let mut cluster = Cluster::connect("couchbase://127.0.0.1", "Administrator", "password")
    /// #   .expect("Could not create Cluster reference!");
    /// # let bucket = cluster
    /// #   .bucket("travel-sample")
    /// #   .expect("Could not open bucket");
    /// # let collection = bucket.default_collection();
    /// #
    /// # async {
    /// let found_doc = collection
    ///     .get_and_touch("airport_1297", Duration::from_secs(5), None)
    ///     .await
    ///     .expect("Error while loading and touching doc");
    ///
    ///     println!(
    ///         "Content Decoded {:?}",
    ///         found_doc.content_as::<Value>()
    ///     );
    /// # };
    /// ```
    pub async fn get_and_touch<S>(
        &self,
        id: S,
        expiration: Duration,
        options: Option<GetAndTouchOptions>,
    ) -> Result<GetResult, CouchbaseError>
    where
        S: Into<String>,
    {
        self.instance.get_and_touch(id.into(), expiration, options).await
    }

    /// Inserts or replaces a new document into the collection.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the document.
    /// * `content` - The content to store inside the document.
    /// * `options` - Options to customize the default behavior.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use couchbase::Cluster;
    /// use serde_derive::Serialize;
    /// use futures::Future;
    ///
    /// #[derive(Debug, Serialize)]
    /// struct Airport {
    ///     airportname: String,
    ///     icao: String,
    ///     iata: String,
    /// }
    /// # let mut cluster = Cluster::connect("couchbase://127.0.0.1", "Administrator", "password")
    /// #     .expect("Could not create Cluster reference!");
    /// # let bucket = cluster
    /// #     .bucket("travel-sample")
    /// #     .expect("Could not open bucket");
    /// # let collection = bucket.default_collection();
    ///
    /// let airport = Airport {
    ///     airportname: "Vienna Airport".into(),
    ///     icao: "LOWW".into(),
    ///     iata: "VIE".into(),
    /// };
    ///
    /// # async {
    /// collection
    ///     .upsert("airport_999", airport, None)
    ///     .await
    ///     .expect("could not upsert airport!");
    /// # };
    /// ```
    pub async fn upsert<S, T>(
        &self,
        id: S,
        content: T,
        options: Option<UpsertOptions>,
    ) -> Result<MutationResult, CouchbaseError>
    where
        S: Into<String>,
        T: Serialize,
    {
        let serialized = match to_vec(&content) {
            Ok(v) => v,
            Err(_e) => return Err(CouchbaseError::EncodingError),
        };
        let flags = JSON_COMMON_FLAG;
        self.instance.upsert(id.into(), serialized, flags, options).await
    }

    /// Inserts a document into the collection.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the document.
    /// * `content` - The content to store inside the document.
    /// * `options` - Options to customize the default behavior.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use couchbase::Cluster;
    /// use serde_derive::Serialize;
    /// use futures::Future;
    ///
    /// #[derive(Debug, Serialize)]
    /// struct Airport {
    ///     airportname: String,
    ///     icao: String,
    ///     iata: String,
    /// }
    /// # let mut cluster = Cluster::connect("couchbase://127.0.0.1", "Administrator", "password")
    /// #     .expect("Could not create Cluster reference!");
    /// # let bucket = cluster
    /// #     .bucket("travel-sample")
    /// #     .expect("Could not open bucket");
    /// # let collection = bucket.default_collection();
    ///
    /// let airport = Airport {
    ///     airportname: "Vienna Airport".into(),
    ///     icao: "LOWW".into(),
    ///     iata: "VIE".into(),
    /// };
    ///
    /// # async {
    /// collection
    ///     .insert("airport_999", airport, None)
    ///     .await
    ///     .expect("could not insert airport!");
    /// # };
    /// ```
    pub async fn insert<S, T>(
        &self,
        id: S,
        content: T,
        options: Option<InsertOptions>,
    ) -> Result<MutationResult, CouchbaseError>
    where
        S: Into<String>,
        T: Serialize,
    {
        let serialized = match to_vec(&content) {
            Ok(v) => v,
            Err(_e) => return Err(CouchbaseError::EncodingError),
        };
        let flags = JSON_COMMON_FLAG;
        self.instance.insert(id.into(), serialized, flags, options).await
    }

    /// Replaces an existing document in the collection.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the document.
    /// * `content` - The content to store inside the document.
    /// * `options` - Options to customize the default behavior.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use couchbase::Cluster;
    /// use serde_derive::Serialize;
    /// use futures::Future;
    ///
    /// #[derive(Debug, Serialize)]
    /// struct Airport {
    ///     airportname: String,
    ///     icao: String,
    ///     iata: String,
    /// }
    /// # let mut cluster = Cluster::connect("couchbase://127.0.0.1", "Administrator", "password")
    /// #     .expect("Could not create Cluster reference!");
    /// # let bucket = cluster
    /// #     .bucket("travel-sample")
    /// #     .expect("Could not open bucket");
    /// # let collection = bucket.default_collection();
    ///
    /// let airport = Airport {
    ///     airportname: "Vienna Airport".into(),
    ///     icao: "LOWW".into(),
    ///     iata: "VIE".into(),
    /// };
    ///
    /// # async {
    /// collection
    ///     .replace("airport_999", airport, None)
    ///     .await
    ///     .expect("could not replace airport!");
    /// # };
    /// ```
    pub async fn replace<S, T>(
        &self,
        id: S,
        content: T,
        options: Option<ReplaceOptions>,
    ) -> Result<MutationResult, CouchbaseError>
    where
        S: Into<String>,
        T: Serialize,
    {
        let serialized = match to_vec(&content) {
            Ok(v) => v,
            Err(_e) => return Err(CouchbaseError::EncodingError),
        };
        let flags = JSON_COMMON_FLAG;
        self.instance.replace(id.into(), serialized, flags, options).await
    }

    /// Removes a document from the collection.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the document.
    /// * `options` - Options to customize the default behavior.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use futures::Future;
    /// # use couchbase::Cluster;
    /// # let mut cluster = Cluster::connect("couchbase://127.0.0.1", "Administrator", "password")
    /// #   .expect("Could not create Cluster reference!");
    /// # let bucket = cluster
    /// #   .bucket("travel-sample")
    /// #   .expect("Could not open bucket");
    /// # let collection = bucket.default_collection();
    /// # async {
    /// let result = collection.remove("document_id", None).await;
    /// # };
    /// ```
    pub async fn remove<S>(
        &self,
        id: S,
        options: Option<RemoveOptions>,
    ) -> Result<MutationResult, CouchbaseError>
    where
        S: Into<String>,
    {
        self.instance.remove(id.into(), options).await
    }

    /// Changes the expiration time on a document.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the document.
    /// * `expiration` - The new expiration of the document.
    /// * `options` - Options to customize the default behavior.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use std::time::Duration;
    /// use futures::Future;
    /// # use couchbase::Cluster;
    /// # let mut cluster = Cluster::connect("couchbase://127.0.0.1", "Administrator", "password")
    /// #   .expect("Could not create Cluster reference!");
    /// # let bucket = cluster
    /// #   .bucket("travel-sample")
    /// #   .expect("Could not open bucket");
    /// # let collection = bucket.default_collection();
    /// # async {
    /// let result = collection.touch("document_id", Duration::from_secs(5), None).await;
    /// # };
    /// ```
    pub async fn touch<S>(
        &self,
        id: S,
        expiration: Duration,
        options: Option<TouchOptions>,
    ) -> Result<MutationResult, CouchbaseError>
    where
        S: Into<String>,
    {
        self.instance.touch(id.into(), expiration, options).await
    }

    /// Unlocks a write-locked document.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the document.
    /// * `cas` - The cas needed to remove the write lock.
    /// * `options` - Options to customize the default behavior.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use couchbase::Cluster;
    /// use futures::Future;
    /// # let mut cluster = Cluster::connect("couchbase://127.0.0.1", "Administrator", "password")
    /// #   .expect("Could not create Cluster reference!");
    /// # let bucket = cluster
    /// #   .bucket("travel-sample")
    /// #   .expect("Could not open bucket");
    /// # let collection = bucket.default_collection();
    /// let cas = 1234; // retrieved from a `getAndLock`
    /// # async {
    /// let result = collection.unlock("document_id", cas, None).await;
    /// # };
    /// ```
    pub async fn unlock<S>(
        &self,
        id: S,
        cas: u64,
        options: Option<UnlockOptions>,
    ) -> Result<MutationResult, CouchbaseError>
    where
        S: Into<String>,
    {
        self.instance.unlock(id.into(), cas, options).await
    }

    /// Checks if a document exists and if so returns a cas value with it.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the document.
    /// * `options` - Options to customize the default behavior.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use couchbase::Cluster;
    /// use futures::Future;
    /// # let mut cluster = Cluster::connect("couchbase://127.0.0.1", "Administrator", "password")
    /// #   .expect("Could not create Cluster reference!");
    /// # let bucket = cluster
    /// #   .bucket("travel-sample")
    /// #   .expect("Could not open bucket");
    /// # let collection = bucket.default_collection();
    /// # async {
    /// let result = collection.exists("document_id", None).await;
    /// # };
    /// ```
    pub async fn exists<S>(
        &self,
        id: S,
        options: Option<ExistsOptions>,
    ) -> Result<ExistsResult, CouchbaseError>
    where
        S: Into<String>,
    {
        self.instance.exists(id.into(), options).await
    }

    /// Extracts fragments of a document.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the document.
    /// * `specs` - The vector of specs that define what to fetch.
    /// * `options` - Options to customize the default behavior.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use couchbase::Cluster;
    /// use futures::Future;
    /// use couchbase::subdoc::LookupInSpec;
    /// # let mut cluster = Cluster::connect("couchbase://127.0.0.1", "Administrator", "password")
    /// #   .expect("Could not create Cluster reference!");
    /// # let bucket = cluster
    /// #   .bucket("travel-sample")
    /// #   .expect("Could not open bucket");
    /// # let collection = bucket.default_collection();
    /// # async {
    /// let partial_result = collection
    ///   .lookup_in("airport_1285", vec![LookupInSpec::get("geo")], None)
    ///   .await;
    /// # };
    /// ```
    pub async fn lookup_in<S>(
        &self,
        id: S,
        specs: Vec<LookupInSpec>,
        options: Option<LookupInOptions>,
    ) -> Result<LookupInResult, CouchbaseError>
    where
        S: Into<String>,
    {
        self.instance.lookup_in(id.into(), specs, options).await
    }

    /// Changes fragments of a document.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the document.
    /// * `specs` - The vector of specs that define what to mutate.
    /// * `options` - Options to customize the default behavior.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use couchbase::Cluster;
    /// use futures::Future;
    /// use couchbase::subdoc::MutateInSpec;
    /// # let mut cluster = Cluster::connect("couchbase://127.0.0.1", "Administrator", "password")
    /// #   .expect("Could not create Cluster reference!");
    /// # let bucket = cluster
    /// #   .bucket("travel-sample")
    /// #   .expect("Could not open bucket");
    /// # let collection = bucket.default_collection();
    /// # async {
    /// let insert_result = collection
    ///     .mutate_in(
    ///         "airport_1285",
    ///         vec![MutateInSpec::upsert("updated", true).expect("could not encode value")],
    ///         None,
    ///     )
    ///     .await;
    /// # };
    /// ```
    pub async fn mutate_in<S>(
        &self,
        id: S,
        specs: Vec<MutateInSpec>,
        options: Option<MutateInOptions>,
    ) -> Result<MutateInResult, CouchbaseError>
    where
        S: Into<String>,
    {
        self.instance.mutate_in(id.into(), specs, options).await
    }
}

/// `SharedCollection` level access to operations.
pub struct SharedCollection {
    instance: Arc<SharedInstance>,
}

impl SharedCollection {
    /// Creates a new `SharedCollection`.
    ///
    /// This function is not intended to be called directly, but rather a new
    /// `SharedCollection` should be retrieved through the `Bucket`.
    ///
    pub(crate) fn new(instance: Arc<SharedInstance>) -> Self {
        Self { instance }
    }

    /// Fetches a document from the collection.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the document.
    /// * `options` - Options to customize the default behavior.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use couchbase::SharedCluster;
    /// use serde_json::Value;
    /// use futures::Future;
    /// # let mut cluster = SharedCluster::connect("couchbase://127.0.0.1", "Administrator", "password")
    /// #   .expect("Could not create Cluster reference!");
    /// # let bucket = cluster
    /// #   .bucket("travel-sample")
    /// #   .expect("Could not open bucket");
    /// # let collection = bucket.default_collection();
    /// #
    /// # async {
    /// let found_doc = collection
    ///     .get("airport_1297", None)
    ///     .await
    ///     .expect("Error while loading doc");
    ///
    ///     println!(
    ///         "Content Decoded {:?}",
    ///         found_doc.content_as::<Value>()
    ///     );
    /// # };
    /// ```
    pub async fn get<S>(
        &self,
        id: S,
        options: Option<GetOptions>,
    ) -> Result<GetResult, CouchbaseError>
    where
        S: Into<String>,
    {
        self.instance.get(id.into(), options).await
    }

    /// Fetches a document from the collection and write locks it.
    ///
    /// Note that the `lock` time can be overridden in the options struct. If none is set explicitly,
    /// the default duration of 30 seconds is used.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the document.
    /// * `options` - Options to customize the default behavior.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use couchbase::SharedCluster;
    /// use futures::Future;
    /// use serde_json::Value;
    /// # let mut cluster = SharedCluster::connect("couchbase://127.0.0.1", "Administrator", "password")
    /// #   .expect("Could not create Cluster reference!");
    /// # let bucket = cluster
    /// #   .bucket("travel-sample")
    /// #   .expect("Could not open bucket");
    /// # let collection = bucket.default_collection();
    /// #
    /// # async {
    /// let found_doc = collection
    ///     .get_and_lock("airport_1297", None)
    ///     .await
    ///     .expect("Error while loading and locking doc");
    ///
    ///     println!(
    ///         "Content Decoded {:?}",
    ///         found_doc.content_as::<Value>()
    ///     );
    /// # };
    /// ```
    pub async fn get_and_lock<S>(
        &self,
        id: S,
        options: Option<GetAndLockOptions>,
    ) -> Result<GetResult, CouchbaseError>
    where
        S: Into<String>,
    {
        self.instance.get_and_lock(id.into(), options).await
    }

    /// Fetches a document from the collection and modifies its expiry.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the document.
    /// * `expiration` - The new expiration of the document.
    /// * `options` - Options to customize the default behavior.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use couchbase::SharedCluster;
    /// use std::time::Duration;
    /// use serde_json::Value;
    /// use futures::Future;
    /// # let mut cluster = SharedCluster::connect("couchbase://127.0.0.1", "Administrator", "password")
    /// #   .expect("Could not create Cluster reference!");
    /// # let bucket = cluster
    /// #   .bucket("travel-sample")
    /// #   .expect("Could not open bucket");
    /// # let collection = bucket.default_collection();
    /// #
    /// # async {
    /// let found_doc = collection
    ///     .get_and_touch("airport_1297", Duration::from_secs(5), None)
    ///     .await
    ///     .expect("Error while loading and touching doc");
    ///
    ///     println!(
    ///         "Content Decoded {:?}",
    ///         found_doc.content_as::<Value>()
    ///     );
    /// # };
    /// ```
    pub async fn get_and_touch<S>(
        &self,
        id: S,
        expiration: Duration,
        options: Option<GetAndTouchOptions>,
    ) -> Result<GetResult, CouchbaseError>
    where
        S: Into<String>,
    {
        self.instance.get_and_touch(id.into(), expiration, options).await
    }

    /// Inserts or replaces a new document into the collection.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the document.
    /// * `content` - The content to store inside the document.
    /// * `options` - Options to customize the default behavior.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use couchbase::SharedCluster;
    /// use serde_derive::Serialize;
    /// use futures::Future;
    ///
    /// #[derive(Debug, Serialize)]
    /// struct Airport {
    ///     airportname: String,
    ///     icao: String,
    ///     iata: String,
    /// }
    /// # let mut cluster = SharedCluster::connect("couchbase://127.0.0.1", "Administrator", "password")
    /// #     .expect("Could not create Cluster reference!");
    /// # let bucket = cluster
    /// #     .bucket("travel-sample")
    /// #     .expect("Could not open bucket");
    /// # let collection = bucket.default_collection();
    ///
    /// let airport = Airport {
    ///     airportname: "Vienna Airport".into(),
    ///     icao: "LOWW".into(),
    ///     iata: "VIE".into(),
    /// };
    ///
    /// # async {
    /// collection
    ///     .upsert("airport_999", airport, None)
    ///     .await
    ///     .expect("could not upsert airport!");
    /// # };
    /// ```
    pub async fn upsert<S, T>(
        &self,
        id: S,
        content: T,
        options: Option<UpsertOptions>,
    ) -> Result<MutationResult, CouchbaseError>
    where
        S: Into<String>,
        T: Serialize,
    {
        let serialized = match to_vec(&content) {
            Ok(v) => v,
            Err(_e) => return Err(CouchbaseError::EncodingError),
        };
        let flags = JSON_COMMON_FLAG;
        self.instance.upsert(id.into(), serialized, flags, options).await
    }

    /// Inserts a document into the collection.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the document.
    /// * `content` - The content to store inside the document.
    /// * `options` - Options to customize the default behavior.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use couchbase::SharedCluster;
    /// use serde_derive::Serialize;
    /// use futures::Future;
    ///
    /// #[derive(Debug, Serialize)]
    /// struct Airport {
    ///     airportname: String,
    ///     icao: String,
    ///     iata: String,
    /// }
    /// # let mut cluster = SharedCluster::connect("couchbase://127.0.0.1", "Administrator", "password")
    /// #     .expect("Could not create Cluster reference!");
    /// # let bucket = cluster
    /// #     .bucket("travel-sample")
    /// #     .expect("Could not open bucket");
    /// # let collection = bucket.default_collection();
    ///
    /// let airport = Airport {
    ///     airportname: "Vienna Airport".into(),
    ///     icao: "LOWW".into(),
    ///     iata: "VIE".into(),
    /// };
    /// # async {
    /// collection
    ///     .insert("airport_999", airport, None)
    ///     .await
    ///     .expect("could not insert airport!");
    /// # };
    /// ```
    pub async fn insert<S, T>(
        &self,
        id: S,
        content: T,
        options: Option<InsertOptions>,
    ) -> Result<MutationResult, CouchbaseError>
    where
        S: Into<String>,
        T: Serialize,
    {
        let serialized = match to_vec(&content) {
            Ok(v) => v,
            Err(_e) => return Err(CouchbaseError::EncodingError),
        };
        let flags = JSON_COMMON_FLAG;
        self.instance.insert(id.into(), serialized, flags, options).await
    }

    /// Replaces an existing document in the collection.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the document.
    /// * `content` - The content to store inside the document.
    /// * `options` - Options to customize the default behavior.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use couchbase::SharedCluster;
    /// use serde_derive::Serialize;
    /// use futures::Future;
    ///
    /// #[derive(Debug, Serialize)]
    /// struct Airport {
    ///     airportname: String,
    ///     icao: String,
    ///     iata: String,
    /// }
    /// # let mut cluster = SharedCluster::connect("couchbase://127.0.0.1", "Administrator", "password")
    /// #     .expect("Could not create Cluster reference!");
    /// # let bucket = cluster
    /// #     .bucket("travel-sample")
    /// #     .expect("Could not open bucket");
    /// # let collection = bucket.default_collection();
    ///
    /// let airport = Airport {
    ///     airportname: "Vienna Airport".into(),
    ///     icao: "LOWW".into(),
    ///     iata: "VIE".into(),
    /// };
    ///
    /// # async {
    /// collection
    ///     .replace("airport_999", airport, None)
    ///     .await
    ///     .expect("could not replace airport!");
    /// # };
    /// ```
    pub async fn replace<S, T>(
        &self,
        id: S,
        content: T,
        options: Option<ReplaceOptions>,
    ) -> Result<MutationResult, CouchbaseError>
    where
        S: Into<String>,
        T: Serialize,
    {
        let serialized = match to_vec(&content) {
            Ok(v) => v,
            Err(_e) => return Err(CouchbaseError::EncodingError),
        };
        let flags = JSON_COMMON_FLAG;
        self.instance.replace(id.into(), serialized, flags, options).await
    }

    /// Removes a document from the collection.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the document.
    /// * `options` - Options to customize the default behavior.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use futures::Future;
    /// # use couchbase::SharedCluster;
    /// # let mut cluster = SharedCluster::connect("couchbase://127.0.0.1", "Administrator", "password")
    /// #   .expect("Could not create Cluster reference!");
    /// # let bucket = cluster
    /// #   .bucket("travel-sample")
    /// #   .expect("Could not open bucket");
    /// # let collection = bucket.default_collection();
    /// # async {
    /// let result = collection.remove("document_id", None).await;
    /// # };
    /// ```
    pub async fn remove<S>(
        &self,
        id: S,
        options: Option<RemoveOptions>,
    ) -> Result<MutationResult, CouchbaseError>
    where
        S: Into<String>,
    {
        self.instance.remove(id.into(), options).await
    }

    /// Changes the expiration time on a document.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the document.
    /// * `expiration` - The new expiration of the document.
    /// * `options` - Options to customize the default behavior.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use std::time::Duration;
    /// use futures::Future;
    /// # use couchbase::SharedCluster;
    /// # let mut cluster = SharedCluster::connect("couchbase://127.0.0.1", "Administrator", "password")
    /// #   .expect("Could not create Cluster reference!");
    /// # let bucket = cluster
    /// #   .bucket("travel-sample")
    /// #   .expect("Could not open bucket");
    /// # let collection = bucket.default_collection();
    /// # async { 
    /// let result = collection.touch("document_id", Duration::from_secs(5), None).await;
    /// # };
    /// ```
    pub async fn touch<S>(
        &self,
        id: S,
        expiration: Duration,
        options: Option<TouchOptions>,
    ) -> Result<MutationResult, CouchbaseError>
    where
        S: Into<String>,
    {
        self.instance.touch(id.into(), expiration, options).await
    }

    /// Unlocks a write-locked document.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the document.
    /// * `cas` - The cas needed to remove the write lock.
    /// * `options` - Options to customize the default behavior.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use couchbase::SharedCluster;
    /// use futures::Future;
    /// # let mut cluster = SharedCluster::connect("couchbase://127.0.0.1", "Administrator", "password")
    /// #   .expect("Could not create Cluster reference!");
    /// # let bucket = cluster
    /// #   .bucket("travel-sample")
    /// #   .expect("Could not open bucket");
    /// # let collection = bucket.default_collection();
    /// let cas = 1234; // retrieved from a `getAndLock`
    /// # async {
    /// let result = collection.unlock("document_id", cas, None).await;
    /// # };
    /// ```
    pub async fn unlock<S>(
        &self,
        id: S,
        cas: u64,
        options: Option<UnlockOptions>,
    ) -> Result<MutationResult, CouchbaseError>
    where
        S: Into<String>,
    {
        self.instance.unlock(id.into(), cas, options).await
    }

    /// Checks if a document exists and if so returns a cas value with it.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the document.
    /// * `options` - Options to customize the default behavior.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use couchbase::SharedCluster;
    /// use futures::Future;
    /// # let mut cluster = SharedCluster::connect("couchbase://127.0.0.1", "Administrator", "password")
    /// #   .expect("Could not create Cluster reference!");
    /// # let bucket = cluster
    /// #   .bucket("travel-sample")
    /// #   .expect("Could not open bucket");
    /// # let collection = bucket.default_collection();
    /// # async {
    /// let result = collection.exists("document_id", None).await;
    /// # };
    /// ```
    pub async fn exists<S>(
        &self,
        id: S,
        options: Option<ExistsOptions>,
    ) -> Result<ExistsResult, CouchbaseError>
    where
        S: Into<String>,
    {
        self.instance.exists(id.into(), options).await
    }

    /// Extracts fragments of a document.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the document.
    /// * `specs` - The vector of specs that define what to fetch.
    /// * `options` - Options to customize the default behavior.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use couchbase::SharedCluster;
    /// use futures::Future;
    /// use couchbase::subdoc::LookupInSpec;
    /// # let mut cluster = SharedCluster::connect("couchbase://127.0.0.1", "Administrator", "password")
    /// #   .expect("Could not create Cluster reference!");
    /// # let bucket = cluster
    /// #   .bucket("travel-sample")
    /// #   .expect("Could not open bucket");
    /// # let collection = bucket.default_collection();
    /// # async {
    /// let partial_result = collection
    ///   .lookup_in("airport_1285", vec![LookupInSpec::get("geo")], None)
    ///   .await;
    /// # };
    /// ```
    pub async fn lookup_in<S>(
        &self,
        id: S,
        specs: Vec<LookupInSpec>,
        options: Option<LookupInOptions>,
    ) -> Result<LookupInResult, CouchbaseError>
    where
        S: Into<String>,
    {
        self.instance.lookup_in(id.into(), specs, options).await
    }

    /// Changes fragments of a document.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the document.
    /// * `specs` - The vector of specs that define what to mutate.
    /// * `options` - Options to customize the default behavior.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use couchbase::SharedCluster;
    /// use futures::Future;
    /// use couchbase::subdoc::MutateInSpec;
    /// # let mut cluster = SharedCluster::connect("couchbase://127.0.0.1", "Administrator", "password")
    /// #   .expect("Could not create Cluster reference!");
    /// # let bucket = cluster
    /// #   .bucket("travel-sample")
    /// #   .expect("Could not open bucket");
    /// # let collection = bucket.default_collection();
    /// # async {
    /// let insert_result = collection
    ///     .mutate_in(
    ///         "airport_1285",
    ///         vec![MutateInSpec::upsert("updated", true).expect("could not encode value")],
    ///         None,
    ///     )
    ///     .await;
    /// # };
    /// ```
    pub async fn mutate_in<S>(
        &self,
        id: S,
        specs: Vec<MutateInSpec>,
        options: Option<MutateInOptions>,
    ) -> Result<MutateInResult, CouchbaseError>
    where
        S: Into<String>,
    {
        self.instance.mutate_in(id.into(), specs, options).await
    }
}
