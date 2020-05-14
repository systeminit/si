// Auth-generated code!
// No touchy!

impl crate::protobuf::KubernetesDeploymentEntity {
    pub fn new(
        name: Option<String>,
        display_name: Option<String>,
        description: Option<String>,
        constraints: Option<crate::protobuf::KubernetesDeploymentComponentConstraints>,
        implicit_constraints: Option<crate::protobuf::KubernetesDeploymentComponentConstraints>,
        properties: Option<crate::protobuf::KubernetesDeploymentEntityProperties>,
        si_properties: Option<si_cea::protobuf::EntitySiProperties>,
    ) -> si_data::Result<crate::protobuf::KubernetesDeploymentEntity> {
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
        let workspace_id = si_properties
            .as_ref()
            .unwrap()
            .workspace_id
            .as_ref()
            .ok_or_else(|| {
                si_data::DataError::ValidationError("siProperties.workspaceId".into())
            })?;
        si_storable.add_to_tenant_ids(workspace_id);

        let mut result: crate::protobuf::KubernetesDeploymentEntity = Default::default();
        result.name = name;
        result.display_name = display_name;
        result.description = description;
        result.constraints = constraints;
        result.implicit_constraints = implicit_constraints;
        result.properties = properties;
        result.si_properties = si_properties;
        result.si_storable = Some(si_storable);

        // TODO: fix
        // implement this!

        Ok(result)
    }

    pub async fn create(
        db: &si_data::Db,
        name: Option<String>,
        display_name: Option<String>,
        description: Option<String>,
        constraints: Option<crate::protobuf::KubernetesDeploymentComponentConstraints>,
        implicit_constraints: Option<crate::protobuf::KubernetesDeploymentComponentConstraints>,
        properties: Option<crate::protobuf::KubernetesDeploymentEntityProperties>,
        si_properties: Option<si_cea::protobuf::EntitySiProperties>,
    ) -> si_data::Result<crate::protobuf::KubernetesDeploymentEntity> {
        let mut result = crate::protobuf::KubernetesDeploymentEntity::new(
            name,
            display_name,
            description,
            constraints,
            implicit_constraints,
            properties,
            si_properties,
        )?;
        db.validate_and_insert_as_new(&mut result).await?;

        Ok(result)
    }

    pub async fn get(
        db: &si_data::Db,
        id: &str,
    ) -> si_data::Result<crate::protobuf::KubernetesDeploymentEntity> {
        let obj = db.get(id).await?;
        Ok(obj)
    }

    pub async fn get_by_natural_key(
        db: &si_data::Db,
        natural_key: &str,
    ) -> si_data::Result<crate::protobuf::KubernetesDeploymentEntity> {
        let obj = db.lookup_by_natural_key(natural_key).await?;
        Ok(obj)
    }

    pub async fn save(&self, db: &si_data::Db) -> si_data::Result<()> {
        db.upsert(self).await?;
        Ok(())
    }

    pub async fn list(
        db: &si_data::Db,
        list_request: crate::protobuf::KubernetesDeploymentEntityListRequest,
    ) -> si_data::Result<si_data::ListResult<crate::protobuf::KubernetesDeploymentEntity>> {
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

    pub fn edit_kubernetes_object(
        &mut self,
        property: crate::protobuf::KubernetesDeploymentEntityPropertiesKubernetesObject,
    ) -> si_cea::CeaResult<()> {
        use si_cea::Entity;

        self.properties_mut()?.kubernetes_object = Some(property);
        self.update_kubernetes_object_yaml_from_kubernetes_object()?;

        Ok(())
    }

    pub fn edit_kubernetes_object_yaml(&mut self, property: String) -> si_cea::CeaResult<()> {
        use si_cea::Entity;

        self.properties_mut()?.kubernetes_object_yaml = Some(property);
        self.update_kubernetes_object_from_kubernetes_object_yaml()?;

        Ok(())
    }

    fn update_kubernetes_object_yaml_from_kubernetes_object(&mut self) -> si_cea::CeaResult<()> {
        use si_cea::Entity;
        use std::convert::TryInto;

        if let Some(ref kubernetes_object) = self.properties()?.kubernetes_object {
            self.properties_mut()?.kubernetes_object_yaml = Some(kubernetes_object.try_into()?);
        }

        Ok(())
    }

    fn update_kubernetes_object_from_kubernetes_object_yaml(&mut self) -> si_cea::CeaResult<()> {
        use si_cea::Entity;
        use std::convert::TryInto;

        if let Some(ref kubernetes_object_yaml) = self.properties()?.kubernetes_object_yaml {
            self.properties_mut()?.kubernetes_object = Some(kubernetes_object_yaml.try_into()?);
        }

        Ok(())
    }
}

impl si_cea::Entity for crate::protobuf::KubernetesDeploymentEntity {
    type EntityProperties = crate::protobuf::KubernetesDeploymentEntityProperties;

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
            "crate::protobuf::KubernetesDeploymentEntity.si_properties \
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
            "crate::protobuf::KubernetesDeploymentEntity.si_properties \
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
            "crate::protobuf::KubernetesDeploymentEntity.si_properties \
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
            "crate::protobuf::KubernetesDeploymentEntity.si_properties \
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
            "crate::protobuf::KubernetesDeploymentEntity.si_properties \
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
            "crate::protobuf::KubernetesDeploymentEntity.si_properties \
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
            "crate::protobuf::KubernetesDeploymentEntity.si_properties \
                has been set or initialized",
        );
        si_properties.billing_account_id = Some(billing_account_id.into());
    }
}

