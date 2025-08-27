use std::{
    result,
    str::Utf8Error,
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
                CreateErrorKind,
                Watch,
            },
            stream::ConsumerError,
        },
    },
    jetstream,
};
use si_frontend_mv_types::{
    index::{
        ChangeSetIndexPointerValue,
        ChangeSetMvIndex,
        DeploymentIndexPointerValue,
        DeploymentMvIndex,
    },
    materialized_view::materialized_view_definitions_checksum,
    object::FrontendObject,
    reference::ReferenceKind,
};
use si_id::{
    ChangeSetId,
    WorkspacePk,
};
use strum::AsRefStr;
use telemetry::prelude::*;
use thiserror::Error;

const NATS_KV_BUCKET_NAME: &str = "FRIGG";

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

    #[instrument(
        name = "frigg.insert_deployment_object",
        level = "debug",
        skip_all,
        fields(
            si.frontend_object.id = %object.id,
            si.frontend_object.kind = %object.kind,
        )
    )]
    pub async fn insert_deployment_object(&self, object: &FrontendObject) -> Result<Subject> {
        let key = Self::deployment_object_key(&object.kind, &object.id, &object.checksum);
        let value = serde_json::to_vec(&object).map_err(Error::Serialize)?;
        if let Err(err) = self.store.create(key.as_str(), value.into()).await {
            if !matches!(err.kind(), CreateErrorKind::AlreadyExists) {
                return Err(Error::Create(err));
            }
        };

        Ok(key)
    }

    #[instrument(
        name = "frigg.get_deployment_object",
        level = "debug",
        skip_all,
        fields(
            si.frontend_object.id = %id,
            si.frontend_object.kind = %kind,
            si.frontend_object.checksum = %checksum,
    ))]
    pub async fn get_deployment_object(
        &self,
        kind: ReferenceKind,
        id: &str,
        checksum: &str,
    ) -> Result<Option<FrontendObject>> {
        match self
            .get_object_raw_bytes(&Self::deployment_object_key(
                &kind.to_string(),
                id,
                checksum,
            ))
            .await?
        {
            Some((bytes, _)) => Ok(Some(
                serde_json::from_slice(bytes.as_ref()).map_err(Error::Deserialize)?,
            )),
            None => Ok(None),
        }
    }

    #[instrument(
        name = "frigg.get_current_deployment_object",
        level = "debug",
        skip_all,
        fields(
            si.frontend_object.id = %id,
            si.frontend_object.kind = %kind,
    ))]
    pub async fn get_current_deployment_object(
        &self,
        kind: ReferenceKind,
        id: &str,
    ) -> Result<Option<FrontendObject>> {
        let maybe_mv_index = self.get_deployment_index().await?.map(|r| r.0);

        self.get_current_deployment_object_with_index(kind, id, maybe_mv_index)
            .await
    }

    #[instrument(
        name = "frigg.get_current_deployment_object_with_index",
        level = "debug",
        skip_all,
        fields(
            si.frontend_object.id = %id,
            si.frontend_object.kind = %kind,
        )
    )]
    pub async fn get_current_deployment_object_with_index(
        &self,
        kind: ReferenceKind,
        id: &str,
        maybe_mv_index: Option<FrontendObject>,
    ) -> Result<Option<FrontendObject>> {
        let kind_str = kind.to_string();
        let Some(current_index) = maybe_mv_index else {
            return Ok(None);
        };
        let mv_index: DeploymentMvIndex =
            serde_json::from_value(current_index.data).map_err(FriggError::Deserialize)?;

        for index_entry in mv_index.mv_list {
            if index_entry.kind == kind_str && index_entry.id == id {
                return Ok(Some(
                    self.get_deployment_object(kind, id, &index_entry.checksum)
                        .await?
                        .ok_or_else(|| FriggError::ObjectNotFoundForDeploymentIndex {
                            kind: kind.to_string(),
                            id: id.to_string(),
                        })?,
                ));
            }
        }

        Ok(None)
    }

    #[instrument(
        name = "frigg.get_current_deployment_objects_by_kind",
        level = "debug",
        skip_all,
        fields(
            si.frontend_object.kind = %kind,
    ))]
    pub async fn get_current_deployment_objects_by_kind(
        &self,
        kind: ReferenceKind,
    ) -> Result<Vec<FrontendObject>> {
        let kind_str = kind.to_string();
        let Some((current_index, _)) = self.get_deployment_index().await? else {
            return Ok(Vec::new());
        };
        let mv_index: DeploymentMvIndex =
            serde_json::from_value(current_index.data).map_err(FriggError::Deserialize)?;

        let mut objects = Vec::new();
        for index_entry in mv_index.mv_list {
            if index_entry.kind == kind_str {
                if let Ok(Some(obj)) = self
                    .get_deployment_object(kind, &index_entry.id, &index_entry.checksum)
                    .await
                {
                    objects.push(obj);
                }
            }
        }

        Ok(objects)
    }

    #[instrument(
        name = "frigg.insert_workspace_object",
        level = "debug",
        skip_all,
        fields(
            si.workspace.id = %workspace_id,
            si.frontend_object.id = %object.id,
            si.frontend_object.kind = %object.kind,
        )
    )]
    pub async fn insert_workspace_object(
        &self,
        workspace_id: WorkspacePk,
        object: &FrontendObject,
    ) -> Result<Subject> {
        let key = Self::workspace_object_key(
            workspace_id,
            &object.kind.to_string(),
            &object.id,
            &object.checksum,
        );
        let value = serde_json::to_vec(&object).map_err(Error::Serialize)?;
        if let Err(err) = self.store.create(key.as_str(), value.into()).await {
            if !matches!(err.kind(), CreateErrorKind::AlreadyExists) {
                return Err(Error::Create(err));
            }
        };

        Ok(key)
    }

    #[instrument(
        name = "frigg.get_workspace_object",
        level = "debug",
        skip_all,
        fields(
            si.workspace.id = %workspace_id,
            si.frontend_object.id = %id,
            si.frontend_object.kind = %kind,
            si.frontend_object.checksum = %checksum,
    ))]
    pub async fn get_workspace_object(
        &self,
        workspace_id: WorkspacePk,
        kind: &str,
        id: &str,
        checksum: &str,
    ) -> Result<Option<FrontendObject>> {
        match self
            .get_object_raw_bytes(&Self::workspace_object_key(
                workspace_id,
                kind,
                id,
                checksum,
            ))
            .await?
        {
            Some((bytes, _)) => Ok(Some(
                serde_json::from_slice(bytes.as_ref()).map_err(Error::Deserialize)?,
            )),
            None => Ok(None),
        }
    }

    #[instrument(
        name = "frigg.get_current_workspace_object",
        level = "debug",
        skip_all,
        fields(
            si.workspace.id = %workspace_id,
            si.frontend_object.id = %id,
            si.frontend_object.kind = %kind,
    ))]
    pub async fn get_current_workspace_object(
        &self,
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
        kind: &str,
        id: &str,
    ) -> Result<Option<FrontendObject>> {
        let Some((current_index, _)) = self
            .get_change_set_index(workspace_id, change_set_id)
            .await?
        else {
            return Ok(None);
        };
        let mv_index: ChangeSetMvIndex =
            serde_json::from_value(current_index.data).map_err(FriggError::Deserialize)?;
        for index_entry in mv_index.mv_list {
            if index_entry.kind == kind && index_entry.id == id {
                return Ok(Some(
                    self.get_workspace_object(workspace_id, kind, id, &index_entry.checksum)
                        .await?
                        .ok_or_else(|| FriggError::ObjectNotFoundForChangesetIndex {
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

    async fn insert_or_update_deployment_index_preamble(
        &self,
        object: &FrontendObject,
    ) -> Result<(Subject, Subject)> {
        let mv_index_kind_string = ReferenceKind::DeploymentMvIndex.to_string();
        if object.kind != mv_index_kind_string {
            return Err(Error::NotIndexKind(object.kind.clone()));
        }

        let index_object_key = self.insert_deployment_object(object).await?;
        let index_pointer_key = Self::deployment_index_key();

        Ok((index_object_key, index_pointer_key))
    }

    /// Insert a new deployment `MvIndex` into the store, and update the associated index pointer
    /// to refer to the newly inserted `MvIndex`.
    ///
    /// Will fail if the index pointer already exists.
    #[instrument(
        name = "frigg.insert_deployment_index",
        level = "debug",
        skip_all,
        fields(
            si.frontend_object.id = %object.id,
            si.frontend_object.kind = %object.kind,
            si.frontend_object.checksum = %object.checksum,
        )
    )]
    pub async fn insert_deployment_index(&self, object: &FrontendObject) -> Result<KvRevision> {
        let (index_object_key, index_pointer_key) = self
            .insert_or_update_deployment_index_preamble(object)
            .await?;
        let index_pointer_value = DeploymentIndexPointerValue {
            index_object_key: index_object_key.into_string(),
            definition_checksum: materialized_view_definitions_checksum(),
            index_checksum: object.checksum.to_owned(),
        };
        let value = serde_json::to_vec(&index_pointer_value).map_err(Error::Serialize)?;
        let new_revision = self.store.create(index_pointer_key, value.into()).await?;

        Ok(new_revision.into())
    }

    /// Insert an updated `MvIndex` into the store, and update the associated index pointer to
    /// refer to the newly inserted `MvIndex`.
    ///
    /// Will fail if the index pointer has been updated since `revision` was fetched.
    #[instrument(
        name = "frigg.update_deployment_index",
        level = "debug",
        skip_all,
        fields(
            si.frontend_object.id = %object.id,
            si.frontend_object.kind = %object.kind,
            si.frontend_object.checksum = %object.checksum,
        )
    )]
    pub async fn update_deployment_index(
        &self,
        object: &FrontendObject,
        revision: KvRevision,
    ) -> Result<KvRevision> {
        let (index_object_key, index_pointer_key) = self
            .insert_or_update_deployment_index_preamble(object)
            .await?;
        let index_pointer_value = DeploymentIndexPointerValue {
            index_object_key: index_object_key.into_string(),
            definition_checksum: materialized_view_definitions_checksum(),
            index_checksum: object.checksum.to_owned(),
        };
        let value = serde_json::to_vec(&index_pointer_value).map_err(Error::Serialize)?;

        let new_revision = self
            .store
            .update(index_pointer_key, value.into(), revision.0)
            .await?;

        Ok(new_revision.into())
    }

    /// Put a new `MvIndex` into the store, and update the associated index pointer to refer
    /// to the newly inserted `MvIndex`.
    ///
    /// Will NOT fail if the index pointer already exists.
    #[instrument(
        name = "frigg.put_deployment_index",
        level = "debug",
        skip_all,
        fields(
            si.frontend_object.id = %object.id,
            si.frontend_object.kind = %object.kind,
            si.frontend_object.checksum = %object.checksum,
        )
    )]
    pub async fn put_deployment_index(&self, object: &FrontendObject) -> Result<KvRevision> {
        let (index_object_key, index_pointer_key) = self
            .insert_or_update_deployment_index_preamble(object)
            .await?;
        let index_pointer_value = DeploymentIndexPointerValue {
            index_object_key: index_object_key.into_string(),
            definition_checksum: materialized_view_definitions_checksum(),
            index_checksum: object.checksum.to_owned(),
        };
        let value = serde_json::to_vec(&index_pointer_value).map_err(Error::Serialize)?;

        let new_revision = self.store.put(index_pointer_key, value.into()).await?;

        Ok(new_revision.into())
    }

    #[instrument(
        name = "frigg.get_deployment_index",
        level = "debug",
        skip_all,
        fields()
    )]
    pub async fn get_deployment_index(&self) -> Result<Option<(FrontendObject, KvRevision)>> {
        let index_pointer_key = Self::deployment_index_key();

        let Some((bytes, revision)) = self.get_object_raw_bytes(&index_pointer_key).await? else {
            return Ok(None);
        };
        let index_pointer_value: DeploymentIndexPointerValue =
            match serde_json::from_slice(bytes.as_ref()).map_err(Error::Deserialize) {
                Ok(value) => value,
                Err(err) => {
                    debug!(
                        "Unable to deserialize deployment index pointer value at {}: {}",
                        index_pointer_key, err
                    );
                    return Ok(None);
                }
            };
        // TEMPORARY: Definition checksum validation bypassed
        // This allows deployment indexes to be used even when definition checksums don't match
        //
        // TODO: Re-enable this validation once the deployment-level MV definition checksum is decoupled from the change set level MV definition checksum.
        /*
        // If the definition checksum for the current set of MVs is not the same as the one the
        // MvIndex was built for, then the MvIndex is out of date and should not be used at all.
        if index_pointer_value.definition_checksum != materialized_view_definitions_checksum() {
            debug!(
                "deployment index pointer is out of date: index checksum: {}, expected checksum: {}",
                index_pointer_value.definition_checksum,
                materialized_view_definitions_checksum()
            );
            return Ok(None);
        }
        */
        debug!(
            "deployment index checksum validation bypassed (temporary): index checksum: {}, expected checksum: {}",
            index_pointer_value.definition_checksum,
            materialized_view_definitions_checksum()
        );

        let object_key = index_pointer_value.index_object_key;
        let bytes = self
            .store
            .get(object_key.to_string())
            .await?
            .ok_or(Error::IndexObjectNotFound(object_key.into()))?;
        let object = serde_json::from_slice(bytes.as_ref()).map_err(Error::Deserialize)?;

        Ok(Some((object, revision)))
    }

    #[instrument(
        name = "frigg.watch_deployment_index",
        level = "debug",
        skip_all,
        fields()
    )]
    pub async fn watch_deployment_index(&self) -> Result<Watch> {
        let index_pointer_key = Self::deployment_index_key();
        self.store
            .watch(index_pointer_key)
            .await
            .map_err(Into::into)
    }

    async fn insert_or_update_change_set_index_preamble(
        &self,
        workspace_id: WorkspacePk,
        change_set_id: &str,
        object: &FrontendObject,
    ) -> Result<(Subject, Subject)> {
        let mv_index_kind_string = ReferenceKind::ChangeSetMvIndex.to_string();
        if object.kind != mv_index_kind_string {
            return Err(Error::NotIndexKind(object.kind.clone()));
        }

        let index_object_key = self.insert_workspace_object(workspace_id, object).await?;
        let index_pointer_key = Self::change_set_index_key(workspace_id, change_set_id);

        Ok((index_object_key, index_pointer_key))
    }

    /// Insert a new change set `MvIndex` into the store, and update the associated index pointer
    /// to refer to the newly inserted `MvIndex`.
    ///
    /// Will fail if the index pointer already exists.
    #[instrument(
        name = "frigg.insert_change_set_index",
        level = "debug",
        skip_all,
        fields(
            si.workspace.id = %workspace_id,
            si.change_set.id = change_set_id,
            si.frontend_object.id = %object.id,
            si.frontend_object.kind = %object.kind,
            si.frontend_object.checksum = %object.checksum,
        )
    )]
    pub async fn insert_change_set_index(
        &self,
        workspace_id: WorkspacePk,
        change_set_id: &str,
        object: &FrontendObject,
    ) -> Result<KvRevision> {
        let (index_object_key, index_pointer_key) = self
            .insert_or_update_change_set_index_preamble(workspace_id, change_set_id, object)
            .await?;
        let index_pointer_value = ChangeSetIndexPointerValue {
            index_object_key: index_object_key.into_string(),
            snapshot_address: object.id.to_owned(),
            definition_checksum: materialized_view_definitions_checksum(),
            index_checksum: object.checksum.to_owned(),
        };
        let value = serde_json::to_vec(&index_pointer_value).map_err(Error::Serialize)?;
        let new_revision = self.store.create(index_pointer_key, value.into()).await?;

        Ok(new_revision.into())
    }

    /// Creates a new index pointer to for the provided [`IndexPointerValue`], allowing a new
    /// change set to reuse the index of another change set and avoid rebuilding the world
    /// unnecessarily.
    ///
    /// Will fail if the index pointer already exists.
    #[instrument(
        name = "frigg.insert_change_set_index_key_for_existing",
        level = "debug",
        skip_all,
        fields(
            si.workspace.id = %workspace_id,
            si.change_set.id = change_set_id,
        )
    )]
    pub async fn insert_change_set_index_key_for_existing_index(
        &self,
        workspace_id: WorkspacePk,
        change_set_id: &str,
        index_pointer_value: ChangeSetIndexPointerValue,
    ) -> Result<KvRevision> {
        let index_pointer_key = Self::change_set_index_key(workspace_id, change_set_id);

        let value = serde_json::to_vec(&index_pointer_value).map_err(Error::Serialize)?;
        let new_revision = self.store.create(index_pointer_key, value.into()).await?;

        Ok(new_revision.into())
    }

    /// Insert an updated `MvIndex` into the store, and update the associated index pointer to
    /// refer to the newly inserted `MvIndex`.
    ///
    /// Will fail if the index pointer has been updated since `revision` was fetched.
    #[instrument(
        name = "frigg.update_change_set_index",
        level = "debug",
        skip_all,
        fields(
            si.workspace.id = %workspace_id,
            si.change_set.id = change_set_id,
            si.frontend_object.id = %object.id,
            si.frontend_object.kind = %object.kind,
            si.frontend_object.checksum = %object.checksum,
        )
    )]
    pub async fn update_change_set_index(
        &self,
        workspace_id: WorkspacePk,
        change_set_id: &str,
        object: &FrontendObject,
        revision: KvRevision,
    ) -> Result<KvRevision> {
        let (index_object_key, index_pointer_key) = self
            .insert_or_update_change_set_index_preamble(workspace_id, change_set_id, object)
            .await?;
        let index_pointer_value = ChangeSetIndexPointerValue {
            index_object_key: index_object_key.into_string(),
            snapshot_address: object.id.to_owned(),
            definition_checksum: materialized_view_definitions_checksum(),
            index_checksum: object.checksum.to_owned(),
        };
        let value = serde_json::to_vec(&index_pointer_value).map_err(Error::Serialize)?;

        let new_revision = self
            .store
            .update(index_pointer_key, value.into(), revision.0)
            .await?;

        Ok(new_revision.into())
    }

    /// Put a new `MvIndex` into the store, and update the associated index pointer to refer
    /// to the newly inserted `MvIndex`.
    ///
    /// Will NOT fail if the index pointer already exists.
    #[instrument(
        name = "frigg.put_change_set_index",
        level = "debug",
        skip_all,
        fields(
            si.workspace.id = %workspace_id,
            si.change_set.id = change_set_id,
            si.frontend_object.id = %object.id,
            si.frontend_object.kind = %object.kind,
            si.frontend_object.checksum = %object.checksum,
        )
    )]
    pub async fn put_change_set_index(
        &self,
        workspace_id: WorkspacePk,
        change_set_id: &str,
        object: &FrontendObject,
    ) -> Result<KvRevision> {
        let (index_object_key, index_pointer_key) = self
            .insert_or_update_change_set_index_preamble(workspace_id, change_set_id, object)
            .await?;
        let index_pointer_value = ChangeSetIndexPointerValue {
            index_object_key: index_object_key.into_string(),
            snapshot_address: object.id.to_owned(),
            definition_checksum: materialized_view_definitions_checksum(),
            index_checksum: object.checksum.to_owned(),
        };
        let value = serde_json::to_vec(&index_pointer_value).map_err(Error::Serialize)?;

        let new_revision = self.store.put(index_pointer_key, value.into()).await?;

        Ok(new_revision.into())
    }

    #[instrument(
        name = "frigg.get_change_set_index",
        level = "debug",
        skip_all,
        fields(
            si.workspace.id = %workspace_id,
            si.change_set.id = %change_set_id,
        )
    )]
    pub async fn get_change_set_index(
        &self,
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
    ) -> Result<Option<(FrontendObject, KvRevision)>> {
        let index_pointer_key =
            Self::change_set_index_key(workspace_id, &change_set_id.to_string());

        let Some((bytes, revision)) = self.get_object_raw_bytes(&index_pointer_key).await? else {
            return Ok(None);
        };
        let index_pointer_value: ChangeSetIndexPointerValue =
            match serde_json::from_slice(bytes.as_ref()).map_err(Error::Deserialize) {
                Ok(value) => value,
                Err(err) => {
                    debug!(
                        "Unable to deserialize change set index pointer value at {}: {}",
                        index_pointer_key, err
                    );
                    return Ok(None);
                }
            };
        // If the definition checksum for the current set of MVs is not the same as the one the
        // MvIndex was built for, then the MvIndex is out of date and should not be used at all.
        if index_pointer_value.definition_checksum != materialized_view_definitions_checksum() {
            debug!(
                "change set index pointer is out of date: index checksum: {}, expected checksum: {}",
                index_pointer_value.definition_checksum,
                materialized_view_definitions_checksum()
            );
            return Ok(None);
        }

        let object_key = index_pointer_value.index_object_key;
        let bytes = self
            .store
            .get(object_key.to_string())
            .await?
            .ok_or(Error::IndexObjectNotFound(object_key.into()))?;
        let object = serde_json::from_slice(bytes.as_ref()).map_err(Error::Deserialize)?;

        Ok(Some((object, revision)))
    }

    #[instrument(
        name = "frigg.get_change_set_index_pointer_value",
        level = "debug",
        skip_all,
        fields(
            si.workspace.id = %workspace_id,
            si.change_set.id = %change_set_id,
        )
    )]
    pub async fn get_change_set_index_pointer_value(
        &self,
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
    ) -> Result<Option<(ChangeSetIndexPointerValue, KvRevision)>> {
        let index_pointer_key =
            Self::change_set_index_key(workspace_id, &change_set_id.to_string());

        let Some((bytes, revision)) = self.get_object_raw_bytes(&index_pointer_key).await? else {
            return Ok(None);
        };

        let index_pointer_value: ChangeSetIndexPointerValue =
            serde_json::from_slice(bytes.as_ref()).map_err(Error::Deserialize)?;
        Ok(Some((index_pointer_value, revision)))
    }

    #[instrument(
        name = "frigg.watch_change_set_index",
        level = "debug",
        skip_all,
        fields(
            si.workspace.id = %workspace_id,
            si.change_set.id = %change_set_id,
        )
    )]
    pub async fn watch_change_set_index(
        &self,
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
    ) -> Result<Watch> {
        let index_pointer_key =
            Self::change_set_index_key(workspace_id, &change_set_id.to_string());
        self.store
            .watch(index_pointer_key)
            .await
            .map_err(Into::into)
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

    // NOTE: this will be useful when garbage-collecting old indexes
    #[allow(dead_code)]
    async fn change_set_index_keys_for_workspace(&self, workspace_id: WorkspacePk) -> Result<Keys> {
        let filter_subject = Self::change_set_index_key(workspace_id, "*").into_string();

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

        Ok(Keys { inner: entries })
    }

    #[inline]
    fn deployment_object_key(kind: &str, id: &str, checksum: &str) -> Subject {
        Subject::from(format!(
            "{}.{}.{kind}.{id}.{checksum}",
            Domain::Object.as_ref(),
            Scope::Deployment.as_ref(),
        ))
    }

    #[inline]
    fn workspace_object_key(
        workspace_id: WorkspacePk,
        kind: &str,
        id: &str,
        checksum: &str,
    ) -> Subject {
        Subject::from(format!(
            "{}.{}.{workspace_id}.{kind}.{id}.{checksum}",
            Domain::Object.as_ref(),
            Scope::Workspace.as_ref(),
        ))
    }

    #[inline]
    fn deployment_index_key() -> Subject {
        Subject::from(format!(
            "{}.{}",
            Domain::Index.as_ref(),
            Scope::Deployment.as_ref()
        ))
    }

    #[inline]
    fn change_set_index_key(workspace_id: WorkspacePk, change_set_id: &str) -> Subject {
        Subject::from(format!(
            "{}.{}.{workspace_id}.{change_set_id}",
            Domain::Index.as_ref(),
            Scope::ChangeSet.as_ref()
        ))
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

// Internal impl of a `Watch` type vendored from the `async-nats` crate.
//
// See: https://github.com/nats-io/nats.rs/blob/7d63f1dd725c86a4f01723ea3194f17e30a0561b/async-nats/src/jetstream/kv/mod.rs#L1263-L1323
mod kv_history {
    use std::{
        result,
        str::FromStr as _,
        task::Poll,
    };

    use futures::StreamExt as _;
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
    pub enum WatcherError {
        #[error("{0}")]
        Default(WatcherErrorKind, String),
        #[error("{0}")]
        Ordered(#[from] OrderedError),
    }

    impl futures::Stream for History {
        type Item = result::Result<Entry, WatcherError>;

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
                            WatcherError::Default(
                                WatcherErrorKind::Other,
                                format!("failed to parse message metadata: {err}"),
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

    pub struct Keys {
        pub inner: History,
    }

    impl futures::Stream for Keys {
        type Item = Result<String, WatcherError>;

        fn poll_next(
            mut self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Option<Self::Item>> {
            loop {
                match self.inner.poll_next_unpin(cx) {
                    Poll::Ready(None) => return Poll::Ready(None),
                    Poll::Ready(Some(res)) => match res {
                        Ok(entry) => {
                            // Skip purged and deleted keys
                            if matches!(entry.operation, Operation::Purge | Operation::Delete) {
                                // Try to poll again if we skip this one
                                continue;
                            } else {
                                return Poll::Ready(Some(Ok(entry.key)));
                            }
                        }
                        Err(e) => return Poll::Ready(Some(Err(e))),
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
