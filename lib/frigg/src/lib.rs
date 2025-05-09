use std::{
    marker::PhantomData,
    ops,
    result,
    str::Utf8Error,
    sync::Arc,
};

use bytes::Bytes;
use kv_history::{
    History,
    Keys,
};
use si_data_nats::{
    NatsClient,
    Subject,
    async_nats::{
        self,
        jetstream::{
            consumer::{
                StreamError,
                push::OrderedConfig,
            },
            context::{
                KeyValueErrorKind,
                RequestError,
            },
            kv::{
                self,
                PurgeError,
                Watch,
            },
            stream::ConsumerError,
        },
    },
    jetstream,
};
use si_frontend_mv_types::{
    index::MvIndex,
    object::FrontendObject,
    reference::{
        IndexReference,
        ReferenceKind,
    },
};
use si_id::{
    ChangeSetId,
    WorkspacePk,
};
use telemetry::prelude::*;
use thiserror::Error;

const NATS_KV_BUCKET_NAME: &str = "FRIGG";

const KEY_PREFIX_INDEX: &str = "index";
const KEY_PREFIX_OBJECT: &str = "object";

#[remain::sorted]
#[derive(Debug, Error)]
pub enum Error {
    #[error("consumer error: {0}")]
    Consumer(#[from] ConsumerError),
    #[error("create error: {0}")]
    Create(#[from] kv::CreateError),
    #[error("error creating kv store: {0}")]
    CreateKeyValue(#[from] async_nats::jetstream::context::CreateKeyValueError),
    #[error("error deserializing kv value: {0}")]
    Deserialize(#[source] serde_json::Error),
    #[error("entry error: {0}")]
    Entry(#[from] kv::EntryError),
    #[error("error getting kv store: {0}")]
    GetKeyValue(#[from] async_nats::jetstream::context::KeyValueError),
    #[error("index object not found at key: {0}")]
    IndexObjectNotFound(Subject),
    #[error("kv watcher error: {0}")]
    KvWatcher(#[from] KvWatcherError),
    #[error("nats request error: {0}")]
    NatsRequest(#[from] RequestError),
    #[error("object kind was expected to be 'MvIndex' but was '{0}'")]
    NotIndexKind(String),
    #[error(
        "object listed in index not found: workspace: {workspace_id}, change set: {change_set_id}, kind: {kind}, id: {id}"
    )]
    ObjectNotFoundFromIndex {
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
        kind: String,
        id: String,
    },
    #[error("failed to parse index key: {0}")]
    ParseIndexKey(&'static str),
    #[error("failed to parse object key: {0}")]
    ParseObjectKey(&'static str),
    #[error("error purging key")]
    Purge(#[source] PurgeError),
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

pub use kv_history::KvWatcherError;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct KvRevision(u64);

impl From<u64> for KvRevision {
    fn from(value: u64) -> Self {
        Self(value)
    }
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

    #[instrument(
        name = "frigg.insert_object",
        level = "debug",
        skip_all,
        fields(
            si.workspace.id = %workspace_id,
            si.frontend_object.id = %object.id,
            si.frontend_object.kind = %object.kind,
        )
    )]
    pub async fn insert_object(
        &self,
        workspace_id: WorkspacePk,
        object: &FrontendObject,
    ) -> Result<Subject> {
        let key = object_key(
            workspace_id,
            &object.kind.to_string(),
            &object.id,
            &object.checksum,
        );
        let value = serde_json::to_vec(&object).map_err(Error::Serialize)?;
        self.store.put(key.as_str(), value.into()).await?;

        Ok(key)
    }

    #[instrument(
        name = "frigg.insert_objects",
        level = "debug",
        skip_all,
        fields(si.workspace.id = %workspace_id),
    )]
    pub async fn insert_objects(
        &self,
        workspace_id: WorkspacePk,
        objects: impl Iterator<Item = &FrontendObject>,
    ) -> Result<()> {
        for object in objects {
            self.insert_object(workspace_id, object).await?;
        }

        Ok(())
    }

    #[instrument(
        name = "frigg.get_object",
        level = "debug",
        skip_all,
        fields(
            si.workspace.id = %workspace_id,
            si.frontend_object.id = %id,
            si.frontend_object.kind = %kind,
            si.frontend_object.checksum = %checksum,
    ))]
    pub async fn get_object(
        &self,
        workspace_id: WorkspacePk,
        kind: &str,
        id: &str,
        checksum: &str,
    ) -> Result<Option<FrontendObject>> {
        match self
            .get_object_raw_bytes(&object_key(workspace_id, kind, id, checksum))
            .await?
        {
            Some((bytes, _)) => Ok(Some(
                serde_json::from_slice(bytes.as_ref()).map_err(Error::Deserialize)?,
            )),
            None => Ok(None),
        }
    }

    #[instrument(
        name = "frigg.get_current_object",
        level = "debug",
        skip_all,
        fields(
            si.workspace.id = %workspace_id,
            si.frontend_object.id = %id,
            si.frontend_object.kind = %kind,
    ))]
    pub async fn get_current_object(
        &self,
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
        kind: &str,
        id: &str,
    ) -> Result<Option<FrontendObject>> {
        let Some((current_index, _)) = self.get_index(workspace_id, change_set_id).await? else {
            return Ok(None);
        };
        let mv_index: MvIndex =
            serde_json::from_value(current_index.data).map_err(FriggError::Deserialize)?;
        for index_entry in mv_index.mv_list {
            if index_entry.kind == kind && index_entry.id == id {
                return Ok(Some(
                    self.get_object(workspace_id, kind, id, &index_entry.checksum)
                        .await?
                        .ok_or_else(|| FriggError::ObjectNotFoundFromIndex {
                            workspace_id,
                            change_set_id,
                            kind: kind.to_string(),
                            id: id.to_string(),
                        })?,
                ));
            }
        }

        Ok(None)
    }

    async fn insert_or_update_index_preamble(
        &self,
        workspace_id: WorkspacePk,
        object: &FrontendObject,
    ) -> Result<(Subject, Subject)> {
        let mv_index_kind_string = ReferenceKind::MvIndex.to_string();
        if object.kind != mv_index_kind_string {
            return Err(Error::NotIndexKind(object.kind.clone()));
        }

        let index_object_key = self.insert_object(workspace_id, object).await?;
        let index_pointer_key = index_key(workspace_id, &object.id);

        Ok((index_object_key, index_pointer_key))
    }

    /// Insert a new `MvIndex` into the store, and update the associated index pointer to refer
    /// to the newly inserted `MvIndex`.
    ///
    /// Will fail if the index pointer already exists.
    #[instrument(
        name = "frigg.insert_index",
        level = "debug",
        skip_all,
        fields(
            si.workspace.id = %workspace_id,
            si.frontend_object.id = %object.id,
            si.frontend_object.kind = %object.kind,
            si.frontend_object.checksum = %object.checksum,
        )
    )]
    pub async fn insert_index(
        &self,
        workspace_id: WorkspacePk,
        object: &FrontendObject,
    ) -> Result<KvRevision> {
        let (index_object_key, index_pointer_key) = self
            .insert_or_update_index_preamble(workspace_id, object)
            .await?;

        let new_revision = self
            .store
            .create(index_pointer_key, index_object_key.into_string().into())
            .await?;

        Ok(new_revision.into())
    }

