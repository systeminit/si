// Auth-generated code!
// No touchy!

use si_data;
use uuid;

impl si_data::Storable for crate::protobuf::KubernetesDeploymentEntity {
    /// # Panics
    ///
    /// * When a system object's `id` is not set (`crate::protobuf::KubernetesDeploymentEntity::generate_id()` must be called first)
    fn get_id(&self) -> &str {
        (self.id.as_ref())
            .expect("crate::protobuf::KubernetesDeploymentEntity::generate_id() must be called before crate::protobuf::KubernetesDeploymentEntity::get_id")
    }

    fn set_id(&mut self, id: impl Into<String>) {
        self.id = Some(id.into());
    }

    fn type_name() -> &'static str {
        "kubernetes_deployment_entity"
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
        if self.description.is_none() {
            return Err(si_data::DataError::ValidationError(
                "missing required description value".into(),
            ));
        }
        if self.display_type_name.is_none() {
            return Err(si_data::DataError::ValidationError(
                "missing required display_type_name value".into(),
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
                "crate::protobuf::KubernetesDeploymentEntity's tenant_ids must be set with crate::protobuf::KubernetesDeploymentEntity.set_natural_key() is called"
            ),
            <Self as si_data::Storable>::type_name(),
            self.name
                .as_ref()
                .expect("crate::protobuf::KubernetesDeploymentEntity.name must be set when crate::protobuf::KubernetesDeploymentEntity.set_natural_key() is called")
        );

        let mut storable = self.si_storable.as_mut().unwrap();
        storable.natural_key = Some(natural_key);
    }

    fn order_by_fields() -> Vec<&'static str> {
        vec!["id", "name", "displayName", "description", "displayTypeName", "entitySiProperties.entityState", "properties.kubernetesObject.apiVersion", "properties.kubernetesObject.kind", "properties.kubernetesObject.kubernetesMetadata.name", "properties.kubernetesObject.kubernetesMetadata.labels", "properties.kubernetesObject.spec.replicas", "properties.kubernetesObject.spec.kubernetesSelector.matchLabels", "properties.kubernetesObject.spec.kubernetesPodTemplateSpec.kubernetesMetadata.name", "properties.kubernetesObject.spec.kubernetesPodTemplateSpec.kubernetesMetadata.labels", "properties.kubernetesObject.spec.kubernetesPodTemplateSpec.kubernetesPodSpec.kubernetesContainer.name", "properties.kubernetesObject.spec.kubernetesPodTemplateSpec.kubernetesPodSpec.kubernetesContainer.image", "properties.kubernetesObject.spec.kubernetesPodTemplateSpec.kubernetesPodSpec.kubernetesContainer.ports.portValues.containerPort", "properties.kubernetesObjectYaml", "constraints.componentName", "constraints.componentDisplayName", "constraints.kubernetesVersion", "constraints.componentName", "constraints.componentDisplayName", "constraints.kubernetesVersion"]
    }
}

impl crate::protobuf::KubernetesDeploymentEntity {
    pub fn new(
        constraints: crate::protobuf::KubernetesDeploymentComponentConstraints,
        properties: crate::protobuf::KubernetesDeploymentEntityProperties,
        name: Option<String>,
        display_name: Option<String>,
        description: Option<String>,
        workspace_id: Option<String>,
    ) -> si_data::Result<crate::protobuf::KubernetesDeploymentEntity> {
        let mut result_obj = crate::protobuf::KubernetesDeploymentEntity {
            ..Default::default()
        };
        result_obj.constraints = constraints;
        result_obj.properties = properties;
        result_obj.name = name;
        result_obj.display_name = display_name;
        result_obj.description = description;
        result_obj.workspace_id = workspace_id;

        let mut si_storable = si_data::protobuf::DataStorable::default();
        si_storable.tenant_ids = vec![];
        result_obj.si_storable = Some(si_storable);

        Ok(result_obj)
    }

    pub async fn create(
        db: &si_data::Db,
        constraints: crate::protobuf::KubernetesDeploymentComponentConstraints,
        properties: crate::protobuf::KubernetesDeploymentEntityProperties,
        name: Option<String>,
        display_name: Option<String>,
        description: Option<String>,
        workspace_id: Option<String>,
    ) -> si_data::Result<crate::protobuf::KubernetesDeploymentEntity> {
        let mut result_obj = crate::protobuf::KubernetesDeploymentEntity::new(
            constraints,
            properties,
            name,
            display_name,
            description,
            workspace_id,
        )?;
        db.validate_and_insert_as_new(&mut result_obj).await?;
        Ok(result_obj)
    }
}
