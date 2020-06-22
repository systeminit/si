// Auth-generated code!
// No touchy!

impl crate::protobuf::Workspace {
    pub fn new(
        name: Option<String>,
        display_name: Option<String>,
        si_properties: Option<crate::protobuf::WorkspaceSiProperties>,
    ) -> si_data::Result<crate::protobuf::Workspace> {
        let mut si_storable = si_data::protobuf::DataStorable::default();
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

        let mut result: crate::protobuf::Workspace = Default::default();
        result.name = name;
        result.display_name = display_name;
        result.si_properties = si_properties;
        result.si_storable = Some(si_storable);

        Ok(result)
    }

    pub async fn create(
        db: &si_data::Db,
        name: Option<String>,
        display_name: Option<String>,
        si_properties: Option<crate::protobuf::WorkspaceSiProperties>,
    ) -> si_data::Result<crate::protobuf::Workspace> {
        let mut result = crate::protobuf::Workspace::new(name, display_name, si_properties)?;
        db.validate_and_insert_as_new(&mut result).await?;

        Ok(result)
    }

    pub async fn get(db: &si_data::Db, id: &str) -> si_data::Result<crate::protobuf::Workspace> {
        let obj = db.get(id).await?;
        Ok(obj)
    }

    pub async fn get_by_natural_key(
        db: &si_data::Db,
        natural_key: &str,
    ) -> si_data::Result<crate::protobuf::Workspace> {
        let obj = db.lookup_by_natural_key(natural_key).await?;
        Ok(obj)
    }

    pub async fn save(&self, db: &si_data::Db) -> si_data::Result<()> {
        db.upsert(self).await?;
        Ok(())
    }

    pub async fn list(
        db: &si_data::Db,
        list_request: crate::protobuf::WorkspaceListRequest,
    ) -> si_data::Result<si_data::ListResult<crate::protobuf::Workspace>> {
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
}

impl si_data::Storable for crate::protobuf::Workspace {
    fn type_name() -> &'static str {
        "workspace"
    }

    fn set_type_name(&mut self) {
        if self.si_storable.is_none() {
            self.si_storable = Some(Default::default());
        }

        let si_storable = self.si_storable.as_mut().expect(
            "crate::protobuf::Workspace.si_storable \
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

    fn generate_id(&mut self) {
        let mut id = format!("{}:{}", Self::type_name(), si_data::uuid_string(),);
        if self.si_storable.is_some() {
            if self.si_storable.as_ref().unwrap().change_set_id.is_some() {
                if self
                    .si_storable
                    .as_ref()
                    .unwrap()
                    .change_set_entry_count
                    .is_some()
                {
                    self.si_storable.as_mut().unwrap().item_id = Some(id.clone());
                    id = format!(
                        "{}:{}:{}",
                        self.si_storable
                            .as_ref()
                            .unwrap()
                            .change_set_id
                            .as_ref()
                            .unwrap(),
                        self.si_storable
                            .as_ref()
                            .unwrap()
                            .change_set_entry_count
                            .unwrap(),
                        id
                    );
                }
            }
        }
        self.set_id(id);
    }

    fn natural_key(&self) -> si_data::Result<Option<&str>> {
        Ok(self
            .si_storable
            .as_ref()
            .ok_or_else(|| si_data::DataError::RequiredField("si_storable".to_string()))?
            .natural_key
            .as_ref()
            .map(String::as_str))
    }

    fn set_natural_key(&mut self) -> si_data::Result<()> {
        let natural_key = format!(
            "{}:{}:{}",
            self.tenant_ids()?
                .first()
                .ok_or_else(|| si_data::DataError::MissingTenantIds)?,
            Self::type_name(),
            self.name
                .as_ref()
                .ok_or_else(|| si_data::DataError::RequiredField("name".to_string()))?,
        );

        if self.si_storable.is_none() {
            self.si_storable = Some(Default::default());
        }

        let si_storable = self.si_storable.as_mut().expect(
            "crate::protobuf::Workspace.si_storable \
                has been set or initialized",
        );
        si_storable.natural_key = Some(natural_key);

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
            "crate::protobuf::Workspace.si_storable \
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
        if self.si_properties.is_none() {
            return Err(si_data::DataError::ValidationError(
                "missing required si_properties value".into(),
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
            "siStorable.naturalKey",
            "siProperties.billingAccountId",
            "siProperties.organizationId",
        ]
    }
}