    /// Insert an updated `MvIndex` into the store, and update the associated index pointer to refer
    /// to the newly inserted `MvIndex`.
    ///
    /// Will fail if the index pointer has been updated since `revision` was fetched.
    #[instrument(
        name = "frigg.update_index",
        level = "debug",
        skip_all,
        fields(
            si.workspace.id = %workspace_id,
            si.frontend_object.id = %object.id,
            si.frontend_object.kind = %object.kind,
            si.frontend_object.checksum = %object.checksum,
        )
    )]
    pub async fn update_index(
        &self,
        workspace_id: WorkspacePk,
        object: &FrontendObject,
        revision: KvRevision,
    ) -> Result<KvRevision> {
        let (index_object_key, index_pointer_key) = self
            .insert_or_update_index_preamble(workspace_id, object)
            .await?;

        let new_revision = self
            .store
            .update(
                index_pointer_key,
                index_object_key.into_string().into(),
                revision.0,
            )
            .await?;

        Ok(new_revision.into())
    }

    /// Put a new `MvIndex` into the store, and update the associated index pointer to refer
    /// to the newly inserted `MvIndex`.
    ///
    /// Will NOT fail if the index pointer already exists.
    #[instrument(
        name = "frigg.put_index",
        level = "debug",
        skip_all,
        fields(
            si.workspace.id = %workspace_id,
            si.frontend_object.id = %object.id,
            si.frontend_object.kind = %object.kind,
            si.frontend_object.checksum = %object.checksum,
        )
    )]
    pub async fn put_index(
        &self,
        workspace_id: WorkspacePk,
        object: &FrontendObject,
    ) -> Result<KvRevision> {
        let (index_object_key, index_pointer_key) = self
            .insert_or_update_index_preamble(workspace_id, object)
            .await?;

        let new_revision = self
            .store
            .put(index_pointer_key, index_object_key.into_string().into())
            .await?;

        Ok(new_revision.into())
    }

    #[instrument(
        name = "frigg.get_index",
        level = "debug",
        skip_all,
        fields(
            si.workspace.id = %workspace_id,
            si.change_set.id = %change_set_id,
        )
    )]
    pub async fn get_index(
        &self,
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
    ) -> Result<Option<(FrontendObject, KvRevision)>> {
        let index_pointer_key = index_key(workspace_id, &change_set_id.to_string());

        let Some((bytes, revision)) = self.get_object_raw_bytes(&index_pointer_key).await? else {
            return Ok(None);
        };

        let object_key = Subject::from_utf8(bytes)?;
        let bytes = self
            .store
            .get(object_key.to_string())
            .await?
            .ok_or(Error::IndexObjectNotFound(object_key))?;
        let object = serde_json::from_slice(bytes.as_ref()).map_err(Error::Deserialize)?;

        Ok(Some((object, revision)))
    }

    #[instrument(
        name = "frigg.get_mv_index",
        level = "debug",
        skip_all,
        fields(
            si.workspace.id = %workspace_id,
            si.change_set.id = %change_set_id,
        )
    )]
    pub async fn get_mv_index(
        &self,
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
    ) -> Result<MvIndex> {
        let (frontend_object, _) = self
            .get_index(workspace_id, change_set_id)
            .await?
            .ok_or_else(|| {
                Error::IndexObjectNotFound(index_key(workspace_id, &change_set_id.to_string()))
            })?;

        let mv_index: MvIndex =
            serde_json::from_value(frontend_object.data).map_err(FriggError::Deserialize)?;

        Ok(mv_index)
    }

    #[instrument(
        name = "frigg.watch_index",
        level = "debug",
        skip_all,
        fields(
            si.workspace.id = %workspace_id,
            si.change_set.id = %change_set_id,
        )
    )]
    pub async fn watch_index(
        &self,
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
    ) -> Result<Watch> {
        let index_pointer_key = index_key(workspace_id, &change_set_id.to_string());
        self.store
            .watch(index_pointer_key)
            .await
            .map_err(Into::into)
    }

    #[instrument(
        name = "frigg.delete_index",
        level = "debug",
        skip_all,
        fields(
            si.workspace.id = %workspace_id,
            si.change_set.id = %change_set_id,
        )
    )]
    pub async fn delete_index(
        &self,
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
    ) -> Result<()> {
        let index_pointer_key = index_key(workspace_id, &change_set_id.to_string());

        // Get subject for frontend object containing the MvIndex
        let (bytes, index_key_pointer_revision) = self
            .get_object_raw_bytes(&index_pointer_key)
            .await?
            .ok_or_else(|| Error::IndexObjectNotFound(index_pointer_key.clone()))?;
        let object_key = Subject::from_utf8(bytes)?;

        // Delete MvIndex key
        self.store.purge(object_key).await?;

        // Delete index pointer key
        self.store
            .purge_expect_revision(index_pointer_key, Some(index_key_pointer_revision.0))
            .await
            .map_err(Error::Purge)?;

        Ok(())
    }

    #[instrument(
        name = "frigg.get_object_raw_bytes",
        level = "debug",
        skip_all,
        fields(
            si.frigg.object.key = %key,
        )
    )]
    async fn get_object_raw_bytes(&self, key: &Subject) -> Result<Option<(Bytes, KvRevision)>> {
        let Some(entry) = self.store.entry(key.to_string()).await? else {
            return Ok(None);
        };

        match entry.operation {
            kv::Operation::Delete | kv::Operation::Purge => Ok(None),
            kv::Operation::Put => Ok(Some((entry.value, entry.revision.into()))),
        }
    }

    #[instrument(
        name = "frigg.index_keys_for_workspace",
        level = "debug",
        skip_all,
        fields(
            si.workspace.id = %workspace_id,
        )
    )]
    pub async fn index_keys_for_workspace(
        &self,
        workspace_id: WorkspacePk,
    ) -> Result<Keys<IndexKey>> {
        let filter_subject = index_key(workspace_id, "*").into_string();

        let mut keys_consumer = self
            .store
            .stream
            .create_consumer(OrderedConfig {
                deliver_subject: self.nats.new_inbox(),
                description: Some("kv index keys consumer".to_string()),
                filter_subject,
                headers_only: true,
                replay_policy: async_nats::jetstream::consumer::ReplayPolicy::Instant,
                // We only need to know the latest state for each key, not the whole history
                deliver_policy: async_nats::jetstream::consumer::DeliverPolicy::LastPerSubject,
                ..Default::default()
            })
            .await?;

        let entries = History {
            done: keys_consumer.info().await?.num_pending == 0,
            subscription: keys_consumer.messages().await?,
            prefix: self.store.prefix.clone(),
            bucket: self.store.name.clone(),
        };

        Ok(Keys {
            inner: entries,
            _phantom: PhantomData,
        })
    }

    #[instrument(
        name = "frigg.object_keys_for_workspace",
        level = "debug",
        skip_all,
        fields(
            si.workspace.id = %workspace_id,
        )
    )]
    pub async fn object_keys_for_workspace(
        &self,
        workspace_id: WorkspacePk,
    ) -> Result<Keys<ObjectKey>> {
        let filter_subject = object_key(workspace_id, "*", "*", "*").into_string();

        let mut keys_consumer = self
            .store
            .stream
            .create_consumer(OrderedConfig {
                deliver_subject: self.nats.new_inbox(),
                description: Some("kv object keys consumer".to_string()),
                filter_subject,
                headers_only: true,
                replay_policy: async_nats::jetstream::consumer::ReplayPolicy::Instant,
                // We only need to know the latest state for each key, not the whole history
                deliver_policy: async_nats::jetstream::consumer::DeliverPolicy::LastPerSubject,
                ..Default::default()
            })
            .await?;

        let entries = History {
            done: keys_consumer.info().await?.num_pending == 0,
            subscription: keys_consumer.messages().await?,
            prefix: self.store.prefix.clone(),
            bucket: self.store.name.clone(),
        };

        Ok(Keys {
            inner: entries,
            _phantom: PhantomData,
        })
    }
}

