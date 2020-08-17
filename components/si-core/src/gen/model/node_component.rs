// Auth-generated code!
// No touchy!

impl crate::protobuf::NodeComponent {
    pub fn new(
        name: Option<String>,
        display_name: Option<String>,
        description: Option<String>,
        constraints: Option<crate::protobuf::NodeComponentConstraints>,
        si_properties: Option<si_cea::protobuf::ComponentSiProperties>,
    ) -> si_data::Result<crate::protobuf::NodeComponent> {
        let mut si_storable = si_data::protobuf::DataStorable::default();
        si_storable.add_to_tenant_ids("global");
        si_properties
            .as_ref()
            .ok_or_else(|| si_data::DataError::ValidationError("siProperties".into()))?;
        let integration_id = si_properties
            .as_ref()
            .unwrap()
            .integration_id
            .as_ref()
            .ok_or_else(|| {
                si_data::DataError::ValidationError("siProperties.integrationId".into())
            })?;
        si_storable.add_to_tenant_ids(integration_id);
        let integration_service_id = si_properties
            .as_ref()
            .unwrap()
            .integration_service_id
            .as_ref()
            .ok_or_else(|| {
                si_data::DataError::ValidationError("siProperties.integrationServiceId".into())
            })?;
        si_storable.add_to_tenant_ids(integration_service_id);

        let mut result: crate::protobuf::NodeComponent = Default::default();
        result.name = name;
        result.display_name = display_name;
        result.description = description;
        result.constraints = constraints;
        result.si_properties = si_properties;
        result.si_storable = Some(si_storable);

        Ok(result)
    }

    pub async fn create(
        db: &si_data::Db,
        name: Option<String>,
        display_name: Option<String>,
        description: Option<String>,
        constraints: Option<crate::protobuf::NodeComponentConstraints>,
        si_properties: Option<si_cea::protobuf::ComponentSiProperties>,
    ) -> si_data::Result<crate::protobuf::NodeComponent> {
        let mut result = crate::protobuf::NodeComponent::new(
            name,
            display_name,
            description,
            constraints,
            si_properties,
        )?;
        db.validate_and_insert_as_new(&mut result).await?;

        Ok(result)
    }

    pub async fn get(
        db: &si_data::Db,
        id: &str,
    ) -> si_data::Result<crate::protobuf::NodeComponent> {
        let obj = db.get(id).await?;
        Ok(obj)
    }

    pub async fn get_by_natural_key(
        db: &si_data::Db,
        natural_key: &str,
    ) -> si_data::Result<crate::protobuf::NodeComponent> {
        let obj = db.lookup_by_natural_key(natural_key).await?;
        Ok(obj)
    }

    pub async fn save(&self, db: &si_data::Db) -> si_data::Result<()> {
        db.upsert(self).await?;
        Ok(())
    }

    pub async fn finalize(&self, db: &si_data::Db) -> si_data::Result<()> {
        tracing::debug!("finalizing_node_component");
        db.upsert(self).await?;

        Ok(())
    }

