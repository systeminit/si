use std::{result, str::Utf8Error};

use bytes::Bytes;
use si_data_nats::{
    async_nats::{self, jetstream::kv},
    jetstream, Subject,
};
use si_frontend_types::{index::MvIndex, object::FrontendObject, reference::ReferenceKind};
use si_id::{ChangeSetId, WorkspacePk};
use thiserror::Error;

const NATS_KV_BUCKET_NAME: &str = "FRIGG";

const KEY_PREFIX_INDEX: &str = "index";
const KEY_PREFIX_OBJECT: &str = "object";

#[remain::sorted]
#[derive(Debug, Error)]
pub enum Error {
    #[error("error creating kv store: {0}")]
    CreateKeyValue(#[from] async_nats::jetstream::context::CreateKeyValueError),
    #[error("error deserializing kv value: {0}")]
    Deserialize(#[source] serde_json::Error),
    #[error("entry error: {0}")]
    Entry(#[from] kv::EntryError),
    #[error("index object not found at key: {0}")]
    IndexObjectNotFound(Subject),
    #[error("object kind was expected to be 'MvIndex' but was '{0}'")]
    NotIndexKind(String),
    #[error("object listed in index not found: workspace: {workspace_id}, change set: {change_set_id}, kind: {kind}, id: {id}")]
    ObjectNotFoundFromIndex {
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
        kind: String,
        id: String,
    },
    #[error("put error: {0}")]
    Put(#[from] kv::PutError),
    #[error("error serializing kv value: {0}")]
    Serialize(#[source] serde_json::Error),
    #[error("update error: {0}")]
    Update(#[from] kv::UpdateError),
    #[error("utf8 encoding error: {0}")]
    Utf8(#[from] Utf8Error),
}

pub type FriggError = Error;

type Result<T> = result::Result<T, Error>;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct KvRevision(u64);

impl From<u64> for KvRevision {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

#[derive(Clone, Debug)]
pub struct FriggStore {
    store: kv::Store,
}

impl FriggStore {
    pub fn new(inner: kv::Store) -> Self {
        Self { store: inner }
    }

    pub async fn insert_object(
        &self,
        workspace_id: WorkspacePk,
        object: &FrontendObject,
    ) -> Result<Subject> {
        let key = Self::object_key(
            workspace_id,
            &object.kind.to_string(),
            &object.id,
            &object.checksum,
        );
        let value = serde_json::to_vec(&object).map_err(Error::Serialize)?;
        self.store.put(key.as_str(), value.into()).await?;

        Ok(key)
    }

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

    pub async fn get_object(
        &self,
        workspace_id: WorkspacePk,
        kind: &str,
        id: &str,
        checksum: &str,
    ) -> Result<Option<FrontendObject>> {
        match self
            .get_object_raw_bytes(&Self::object_key(workspace_id, kind, id, checksum))
            .await?
        {
            Some((bytes, _)) => Ok(Some(
                serde_json::from_slice(bytes.as_ref()).map_err(Error::Deserialize)?,
            )),
            None => Ok(None),
        }
    }

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

    pub async fn insert_index(
        &self,
        workspace_id: WorkspacePk,
        object: &FrontendObject,
    ) -> Result<KvRevision> {
        self.update_index(workspace_id, object, 0.into()).await
    }

    pub async fn update_index(
        &self,
        workspace_id: WorkspacePk,
        object: &FrontendObject,
        revision: KvRevision,
    ) -> Result<KvRevision> {
        let mv_index_kind_string = ReferenceKind::MvIndex.to_string();
        if object.kind != mv_index_kind_string {
            return Err(Error::NotIndexKind(object.kind.clone()));
        }

        // Insert the index as an object and get back the key name where it's stored
        let index_object_key = self.insert_object(workspace_id, object).await?;

        let index_pointer_key = Self::index_key(workspace_id, &object.id);

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

    pub async fn get_index(
        &self,
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
    ) -> Result<Option<(FrontendObject, KvRevision)>> {
        let index_pointer_key = Self::index_key(workspace_id, &change_set_id.to_string());

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

    async fn get_object_raw_bytes(&self, key: &Subject) -> Result<Option<(Bytes, KvRevision)>> {
        let Some(entry) = self.store.entry(key.to_string()).await? else {
            return Ok(None);
        };

        match entry.operation {
            kv::Operation::Delete | kv::Operation::Purge => Ok(None),
            kv::Operation::Put => Ok(Some((entry.value, entry.revision.into()))),
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
}

pub async fn frigg_kv(context: &jetstream::Context, prefix: Option<&str>) -> Result<kv::Store> {
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