#[derive(Clone, Debug)]
pub struct IndexKey {
    inner: Arc<str>,
    workspace_id: WorkspacePk,
    workspace_id_str_range: ops::Range<usize>,
    change_set_id: ChangeSetId,
    change_set_id_str_range: ops::Range<usize>,
}

impl IndexKey {
    pub fn workspace_id(&self) -> WorkspacePk {
        self.workspace_id
    }

    pub fn workspace_id_str(&self) -> &str {
        &self.inner.as_ref()[self.workspace_id_str_range.start..self.workspace_id_str_range.end]
    }

    pub fn change_set_id(&self) -> ChangeSetId {
        self.change_set_id
    }

    pub fn change_set_id_str(&self) -> &str {
        &self.inner.as_ref()[self.change_set_id_str_range.start..self.change_set_id_str_range.end]
    }

    pub fn to_subject(&self) -> Subject {
        self.inner.as_ref().into()
    }
}

impl TryFrom<String> for IndexKey {
    type Error = Error;

    fn try_from(value: String) -> result::Result<Self, Self::Error> {
        let mut parts = value.splitn(4, '.');

        if let (Some(prefix), Some(workspace_id_str), Some(_change_set_id_str), None) =
            (parts.next(), parts.next(), parts.next(), parts.next())
        {
            if prefix != KEY_PREFIX_INDEX {
                return Err(Self::Error::ParseIndexKey("incorrect key prefix"));
            }

            let prefix_len = prefix.len();
            let workspace_len = workspace_id_str.len();
            let inner: Arc<str> = value.into();
            let inner_len = inner.len();

            let workspace_id_str_range = ops::Range {
                start: prefix_len + 1,
                end: prefix_len + 1 + workspace_len,
            };
            let change_set_id_str_range = ops::Range {
                start: prefix_len + 1 + workspace_len + 1,
                end: inner_len,
            };

            let workspace_id = inner.as_ref()
                [workspace_id_str_range.start..workspace_id_str_range.end]
                .parse()
                .map_err(|_| Self::Error::ParseIndexKey("failed to parse workspace id"))?;
            let change_set_id = inner.as_ref()
                [change_set_id_str_range.start..change_set_id_str_range.end]
                .parse()
                .map_err(|_| Self::Error::ParseIndexKey("failed to parse change set id"))?;

            Ok(Self {
                inner,
                workspace_id,
                workspace_id_str_range,
                change_set_id,
                change_set_id_str_range,
            })
        } else {
            Err(Self::Error::ParseIndexKey("invalid index key format"))
        }
    }
}

