use si_data_nats::{
    Subject,
    async_nats::jetstream::kv::{
        CreateErrorKind,
        Watch,
    },
};
use si_frontend_mv_types::{
    definition_checksum::materialized_view_definition_checksums,
    index::deployment::{
        DeploymentIndexPointerValueV2,
        DeploymentIndexPointerVersion,
    },
    object::FrontendObject,
    reference::{
        IndexReference,
        ReferenceKind,
    },
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
};

impl FriggStore {
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
        kind: &str,
        id: &str,
        checksum: &str,
    ) -> Result<Option<FrontendObject>> {
        match self
            .get_object_raw_bytes(&Self::deployment_object_key(kind, id, checksum))
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
        kind: &str,
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
        kind: &str,
        id: &str,
        maybe_mv_index: Option<FrontendObject>,
    ) -> Result<Option<FrontendObject>> {
        let Some(current_index) = maybe_mv_index else {
            return Ok(None);
        };
        let mv_list = Self::mv_list_from_deployment_mv_index_version_data(current_index.data)?;

        for index_entry in mv_list {
            if index_entry.kind == kind && index_entry.id == id {
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

    pub async fn get_current_deployment_object_with_mvlist(
        &self,
        kind: &str,
        id: &str,
        mv_list: &[IndexReference],
    ) -> Result<Option<FrontendObject>> {
        for index_entry in mv_list {
            if index_entry.kind == kind && index_entry.id == id {
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
        kind: &str,
    ) -> Result<Vec<FrontendObject>> {
        let Some((current_index, _)) = self.get_deployment_index().await? else {
            return Ok(Vec::new());
        };
        let mv_list = Self::mv_list_from_deployment_mv_index_version_data(current_index.data)?;

        let mut objects = Vec::new();
        for index_entry in mv_list {
            if index_entry.kind == kind {
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

    pub fn mv_list_from_deployment_mv_index_version_data(
        versioned_index_data: serde_json::Value,
    ) -> Result<Vec<IndexReference>> {
        Ok(
            match serde_json::from_value::<
                si_frontend_mv_types::index::deployment::DeploymentMvIndexVersion,
            >(versioned_index_data)
            .map_err(FriggError::Deserialize)?
            {
                si_frontend_mv_types::index::deployment::DeploymentMvIndexVersion::V1(v1_index) => {
                    v1_index.mv_list
                }
                si_frontend_mv_types::index::deployment::DeploymentMvIndexVersion::V2(v2_index) => {
                    v2_index.mv_list
                }
            },
        )
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
        let index_pointer_value = DeploymentIndexPointerValueV2 {
            index_object_key: index_object_key.into_string(),
            definition_checksums: materialized_view_definition_checksums().clone(),
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
        let index_pointer_value = DeploymentIndexPointerValueV2 {
            index_object_key: index_object_key.into_string(),
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
        let index_pointer_value = DeploymentIndexPointerValueV2 {
            index_object_key: index_object_key.into_string(),
            definition_checksums: materialized_view_definition_checksums().clone(),
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
        let index_pointer_value =
            match serde_json::from_slice::<DeploymentIndexPointerVersion>(bytes.as_ref())
                .map_err(Error::Deserialize)?
            {
                DeploymentIndexPointerVersion::V1(_) => return Ok(None),
                DeploymentIndexPointerVersion::V2(index) => index,
            };

        // TEMPORARY: Definition checksum validation bypassed
        // This allows deployment indexes to be used even when definition checksums don't match
        //
        // TODO: Re-enable this validation once the deployment-level MV definition checksum is decoupled from the change set level MV definition checksum.
        /*
        // If the definition checksum for the current set of MVs is not the same as the one the
        // MvIndex was built for, then the MvIndex is out of date and should not be used at all.
        if index_pointer_value.definition_checksums != materialized_view_definition_checksums() {
            debug!(
                "deployment index pointer is out of date: index checksums: {:?}, expected checksums: {:?}",
                index_pointer_value.definition_checksums,
                materialized_view_definition_checksums()
            );
            return Ok(None);
        }
        */
        debug!(
            "deployment index checksum validation bypassed (temporary): index checksums: {:?}, expected checksums: {:?}",
            index_pointer_value.definition_checksums,
            materialized_view_definition_checksums()
        );

        let object_key = index_pointer_value.index_object_key;
        let bytes = self
            .store
            .get(object_key.to_string())
            .await
            .map_err(Error::EntryGetDeploymentIndex)?
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

    #[inline]
    fn deployment_object_key(kind: &str, id: &str, checksum: &str) -> Subject {
        Subject::from(format!(
            "{}.{}.{kind}.{id}.{checksum}",
            Domain::Object.as_ref(),
            Scope::Deployment.as_ref(),
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
}
