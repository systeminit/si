use std::{result, str::Utf8Error};

use bytes::Bytes;
use si_data_nats::{
    async_nats::{self, jetstream::kv},
    jetstream, Subject,
};
use si_events::workspace_snapshot::Checksum;
use si_frontend_types::{object::FrontendObject, reference::ReferenceKind};
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
    #[error("object kind was expected to be 'ReferenceKind::MvIndex' but was '{0}'")]
    NotIndexKind(ReferenceKind),
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
        kind: String,
        id: String,
        checksum: Checksum,
    ) -> Result<Option<FrontendObject>> {
        match self
            .get_object_raw_bytes(&Self::object_key(
                workspace_id,
                &kind,
                &id,
                &checksum.to_string(),
            ))
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
        kind: String,
        id: String,
    ) -> Result<Option<FrontendObject>> {
        // We want to consult an index to determine this which involves:
        //
        // - Retrieve current index
        // - Consult it for the checksum of the kind/id we were given
        // - Fetch that object
        todo!()
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
        if object.kind != ReferenceKind::MvIndex {
            return Err(Error::NotIndexKind(object.kind));
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

        let Some((bytes, revision)) = self
            .get_object_raw_bytes(&index_pointer_key)
            .await?
        else {
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