impl TryFrom<Subject> for IndexKey {
    type Error = Error;

    #[inline]
    fn try_from(value: Subject) -> result::Result<Self, Self::Error> {
        value.to_string().try_into()
    }
}

impl TryFrom<(WorkspacePk, ChangeSetId)> for IndexKey {
    type Error = Error;

    #[inline]
    fn try_from(value: (WorkspacePk, ChangeSetId)) -> result::Result<Self, Self::Error> {
        let inner = index_key(value.0, &value.1.to_string());
        inner.try_into()
    }
}

impl From<IndexKey> for String {
    fn from(value: IndexKey) -> Self {
        value.inner.to_string()
    }
}

impl AsRef<str> for IndexKey {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

#[derive(Clone, Debug)]
pub struct ObjectKey {
    inner: Arc<str>,
    workspace_id: WorkspacePk,
    workspace_id_str_range: ops::Range<usize>,
    kind_range: ops::Range<usize>,
    id_range: ops::Range<usize>,
    checksum_range: ops::Range<usize>,
}

impl ObjectKey {
    pub fn workspace_id(&self) -> WorkspacePk {
        self.workspace_id
    }

    pub fn workspace_id_str(&self) -> &str {
        &self.inner.as_ref()[self.workspace_id_str_range.start..self.workspace_id_str_range.end]
    }

