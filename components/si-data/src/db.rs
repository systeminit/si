use couchbase::{options::QueryOptions, options::ScanConsistency, SharedBucket, SharedCluster};
use futures::stream::StreamExt;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{self, json};
use si_settings::Settings;
use sodiumoxide::crypto::secretbox;
use tracing::{debug, event, info_span, span, Level};
use tracing_futures::Instrument as _;

use std::collections::HashMap;
use std::sync::Arc;

use crate::error::{DataError, Result};
use crate::migrateable::Migrateable;
use crate::mvcc::TxnId;
use crate::protobuf::{DataPageToken, DataPageTokenOrderByDirection, DataQuery};
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
    pub total_count: u32,
    pub next_item_id: String,
    pub page_token: String,
}

impl<I: DeserializeOwned + std::fmt::Debug> ListResult<I> {
    pub fn take_items(self) -> Vec<I> {
        self.items
    }

    pub fn total_count(&self) -> u32 {
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

impl std::fmt::Display for DataPageTokenOrderByDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            &DataPageTokenOrderByDirection::Unknown => "ASC".to_string(),
            &DataPageTokenOrderByDirection::Asc => "ASC".to_string(),
            &DataPageTokenOrderByDirection::Desc => "DESC".to_string(),
        };
        write!(f, "{}", msg)
    }
}

