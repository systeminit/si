use std::{
    result,
    str::Utf8Error,
};

use bytes::Bytes;
use si_data_nats::{
    NatsClient,
    Subject,
    async_nats::{
        self,
        jetstream::{
            consumer::StreamError,
            context::{
                KeyValueErrorKind,
                RequestError,
            },
            kv::{
                self,
            },
            stream::ConsumerError,
        },
    },
    jetstream,
};
use si_id::{
    ChangeSetId,
    WorkspacePk,
};
use strum::AsRefStr;
use thiserror::Error;

mod deployment;
mod kv_history;
mod workspace;

const NATS_KV_BUCKET_NAME: &str = "FRIGG";

#[remain::sorted]
#[derive(Debug, Error)]
pub enum Error {
    #[error("change set index definition checksum found does not match what's expected")]
    ChangeSetIndexDefinitionChecksumMismatch,
    #[error("old change set index pointer version found: {0}")]
    ChangeSetIndexOldPointerVersion(&'static str),
    #[error("change set index pointer value not found")]
    ChangeSetIndexPointerValueNotFound,
    #[error("consumer error: {0}")]
    Consumer(#[from] ConsumerError),
    #[error("create error: {0}")]
    Create(#[from] kv::CreateError),
    #[error("error creating kv store: {0}")]
    CreateKeyValue(#[from] async_nats::jetstream::context::CreateKeyValueError),
    #[error("error deserializing kv value: {0}")]
    Deserialize(#[source] serde_json::Error),
    #[error("entry error when getting change set index")]
    EntryGetChangeSetIndex(#[source] kv::EntryError),
    #[error("entry error when getting deployment index")]
    EntryGetDeploymentIndex(#[source] kv::EntryError),
    #[error("entry error when performing get object raw bytes: {0}")]
    EntryGetObjectRawBytes(#[source] kv::EntryError),
    #[error("error getting kv store: {0}")]
    GetKeyValue(#[from] async_nats::jetstream::context::KeyValueError),
    #[error("index object not found at key: {0}")]
    IndexObjectNotFound(Subject),
    #[error("nats request error: {0}")]
    NatsRequest(#[from] RequestError),
    #[error("object kind was expected to be 'MvIndex' but was '{0}'")]
    NotIndexKind(String),
    #[error(
        "object listed in changeset index not found: workspace: {workspace_id}, change set: {change_set_id}, kind: {kind}, id: {id}"
    )]
    ObjectNotFoundForChangesetIndex {
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
        kind: String,
        id: String,
    },
    #[error("object listed in deployment index not found: kind: {kind}, id: {id}")]
    ObjectNotFoundForDeploymentIndex { kind: String, id: String },
    #[error("put error: {0}")]
    Put(#[from] kv::PutError),
    #[error("error serializing kv value: {0}")]
    Serialize(#[source] serde_json::Error),
    #[error("stream error: {0}")]
    Stream(#[from] StreamError),
    #[error("update error: {0}")]
    Update(#[from] kv::UpdateError),
    #[error("utf8 encoding error: {0}")]
    Utf8(#[from] Utf8Error),
    #[error("kv watch error: {0}")]
    Watch(#[from] async_nats::jetstream::kv::WatchError),
}

pub type FriggError = Error;

type Result<T> = result::Result<T, Error>;

impl Error {
    /// An error corresponding to a `ChangeSetIndex` that is either missing from the store or is
    /// invalid for any reason.
    pub fn is_missing_or_invalid_change_set_index(&self) -> bool {
        matches!(
            self,
            Error::ChangeSetIndexDefinitionChecksumMismatch
                | Error::ChangeSetIndexOldPointerVersion(_)
                | Error::ChangeSetIndexPointerValueNotFound
        )
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct KvRevision(u64);

impl From<u64> for KvRevision {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

#[remain::sorted]
#[derive(AsRefStr, Debug, PartialEq)]
#[strum(serialize_all = "snake_case")]
enum Domain {
    Index,
    Object,
}

#[remain::sorted]
#[derive(AsRefStr, Debug, PartialEq)]
#[strum(serialize_all = "snake_case")]
enum Scope {
    ChangeSet,
    Deployment,
    Workspace,
}

#[derive(Clone, Debug)]
pub struct FriggStore {
    nats: NatsClient,
    store: kv::Store,
}

impl FriggStore {
    pub fn new(nats: NatsClient, store: kv::Store) -> Self {
        Self { nats, store }
    }

    async fn get_object_raw_bytes(&self, key: &Subject) -> Result<Option<(Bytes, KvRevision)>> {
        let Some(entry) = self
            .store
            .entry(key.to_string())
            .await
            .map_err(Error::EntryGetObjectRawBytes)?
        else {
            return Ok(None);
        };

        match entry.operation {
            kv::Operation::Delete | kv::Operation::Purge => Ok(None),
            kv::Operation::Put => Ok(Some((entry.value, entry.revision.into()))),
        }
    }
}

pub async fn frigg_kv(context: &jetstream::Context, prefix: Option<&str>) -> Result<kv::Store> {
    let bucket = nats_std::jetstream::prefixed(prefix, NATS_KV_BUCKET_NAME);

    let kv = match context.get_key_value(bucket.clone()).await {
        Ok(kv) => kv,
        Err(err) => match err.kind() {
            KeyValueErrorKind::GetBucket | KeyValueErrorKind::JetStream => {
                create_kv(context, bucket).await?
            }
            _ => return Err(err.into()),
        },
    };

    Ok(kv)
}

async fn create_kv(context: &jetstream::Context, bucket: String) -> Result<kv::Store> {
    context
        .create_key_value(kv::Config {
            bucket,
            description: "Frigg store data".to_owned(),
            ..Default::default()
        })
        .await
        .map_err(Into::into)
}
