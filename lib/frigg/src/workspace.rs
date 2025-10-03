use kv_history::{
    History,
    Keys,
};
use serde::{
    Deserialize,
    Serialize,
    de::DeserializeOwned,
};
use si_data_nats::{
    Subject,
    async_nats::jetstream::{
        consumer::push::OrderedConfig,
        kv::{
            CreateErrorKind,
            Watch,
        },
    },
};
use si_frontend_mv_types::{
    definition_checksum::materialized_view_definition_checksums,
    index::change_set::{
        ChangeSetIndexPointerValueV2,
        ChangeSetIndexPointerVersion,
        ChangeSetMvIndexVersion,
    },
    object::FrontendObject,
    reference::ReferenceKind,
};
use si_id::{
    ChangeSetId,
    WorkspacePk,
};
use telemetry::prelude::*;

use crate::{
    Domain,
    Error,
    FriggError,
    FriggStore,
    KvRevision,
    Result,
    Scope,
    kv_history,
};

impl FriggStore {
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
        name = "frigg.get_workspace_object",
        level = "debug",
        skip_all,
        fields(
            si.workspace.id = %workspace_id,
            si.frontend_object.id = %id,
            si.frontend_object.kind = %kind,
            si.frontend_object.checksum = %checksum,
    ))]
    pub async fn get_workspace_object_data<T: DeserializeOwned>(
        &self,
        workspace_id: WorkspacePk,
        kind: &str,
        id: &str,
        checksum: &str,
    ) -> Result<Option<T>> {
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
                serde_json::from_slice::<FrontendObjectData<T>>(bytes.as_ref())
                    .map_err(Error::Deserialize)?
                    .data,
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
        let maybe_mv_index = self
            .get_change_set_index(workspace_id, change_set_id)
            .await?
            .map(|r| r.0);

        self.get_current_workspace_object_with_index(
            workspace_id,
            change_set_id,
            kind,
            id,
            maybe_mv_index,
        )
        .await
    }

    #[instrument(
        name = "frigg.get_current_workspace_object_with_index",
        level = "debug",
        skip_all,
        fields(
            si.workspace.id = %workspace_id,
            si.change_set.id = %change_set_id,
            si.frontend_object.id = %id,
            si.frontend_object.kind = %kind,
        )
    )]
    pub async fn get_current_workspace_object_with_index(
        &self,
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
        kind: &str,
        id: &str,
        maybe_mv_index: Option<FrontendObject>,
    ) -> Result<Option<FrontendObject>> {
        let Some(current_index) = maybe_mv_index else {
            return Ok(None);
        };
        let mv_list =
            match serde_json::from_value(current_index.data).map_err(FriggError::Deserialize)? {
                ChangeSetMvIndexVersion::V1(v1_index) => v1_index.mv_list,
                ChangeSetMvIndexVersion::V2(v2_index) => v2_index.mv_list,
            };
        for index_entry in mv_list {
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
        let index_pointer_value = ChangeSetIndexPointerValueV2 {
            index_object_key: index_object_key.into_string(),
            snapshot_address: object.id.to_owned(),
            definition_checksums: materialized_view_definition_checksums().clone(),
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
        index_pointer_value: ChangeSetIndexPointerValueV2,
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
        let index_pointer_value = ChangeSetIndexPointerValueV2 {
            index_object_key: index_object_key.into_string(),
            snapshot_address: object.id.to_owned(),
            definition_checksums: materialized_view_definition_checksums().clone(),
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
        let index_pointer_value = ChangeSetIndexPointerValueV2 {
            index_object_key: index_object_key.into_string(),
            snapshot_address: object.id.to_owned(),
            definition_checksums: materialized_view_definition_checksums().clone(),
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
        let Some((index_pointer_value, revision)) = self
            .get_change_set_index_pointer_value(workspace_id, change_set_id)
            .await?
        else {
            return Ok(None);
        };

        // If the definition checksum for the current set of MVs is not the same as the one the
        // MvIndex was built for, then the MvIndex is out of date and should not be used at all.
        if &index_pointer_value.definition_checksums != materialized_view_definition_checksums() {
            debug!(
                "change set index pointer is out of date: index checksums: {:?}, expected checksums: {:?}",
                index_pointer_value.definition_checksums,
                materialized_view_definition_checksums()
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
    ) -> Result<Option<(ChangeSetIndexPointerValueV2, KvRevision)>> {
        let index_pointer_key =
            Self::change_set_index_key(workspace_id, &change_set_id.to_string());

        let Some((bytes, revision)) = self.get_object_raw_bytes(&index_pointer_key).await? else {
            return Ok(None);
        };

        let index_pointer_value =
            match serde_json::from_slice::<ChangeSetIndexPointerVersion>(bytes.as_ref())
                .map_err(Error::Deserialize)?
            {
                ChangeSetIndexPointerVersion::V1(_) => return Ok(None),
                ChangeSetIndexPointerVersion::V2(index) => index,
            };

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
                replay_policy: si_data_nats::async_nats::jetstream::consumer::ReplayPolicy::Instant,
                // We only need to know the latest state for each key, not the whole history
                deliver_policy:
                    si_data_nats::async_nats::jetstream::consumer::DeliverPolicy::LastPerSubject,
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
    fn change_set_index_key(workspace_id: WorkspacePk, change_set_id: &str) -> Subject {
        Subject::from(format!(
            "{}.{}.{workspace_id}.{change_set_id}",
            Domain::Index.as_ref(),
            Scope::ChangeSet.as_ref()
        ))
    }
}

// Used for deserializing the data field directly to a type.
// This is just a subset of FrontendObject with a different type on the data field.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FrontendObjectData<T> {
    data: T,
}