    pub async fn list(
        db: &si_data::Db,
        list_request: crate::protobuf::NodeComponentListRequest,
    ) -> si_data::Result<si_data::ListResult<crate::protobuf::NodeComponent>> {
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

    pub async fn pick_by_expressions(
        db: &si_data::Db,
        items: Vec<si_data::DataQueryItems>,
        boolean_term: si_data::DataQueryBooleanTerm,
    ) -> si_data::Result<Self> {
        let query = si_data::DataQuery {
            items,
            boolean_term: boolean_term as i32,
            ..Default::default()
        };

        let mut check_result: si_data::ListResult<Self> =
            db.list(&Some(query), 1, "", 0, "global", "").await?;
        if check_result.len() == 1 {
            return Ok(check_result.items.pop().unwrap());
        } else {
            return Err(si_data::DataError::PickComponent(
                "a match was not found".to_string(),
            ));
        }
    }

    pub async fn pick_by_string_field<F, V>(
        db: &si_data::Db,
        field: F,
        value: V,
    ) -> si_data::Result<Option<Self>>
    where
        F: Into<String> + Send,
        V: Into<String> + Send,
    {
        let value = value.into();
        let field = field.into();

        if value != "" {
            let query = si_data::DataQuery::generate_for_string(
                field.clone(),
                si_data::DataQueryItemsExpressionComparison::Equals,
                value.clone(),
            );
            let mut check_result: si_data::ListResult<Self> =
                db.list(&Some(query), 1, "", 0, "global", "").await?;
            if check_result.len() == 1 {
                return Ok(Some(check_result.items.pop().unwrap()));
            } else {
                return Err(si_data::DataError::PickComponent(format!(
                    "{}={} must match exactly, and was not found",
                    field, value
                )));
            }
        }
        Ok(None)
    }

    pub async fn pick_by_component_name(
        db: &si_data::Db,
        req: &crate::protobuf::NodeComponentConstraints,
    ) -> si_data::Result<Option<(crate::protobuf::NodeComponentConstraints, Self)>> {
        match &req.component_name {
            Some(name) => match Self::pick_by_string_field(db, "name", name).await? {
                Some(component) => Ok(Some((
                    crate::protobuf::NodeComponentConstraints::default(),
                    component,
                ))),
                None => Ok(None),
            },
            None => Ok(None),
        }
    }

    pub async fn pick_by_component_display_name(
        db: &si_data::Db,
        req: &crate::protobuf::NodeComponentConstraints,
    ) -> si_data::Result<Option<(crate::protobuf::NodeComponentConstraints, Self)>> {
        match &req.component_display_name {
            Some(display_name) => {
                match Self::pick_by_string_field(db, "displayName", display_name).await? {
                    Some(component) => Ok(Some((
                        crate::protobuf::NodeComponentConstraints::default(),
                        component,
                    ))),
                    None => Ok(None),
                }
            }
            None => Ok(None),
        }
    }
}

impl si_data::Storable for crate::protobuf::NodeComponent {
    fn type_name() -> &'static str {
        "node_component"
    }

    fn set_type_name(&mut self) {
        if self.si_storable.is_none() {
            self.si_storable = Some(Default::default());
        }

        let si_storable = self.si_storable.as_mut().expect(
            "crate::protobuf::NodeComponent.si_storable \
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
            "crate::protobuf::NodeComponent.si_storable \
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
            "crate::protobuf::NodeComponent.si_storable \
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
        if self.constraints.is_none() {
            return Err(si_data::DataError::ValidationError(
                "missing required constraints value".into(),
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
        let integration_id = match &self.si_properties {
            Some(cip) => cip
                .integration_id
                .as_ref()
                .map(String::as_ref)
                .unwrap_or("No integration_id found for referential integrity check"),
            None => "No integration_id found for referential integrity check",
        };
        let integration_service_id = match &self.si_properties {
            Some(cip) => cip
                .integration_service_id
                .as_ref()
                .map(String::as_ref)
                .unwrap_or("No integration_service_id found for referential integrity check"),
            None => "No integration_service_id found for referential integrity check",
        };
        vec![
            si_data::Reference::HasOne("integration_id", integration_id),
            si_data::Reference::HasOne("integration_service_id", integration_service_id),
        ]
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
            "constraints.componentName",
            "constraints.componentDisplayName",
            "constraints.nodeKind",
            "siStorable.naturalKey",
        ]
    }
}

impl si_data::Migrateable for crate::protobuf::NodeComponent {
    fn get_version(&self) -> i32 {
        match self.si_properties.as_ref().map(|p| p.version) {
            Some(v) => v.unwrap_or(0),
            None => 0,
        }
    }
}

impl crate::protobuf::NodeComponentConstraintsNodeKind {
    pub fn iterator() -> impl Iterator<Item = Self> {
        [Self::Entity].iter().copied()
    }

    pub fn default_value() -> Self {
        Self::Entity
    }

    pub fn to_i32_string(&self) -> String {
        (*self as i32).to_string()
    }
}

impl std::fmt::Display for crate::protobuf::NodeComponentConstraintsNodeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unknown => f.write_str("<UNKNOWN>"),
            Self::Entity => f.write_str("entity"),
        }
    }
}

#[derive(thiserror::Error, Debug)]
#[error("invalid NodeComponentConstraintsNodeKind value: {0}")]
pub struct InvalidNodeComponentConstraintsNodeKind(String);

impl std::str::FromStr for crate::protobuf::NodeComponentConstraintsNodeKind {
    type Err = InvalidNodeComponentConstraintsNodeKind;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "entity" => Ok(Self::Entity),
            invalid => Err(InvalidNodeComponentConstraintsNodeKind(invalid.to_string())),
        }
    }
}
