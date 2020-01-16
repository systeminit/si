use couchbase::{self, options::QueryOptions, SharedBucket, SharedCluster};
use futures::stream::StreamExt;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{self, json};
use si_settings::Settings;
use sodiumoxide::crypto::secretbox;
use tracing::{event, span, Level};
use tracing_futures::Instrument;
use uuid::Uuid;

use std::collections::HashMap;
use std::sync::Arc;

use crate::error::{Error, Result};
use crate::ssh_key;

pub mod page_token;
pub mod query;

#[derive(Debug)]
pub struct Db {
    // Eventually, this should become a real thread pool.
    cluster: SharedCluster,
    bucket: Arc<SharedBucket>,
    pub page_secret_key: secretbox::Key,
}

#[derive(Debug, Deserialize)]
pub struct IdResult {
    id: String,
}

#[derive(Debug)]
pub struct ListResult<I: DeserializeOwned + std::fmt::Debug> {
    pub items: Vec<I>,
    pub total_count: i32,
    pub next_item_id: String,
}

impl<I: DeserializeOwned + std::fmt::Debug> ListResult<I> {
    pub fn take_items(self) -> Vec<I> {
        self.items
    }
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
        let bucket = cluster.bucket("si")?;

        event!(Level::INFO, "couchbase cluster connected");

        Ok(Db {
            cluster: cluster,
            bucket: bucket,
            page_secret_key: settings.paging.key.clone(),
        })
    }

    #[tracing::instrument]
    pub async fn insert<S, T>(&self, id: S, content: T) -> Result<couchbase::result::MutationResult>
    where
        S: Into<String> + std::fmt::Debug + std::fmt::Display,
        T: Serialize + std::fmt::Debug,
    {
        let bucket = self.bucket.clone();
        let collection = bucket.default_collection();
        collection
            .insert(id, content, None)
            .await
            .map_err(Error::CouchbaseError)
    }

    #[tracing::instrument]
    pub async fn list<
        I: DeserializeOwned + Storable + std::fmt::Debug,
        S: AsRef<str> + std::fmt::Debug,
        D: std::fmt::Display + std::fmt::Debug,
    >(
        &self,
        type_name: S,
        query: &Option<ssh_key::Query>,
        page_size: i32,
        order_by: S,
        order_by_direction: D,
        item_id: S,
    ) -> Result<ListResult<I>> {
        let mut named_params = HashMap::new();
        named_params.insert("type_name".into(), json![type_name.as_ref()]);
        named_params.insert("order_by".into(), json![order_by.as_ref()]);
        let named_options = QueryOptions::new().set_named_parameters(named_params);

        let query = match query {
            Some(q) => format!(
                "SELECT si.* FROM `si` WHERE typeName = $type_name AND {} ORDER BY si.[$order_by] {}",
                q.as_n1ql()?,
                order_by_direction,
            ),
            None => format!("SELECT si.* FROM `si` WHERE typeName = $type_name ORDER BY si.[$order_by] {}", order_by_direction),
        };
        event!(Level::DEBUG, ?query);
        let mut result = self.cluster.query(query, Some(named_options)).await?;

        event!(Level::DEBUG, ?result);

        let mut result_stream = result.rows_as::<I>()?;
        let result_meta = result.meta().await?;
        event!(Level::WARN, ?result_meta);

        let mut final_vec: Vec<I> = Vec::new();

        let mut real_item_id = item_id.as_ref().to_string();
        let mut include = false;
        let mut count = 0;
        let mut next_component = String::new();

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
                        next_component = item.get_id().to_string();
                        break;
                    } else if include {
                        final_vec.push(item);
                        count = count + 1;
                    }
                }
                Err(e) => return Err(Error::CouchbaseError(e)),
            }
        }

        Ok(ListResult {
            items: final_vec,
            total_count: result_meta.metrics.result_count as i32,
            next_item_id: next_component,
        })
    }

    #[tracing::instrument]
    pub async fn migrate_component<I: Migrateable + Storable + Serialize + std::fmt::Debug>(
        &self,
        component: &mut I,
    ) -> Result<couchbase::result::MutationResult> {
        let positional_options =
            QueryOptions::new().set_positional_parameters(vec![json![component.natural_key()]]);
        let mut result = self
            .cluster
            .query(
                "select id from `si` where naturalKey = $1",
                Some(positional_options),
            )
            .await
            .map_err(Error::CouchbaseError)?;

        let mut seen_id = String::from("nopermcnoperson");
        let mut result_iter = result.rows_as::<IdResult>()?;

        // We only want the first result
        if let Some(item) = result_iter.next().await {
            match item {
                Ok(i) => {
                    seen_id = i.id;
                    event!(Level::DEBUG, ?seen_id, "found component id");
                }
                Err(e) => return Err(Error::CouchbaseError(e)),
            };
        }

        if seen_id == "nopermcnoperson" {
            component.generate_id();
        } else {
            component.set_id(seen_id);
        }

        self.upsert(component).await
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
            .map_err(Error::CouchbaseError)
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
                .get(id, None)
                .await
                .map_err(Error::CouchbaseError)?
        };
        event!(Level::DEBUG, ?item);
        Ok(item.content_as::<T>()?)
    }
}

