// Auth-generated code!
// No touchy!

impl crate::protobuf::ServiceEntity {
    pub fn new(
        name: Option<String>,
        display_name: Option<String>,
        description: Option<String>,
        constraints: Option<crate::protobuf::ServiceComponentConstraints>,
        implicit_constraints: Option<crate::protobuf::ServiceComponentConstraints>,
        properties: Option<crate::protobuf::ServiceEntityProperties>,
        si_properties: Option<si_cea::protobuf::EntitySiProperties>,
        change_set_id: Option<String>,
    ) -> si_cea::CeaResult<crate::protobuf::ServiceEntity> {
        let mut si_storable = si_data::protobuf::DataStorable::default();
        si_storable.change_set_id = change_set_id;
        si_storable.change_set_event_type =
            si_data::protobuf::DataStorableChangeSetEventType::Create as i32;
        si_properties
            .as_ref()
            .ok_or_else(|| si_data::DataError::ValidationError("siProperties".into()))?;
        let billing_account_id = si_properties
            .as_ref()
            .unwrap()
            .billing_account_id
            .as_ref()
            .ok_or_else(|| {
                si_data::DataError::ValidationError("siProperties.billingAccountId".into())
            })?;
        si_storable.add_to_tenant_ids(billing_account_id);
        let organization_id = si_properties
            .as_ref()
            .unwrap()
            .organization_id
            .as_ref()
            .ok_or_else(|| {
                si_data::DataError::ValidationError("siProperties.organizationId".into())
            })?;
        si_storable.add_to_tenant_ids(organization_id);
        let workspace_id = si_properties
            .as_ref()
            .unwrap()
            .workspace_id
            .as_ref()
            .ok_or_else(|| {
                si_data::DataError::ValidationError("siProperties.workspaceId".into())
            })?;
        si_storable.add_to_tenant_ids(workspace_id);

        let mut result: crate::protobuf::ServiceEntity = Default::default();
        result.name = name;
        result.display_name = display_name;
        result.description = description;
        result.constraints = constraints;
        result.implicit_constraints = implicit_constraints;
        result.properties = properties;
        result.si_properties = si_properties;
        result.si_storable = Some(si_storable);

        Ok(result)
    }

    pub async fn create(
        db: &si_data::Db,
        name: Option<String>,
        display_name: Option<String>,
        description: Option<String>,
        constraints: Option<crate::protobuf::ServiceComponentConstraints>,
        implicit_constraints: Option<crate::protobuf::ServiceComponentConstraints>,
        properties: Option<crate::protobuf::ServiceEntityProperties>,
        si_properties: Option<si_cea::protobuf::EntitySiProperties>,
        change_set_id: Option<String>,
    ) -> si_cea::CeaResult<crate::protobuf::ServiceEntity> {
        let mut result = crate::protobuf::ServiceEntity::new(
            name,
            display_name,
            description,
            constraints,
            implicit_constraints,
            properties,
            si_properties,
            change_set_id,
        )?;
        db.validate_and_insert_as_new(&mut result).await?;

        Ok(result)
    }

    pub async fn get(
        db: &si_data::Db,
        id: &str,
    ) -> si_data::Result<crate::protobuf::ServiceEntity> {
        let obj = db.get(id).await?;
        Ok(obj)
    }

    pub async fn get_by_natural_key(
        db: &si_data::Db,
        natural_key: &str,
    ) -> si_data::Result<crate::protobuf::ServiceEntity> {
        let obj = db.lookup_by_natural_key(natural_key).await?;
        Ok(obj)
    }

    pub async fn save(&self, db: &si_data::Db) -> si_data::Result<()> {
        db.upsert(self).await?;
        Ok(())
    }

    pub async fn finalize(&self, db: &si_data::Db) -> si_data::Result<()> {
        tracing::debug!("finalizing_service_entity");
        db.upsert(self).await?;

        Ok(())
    }

    pub async fn list(
        db: &si_data::Db,
        list_request: crate::protobuf::ServiceEntityListRequest,
    ) -> si_data::Result<si_data::ListResult<crate::protobuf::ServiceEntity>> {
        let result = match list_request.page_token {
            Some(token) => db.list_by_page_token(token).await?,
            None => {
                let page_size = match list_request.page_size {
                    Some(page_size) => page_size,
                    None => 10,
                };
                let order_by = match list_request.order_by {
                    Some(order_by) => order_by,
                    // The empty string is the signal for a default, thanks protobuf history
                    None => "".to_string(),
                };
                let contained_within = match list_request.scope_by_tenant_id {
                    Some(contained_within) => contained_within,
                    None => return Err(si_data::DataError::MissingScopeByTenantId),
                };
                db.list(
                    &list_request.query,
                    page_size,
                    order_by,
                    list_request.order_by_direction,
                    contained_within,
                    "",
                )
                .await?
            }
        };
        Ok(result)
    }

