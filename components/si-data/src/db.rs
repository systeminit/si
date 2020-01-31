use couchbase::{options::QueryOptions, options::ScanConsistency, SharedBucket, SharedCluster};
use futures::stream::StreamExt;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{self, json};
use si_settings::Settings;
use sodiumoxide::crypto::secretbox;
use tracing::{debug, event, info, span, Level};

use std::collections::HashMap;
use std::sync::Arc;

use crate::data::{OrderByDirection, PageToken, Query};
use crate::error::{DataError, Result};
use crate::migrateable::Migrateable;
use crate::storable::{Reference, Storable};

#[derive(Debug, Deserialize)]
pub struct IdResult {
    id: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LookupObject {
    id: String,
    object_id: String,
    type_name: String,
    tenant_ids: Vec<String>,
}

#[derive(Debug)]
pub struct ListResult<I: DeserializeOwned + std::fmt::Debug> {
    pub items: Vec<I>,
    pub total_count: i32,
    pub next_item_id: String,
    pub page_token: String,
}

impl<I: DeserializeOwned + std::fmt::Debug> ListResult<I> {
    pub fn take_items(self) -> Vec<I> {
        self.items
    }

    pub fn total_count(&self) -> i32 {
        self.total_count
    }

    pub fn page_token(&self) -> &str {
        &self.page_token
    }
}

impl<T: DeserializeOwned + std::fmt::Debug> std::ops::Deref for ListResult<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.items
    }
}

impl std::fmt::Display for OrderByDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            &OrderByDirection::Asc => "ASC".to_string(),
            &OrderByDirection::Desc => "DESC".to_string(),
        };
        write!(f, "{}", msg)
    }
}

#[derive(Debug, Clone)]
pub struct Db {
    // Eventually, this should become a real thread pool.
    pub cluster: SharedCluster,
    bucket: Arc<SharedBucket>,
    pub bucket_name: Arc<String>,
    pub scan_consistency: ScanConsistency,
    pub page_secret_key: secretbox::Key,
}

impl Db {
    pub fn new(settings: &Settings) -> Result<Self> {
        let span = span!(Level::DEBUG, "db_new");
        let _start_span = span.enter();
        event!(Level::INFO, ?settings.db.cluster_url, ?settings.db.cluster_user, ?settings.db.cluster_password);
        let mut cluster = SharedCluster::connect(
            &settings.db.cluster_url,
            &settings.db.cluster_user,
            &settings.db.cluster_password,
        )?;

        // Buckets are cached forever, so you are supposed to connect once, early.
        // Later, when you ask for a bucket, you can get it back.
        let bucket = cluster.bucket(&settings.db.bucket_name)?;

        event!(Level::INFO, "couchbase cluster connected");

        let scan_consistency = match settings.db.scan_consistency.as_ref() {
            "NotBounded" => ScanConsistency::NotBounded,
            "RequestPlus" => ScanConsistency::RequestPlus,
            _ => ScanConsistency::NotBounded,
        };

        Ok(Db {
            cluster: cluster,
            bucket: bucket,
            bucket_name: Arc::new(settings.db.bucket_name.clone()),
            scan_consistency: scan_consistency,
            page_secret_key: settings.paging.key.clone(),
        })
    }

    #[tracing::instrument]
    pub async fn check_natural_key_exists(&self, natural_key: Option<&str>) -> Result<()> {
        match natural_key {
            Some(nk) => {
                if self.exists(nk).await? == true {
                    Err(DataError::NaturalKeyExists(nk.to_string()))
                } else {
                    Ok(())
                }
            }
            None => Ok(()),
        }
    }