pub trait Storable {
    fn get_id(&self) -> &str;
}

pub trait Migrateable {
    fn set_natural_key(&mut self);
    fn natural_key(&self) -> &str;
    fn generate_id(&mut self);
    fn set_id<T: Into<String>>(&mut self, id: T);
}

impl Storable for ssh_key::Entity {
    #[tracing::instrument]
    fn get_id(&self) -> &str {
        &self.id
    }
}

impl Storable for ssh_key::Component {
    #[tracing::instrument]
    fn get_id(&self) -> &str {
        &self.id
    }
}

impl Migrateable for ssh_key::Component {
    #[tracing::instrument]
    fn generate_id(&mut self) {
        self.id = format!("component:sshkey:{}", Uuid::new_v4());
    }

    fn set_id<T: Into<String>>(&mut self, id: T) {
        let span = span!(Level::TRACE, "set_id");
        let _entered_span = span.enter();
        self.id = id.into();
    }

    #[tracing::instrument]
    fn set_natural_key(&mut self) {
        self.natural_key = format!("{}/{}", self.integration_service_id, self.name);
    }

    #[tracing::instrument]
    fn natural_key(&self) -> &str {
        &self.natural_key
    }
}

#[tracing::instrument]
pub fn migration_data() -> Vec<ssh_key::Component> {
    let key_types = [
        ssh_key::KeyType::Rsa,
        ssh_key::KeyType::Dsa,
        ssh_key::KeyType::Ecdsa,
        ssh_key::KeyType::Ed25519,
    ];
    let key_formats = [
        ssh_key::KeyFormat::Rfc4716,
        ssh_key::KeyFormat::Pkcs8,
        ssh_key::KeyFormat::Pem,
    ];

    let mut data: Vec<ssh_key::Component> = vec![];

    for key_type in key_types.iter() {
        let valid_bits = match key_type {
            ssh_key::KeyType::Rsa => vec![1024, 2048, 3072, 4096],
            ssh_key::KeyType::Dsa => vec![1024],
            ssh_key::KeyType::Ecdsa => vec![256, 384, 521],
            ssh_key::KeyType::Ed25519 => vec![256],
        };

        for key_format in key_formats.iter() {
            for bits in valid_bits.iter() {
                let mut name: String = String::new();
                match key_type {
                    ssh_key::KeyType::Rsa => name.push_str("RSA "),
                    ssh_key::KeyType::Dsa => name.push_str("DSA "),
                    ssh_key::KeyType::Ecdsa => name.push_str("ECDSA "),
                    ssh_key::KeyType::Ed25519 => name.push_str("ED25519 "),
                };
                name.push_str(&format!("{}", bits));
                match key_format {
                    ssh_key::KeyFormat::Rfc4716 => name.push_str(" RFC4716"),
                    ssh_key::KeyFormat::Pkcs8 => name.push_str(" PKCS8"),
                    ssh_key::KeyFormat::Pem => name.push_str(" PEM"),
                };

                let mut c = ssh_key::Component {
                    display_name: name.clone(),
                    description: name.clone(),
                    key_type: *key_type as i32,
                    key_format: *key_format as i32,
                    bits: bits.clone(),
                    integration_id: "integration:9a38d1b0-4936-4082-8b95-64bebc5459c8".to_string(),
                    integration_service_id:
                        "integration:service:415e7492-0932-4cda-a888-7c176a422680".to_string(),
                    type_name: "component:ssh_key".to_string(),
                    display_type_name: "SSH Key".to_string(),
                    name: name,
                    ..Default::default()
                };
                c.set_natural_key();
                data.push(c);
            }
        }
    }
    data
}