    pub fn kind(&self) -> &str {
        &self.inner.as_ref()[self.kind_range.start..self.kind_range.end]
    }

    pub fn id(&self) -> &str {
        &self.inner.as_ref()[self.id_range.start..self.id_range.end]
    }

    pub fn checksum(&self) -> &str {
        &self.inner.as_ref()[self.checksum_range.start..self.checksum_range.end]
    }

    pub fn to_subject(&self) -> Subject {
        self.inner.as_ref().into()
    }

    pub fn to_index_reference(&self) -> IndexReference {
        IndexReference {
            kind: self.kind().to_string(),
            id: self.id().to_string(),
            checksum: self.checksum().to_string(),
        }
    }
}

impl TryFrom<String> for ObjectKey {
    type Error = Error;

    fn try_from(value: String) -> result::Result<Self, Self::Error> {
        let mut parts = value.splitn(6, '.');

        if let (Some(prefix), Some(workspace_id_str), Some(kind), Some(id), Some(_checksum), None) = (
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
        ) {
            if prefix != KEY_PREFIX_OBJECT {
                return Err(Self::Error::ParseObjectKey("incorrect key prefix"));
            }

            let prefix_len = prefix.len();
            let workspace_len = workspace_id_str.len();
            let kind_len = kind.len();
            let id_len = id.len();
            let inner: Arc<str> = value.into();
            let inner_len = inner.len();

            let workspace_id_str_range = ops::Range {
                start: prefix_len + 1,
                end: prefix_len + 1 + workspace_len,
            };
            let kind_range = ops::Range {
                start: prefix_len + 1 + workspace_len + 1,
                end: prefix_len + 1 + workspace_len + 1 + kind_len,
            };
            let id_range = ops::Range {
                start: prefix_len + 1 + workspace_len + 1 + kind_len + 1,
                end: prefix_len + 1 + workspace_len + 1 + kind_len + 1 + id_len,
            };
            let checksum_range = ops::Range {
                start: prefix_len + 1 + workspace_len + 1 + kind_len + 1 + id_len + 1,
                end: inner_len,
            };

            let workspace_id = inner.as_ref()
                [workspace_id_str_range.start..workspace_id_str_range.end]
                .parse()
                .map_err(|_| Self::Error::ParseObjectKey("failed to parse workspace id"))?;

            Ok(Self {
                inner,
                workspace_id,
                workspace_id_str_range,
                kind_range,
                id_range,
                checksum_range,
            })
        } else {
            Err(Self::Error::ParseObjectKey("invalid object key format"))
        }
    }
}

impl TryFrom<Subject> for ObjectKey {
    type Error = Error;