    pub async fn update(
        &mut self,
        db: &si_data::Db,
        change_set_id: Option<String>,
        update: Option<crate::protobuf::ServiceEntityUpdateRequestUpdate>,
    ) -> si_cea::CeaResult<()> {
        let item_id = self.id.as_ref().map(|change_set_enabled_id| {
            let split_result: Vec<&str> = change_set_enabled_id.split(":").collect();
            let item_id = if split_result.len() == 2 {
                format!("{}:{}", split_result[0], split_result[1])
            } else {
                format!("{}:{}", split_result[3], split_result[4])
            };
            item_id
        });
        self.si_storable.as_mut().map(|si_storable| {
            si_storable.change_set_id = change_set_id;
            si_storable.deleted = Some(false);
            si_storable.set_change_set_event_type(si_data::DataStorableChangeSetEventType::Update);
            si_storable.item_id = item_id;
            si_storable.change_set_executed = Some(false);
            si_storable
        });

        if let Some(update) = update {
            if update.name.is_some() {
                self.name = update.name;
            }
            if update.display_name.is_some() {
                self.display_name = update.display_name;
            }
            if update.description.is_some() {
                self.description = update.description;
            }
            if update.properties.is_some() {
                self.properties = update.properties;
            }
        }

        db.update(self).await?;
        Ok(())
    }

    pub async fn delete(
        &mut self,
        db: &si_data::Db,
        change_set_id: Option<String>,
    ) -> si_cea::CeaResult<()> {
        let item_id = self.id.as_ref().map(|change_set_enabled_id| {
            let split_result: Vec<&str> = change_set_enabled_id.split(":").collect();
            let item_id = if split_result.len() == 2 {
                format!("{}:{}", split_result[0], split_result[1])
            } else {
                format!("{}:{}", split_result[3], split_result[4])
            };
            item_id
        });
        self.si_storable.as_mut().map(|si_storable| {
            si_storable.change_set_id = change_set_id;
            si_storable.deleted = Some(true);
            si_storable.set_change_set_event_type(si_data::DataStorableChangeSetEventType::Delete);
            si_storable.item_id = item_id;
            si_storable.change_set_executed = Some(false);
            si_storable
        });

        db.delete(self).await?;
        Ok(())
    }
}

impl si_cea::Entity for crate::protobuf::ServiceEntity {
    type EntityProperties = crate::protobuf::ServiceEntityProperties;

    fn entity_state(&self) -> si_data::Result<si_cea::EntitySiPropertiesEntityState> {
        Ok(self
            .si_properties
            .as_ref()
            .ok_or_else(|| si_data::DataError::RequiredField("si_properties".to_string()))?
            .entity_state())
    }

    fn set_entity_state(&mut self, state: si_cea::EntitySiPropertiesEntityState) {
        if self.si_properties.is_none() {
            self.si_properties = Some(Default::default());
        }

        let si_properties = self.si_properties.as_mut().expect(
            "crate::protobuf::ServiceEntity.si_properties \
                has been set or initialized",
        );
        si_properties.set_entity_state(state);
    }

    fn properties(&self) -> si_data::Result<&Self::EntityProperties> {
        self.properties
            .as_ref()
            .ok_or_else(|| si_data::DataError::RequiredField("properties".to_string()))
    }

    fn properties_mut(&mut self) -> si_data::Result<&mut Self::EntityProperties> {
        self.properties
            .as_mut()
            .ok_or_else(|| si_data::DataError::RequiredField("properties".to_string()))
    }

    fn integration_id(&self) -> si_data::Result<&str> {
        self.si_properties
            .as_ref()
            .ok_or_else(|| si_data::DataError::RequiredField("si_properties".to_string()))?
            .integration_id
            .as_ref()
            .map(String::as_str)
            .ok_or_else(|| si_data::DataError::RequiredField("integration_id".to_string()))
    }

    fn set_integration_id(&mut self, integration_id: impl Into<String>) {
        if self.si_properties.is_none() {
            self.si_properties = Some(Default::default());
        }

        let si_properties = self.si_properties.as_mut().expect(
            "crate::protobuf::ServiceEntity.si_properties \
                has been set or initialized",
        );
        si_properties.integration_id = Some(integration_id.into());
    }

    fn integration_service_id(&self) -> si_data::Result<&str> {
        self.si_properties
            .as_ref()
            .ok_or_else(|| si_data::DataError::RequiredField("si_properties".to_string()))?
            .integration_service_id
            .as_ref()
            .map(String::as_str)
            .ok_or_else(|| si_data::DataError::RequiredField("integration_service_id".to_string()))
    }