#[derive(Debug, Clone)]
pub struct Db {
    // Eventually, this should become a real thread pool.
    pub cluster: SharedCluster,
    pub bucket: Arc<SharedBucket>,
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
            cluster,
            bucket,
            bucket_name: Arc::new(settings.db.bucket_name.clone()),
            scan_consistency,
            page_secret_key: settings.paging.key.clone(),
        })
    }

    pub async fn create_indexes(&self) -> Result<()> {
        async {
            debug!("creating index on siStorable.typeName");
            let mut result = self
                .cluster
                .query(
                    format!(
                        "CREATE INDEX `idx_si_storable_typename` on `{}`(siStorable.typeName)",
                        self.bucket_name
                    ),
                    None,
                )
                .await?;
            debug!("awaiting results");
            let meta = result.meta().await?;
            match meta.errors {
                Some(error) => debug!(?error, "index already exists"),
                None => debug!("created primary index"),
            }

            debug!("creating index on siStorable.typeName and siStorable.tenantIds");
            let mut result = self
                .cluster
                .query(
                    format!("CREATE INDEX `idx_si_storable_typename_and_tenancy` ON `{}` (siStorable.typeName, DISTINCT ARRAY t FOR t IN siStorable.tenantIds END)", self.bucket_name),
                    None,
                )
                .await?;
            debug!("awaiting results");
            let meta = result.meta().await?;
            match meta.errors {
                Some(error) => debug!(?error, "index already exists"),
                None => debug!("created primary index"),
            }

            Ok(())
        }
        .instrument(info_span!("db.create_indexes"))
        .await
    }

    pub async fn check_natural_key_exists(&self, natural_key: Option<&str>) -> Result<()> {
        let span = info_span!(
            "db.check_natural_key_exists",
            ?natural_key,
            exists = tracing::field::Empty
        );
        async {
            let span = tracing::Span::current();
            match natural_key {
                Some(nk) => {
                    if self.exists(nk).await? == true {
                        span.record("exists", &tracing::field::display(false));
                        Err(DataError::NaturalKeyExists(nk.to_string()))
                    } else {
                        span.record("exists", &tracing::field::display(true));
                        Ok(())
                    }
                }
                None => Ok(()),
            }
        }
        .instrument(span)
        .await
    }

    pub async fn validate_and_insert_as_new<'a, T>(&self, content: &'a mut T) -> Result<&'a mut T>
    where
        T: Storable + Serialize + std::fmt::Debug,
    {
        let span = info_span!(
            "db.validate_and_insert_as_new",
            db.storable.id = tracing::field::Empty,
            db.storable.type_name = tracing::field::Empty,
            db.storable.natural_key = tracing::field::Empty,
            db.storable.tenant_ids = tracing::field::Empty,
            error = tracing::field::Empty,
            component = tracing::field::display("si-data"),
        );
        async {
            let span = tracing::Span::current();
            event!(Level::TRACE, "generating_id");
            // We generate a new ID for every inserted object, no matter what
            content.generate_id();
            span.record("db.storable.id", &tracing::field::display(content.id()?));

            event!(Level::TRACE, "set_type_name");
            // We set the type name, always.
            content.set_type_name();
            span.record(
                "db.storable.type_name",
                &tracing::field::display(<T as Storable>::type_name()),
            );

            event!(Level::TRACE, "check_tenant_ids");
            // We must have a tenant ID already in the list; otherwise, this object is
            // invalid and should be rejected. The first item in the list is our primary
            // tenancy.
            if content.tenant_ids()?.is_empty() {
                span.record("error", &"DataError::MissingTenantIds");
                return Err(DataError::MissingTenantIds);
            }

            event!(Level::TRACE, "add_self_to_tenant_ids");
            // The object itself should always be in the tenant id list, ideally last.
            content.add_to_tenant_ids(content.id()?.to_string());

            let tenant_id_list = content.tenant_ids()?.join(", ");
            span.record(
                "db.storable.tenant_ids",
                &tracing::field::display(&tenant_id_list[..]),
            );

            event!(Level::TRACE, "set_natural_key");
            // We set the natural key, if the object needs one.
            content.set_natural_key()?;

            span.record(
                "db.storable.natural_key",
                &tracing::field::display(&content.natural_key()?.unwrap_or("None")),
            );

            event!(Level::TRACE, "check_natural_key_exists");
            // Check for the natural key - it should not already exist, assuming we have one.
            self.check_natural_key_exists(content.natural_key()?)
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
            for tenant_id in content.tenant_ids()?.iter() {
                if tenant_id == "global" || tenant_id == content.id()? {
                    continue;
                }
                if self.exists(tenant_id).await? == false {
                    return Err(DataError::TenantIdIntegrity(tenant_id.to_string()));
                }
            }

            event!(Level::TRACE, "insert");
            self.insert(content).await?;

            if let Some(nk) = content.natural_key()? {
                let id = String::from(nk);
                let lookup_object = LookupObject {
                    id: String::from(nk),
                    object_id: content.id()?.to_string(),
                    type_name: "lookup_object".to_string(),
                    tenant_ids: Vec::from(content.tenant_ids()?),
                };
                let bucket = self.bucket.clone();
                let collection = bucket.default_collection();
                debug!(?id, ?lookup_object, "insert_natural_key_lookup_object");
                collection.insert(id, lookup_object, None).await?;
            }

            Ok(content)
        }
        .instrument(span)
        .await
    }

    pub async fn insert<'a, T>(&self, content: &'a T) -> Result<&'a T>
    where
        T: Storable + Serialize + std::fmt::Debug,
    {
        let bucket_name = format!("{}", self.bucket_name);
        let span = info_span!(
            "db.insert",
            db.storable.id = tracing::field::Empty,
            db.storable.type_name = tracing::field::Empty,
            db.storable.natural_key = tracing::field::Empty,
            db.storable.tenant_ids = tracing::field::Empty,
            db.bucket_name = tracing::field::display(&bucket_name[..]),
            error = tracing::field::Empty,
            component = tracing::field::display("si-data"),
        );
        async {
            let span = tracing::Span::current();
            span.record("db.storable.id", &tracing::field::display(&content.id()?));
            span.record(
                "db.storable.type_name",
                &tracing::field::display(&<T as Storable>::type_name()),
            );
            span.record(
                "db.storable.tenant_ids",
                &tracing::field::display(&content.tenant_ids()?.join(", ")[..]),
            );
            span.record(
                "db.storable.natural_key",
                &tracing::field::display(&content.natural_key()?.unwrap_or("None")),
            );
            let bucket = self.bucket.clone();
            let collection = bucket.default_collection();
            let id = String::from(content.id()?);
            debug!(?id, ?content, "insert");
            collection.insert(id, content, None).await?;
            Ok(content)
        }
        .instrument(span)
        .await
    }

    pub async fn list_by_page_token_raw<S>(
        &self,
        page_token: S,
    ) -> Result<ListResult<serde_json::Value>>
    where
        S: AsRef<str> + std::fmt::Debug,
    {
        let span = info_span!(
            "db.list_by_page_token_raw",
            db.list.query = tracing::field::Empty,
            db.list.order_by = tracing::field::Empty,
            db.list.page_size = tracing::field::Empty,
            db.list.item_id = tracing::field::Empty,
            db.list.order_by_direction = tracing::field::Empty,
            db.list.scope_by_tenant_id = tracing::field::Empty,
            component = tracing::field::display("si-data"),
        );
        async {
            let span = tracing::Span::current();
            let page_token = DataPageToken::unseal(page_token.as_ref(), &self.page_secret_key)?;
            let query = page_token.query;
            span.record("db.list.query", &tracing::field::debug(&query));

            let order_by = page_token
                .order_by
                .ok_or_else(|| DataError::RequiredField("page_token.order_by".into()))?;
            span.record("db.list.order_by", &tracing::field::display(&order_by));

            let page_size = page_token
                .page_size
                .ok_or_else(|| DataError::RequiredField("page_token.page_size".into()))?;
            span.record("db.list.page_size", &tracing::field::display(&page_size));

            let item_id = page_token
                .item_id
                .ok_or_else(|| DataError::RequiredField("page_token.item_id".into()))?;
            span.record("db.list.item_id", &tracing::field::display(&item_id));

            let order_by_direction = page_token.order_by_direction;
            span.record(
                "db.list.order_by_direction",
                &tracing::field::display(&order_by_direction),
            );

            let contained_within = page_token
                .contained_within
                .ok_or_else(|| DataError::RequiredField("page_token.contained_within".into()))?;
            span.record(
                "db.list.contained_within",
                &tracing::field::display(&contained_within),
            );

            //let order_by_direction = OrderByDirection::from_i32(page_token.order_by_direction)
            //.ok_or_else(|| DataError::InvalidOrderByDirection)?;
            self.list_raw(
                &query,
                page_size,
                &order_by,
                order_by_direction,
                &contained_within,
                &item_id,
            )
            .await
        }
        .instrument(span)
        .await
    }

    pub async fn list_by_page_token<S, I>(&self, page_token: S) -> Result<ListResult<I>>
    where
        S: AsRef<str> + std::fmt::Debug,
        I: DeserializeOwned + Storable + std::fmt::Debug,
    {
        let span = info_span!(
            "db.list_by_page_token",
            db.list.query = tracing::field::Empty,
            db.list.order_by = tracing::field::Empty,
            db.list.page_size = tracing::field::Empty,
            db.list.item_id = tracing::field::Empty,
            db.list.order_by_direction = tracing::field::Empty,
            db.list.scope_by_tenant_id = tracing::field::Empty,
            component = tracing::field::display("si-data"),
        );
        async {
            let span = tracing::Span::current();
            let page_token = DataPageToken::unseal(page_token.as_ref(), &self.page_secret_key)?;
            let query = page_token.query;
            span.record("db.list.query", &tracing::field::debug(&query));

            let order_by = page_token
                .order_by
                .ok_or_else(|| DataError::RequiredField("page_token.order_by".into()))?;
            span.record("db.list.order_by", &tracing::field::display(&order_by));

            let page_size = page_token
                .page_size
                .ok_or_else(|| DataError::RequiredField("page_token.page_size".into()))?;
            span.record("db.list.page_size", &tracing::field::display(&page_size));

            let item_id = page_token
                .item_id
                .ok_or_else(|| DataError::RequiredField("page_token.item_id".into()))?;
            span.record("db.list.item_id", &tracing::field::display(&item_id));

            let order_by_direction = page_token.order_by_direction;
            span.record(
                "db.list.order_by_direction",
                &tracing::field::display(&order_by_direction),
            );

            let contained_within = page_token
                .contained_within
                .ok_or_else(|| DataError::RequiredField("page_token.contained_within".into()))?;
            span.record(
                "db.list.contained_within",
                &tracing::field::display(&contained_within),
            );

            //let order_by_direction = OrderByDirection::from_i32(page_token.order_by_direction)
            //.ok_or_else(|| DataError::InvalidOrderByDirection)?;
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
        .instrument(span)
        .await
    }

    // TODO: Pick up on getting the list of raw things. Removing
    // all the safety that storable provides. We either bind it
    // driectly to uitem, which It hink probalby sucks but will
    // work, or we need a minimal trait that just does ID.
    //
    // I think it's probably the minimal trait route.
    pub async fn list_raw<
        O: AsRef<str> + std::fmt::Debug,
        C: AsRef<str> + std::fmt::Debug + std::fmt::Display,
        S: AsRef<str> + std::fmt::Debug,
    >(
        &self,
        query: &Option<DataQuery>,
        page_size: u32,
        order_by: O,
        order_by_direction: i32,
        contained_within: C,
        item_id: S,
    ) -> Result<ListResult<serde_json::Value>> {
        let span = info_span!(
            "db.list_raw",
            db.list.query = tracing::field::Empty,
            db.list.order_by = tracing::field::Empty,
            db.list.page_size = tracing::field::Empty,
            db.list.item_id = tracing::field::Empty,
            db.list.order_by_direction = tracing::field::Empty,
            db.list.scope_by_tenant_id = tracing::field::Empty,
            db.list.next_page_token = tracing::field::Empty,
            db.list.items_count = tracing::field::Empty,
            db.storable.type_name = tracing::field::Empty,
            db.cb.querymeta.request_id = tracing::field::Empty,
            db.cb.querymeta.status = tracing::field::Empty,
            db.cb.querymeta.errors = tracing::field::Empty,
            db.cb.querymeta.client_context_id = tracing::field::Empty,
            db.cb.querymeta.elapsed_time = tracing::field::Empty,
            db.cb.querymeta.execution_time = tracing::field::Empty,
            db.cb.querymeta.result_count = tracing::field::Empty,
            db.cb.querymeta.result_size = tracing::field::Empty,
            component = tracing::field::display("si-data"),
        );
        async {
            let span = tracing::Span::current();

            span.record("db.storable.type_name", &tracing::field::display("*"));

            // The empty string is the default order_by; and it should be
            // naturalKey
            let order_by = match order_by.as_ref() {
                "" => "siStorable.naturalKey",
                ob => ob,
            };
            span.record("db.list.order_by", &tracing::field::display(&order_by));

            //<I as Storable>::is_order_by_valid(order_by, <I as Storable>::order_by_fields())?;

            // If you don't send a valid order by direction, you fucked with
            // the protobuf you sent by hand
            let order_by_direction = DataPageTokenOrderByDirection::from_i32(order_by_direction)
                .ok_or_else(|| DataError::InvalidOrderByDirection)?;
            span.record("db.list.order_by_direction", &tracing::field::display(&order_by_direction));

            // The default page size is 10, and the inbound default is 0
            let page_size = if page_size == 0 { 10 } else { page_size };
            span.record("db.list.page_size", &tracing::field::display(&order_by));

            let mut named_params = HashMap::new();
            named_params.insert("order_by".into(), json![order_by]);
            let named_options = QueryOptions::new()
                .set_named_parameters(named_params)
                .set_scan_consistency(self.scan_consistency);

            let cbquery = match query {
                Some(q) => format!(
                   "SELECT {bucket}.* FROM `{bucket}` WHERE siStorable.typeName IS VALUED AND ARRAY_CONTAINS(siStorable.tenantIds, \"{tenant_id}\") AND {query} ORDER BY {bucket}.[$order_by] {order_by_direction}",
                    query=q.as_n1ql(&self.bucket_name)?,
                    order_by_direction=order_by_direction.to_string(),
                    bucket=self.bucket_name,
                    tenant_id=contained_within,
                ),
                None => format!("SELECT {bucket}.* FROM `{bucket}` WHERE siStorable.typeName IS VALUED AND ARRAY_CONTAINS(siStorable.tenantIds, \"{tenant_id}\") ORDER BY {bucket}.[$order_by] {}", order_by_direction, bucket=self.bucket_name, tenant_id=contained_within),
            };
            span.record("db.list.query", &tracing::field::display(&cbquery));

            let mut result = {
                let span = info_span!("db.cb.query", db.cb.query = &tracing::field::display(&cbquery), db.cb.query.success = tracing::field::Empty);
                let result = self.cluster.query(cbquery, Some(named_options)).await?;
                span.record("db.cb.query.success", &tracing::field::display(true));
                result
            };

            let mut result_stream = result.rows_as::<serde_json::Value>()?;
            let result_meta = result.meta().await?;
            span.record("db.cb.querymeta.request_id", &tracing::field::display(&result_meta.request_id));
            span.record("db.cb.querymeta.status", &tracing::field::display(&result_meta.status));
            span.record("db.cb.querymeta.errors", &tracing::field::debug(&result_meta.errors));
            span.record("db.cb.querymeta.client_context_id", &tracing::field::display(&result_meta.client_context_id));
            span.record("db.cb.querymeta.elapsed_time", &tracing::field::display(&result_meta.metrics.elapsed_time));
            span.record("db.cb.querymeta.execution_time", &tracing::field::display(&result_meta.metrics.execution_time));
            span.record("db.cb.querymeta.result_count", &tracing::field::display(&result_meta.metrics.result_count));
            span.record("db.cb.querymeta.result_size", &tracing::field::display(&result_meta.metrics.result_size));

            let mut final_vec: Vec<serde_json::Value> = Vec::new();

            let mut real_item_id = item_id.as_ref().to_string();
            let mut include = false;
            let mut count = 0;
            let mut next_item_id = String::new();

            // Probably a way to optimize this for really long result sets by
            // using some fancy combinator on the stream iterator; but... this is
            // fine until it ain't. :)
            while let Some(r) = result_stream.next().await {
                match r {
                    Ok(item) => {
                        if count == 0 && real_item_id == "" {
                            real_item_id = item.get("id").ok_or(DataError::MissingId)?.to_string();
                        }
                        if real_item_id == item.get("id").ok_or(DataError::MissingId)?.to_string() {
                            include = true;
                            count = count + 1;
                            final_vec.push(item);
                        } else if count == page_size {
                            next_item_id = item.get("id").ok_or(DataError::MissingId)?.to_string();
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
                let mut next_page_token = DataPageToken::default();
                next_page_token.query = query.clone();
                next_page_token.page_size = Some(page_size);
                next_page_token.order_by = Some(String::from(order_by));
                next_page_token.order_by_direction = order_by_direction as i32;
                next_page_token.item_id = Some(next_item_id.clone());
                next_page_token.contained_within = Some(contained_within.to_string());
                next_page_token.seal(&self.page_secret_key)?
            };
            span.record("db.list.next_page_token", &tracing::field::display(&page_token));
            span.record("db.list.items_count", &tracing::field::display(&final_vec.len()));

            Ok(ListResult {
                items: final_vec,
                total_count: result_meta.metrics.result_count as u32,
                next_item_id,
                page_token,
            })
        }
        .instrument(span)
        .await
    }

    // TODO: Do you want to select the things that are in the system, but
    // replace things that are in a changeset? This query does that for you.
    // It is fucking bonkers.
    //
    // SELECT `si`.*
    // FROM `si` AS a
    // WHERE a.siStorable.typeName = "user"
    //     AND (a.siStorable.changeSetId = "change_set:560fb205-8699-44ea-a1ac-44252eb950c9"
    //         OR (a.siStorable.changeSetId IS NOT VALUED
    //           AND a.id NOT IN (
    //             SELECT RAW id FROM `si` as b where b.siStorable.typeName = "user"
    //                    AND b.siStorable.changeSetId = "change_set:560fb205-8699-44ea-a1ac-44252eb950c9")))
    //
    pub async fn list<
        I: DeserializeOwned + Storable + std::fmt::Debug,
        O: AsRef<str> + std::fmt::Debug,
        C: AsRef<str> + std::fmt::Debug + std::fmt::Display,
        S: AsRef<str> + std::fmt::Debug,
    >(
        &self,
        query: &Option<DataQuery>,
        page_size: u32,
        order_by: O,
        order_by_direction: i32,
        contained_within: C,
        item_id: S,
    ) -> Result<ListResult<I>> {
        let span = info_span!(
            "db.list",
            db.list.query = tracing::field::Empty,
            db.list.order_by = tracing::field::Empty,
            db.list.page_size = tracing::field::Empty,
            db.list.item_id = tracing::field::Empty,
            db.list.order_by_direction = tracing::field::Empty,
            db.list.scope_by_tenant_id = tracing::field::Empty,
            db.list.next_page_token = tracing::field::Empty,
            db.list.items_count = tracing::field::Empty,
            db.storable.type_name = tracing::field::Empty,
            db.cb.querymeta.request_id = tracing::field::Empty,
            db.cb.querymeta.status = tracing::field::Empty,
            db.cb.querymeta.errors = tracing::field::Empty,
            db.cb.querymeta.client_context_id = tracing::field::Empty,
            db.cb.querymeta.elapsed_time = tracing::field::Empty,
            db.cb.querymeta.execution_time = tracing::field::Empty,
            db.cb.querymeta.result_count = tracing::field::Empty,
            db.cb.querymeta.result_size = tracing::field::Empty,
            component = tracing::field::display("si-data"),
        );
        async {
            let span = tracing::Span::current();

            let type_name = <I as Storable>::type_name();
            span.record("db.storable.type_name", &tracing::field::display(&type_name));

            // The empty string is the default order_by; and it should be
            // naturalKey
            let order_by = match order_by.as_ref() {
                "" => "siStorable.naturalKey",
                ob => ob,
            };
            span.record("db.list.order_by", &tracing::field::display(&order_by));

            <I as Storable>::is_order_by_valid(order_by, <I as Storable>::order_by_fields())?;

            // If you don't send a valid order by direction, you fucked with
            // the protobuf you sent by hand
            let order_by_direction = DataPageTokenOrderByDirection::from_i32(order_by_direction)
                .ok_or_else(|| DataError::InvalidOrderByDirection)?;
            span.record("db.list.order_by_direction", &tracing::field::display(&order_by_direction));

            // The default page size is 10, and the inbound default is 0
            let page_size = if page_size == 0 { 10 } else { page_size };
            span.record("db.list.page_size", &tracing::field::display(&order_by));

            let mut named_params = HashMap::new();
            named_params.insert("type_name".into(), json![type_name]);
            named_params.insert("order_by".into(), json![order_by]);
            named_params.insert("tenant_id".into(), json![contained_within.as_ref()]);

            // Base query
            //   + View Context Filter?
            //   + User Query?
            let cbquery = match query {
                Some(q) => {
                    if let Some(view_context) = q.view_context.as_ref() {
                        named_params.insert("view_context".into(), json![view_context]);

                        if let Some(change_set_id) = q.change_set_id.as_ref() {
                            named_params.insert("change_set_id".into(), json![change_set_id]);

                            if q.items.is_empty() {
                                // View Context Filter & Change Set ID
                                format!(
                                    "SELECT a.* \
                                       FROM `{bucket}` AS a \
                                       WHERE a.siStorable.typeName = $type_name \
                                           AND ARRAY_CONTAINS(a.siStorable.tenantIds, $tenant_id) \
                                           AND ARRAY_CONTAINS(a.siStorable.viewContext, $view_context) \
                                           AND (a.siStorable.changeSetId = $change_set_id \
                                                OR (a.siStorable.changeSetId IS NOT VALUED \
                                                    AND a.id NOT IN ( \
                                                      SELECT RAW siStorable.itemId FROM `{bucket}` AS b WHERE b.siStorable.typeName = $type_name \
                                                             AND b.siStorable.changeSetId = $change_set_id))) \
                                       ORDER BY a.[$order_by] {order_by_direction}",
                                    order_by_direction=order_by_direction.to_string(),
                                    bucket=self.bucket_name,
                                )
                            } else {
                                // View Context Filter & Change Set ID & Query
                                format!(
                                    "SELECT {bucket}.* \
                                       FROM `{bucket}` \
                                       WHERE siStorable.typeName = $type_name \
                                           AND ARRAY_CONTAINS(siStorable.tenantIds, $tenant_id) \
                                           AND ARRAY_CONTAINS(siStorable.viewContext, $view_context) \
                                           AND {query} \
                                           AND (a.siStorable.changeSetId = $change_set_id \
                                                OR (a.siStorable.changeSetId IS NOT VALUED \
                                                    AND a.id NOT IN ( \
                                                      SELECT RAW siStorable.itemId FROM `{bucket}` AS b WHERE b.siStorable.typeName = $type_name \
                                                             AND b.siStorable.changeSetId = $change_set_id))) \
                                       ORDER BY {bucket}.[$order_by] {order_by_direction}",
                                    query=q.as_n1ql(&self.bucket_name)?,
                                    order_by_direction=order_by_direction.to_string(),
                                    bucket=self.bucket_name,
                                )
                            }
                        } else {
                            if q.items.is_empty() {
                                // View Context Filter
                                format!(
                                    "SELECT {bucket}.* \
                                       FROM `{bucket}` \
                                       WHERE siStorable.typeName = $type_name \
                                           AND siStorable.changeSetId IS NOT VALUED \
                                           AND ARRAY_CONTAINS(siStorable.tenantIds, $tenant_id) \
                                           AND ARRAY_CONTAINS(siStorable.viewContext, $view_context) \
                                       ORDER BY {bucket}.[$order_by] {order_by_direction}",
                                    order_by_direction=order_by_direction.to_string(),
                                    bucket=self.bucket_name,
                                )
                            } else {
                                // View Context Filter & Query
                                format!(
                                    "SELECT {bucket}.* \
                                       FROM `{bucket}` \
                                       WHERE siStorable.typeName = $type_name \
                                           AND siStorable.changeSetId IS NOT VALUED \
                                           AND ARRAY_CONTAINS(siStorable.tenantIds, $tenant_id) \
                                           AND ARRAY_CONTAINS(siStorable.viewContext, $view_context) \
                                           AND {query} \
                                       ORDER BY {bucket}.[$order_by] {order_by_direction}",
                                    query=q.as_n1ql(&self.bucket_name)?,
                                    order_by_direction=order_by_direction.to_string(),
                                    bucket=self.bucket_name,
                                )
                            }
                        }
                    } else if let Some(change_set_id) = q.change_set_id.as_ref() {
                        named_params.insert("change_set_id".into(), json![change_set_id]);

                        if q.items.is_empty() {
                            // Change Set ID only
                            format!(
                                "SELECT a.* \
                                       FROM `{bucket}` AS a \
                                       WHERE a.siStorable.typeName = $type_name \
                                           AND ARRAY_CONTAINS(a.siStorable.tenantIds, $tenant_id) \
                                           AND (a.siStorable.changeSetId = $change_set_id \
                                                OR (a.siStorable.changeSetId IS NOT VALUED \
                                                    AND a.id NOT IN ( \
                                                      SELECT RAW siStorable.itemId FROM `{bucket}` AS b WHERE b.siStorable.typeName = $type_name \
                                                      AND b.siStorable.changeSetId = $change_set_id))) \
                                       ORDER BY a.[$order_by] {order_by_direction}",
                                       order_by_direction=order_by_direction.to_string(),
                                       bucket=self.bucket_name,
                            )
                        } else {
                            // Change Set ID & Query
                            format!(
                                "SELECT {bucket}.*  \
                                       FROM `{bucket}` \
                                       WHERE siStorable.typeName = $type_name \
                                           AND ARRAY_CONTAINS(siStorable.tenantIds, $tenant_id) \
                                           AND {query} \
                                           AND (a.siStorable.changeSetId = $change_set_id \
                                                OR (a.siStorable.changeSetId IS NOT VALUED \
                                                    AND a.id NOT IN ( \
                                                      SELECT RAW siStorable.itemId FROM `{bucket}` AS b WHERE b.siStorable.typeName = $type_name  \
                                                             AND b.siStorable.changeSetId = $change_set_id))) \
                                       ORDER BY {bucket}.[$order_by] {order_by_direction}",
                                       query=q.as_n1ql(&self.bucket_name)?,
                                       order_by_direction=order_by_direction.to_string(),
                                       bucket=self.bucket_name,
                            )
                        }
                    } else {
                        // No filters or change set id, but a query was provided - it must have items to
                        // complete
                        format!(
                            "SELECT {bucket}.* \
                               FROM `{bucket}` \
                               WHERE siStorable.typeName = $type_name \
                                 AND siStorable.changeSetId IS NOT VALUED \
                                 AND ARRAY_CONTAINS(siStorable.tenantIds, $tenant_id) \
                                 AND {query} \
                                 ORDER BY {bucket}.[$order_by] {order_by_direction}",
                            query=q.as_n1ql(&self.bucket_name)?,
                            order_by_direction=order_by_direction.to_string(),
                            bucket=self.bucket_name,
                        )
                    }
                },
                None => format!("SELECT {bucket}.* \
                                  FROM `{bucket}` \
                                  WHERE siStorable.typeName = $type_name \
                                    AND siStorable.changeSetId IS NOT VALUED \
                                    AND ARRAY_CONTAINS(siStorable.tenantIds, $tenant_id) \
                                    ORDER BY {bucket}.[$order_by] {}", 
                                    order_by_direction, 
                                    bucket=self.bucket_name, ),
            };
            span.record("db.list.query", &tracing::field::display(&cbquery));

            let named_options = QueryOptions::new()
                .set_named_parameters(named_params)
                .set_scan_consistency(self.scan_consistency);

            let mut result = {
                let span = info_span!("db.cb.query", db.cb.query = &tracing::field::display(&cbquery), db.cb.query.success = tracing::field::Empty);
                let result = self.cluster.query(cbquery, Some(named_options)).await?;
                span.record("db.cb.query.success", &tracing::field::display(true));
                result
            };

            let mut result_stream = result.rows_as::<I>()?;
            let result_meta = result.meta().await?;
            span.record("db.cb.querymeta.request_id", &tracing::field::display(&result_meta.request_id));
            span.record("db.cb.querymeta.status", &tracing::field::display(&result_meta.status));
            span.record("db.cb.querymeta.errors", &tracing::field::debug(&result_meta.errors));
            span.record("db.cb.querymeta.client_context_id", &tracing::field::display(&result_meta.client_context_id));
            span.record("db.cb.querymeta.elapsed_time", &tracing::field::display(&result_meta.metrics.elapsed_time));
            span.record("db.cb.querymeta.execution_time", &tracing::field::display(&result_meta.metrics.execution_time));
            span.record("db.cb.querymeta.result_count", &tracing::field::display(&result_meta.metrics.result_count));
            span.record("db.cb.querymeta.result_size", &tracing::field::display(&result_meta.metrics.result_size));

            let mut final_vec: Vec<I> = Vec::new();

            let mut real_item_id = item_id.as_ref().to_string();
            let mut include = false;
            let mut count = 0;
            let mut next_item_id = String::new();

            // Probably a way to optimize this for really long result sets by
            // using some fancy combinator on the stream iterator; but... this is
            // fine until it ain't. :)
            while let Some(r) = result_stream.next().await {
                match r {
                    Ok(item) => {
                        if count == 0 && real_item_id == "" {
                            real_item_id = item.id()?.to_string();
                        }
                        if real_item_id == item.id()? {
                            include = true;
                            count = count + 1;
                            final_vec.push(item);
                        } else if count == page_size {
                            next_item_id = item.id()?.to_string();
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
                let mut next_page_token = DataPageToken::default();
                next_page_token.query = query.clone();
                next_page_token.page_size = Some(page_size);
                next_page_token.order_by = Some(String::from(order_by));
                next_page_token.order_by_direction = order_by_direction as i32;
                next_page_token.item_id = Some(next_item_id.clone());
                next_page_token.contained_within = Some(contained_within.to_string());
                next_page_token.seal(&self.page_secret_key)?
            };
            span.record("db.list.next_page_token", &tracing::field::display(&page_token));
            span.record("db.list.items_count", &tracing::field::display(&final_vec.len()));

            Ok(ListResult {
                items: final_vec,
                total_count: result_meta.metrics.result_count as u32,
                next_item_id,
                page_token,
            })
        }
        .instrument(span)
        .await
    }

    pub async fn migrate<
        I: Migrateable + Storable + DeserializeOwned + Serialize + std::fmt::Debug,
    >(
        &self,
        item: &mut I,
    ) -> Result<()> {
        let span = info_span!(
            "db.migrate",
            db.migrate.updated = tracing::field::Empty,
            db.migrate.identical = tracing::field::Empty,
            db.migrate.outdated = tracing::field::Empty,
            db.migrate.existed = tracing::field::Empty,
            db.migrate.new = tracing::field::Empty,
            db.storable.id = tracing::field::Empty,
            db.storable.natural_key = tracing::field::Empty,
            db.storable.type_name = tracing::field::Empty,
            db.storable.tenant_ids = tracing::field::Empty,
            error = tracing::field::Empty,
            component = tracing::field::display("si-data"),
        );

        async {
            let span = tracing::Span::current();

            item.set_type_name();
            span.record(
                "db.storable.type_name",
                &tracing::field::display(<I as Storable>::type_name()),
            );

            item.set_natural_key()?;

            let natural_key = item
                .natural_key()?
                .ok_or_else(|| DataError::NaturalKeyMissing)?;

            span.record(
                "db.storable.natural_key",
                &tracing::field::display(&natural_key),
            );

            let existing_item: Result<I> = self.lookup_by_natural_key(natural_key).await;
            match existing_item {
                Ok(real_item) => {
                    span.record("db.migrate.existed", &tracing::field::display(true));

                    let existing_id = real_item.id()?;
                    item.set_id(existing_id);
                    span.record("db.storable.id", &tracing::field::display(existing_id));
                    item.add_to_tenant_ids(existing_id.to_string());
                    span.record(
                        "db.storable.tenant_ids",
                        &tracing::field::debug(item.tenant_ids()?),
                    );
                    if item.get_version() > real_item.get_version() {
                        span.record("db.migrate.updated", &tracing::field::display(true));
                        self.upsert(item).await?;
                    } else if item.get_version() < real_item.get_version() {
                        span.record("db.migrate.outdated", &tracing::field::display(true));
                    } else {
                        span.record("db.migrate.identical", &tracing::field::display(true));
                    }
                }
                Err(_e) => {
                    span.record("db.migrate.new", &tracing::field::display(true));
                    self.validate_and_insert_as_new(item).await?;
                }
            }

            Ok(())
        }
        .instrument(span)
        .await
    }

    pub async fn upsert<T>(&self, content: &T) -> Result<couchbase::result::MutationResult>
    where
        T: Serialize + Storable + std::fmt::Debug,
    {
        let bucket_name = format!("{}", self.bucket_name);
        let span = info_span!(
            "db.upsert",
            db.storable.id = %content.id()?,
            db.storable.natural_key = %content.natural_key()?.unwrap_or("None"),
            db.storable.type_name = %<T as Storable>::type_name(),
            db.storable.tenant_ids = ?content.tenant_ids()?,
            db.bucket_name = tracing::field::display(&bucket_name[..]),
            component = tracing::field::display("si-data"),
        );
        async {
            let bucket = self.bucket.clone();
            let collection = bucket.default_collection();
            let id = content.id()?.clone();

            collection
                .upsert(id, content, None)
                .await
                .map_err(DataError::CouchbaseError)
        }
        .instrument(span)
        .await
    }

    pub async fn lookup_by_natural_key<S1, T>(&self, natural_key: S1) -> Result<T>
    where
        S1: Into<String> + std::fmt::Debug,
        T: DeserializeOwned + std::fmt::Debug,
    {
        let key = natural_key.into();
        let span = info_span!(
            "db.lookup_by_natural_key",
            db.lookup.id = tracing::field::Empty,
            db.lookup.natural_key = %key,
            db.lookup.object_id = tracing::field::Empty,
            db.storable.tenant_ids = tracing::field::Empty,
            component = tracing::field::display("si-data"),
        );

        async {
            let span = tracing::Span::current();
            let lookup_object: LookupObject = self.get(key).await?;
            span.record("db.lookup.id", &tracing::field::display(&lookup_object.id));
            span.record(
                "db.lookup.object_id",
                &tracing::field::display(&lookup_object.object_id),
            );
            span.record(
                "db.storable.tenant_ids",
                &tracing::field::debug(&lookup_object.tenant_ids),
            );
            self.get(lookup_object.object_id).await
        }
        .instrument(span)
        .await
    }

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
        let span = info_span!(
            "db.lookup",
            db.lookup.id = tracing::field::Empty,
            db.lookup.natural_key = %key,
            db.lookup.object_id = tracing::field::Empty,
            db.storable.tenant_ids = tracing::field::Empty,
            component = tracing::field::display("si-data"),
        );
        async {
            let span = tracing::Span::current();
            let lookup_object: LookupObject = self.get(key).await?;
            span.record("db.lookup.id", &tracing::field::display(&lookup_object.id));
            span.record(
                "db.lookup.object_id",
                &tracing::field::display(&lookup_object.object_id),
            );
            span.record(
                "db.storable.tenant_ids",
                &tracing::field::debug(&lookup_object.tenant_ids),
            );
            self.get(lookup_object.object_id).await
        }
        .instrument(span)
        .await
    }

    pub async fn get<S, T>(&self, id: S) -> Result<T>
    where
        S: Into<String> + std::fmt::Debug,
        T: DeserializeOwned + std::fmt::Debug,
    {
        let id_string = id.into();
        let span = info_span!(
            "db.get",
            db.get.id = %id_string,
            db.bucket_name = tracing::field::display(&self.bucket_name),
            component = tracing::field::display("si-data"),
        );
        async {
            let item = {
                let bucket = self.bucket.clone();
                let collection = bucket.default_collection();
                collection
                    .get(id_string, None)
                    .await
                    .map_err(DataError::CouchbaseError)?
            };
            Ok(item.content_as::<T>()?)
        }
        .instrument(span)
        .await
    }

    pub async fn get_storable<S, T>(&self, id: S) -> Result<T>
    where
        S: Into<String> + std::fmt::Debug,
        T: Storable + DeserializeOwned + std::fmt::Debug,
    {
        let id_string = id.into();
        let span = info_span!(
            "db.get_storable",
            db.get.id = %id_string,
            db.storable.id = tracing::field::Empty,
            db.storable.type_name = tracing::field::Empty,
            db.storable.natural_key = tracing::field::Empty,
            db.storable.tenant_ids = tracing::field::Empty,
            db.storable.natural_key = tracing::field::Empty,
            db.bucket_name = tracing::field::display(&self.bucket_name),
            component = tracing::field::display("si-data"),
        );
        async {
            let span = tracing::Span::current();
            let item = {
                let bucket = self.bucket.clone();
                let collection = bucket.default_collection();
                collection
                    .get(id_string, None)
                    .await
                    .map_err(DataError::CouchbaseError)?
            };
            let ditem = item.content_as::<T>()?;
            span.record("db.storable.id", &tracing::field::display(&ditem.id()?));
            span.record(
                "db.storable.type_name",
                &tracing::field::display(&<T as Storable>::type_name()),
            );
            span.record(
                "db.storable.tenant_ids",
                &tracing::field::display(&ditem.tenant_ids()?.join(", ")[..]),
            );
            span.record(
                "db.storable.natural_key",
                &tracing::field::display(&ditem.natural_key()?.unwrap_or("None")),
            );
            Ok(ditem)
        }
        .instrument(span)
        .await
    }

    pub async fn exists<S>(&self, id: S) -> Result<bool>
    where
        S: Into<String> + std::fmt::Debug,
    {
        let id_string = id.into();
        let span = info_span!(
            "db.exists",
            db.get.id = %id_string,
            db.exists = tracing::field::Empty,
            db.bucket_name = tracing::field::display(&self.bucket_name),
            component = tracing::field::display("si-data"),
        );
        async {
            let span = tracing::Span::current();
            let bucket = self.bucket.clone();
            let collection = bucket.default_collection();
            match collection.exists(id_string, None).await {
                Ok(_cas) => {
                    span.record("db.exists", &tracing::field::display(true));
                    return Ok(true);
                }
                Err(couchbase::CouchbaseError::Success) => {
                    span.record("db.exists", &tracing::field::display(false));
                    return Ok(false);
                }
                Err(couchbase::CouchbaseError::KeyDoesNotExist) => {
                    span.record("db.exists", &tracing::field::display(false));
                    return Ok(false);
                }
                Err(e) => {
                    return Err(DataError::CouchbaseError(e));
                }
            }
        }
        .instrument(span)
        .await
    }

    pub async fn remove<S>(&self, id: S) -> Result<couchbase::result::MutationResult>
    where
        S: Into<String> + std::fmt::Debug,
    {
        let id_string = id.into();
        let span = info_span!(
            "db.remove",
            db.get.id = %id_string,
            db.bucket_name = tracing::field::display(&self.bucket_name),
            component = tracing::field::display("si-data"),
        );
        async {
            let bucket = self.bucket.clone();
            let collection = bucket.default_collection();
            collection
                .remove(id_string, None)
                .await
                .map_err(DataError::CouchbaseError)
        }
        .instrument(span)
        .await
    }

    pub async fn query<I>(
        &self,
        query: String,
        named_params: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<Vec<I>>
    where
        I: DeserializeOwned + Storable + std::fmt::Debug,
    {
        let span = info_span!(
            "db.query",
            db.list.query = %query,
            db.list.named_params = tracing::field::debug(&named_params),
            db.cb.querymeta.request_id = tracing::field::Empty,
            db.cb.querymeta.status = tracing::field::Empty,
            db.cb.querymeta.errors = tracing::field::Empty,
            db.cb.querymeta.client_context_id = tracing::field::Empty,
            db.cb.querymeta.elapsed_time = tracing::field::Empty,
            db.cb.querymeta.execution_time = tracing::field::Empty,
            db.cb.querymeta.result_count = tracing::field::Empty,
            db.cb.querymeta.result_size = tracing::field::Empty,
            component = tracing::field::display("si-data"),
        );
        async {
            let span = tracing::Span::current();
            let query_options = QueryOptions::new().set_scan_consistency(self.scan_consistency);
            let named_options = match named_params {
                Some(hashmap) => Some(query_options.set_named_parameters(hashmap)),
                None => Some(query_options),
            };
            let mut result = self.cluster.query(query, named_options).await?;
            let result_meta = result.meta().await?;
            span.record(
                "db.cb.querymeta.request_id",
                &tracing::field::display(&result_meta.request_id),
            );
            span.record(
                "db.cb.querymeta.status",
                &tracing::field::display(&result_meta.status),
            );
            span.record(
                "db.cb.querymeta.errors",
                &tracing::field::debug(&result_meta.errors),
            );
            span.record(
                "db.cb.querymeta.client_context_id",
                &tracing::field::display(&result_meta.client_context_id),
            );
            span.record(
                "db.cb.querymeta.elapsed_time",
                &tracing::field::display(&result_meta.metrics.elapsed_time),
            );
            span.record(
                "db.cb.querymeta.execution_time",
                &tracing::field::display(&result_meta.metrics.execution_time),
            );
            span.record(
                "db.cb.querymeta.result_count",
                &tracing::field::display(&result_meta.metrics.result_count),
            );
            span.record(
                "db.cb.querymeta.result_size",
                &tracing::field::display(&result_meta.metrics.result_size),
            );
            let mut result_stream = result.rows_as::<I>()?;
            let mut final_vec: Vec<I> = Vec::new();
            while let Some(r) = result_stream.next().await {
                match r {
                    Ok(v) => final_vec.push(v),
                    Err(e) => return Err(DataError::CouchbaseError(e)),
                }
            }
            Ok(final_vec)
        }
        .instrument(span)
        .await
    }
}
