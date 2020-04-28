// Auth-generated code!
// No touchy!

use si_data;
use uuid;

impl si_data::Storable for crate::protobuf::IntegrationService {
    /// # Panics
    ///
    /// * When a system object's `id` is not set (`crate::protobuf::IntegrationService::generate_id()` must be called first)
    fn get_id(&self) -> &str {
        (self.id.as_ref())
            .expect("crate::protobuf::IntegrationService::generate_id() must be called before crate::protobuf::IntegrationService::get_id")
    }

    fn set_id(&mut self, id: impl Into<String>) {
        self.id = Some(id.into());
    }

    fn type_name() -> &'static str {
        "integration_service"
    }

    fn set_type_name(&mut self) {
        if let None = self.si_storable {
            self.si_storable = Some(Default::default());
        }

        let storable = self.si_storable.as_mut().unwrap();
        storable.type_name = Some(<Self as si_data::Storable>::type_name().to_string());
    }

    fn generate_id(&mut self) {
        self.set_id(format!(
            "{}:{}",
            <Self as si_data::Storable>::type_name(),
            uuid::Uuid::new_v4(),
        ));
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

    fn get_tenant_ids(&self) -> &[String] {
        match &self.si_storable {
            Some(storable) => &storable.tenant_ids,
            None => &[],
        }
    }

    fn add_to_tenant_ids(&mut self, id: impl Into<String>) {
        if let None = self.si_storable {
            self.si_storable = Some(Default::default());
        }

        let storable = self.si_storable.as_mut().unwrap();
        storable.tenant_ids.push(id.into());
    }

    fn referential_fields(&self) -> Vec<si_data::Reference> {
        Vec::new()
    }

    fn get_natural_key(&self) -> Option<&str> {
        self.si_storable
            .as_ref()
            .and_then(|s| s.natural_key.as_ref().map(String::as_ref))
    }

    /// # Panics
    ///
    /// This method will panic if any required information is missing to generate a natural key:
    ///
    /// * When `tenant_ids` are not set
    /// * When `name` is not set
    fn set_natural_key(&mut self) {
        if let None = self.si_storable {
            self.si_storable = Some(Default::default());
        }
        let natural_key = format!(
            "{}:{}:{}",
            self.get_tenant_ids().first().expect(
                "crate::protobuf::IntegrationService's tenant_ids must be set with crate::protobuf::IntegrationService.set_natural_key() is called"
            ),
            <Self as si_data::Storable>::type_name(),
            self.name
                .as_ref()
                .expect("crate::protobuf::IntegrationService.name must be set when crate::protobuf::IntegrationService.set_natural_key() is called")
        );

        let mut storable = self.si_storable.as_mut().unwrap();
        storable.natural_key = Some(natural_key);
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
        match self.version {
            Some(v) => v,
            None => 0,
        }
    }
}

impl crate::protobuf::IntegrationService {
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
                    None => "".to_string(), // The empty string is the signal for a default, thanks protobuf history
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
}
