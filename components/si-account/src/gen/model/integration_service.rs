// Auth-generated code!
// No touchy!

impl crate::protobuf::IntegrationService {
    pub fn new(
        name: Option<String>,
        display_name: Option<String>,
        version: Option<i32>,
        si_properties: Option<crate::protobuf::IntegrationServiceSiProperties>,
    ) -> si_data::Result<crate::protobuf::IntegrationService> {
        let mut si_storable = si_data::protobuf::DataStorable::default();
        si_storable.add_to_tenant_ids("global");
        si_properties
            .as_ref()
            .ok_or(si_data::DataError::ValidationError("siProperties".into()))?;
        let integration_id = si_properties
            .as_ref()
            .unwrap()
            .integration_id
            .as_ref()
            .ok_or(si_data::DataError::ValidationError(
                "siProperties.integrationId".into(),
            ))?;
        si_storable.add_to_tenant_ids(integration_id);

        let mut result_obj = crate::protobuf::IntegrationService {
            ..Default::default()
        };
        result_obj.name = name;
        result_obj.display_name = display_name;
        result_obj.version = version;
        result_obj.si_properties = si_properties;
        result_obj.si_storable = Some(si_storable);

        Ok(result_obj)
    }

    pub async fn create(
        db: &si_data::Db,
        name: Option<String>,
        display_name: Option<String>,
        version: Option<i32>,
        si_properties: Option<crate::protobuf::IntegrationServiceSiProperties>,
    ) -> si_data::Result<crate::protobuf::IntegrationService> {
        let mut result_obj =
            crate::protobuf::IntegrationService::new(name, display_name, version, si_properties)?;
        db.validate_and_insert_as_new(&mut result_obj).await?;
        Ok(result_obj)
    }

    pub async fn get(
        db: &si_data::Db,
        id: &str,
    ) -> si_data::Result<crate::protobuf::IntegrationService> {
        let obj = db.get(id).await?;
        Ok(obj)
    }

    pub async fn get_by_natural_key(
        db: &si_data::Db,
        natural_key: &str,
    ) -> si_data::Result<crate::protobuf::IntegrationService> {
        let obj = db.lookup_by_natural_key(natural_key).await?;
        Ok(obj)
    }

    pub async fn save(&self, db: &si_data::Db) -> si_data::Result<()> {
        db.upsert(self).await?;
        Ok(())
    }

    pub async fn list(
        db: &si_data::Db,
        list_request: crate::protobuf::IntegrationServiceListRequest,
    ) -> si_data::Result<si_data::ListResult<crate::protobuf::IntegrationService>> {
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

impl si_data::Storable for crate::protobuf::IntegrationService {
    fn type_name() -> &'static str {
        "integration_service"
    }

    fn set_type_name(&mut self) {
        if self.si_storable.is_none() {
            self.si_storable = Some(Default::default());
        }

        let si_storable = self.si_storable.as_mut().expect(
            "crate::protobuf::IntegrationService.si_storable \
                has been set or initialized",
        );
        si_storable.type_name = Some(Self::type_name().to_string());
    }

    fn id(&self) -> si_data::Result<&str> {
        self.id
            .as_ref()
            .map(String::as_str)
            .ok_or(si_data::DataError::RequiredField("id".to_string()))
    }

    fn set_id(&mut self, id: impl Into<String>) {
        self.id = Some(id.into());
    }

    fn generate_id(&mut self) {
        self.set_id(format!("{}:{}", Self::type_name(), si_data::uuid_string(),));
    }

    fn natural_key(&self) -> si_data::Result<Option<&str>> {
        Ok(self
            .si_storable
            .as_ref()
            .ok_or(si_data::DataError::RequiredField("si_storable".to_string()))?
            .natural_key
            .as_ref()
            .map(String::as_str))
    }

    fn set_natural_key(&mut self) -> si_data::Result<()> {
        let natural_key = format!(
            "{}:{}:{}",
            self.tenant_ids()?
                .first()
                .ok_or(si_data::DataError::MissingTenantIds)?,
            Self::type_name(),
            self.name
                .as_ref()
                .ok_or(si_data::DataError::RequiredField("name".to_string()))?,
        );

        if self.si_storable.is_none() {
            self.si_storable = Some(Default::default());
        }

        let si_storable = self.si_storable.as_mut().expect(
            "crate::protobuf::IntegrationService.si_storable \
                has been set or initialized",
        );
        si_storable.natural_key = Some(natural_key);

        Ok(())
    }

    fn tenant_ids(&self) -> si_data::Result<&[String]> {
        Ok(self
            .si_storable
            .as_ref()
            .ok_or(si_data::DataError::RequiredField("si_storable".to_string()))?
            .tenant_ids
            .as_slice())
    }

    fn add_to_tenant_ids(&mut self, id: impl Into<String>) {
        if self.si_storable.is_none() {
            self.si_storable = Some(Default::default());
        }

        let si_storable = self.si_storable.as_mut().expect(
            "crate::protobuf::IntegrationService.si_storable \
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
        if self.version.is_none() {
            return Err(si_data::DataError::ValidationError(
                "missing required version value".into(),
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
            "siProperties.integrationId",
        ]
    }
}

impl si_data::Migrateable for crate::protobuf::IntegrationService {
    fn get_version(&self) -> i32 {
        match self.si_properties.as_ref().map(|p| p.version) {
            Some(v) => v.unwrap_or(0),
            None => 0,
        }
    }
}