    #[inline]
    fn try_from(value: Subject) -> result::Result<Self, Self::Error> {
        value.to_string().try_into()
    }
}

impl TryFrom<(WorkspacePk, &str, &str, &str)> for ObjectKey {
    type Error = Error;

    #[inline]
    fn try_from(value: (WorkspacePk, &str, &str, &str)) -> result::Result<Self, Self::Error> {
        let inner = object_key(value.0, value.1, value.2, value.3);
        inner.try_into()
    }
}

impl From<ObjectKey> for String {
    fn from(value: ObjectKey) -> Self {
        value.inner.to_string()
    }
}

impl AsRef<str> for ObjectKey {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

pub async fn frigg_kv(context: &jetstream::Context, prefix: Option<&str>) -> Result<kv::Store> {
    let bucket = nats_stream_name(prefix, NATS_KV_BUCKET_NAME);

    let kv = match context.get_key_value(bucket).await {
        Ok(kv) => kv,
        Err(err) => match err.kind() {
            KeyValueErrorKind::GetBucket | KeyValueErrorKind::JetStream => {
                frigg_create_kv(context, context.metadata().subject_prefix()).await?
            }
            _ => return Err(err.into()),
        },
    };

    Ok(kv)
}

pub async fn frigg_create_kv(
    context: &jetstream::Context,
    prefix: Option<&str>,
) -> Result<kv::Store> {
    let bucket = nats_stream_name(prefix, NATS_KV_BUCKET_NAME);

    let kv = context
        .create_key_value(kv::Config {
            bucket,
            description: "Frigg store data".to_owned(),
            ..Default::default()
        })
        .await?;

    Ok(kv)
}

fn nats_stream_name(prefix: Option<&str>, suffix: impl AsRef<str>) -> String {
    let suffix = suffix.as_ref();

    match prefix {
        Some(prefix) => format!("{prefix}_{suffix}"),
        None => suffix.to_owned(),
    }
}

#[inline]
fn object_key(workspace_id: WorkspacePk, kind: &str, id: &str, checksum: &str) -> Subject {
    Subject::from(format!(
        "{KEY_PREFIX_OBJECT}.{workspace_id}.{kind}.{id}.{checksum}"
    ))
}

#[inline]
fn index_key(workspace_id: WorkspacePk, change_set_id: &str) -> Subject {
    Subject::from(format!("{KEY_PREFIX_INDEX}.{workspace_id}.{change_set_id}"))
}

// Internal impl of a `Watch` type vendored from the `async-nats` crate.
//
// See: https://github.com/nats-io/nats.rs/blob/7d63f1dd725c86a4f01723ea3194f17e30a0561b/async-nats/src/jetstream/kv/mod.rs#L1263-L1323
mod kv_history {
    use std::{
        marker::PhantomData,
        result,
        str::FromStr,
        task::Poll,
    };