impl si_data::Storable for crate::protobuf::KubernetesDeploymentEntity {
    fn type_name() -> &'static str {
        "kubernetes_deployment_entity"
    }

    fn set_type_name(&mut self) {
        if self.si_storable.is_none() {
            self.si_storable = Some(Default::default());
        }

        let si_storable = self.si_storable.as_mut().expect(
            "crate::protobuf::KubernetesDeploymentEntity.si_storable \
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

    fn generate_id(&mut self) {
        self.set_id(format!("{}:{}", Self::type_name(), si_data::uuid_string(),));
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
            "crate::protobuf::KubernetesDeploymentEntity.si_storable \
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
            "crate::protobuf::KubernetesDeploymentEntity.si_storable \
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
        vec!["siStorable.naturalKey", "id", "name", "displayName", "description", "siStorable.naturalKey", "entitySiProperties.entityState", "siStorable.naturalKey", "siStorable.naturalKey", "properties.kubernetesObject.apiVersion", "properties.kubernetesObject.kind", "siStorable.naturalKey", "properties.kubernetesObject.kubernetesMetadata.name", "properties.kubernetesObject.kubernetesMetadata.labels", "siStorable.naturalKey", "properties.kubernetesObject.spec.replicas", "siStorable.naturalKey", "properties.kubernetesObject.spec.kubernetesSelector.matchLabels", "siStorable.naturalKey", "siStorable.naturalKey", "properties.kubernetesObject.spec.kubernetesPodTemplateSpec.kubernetesMetadata.name", "properties.kubernetesObject.spec.kubernetesPodTemplateSpec.kubernetesMetadata.labels", "siStorable.naturalKey", "siStorable.naturalKey", "properties.kubernetesObject.spec.kubernetesPodTemplateSpec.kubernetesPodSpec.kubernetesContainer.name", "properties.kubernetesObject.spec.kubernetesPodTemplateSpec.kubernetesPodSpec.kubernetesContainer.image", "siStorable.naturalKey", "siStorable.naturalKey", "properties.kubernetesObject.spec.kubernetesPodTemplateSpec.kubernetesPodSpec.kubernetesContainer.ports.portValues.containerPort", "properties.kubernetesObjectYaml", "siStorable.naturalKey", "constraints.componentName", "constraints.componentDisplayName", "constraints.kubernetesVersion", "siStorable.naturalKey", "constraints.componentName", "constraints.componentDisplayName", "constraints.kubernetesVersion"]
    }
}

impl std::convert::TryFrom<&crate::protobuf::KubernetesDeploymentEntityPropertiesKubernetesObject>
    for String
{
    type Error = si_cea::CeaError;

    fn try_from(
        value: &crate::protobuf::KubernetesDeploymentEntityPropertiesKubernetesObject,
    ) -> std::result::Result<Self, Self::Error> {
        Ok(serde_yaml::to_string(value)?)
    }
}

impl std::convert::TryFrom<&String>
    for crate::protobuf::KubernetesDeploymentEntityPropertiesKubernetesObject
{
    type Error = si_cea::CeaError;

    fn try_from(value: &String) -> std::result::Result<Self, Self::Error> {
        Ok(serde_yaml::from_str(value)?)
    }
}