    fn set_integration_service_id(&mut self, integration_service_id: impl Into<String>) {
        if self.si_properties.is_none() {
            self.si_properties = Some(Default::default());
        }

        let si_properties = self.si_properties.as_mut().expect(
            "crate::protobuf::ServiceEntity.si_properties \
                has been set or initialized",
        );
        si_properties.integration_service_id = Some(integration_service_id.into());
    }

    fn component_id(&self) -> si_data::Result<&str> {
        self.si_properties
            .as_ref()
            .ok_or_else(|| si_data::DataError::RequiredField("si_properties".to_string()))?
            .component_id
            .as_ref()
            .map(String::as_str)
            .ok_or_else(|| si_data::DataError::RequiredField("component_id".to_string()))
    }

    fn set_component_id(&mut self, component_id: impl Into<String>) {
        if self.si_properties.is_none() {
            self.si_properties = Some(Default::default());
        }

        let si_properties = self.si_properties.as_mut().expect(
            "crate::protobuf::ServiceEntity.si_properties \
                has been set or initialized",
        );
        si_properties.component_id = Some(component_id.into());
    }

    fn workspace_id(&self) -> si_data::Result<&str> {
        self.si_properties
            .as_ref()
            .ok_or_else(|| si_data::DataError::RequiredField("si_properties".to_string()))?
            .workspace_id
            .as_ref()
            .map(String::as_str)
            .ok_or_else(|| si_data::DataError::RequiredField("workspace_id".to_string()))
    }

    fn set_workspace_id(&mut self, workspace_id: impl Into<String>) {
        if self.si_properties.is_none() {
            self.si_properties = Some(Default::default());
        }

        let si_properties = self.si_properties.as_mut().expect(
            "crate::protobuf::ServiceEntity.si_properties \
                has been set or initialized",
        );
        si_properties.workspace_id = Some(workspace_id.into());
    }

    fn organization_id(&self) -> si_data::Result<&str> {
        self.si_properties
            .as_ref()
            .ok_or_else(|| si_data::DataError::RequiredField("si_properties".to_string()))?
            .organization_id
            .as_ref()
            .map(String::as_str)
            .ok_or_else(|| si_data::DataError::RequiredField("organization_id".to_string()))
    }

    fn set_organization_id(&mut self, organization_id: impl Into<String>) {
        if self.si_properties.is_none() {
            self.si_properties = Some(Default::default());
        }

        let si_properties = self.si_properties.as_mut().expect(
            "crate::protobuf::ServiceEntity.si_properties \
                has been set or initialized",
        );
        si_properties.organization_id = Some(organization_id.into());
    }

    fn billing_account_id(&self) -> si_data::Result<&str> {
        self.si_properties
            .as_ref()
            .ok_or_else(|| si_data::DataError::RequiredField("si_properties".to_string()))?
            .billing_account_id
            .as_ref()
            .map(String::as_str)
            .ok_or_else(|| si_data::DataError::RequiredField("billing_account_id".to_string()))
    }

    fn set_billing_account_id(&mut self, billing_account_id: impl Into<String>) {
        if self.si_properties.is_none() {
            self.si_properties = Some(Default::default());
        }

        let si_properties = self.si_properties.as_mut().expect(
            "crate::protobuf::ServiceEntity.si_properties \
                has been set or initialized",
        );
        si_properties.billing_account_id = Some(billing_account_id.into());
    }
}

impl si_agent::TypeHint for crate::protobuf::ServiceEntity {
    fn type_name() -> &'static str {
        "service_entity"
    }
}