    use futures::StreamExt as _;
    use pin_project_lite::pin_project;
    use si_data_nats::async_nats::{
        self,
        jetstream::{
            consumer::push::{
                Ordered,
                OrderedError,
            },
            kv::{
                Entry,
                Operation,
                ParseOperationError,
                WatcherErrorKind,
            },
        },
    };
    use thiserror::Error;

    const KV_OPERATION: &str = "KV-Operation";

    /// A structure representing the history of a key-value bucket, yielding past values.
    pub struct History {
        pub subscription: Ordered,
        pub done: bool,
        pub prefix: String,
        pub bucket: String,
    }

    #[derive(Debug, Error)]
    pub enum KvWatcherError {
        #[error("{0}")]
        Default(WatcherErrorKind, String),
        #[error("{0}")]
        Ordered(#[from] OrderedError),
    }

    impl futures::Stream for History {
        type Item = result::Result<Entry, KvWatcherError>;

        fn poll_next(
            mut self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Option<Self::Item>> {
            if self.done {
                return Poll::Ready(None);
            }
            match self.subscription.poll_next_unpin(cx) {
                Poll::Ready(message) => match message {
                    None => Poll::Ready(None),
                    Some(message) => {
                        let message = message?;
                        let info = message.info().map_err(|err| {
                            KvWatcherError::Default(
                                WatcherErrorKind::Other,
                                format!("failed to parse message metadata: {}", err),
                            )
                        })?;
                        if info.pending == 0 {
                            self.done = true;
                        }

                        let operation =
                            kv_operation_from_message(&message).unwrap_or(Operation::Put);

                        let key = message
                            .subject
                            .strip_prefix(&self.prefix)
                            .map(|s| s.to_string())
                            .unwrap();

                        Poll::Ready(Some(Ok(Entry {
                            bucket: self.bucket.clone(),
                            key,
                            value: message.payload.clone(),
                            revision: info.stream_sequence,
                            created: info.published,
                            delta: info.pending,
                            operation,
                            seen_current: self.done,
                        })))
                    }
                },
                std::task::Poll::Pending => Poll::Pending,
            }
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            (0, None)
        }
    }

    pin_project! {
        pub struct Keys<T> {
            #[pin]
            pub(super) inner: History,
            pub(super) _phantom: PhantomData<T>,
        }
    }

    impl<T> futures::Stream for Keys<T>
    where
        T: TryFrom<String>,
        T::Error: Into<super::Error>,
    {
        type Item = Result<T, super::Error>;

        fn poll_next(
            self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Option<Self::Item>> {
            let mut this = self.project();

            loop {
                match this.inner.poll_next_unpin(cx) {
                    Poll::Ready(None) => return Poll::Ready(None),
                    Poll::Ready(Some(res)) => match res {
                        Ok(entry) => {
                            // Skip purged and deleted keys
                            if matches!(entry.operation, Operation::Purge | Operation::Delete) {
                                // Try to poll again if we skip this one
                                continue;
                            } else {
                                return Poll::Ready(Some(entry.key.try_into().map_err(Into::into)));
                            }
                        }
                        Err(e) => return Poll::Ready(Some(Err(e.into()))),
                    },
                    Poll::Pending => return Poll::Pending,
                }
            }
        }
    }

    fn kv_operation_from_message(
        message: &async_nats::message::Message,
    ) -> Result<Operation, ParseOperationError> {
        let headers = match message.headers.as_ref() {
            Some(headers) => headers,
            None => return Ok(Operation::Put),
        };
        if let Some(op) = headers.get(KV_OPERATION) {
            Operation::from_str(op.as_str())
        } else {
            Ok(Operation::Put)
        }
    }
}