    #[tracing::instrument]
    pub async fn validate_and_insert_as_new<'a, T>(&self, content: &'a mut T) -> Result<&'a mut T>
    where
        T: Storable + Serialize + std::fmt::Debug,
    {
        event!(Level::TRACE, "generating_id");
        // We generate a new ID for every inserted object, no matter what
        content.generate_id();

        event!(Level::TRACE, "set_type_name");
        // We set the type name, always.
        content.set_type_name();

        event!(Level::TRACE, "check_tenant_ids");
        // We must have a tenant ID already in the list; otherwise, this object is
        // invalid and should be rejected. The first item in the list is our primary
        // tenancy.
        if content.get_tenant_ids().len() == 0 {
            return Err(DataError::MissingTenantIds);
        }

        event!(Level::TRACE, "add_self_to_tenant_ids");
        // The object itself should always be in the tenant id list, ideally last.
        content.add_to_tenant_ids(content.get_id().to_string());

        event!(Level::TRACE, "set_natural_key");
        // We set the natural key, if the object needs one.
        content.set_natural_key();

        event!(Level::TRACE, "check_natural_key_exists");
        // Check for the natural key - it should not already exist, assuming we have one.
        self.check_natural_key_exists(content.get_natural_key())
            .await?;

        event!(Level::TRACE, "check_model_validation");
        // Check model provided validation
        content.validate()?;

        event!(Level::TRACE, "check_referential_integrity");
        // Check referential integrity; every ID referred to must
        // exist.
        for referential_field in content.referential_fields().iter() {
            match referential_field {
                Reference::HasOne(field_name, reference_id) => {
                    event!(
                        Level::TRACE,
                        ?field_name,
                        ?reference_id,
                        "check_referential_integrity_has_one"
                    );
                    if self.exists(*reference_id).await? == false {
                        event!(Level::TRACE, "check_referential_integrity_failed");
                        return Err(DataError::ReferentialIntegrity(
                            field_name.to_string(),
                            reference_id.to_string(),
                        ));
                    }
                }
                Reference::HasMany(field_name, reference_id_list) => {
                    for reference_id in reference_id_list.iter() {
                        event!(
                            Level::TRACE,
                            ?field_name,
                            ?reference_id,
                            "check_referential_integrity_has_many"
                        );
                        if self.exists(reference_id).await? == false {
                            event!(Level::TRACE, "check_referential_integrity_failed");
                            return Err(DataError::ReferentialIntegrity(
                                field_name.to_string(),
                                reference_id.to_string(),
                            ));
                        }
                    }
                }
            }
        }

        event!(Level::TRACE, "check_tenant_ids");
        // Check tenant_ids - do not allow objects with non-existent tenant ids in to the
        // database.
        for tenant_id in content.get_tenant_ids().iter() {
            if tenant_id == "global" || tenant_id == content.get_id() {
                continue;
            }
            if self.exists(tenant_id).await? == false {
                return Err(DataError::TenantIdIntegrity(tenant_id.to_string()));
            }
        }

        event!(Level::TRACE, "insert");
        self.insert(content).await?;

        if let Some(nk) = content.get_natural_key() {
            let id = String::from(nk);
            let lookup_object = LookupObject {
                id: String::from(nk),
                object_id: content.get_id().to_string(),
                type_name: "lookup_object".to_string(),
                tenant_ids: Vec::from(content.get_tenant_ids()),
            };
            let bucket = self.bucket.clone();
            let collection = bucket.default_collection();
            debug!(?id, ?lookup_object, "insert_natural_key_lookup_object");
            collection.insert(id, lookup_object, None).await?;
        }

        Ok(content)
    }

    #[tracing::instrument]
    pub async fn insert<'a, T>(&self, content: &'a T) -> Result<&'a T>
    where
        T: Storable + Serialize + std::fmt::Debug,
    {
        let bucket = self.bucket.clone();
        let collection = bucket.default_collection();
        let id = String::from(content.get_id());
        debug!(?id, ?content, "insert");
        collection.insert(id, content, None).await?;
        Ok(content)
    }

    pub async fn list_by_page_token<S, I>(&self, page_token: S) -> Result<ListResult<I>>
    where
        S: AsRef<str> + std::fmt::Debug,
        I: DeserializeOwned + Storable + std::fmt::Debug,
    {
        let page_token = PageToken::unseal(page_token.as_ref(), &self.page_secret_key)?;
        let query = page_token.query;
        let order_by = page_token.order_by;
        let page_size = page_token.page_size;
        let item_id = page_token.item_id;
        let order_by_direction = page_token.order_by_direction;
        let contained_within = page_token.contained_within;
        //let order_by_direction = OrderByDirection::from_i32(page_token.order_by_direction)
        //.ok_or(DataError::InvalidOrderByDirection)?;
        self.list(
            &query,
            page_size,
            &order_by,
            order_by_direction,
            &contained_within,
            &item_id,
        )
        .await
    }

    #[tracing::instrument]
    pub async fn list<
        I: DeserializeOwned + Storable + std::fmt::Debug,
        O: AsRef<str> + std::fmt::Debug,
        C: AsRef<str> + std::fmt::Debug + std::fmt::Display,
        S: AsRef<str> + std::fmt::Debug,
    >(
        &self,
        query: &Option<Query>,
        page_size: i32,
        order_by: O,
        order_by_direction: i32,
        contained_within: C,
        item_id: S,
    ) -> Result<ListResult<I>> {
        let type_name = <I as Storable>::type_name();

        // The empty string is the default order_by; and it should be
        // naturalKey
        let order_by = match order_by.as_ref() {
            "" => "naturalKey",
            ob => ob,
        };

        <I as Storable>::is_order_by_valid(order_by, <I as Storable>::order_by_fields())?;

        // If you don't send a valid order by direction, you fucked with
        // the protobuf you sent by hand
        let order_by_direction = OrderByDirection::from_i32(order_by_direction)
            .ok_or(DataError::InvalidOrderByDirection)?;

        // The default page size is 10, and the inbound default is 0
        let page_size = if page_size == 0 { 10 } else { page_size };

        let mut named_params = HashMap::new();
        named_params.insert("type_name".into(), json![type_name]);
        named_params.insert("order_by".into(), json![order_by]);
        let named_options = QueryOptions::new()
            .set_named_parameters(named_params)
            .set_scan_consistency(self.scan_consistency);

        let cbquery = match query {
            Some(q) => format!(
               "SELECT {bucket}.* FROM `{bucket}` WHERE typeName = $type_name AND ARRAY_CONTAINS(tenantIds, \"{tenant_id}\") AND {query} ORDER BY {bucket}.[$order_by] {order_by_direction}",
                query=q.as_n1ql(&self.bucket_name)?,
                order_by_direction=order_by_direction.to_string(),
                bucket=self.bucket_name,
                tenant_id=contained_within,
            ),
            None => format!("SELECT {bucket}.* FROM `{bucket}` WHERE typeName = $type_name AND ARRAY_CONTAINS(tenantIds, \"{tenant_id}\") ORDER BY {bucket}.[$order_by] {}", order_by_direction, bucket=self.bucket_name, tenant_id=contained_within),
        };
        event!(Level::DEBUG, ?cbquery, ?named_options);
        let mut result = self.cluster.query(cbquery, Some(named_options)).await?;

        event!(Level::DEBUG, ?result);

        let mut result_stream = result.rows_as::<I>()?;
        let result_meta = result.meta().await?;
        event!(Level::WARN, ?result_meta);

        let mut final_vec: Vec<I> = Vec::new();

        let mut real_item_id = item_id.as_ref().to_string();
        let mut include = false;
        let mut count = 0;
        let mut next_item_id = String::new();

        // Probably a way to optimize this for really long result sets by
        // using some fancy combinator on the stream iterator; but... this is
        // fine until it ain't. :)
        while let Some(r) = result_stream.next().await {
            event!(Level::DEBUG, ?r);
            match r {
                Ok(item) => {
                    if count == 0 && real_item_id == "" {
                        real_item_id = item.get_id().to_string();
                    }
                    if real_item_id == item.get_id() {
                        include = true;
                        count = count + 1;
                        final_vec.push(item);
                    } else if count == page_size {
                        next_item_id = item.get_id().to_string();
                        break;
                    } else if include {
                        final_vec.push(item);
                        count = count + 1;
                    }
                }
                Err(e) => return Err(DataError::CouchbaseError(e)),
            }
        }

        let page_token = if next_item_id == "" {
            String::from("")
        } else {
            let mut next_page_token = PageToken::default();
            next_page_token.query = query.clone();
            next_page_token.page_size = page_size;
            next_page_token.order_by = String::from(order_by);
            next_page_token.order_by_direction = order_by_direction as i32;
            next_page_token.item_id = next_item_id.clone();
            next_page_token.contained_within = contained_within.to_string();
            next_page_token.seal(&self.page_secret_key)?
        };

        Ok(ListResult {
            items: final_vec,
            total_count: result_meta.metrics.result_count as i32,
            next_item_id: next_item_id,
            page_token: page_token,
        })
    }

    #[tracing::instrument]
    pub async fn migrate<
        I: Migrateable + Storable + DeserializeOwned + Serialize + std::fmt::Debug,
    >(
        &self,
        item: &mut I,
    ) -> Result<()> {
        item.set_type_name();
        item.set_natural_key();

        let natural_key = item.get_natural_key().ok_or(DataError::NaturalKeyMissing)?;

        let existing_item: Result<I> = self.lookup_by_natural_key(natural_key).await;
        match existing_item {
            Ok(real_item) => {
                let existing_id = real_item.get_id();
                item.set_id(existing_id);
                item.set_type_name();
                item.set_natural_key();
                item.add_to_tenant_ids(existing_id.to_string());
                if item.get_version() > real_item.get_version() {
                    info!(
                        current_item = item.get_version(),
                        real_item = real_item.get_version(),
                        migrate = "update"
                    );
                    self.upsert(item).await?;
                } else if item.get_version() < real_item.get_version() {
                    debug!(
                        current_item = item.get_version(),
                        real_item = real_item.get_version(),
                        migrate = "newer existing"
                    );
                } else {
                    debug!(
                        current_item = item.get_version(),
                        real_item = real_item.get_version(),
                        migrate = "identical"
                    );
                }
            }
            Err(e) => {
                info!(migrate = "new", ?e);
                self.validate_and_insert_as_new(item).await?;
            }
        }

        Ok(())
    }

    #[tracing::instrument]
    pub async fn upsert<T>(&self, content: &T) -> Result<couchbase::result::MutationResult>
    where
        T: Serialize + Storable + std::fmt::Debug,
    {
        let bucket = self.bucket.clone();
        let collection = bucket.default_collection();
        let id = content.get_id().clone();

        collection
            .upsert(id, content, None)
            .await
            .map_err(DataError::CouchbaseError)
    }

    #[tracing::instrument]
    pub async fn lookup_by_natural_key<S1, T>(&self, natural_key: S1) -> Result<T>
    where
        S1: Into<String> + std::fmt::Debug,
        T: DeserializeOwned + std::fmt::Debug,
    {
        let key = natural_key.into();
        event!(Level::DEBUG, ?key, "looking up");
        let lookup_object: LookupObject = self.get(key).await?;
        event!(Level::DEBUG, ?lookup_object, "returning");
        self.get(lookup_object.object_id).await
    }

    #[tracing::instrument]
    pub async fn lookup<S1, S2, S3, T>(
        &self,
        tenant_id: S1,
        type_name: S2,
        natural_key: S3,
    ) -> Result<T>
    where
        S1: Into<String> + std::fmt::Debug,
        S2: Into<String> + std::fmt::Debug,
        S3: Into<String> + std::fmt::Debug,
        T: DeserializeOwned + std::fmt::Debug,
    {
        let key = format!(
            "{}:{}:{}",
            tenant_id.into(),
            type_name.into(),
            natural_key.into()
        );
        event!(Level::DEBUG, ?key, "looking up");
        let lookup_object: LookupObject = self.get(key).await?;
        event!(Level::DEBUG, ?lookup_object, "returning");
        self.get(lookup_object.object_id).await
    }

    #[tracing::instrument]
    pub async fn get<S, T>(&self, id: S) -> Result<T>
    where
        S: Into<String> + std::fmt::Debug,
        T: DeserializeOwned + std::fmt::Debug,
    {
        let item = {
            let bucket = self.bucket.clone();
            let collection = bucket.default_collection();
            collection
                .get(id.into(), None)
                .await
                .map_err(DataError::CouchbaseError)?
        };
        event!(Level::DEBUG, ?item);
        Ok(item.content_as::<T>()?)
    }

    #[tracing::instrument]
    pub async fn get_storable<S, T>(&self, id: S) -> Result<T>
    where
        S: Into<String> + std::fmt::Debug,
        T: Storable + DeserializeOwned + std::fmt::Debug,
    {
        let item = {
            let bucket = self.bucket.clone();
            let collection = bucket.default_collection();
            collection
                .get(id.into(), None)
                .await
                .map_err(DataError::CouchbaseError)?
        };
        event!(Level::DEBUG, ?item);
        Ok(item.content_as::<T>()?)
    }

    #[tracing::instrument]
    pub async fn exists<S>(&self, id: S) -> Result<bool>
    where
        S: Into<String> + std::fmt::Debug,
    {
        let bucket = self.bucket.clone();
        let collection = bucket.default_collection();
        match collection.exists(id, None).await {
            Ok(cas) => {
                event!(Level::DEBUG, ?cas, "true");
                return Ok(true);
            }
            Err(couchbase::CouchbaseError::Success) => {
                event!(Level::DEBUG, "false");
                return Ok(false);
            }
            Err(couchbase::CouchbaseError::KeyDoesNotExist) => {
                event!(Level::DEBUG, "false");
                return Ok(false);
            }
            Err(e) => {
                return Err(DataError::CouchbaseError(e));
            }
        }
    }

    #[tracing::instrument]
    pub async fn remove<S>(&self, id: S) -> Result<couchbase::result::MutationResult>
    where
        S: Into<String> + std::fmt::Debug,
    {
        let bucket = self.bucket.clone();
        let collection = bucket.default_collection();
        collection
            .remove(id, None)
            .await
            .map_err(DataError::CouchbaseError)
    }

    #[tracing::instrument]
    pub async fn query<I>(
        &self,
        query: String,
        named_params: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<Vec<I>>
    where
        I: DeserializeOwned + Storable + std::fmt::Debug,
    {
        debug!("query");
        let query_options = QueryOptions::new().set_scan_consistency(self.scan_consistency);
        let named_options = match named_params {
            Some(hashmap) => Some(query_options.set_named_parameters(hashmap)),
            None => Some(query_options),
        };
        let mut result = self.cluster.query(query, named_options).await?;
        let mut result_stream = result.rows_as::<I>()?;
        let mut final_vec: Vec<I> = Vec::new();
        while let Some(r) = result_stream.next().await {
            match r {
                Ok(v) => final_vec.push(v),
                Err(e) => return Err(DataError::CouchbaseError(e)),
            }
        }
        debug!(?final_vec, "query_result");

        Ok(final_vec)
    }
}