impl si_data::Storable for crate::protobuf::ServiceEntity {
    fn type_name() -> &'static str {
        "service_entity"
    }

    fn set_type_name(&mut self) {
        if self.si_storable.is_none() {
            self.si_storable = Some(Default::default());
        }

        let si_storable = self.si_storable.as_mut().expect(
            "crate::protobuf::ServiceEntity.si_storable \
                has been set or initialized",
        );
        si_storable.type_name = Some(Self::type_name().to_string());
    }

    fn id(&self) -> si_data::Result<&str> {
        self.id
            .as_ref()
            .map(String::as_str)
            .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))
    }

    fn set_id(&mut self, id: impl Into<String>) {
        self.id = Some(id.into());
    }

    fn change_set_id(&self) -> si_data::Result<Option<&str>> {
        Ok(self
            .si_storable
            .as_ref()
            .ok_or_else(|| si_data::DataError::RequiredField("si_storable".to_string()))?
            .change_set_id
            .as_ref()
            .map(String::as_str))
    }

    fn set_change_set_entry_count(&mut self, entry_count: u64) -> si_data::Result<()> {
        self.si_storable
            .as_mut()
            .ok_or_else(|| si_data::DataError::RequiredField("si_storable".to_string()))?
            .change_set_entry_count
            .replace(entry_count);
        Ok(())
    }

    // How this should work:
    //
    //  * Do we have an ID?
    //      * Are we in a change set?
    //          * Update the order
    //          * Set the new ID
    //      * keep the current ID
    //  * We don't have an ID
    //      * Generate a new real object id
    //          * Set the item ID to it
    //      * Make the change-set id, and set that as the real one.
    //
    // This needs to possibly error now!
    fn generate_id(&mut self) {
        if let Ok(_current_id) = self.id() {
            if let Some(change_set_id) = self
                .si_storable
                .as_ref()
                .map(|si_storable| si_storable.change_set_id.as_ref())
                .flatten()
            {
                let real_id = self
                    .si_storable
                    .as_ref()
                    .map(|si_storable| si_storable.item_id.as_ref())
                    .flatten()
                    .expect("must have a real item_id");
                let change_set_entry_count = self
                    .si_storable
                    .as_ref()
                    .map(|si_storable| si_storable.change_set_entry_count.as_ref())
                    .flatten()
                    .expect("must have a change_set_entry_count");
                let new_id = format!("{}:{}:{}", change_set_id, change_set_entry_count, real_id);
                self.set_id(new_id);
            }
        } else {
            let real_id = format!("{}:{}", Self::type_name(), si_data::uuid_string(),);
            self.si_storable
                .as_mut()
                .map(|si_storable| si_storable.item_id = Some(real_id.clone()));
            if let Some(change_set_id) = self
                .si_storable
                .as_ref()
                .map(|si_storable| si_storable.change_set_id.as_ref())
                .flatten()
            {
                let change_set_entry_count = self
                    .si_storable
                    .as_ref()
                    .map(|si_storable| si_storable.change_set_entry_count.as_ref())
                    .flatten()
                    .expect("must have a change_set_entry_count");
                let new_id = format!("{}:{}:{}", change_set_id, change_set_entry_count, real_id);
                self.set_id(new_id);
            } else {
                self.set_id(real_id);
            }
        }
    }

    fn natural_key(&self) -> si_data::Result<Option<&str>> {
        Ok(None)
    }

    fn set_natural_key(&mut self) -> si_data::Result<()> {
        Ok(())
    }

    fn tenant_ids(&self) -> si_data::Result<&[String]> {
        Ok(self
            .si_storable
            .as_ref()
            .ok_or_else(|| si_data::DataError::RequiredField("si_storable".to_string()))?
            .tenant_ids
            .as_slice())
    }

    fn add_to_tenant_ids(&mut self, id: impl Into<String>) {
        if self.si_storable.is_none() {
            self.si_storable = Some(Default::default());
        }

        let si_storable = self.si_storable.as_mut().expect(
            "crate::protobuf::ServiceEntity.si_storable \
                has been set or initialized",
        );
        si_storable.tenant_ids.push(id.into());
    }

    fn validate(&self) -> si_data::error::Result<()> {
        if self.id.is_none() {
            return Err(si_data::DataError::ValidationError(
                "missing required id value".into(),
            ));
        }
        if self.name.is_none() {
            return Err(si_data::DataError::ValidationError(
                "missing required name value".into(),
            ));
        }
        if self.display_name.is_none() {
            return Err(si_data::DataError::ValidationError(
                "missing required display_name value".into(),
            ));
        }
        if self.si_storable.is_none() {
            return Err(si_data::DataError::ValidationError(
                "missing required si_storable value".into(),
            ));
        }
        if self.description.is_none() {
            return Err(si_data::DataError::ValidationError(
                "missing required description value".into(),
            ));
        }
        if self.si_properties.is_none() {
            return Err(si_data::DataError::ValidationError(
                "missing required si_properties value".into(),
            ));
        }
        if self.properties.is_none() {
            return Err(si_data::DataError::ValidationError(
                "missing required properties value".into(),
            ));
        }

        Ok(())
    }

    fn referential_fields(&self) -> Vec<si_data::Reference> {
        Vec::new()
    }

    fn order_by_fields() -> Vec<&'static str> {
        vec![
            "siStorable.naturalKey",
            "id",
            "name",
            "displayName",
            "siStorable.naturalKey",
            "dataStorable.viewContext",
            "dataStorable.changeSetId",
            "dataStorable.itemId",
            "dataStorable.changeSetEntryCount",
            "dataStorable.changeSetEventType",
            "dataStorable.changeSetExecuted",
            "dataStorable.deleted",
            "description",
            "siStorable.naturalKey",
            "entitySiProperties.entityState",
            "siStorable.naturalKey",
            "properties.image",
            "properties.port",
            "properties.replicas",
            "properties.deploymentTarget",
            "siStorable.naturalKey",
            "constraints.componentName",
            "constraints.componentDisplayName",
            "siStorable.naturalKey",
            "constraints.componentName",
            "constraints.componentDisplayName",
        ]
    }
}
