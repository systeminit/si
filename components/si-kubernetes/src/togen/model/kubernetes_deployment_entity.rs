use crate::model::KubernetesDeploymentComponent;
use crate::protobuf::{
    KubernetesDeploymentComponentConstraints, KubernetesDeploymentEntityListEntitiesReply,
    KubernetesDeploymentEntityListEntitiesRequest, KubernetesDeploymentEntityProperties,
    KubernetesDeploymentEntityPropertiesKubernetesObject,
};
use si_cea::entity::codegen_prelude::*;
use std::convert::TryInto;

pub use crate::protobuf::KubernetesDeploymentEntity;

type Component = KubernetesDeploymentComponent;
type ComponentConstraints = KubernetesDeploymentComponentConstraints;
type EntityProperties = KubernetesDeploymentEntityProperties;

impl KubernetesDeploymentEntity {
    pub async fn create(
        db: &Db,
        name: String,
        display_name: String,
        description: String,
        display_type_name: String,
        properties: EntityProperties,
        constraints: ComponentConstraints,
        component: Component,
        implicit_constraints: ComponentConstraints,
        workspace: Workspace,
    ) -> CeaResult<Self> {
        let mut si_storable = DataStorable::default();
        si_storable.tenant_ids = vec![
            workspace.billing_account_id.clone(),
            workspace.organization_id.clone(),
            workspace.id.clone(),
        ];

        let si_properties = EntitySiProperties::new(
            &workspace,
            component.si_properties.as_ref().expect("TODO: fix"),
            component.id.expect("TODO: fix"),
        );

        let mut entity = Self {
            id: None,
            name: Some(name),
            display_name: Some(display_name),
            description: Some(description),
            display_type_name: Some(display_type_name),
            si_storable: Some(si_storable),
            si_properties: Some(si_properties),
            constraints: Some(constraints),
            implicit_constraints: Some(implicit_constraints),
            properties: Some(properties),
        };

        match (
            entity.properties()?.kubernetes_object.as_ref(),
            entity.properties()?.kubernetes_object_yaml.as_ref(),
        ) {
            (Some(_), None) => entity.update_kubernetes_object_yaml_from_kubernetes_object()?,
            (None, Some(_)) => entity.update_kubernetes_object_from_kubernetes_object_yaml()?,
            (Some(_), Some(_)) => panic!("TODO: both object & yaml are present"),
            (None, None) => panic!("TODO: neither object nor yaml are present"),
        }

        db.validate_and_insert_as_new(&mut entity).await?;

        Ok(entity)
    }

    pub fn edit_kubernetes_object(
        &mut self,
        property: KubernetesDeploymentEntityPropertiesKubernetesObject,
    ) -> CeaResult<()> {
        self.properties_mut()?.kubernetes_object = Some(property);
        self.update_kubernetes_object_yaml_from_kubernetes_object()?;

        Ok(())
    }

    pub fn edit_kubernetes_object_yaml(&mut self, property: String) -> CeaResult<()> {
        self.properties_mut()?.kubernetes_object_yaml = Some(property);
        self.update_kubernetes_object_from_kubernetes_object_yaml()?;

        Ok(())
    }

    fn properties(&self) -> CeaResult<&EntityProperties> {
        self.properties
            .as_ref()
            .ok_or(CeaError::MissingEntityProperties)
    }

    fn properties_mut(&mut self) -> CeaResult<&mut EntityProperties> {
        self.properties
            .as_mut()
            .ok_or(CeaError::MissingEntityProperties)
    }

    fn update_kubernetes_object_yaml_from_kubernetes_object(&mut self) -> CeaResult<()> {
        if let Some(ref kubernetes_object) = self.properties()?.kubernetes_object {
            self.properties_mut()?.kubernetes_object_yaml = Some(kubernetes_object.try_into()?);
        }
        Ok(())
    }

    fn update_kubernetes_object_from_kubernetes_object_yaml(&mut self) -> CeaResult<()> {
        if let Some(ref kubernetes_object_yaml) = self.properties()?.kubernetes_object_yaml {
            self.properties_mut()?.kubernetes_object = Some(kubernetes_object_yaml.try_into()?);
        }
        Ok(())
    }
}

impl Entity for KubernetesDeploymentEntity {
    fn id(&self) -> &str {
        self.id.as_ref().expect("TODO: fix")
    }

    fn set_id(&mut self, id: impl Into<String>) {
        self.id = Some(id.into());
    }

    fn state(&self) -> i32 {
        self.si_properties.as_ref().expect("TODO: fix").entity_state
    }

    fn set_state(&mut self, state: i32) {
        self.si_properties.as_mut().expect("TODO: fix").entity_state = state;
    }

    fn component_id(&self) -> &str {
        self.si_properties
            .as_ref()
            .expect("TODO: fix")
            .component_id
            .as_ref()
            .expect("TODO: fix")
    }

    fn set_component_id(&mut self, component_id: impl Into<String>) {
        self.si_properties.as_mut().expect("TODO: fix").component_id = Some(component_id.into());
    }

    fn integration_id(&self) -> &str {
        self.si_properties
            .as_ref()
            .expect("TODO: fix")
            .integration_id
            .as_ref()
            .expect("TODO: fix")
    }

    fn set_integration_id(&mut self, integration_id: impl Into<String>) {
        self.si_properties
            .as_mut()
            .expect("TODO: fix")
            .integration_id = Some(integration_id.into());
    }

    fn integration_service_id(&self) -> &str {
        self.si_properties
            .as_ref()
            .expect("TODO: fix")
            .integration_service_id
            .as_ref()
            .expect("TODO: fix")
    }

    fn set_integration_service_id(&mut self, integration_service_id: impl Into<String>) {
        self.si_properties
            .as_mut()
            .expect("TODO: fix")
            .integration_service_id = Some(integration_service_id.into());
    }

    fn workspace_id(&self) -> &str {
        self.si_properties
            .as_ref()
            .expect("TODO: fix")
            .workspace_id
            .as_ref()
            .expect("TODO: fix")
    }

    fn set_workspace_id(&mut self, workspace_id: impl Into<String>) {
        self.si_properties.as_mut().expect("TODO: fix").workspace_id = Some(workspace_id.into());
    }

    fn organization_id(&self) -> &str {
        self.si_properties
            .as_ref()
            .expect("TODO: fix")
            .organization_id
            .as_ref()
            .expect("TODO: fix")
    }

    fn set_organization_id(&mut self, organization_id: impl Into<String>) {
        self.si_properties
            .as_mut()
            .expect("TODO: fix")
            .organization_id = Some(organization_id.into());
    }

    fn billing_account_id(&self) -> &str {
        self.si_properties
            .as_ref()
            .expect("TODO: fix")
            .billing_account_id
            .as_ref()
            .expect("TODO: fix")
    }

    fn set_billing_account_id(&mut self, billing_account_id: impl Into<String>) {
        self.si_properties
            .as_mut()
            .expect("TODO: fix")
            .billing_account_id = Some(billing_account_id.into());
    }

    fn validate(&self) -> DataResult<()> {
        if self.id.is_none() {
            return Err(DataError::ValidationError(
                "missing required id value".into(),
            ));
        }
        if self.name.is_none() {
            return Err(DataError::ValidationError(
                "missing required name value".into(),
            ));
        }
        if self.display_name.is_none() {
            return Err(DataError::ValidationError(
                "missing required display_name value".into(),
            ));
        }
        if self.si_storable.is_none() {
            return Err(DataError::ValidationError(
                "missing required si_storable value".into(),
            ));
        }
        if self.si_properties.is_none() {
            return Err(DataError::ValidationError(
                "missing required si_properties value".into(),
            ));
        }
        Ok(())
    }
}

impl Storable for KubernetesDeploymentEntity {
    /// # Panics
    ///
    /// * When a component's `id` is not set (`KubernetesDeploymentEntity::generate_id()` must be called first)
    fn get_id(&self) -> &str {
        (self.id.as_ref()).expect("KubernetesDeploymentEntity::generate_id() must be called before KubernetesDeploymentEntity::get_id")
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
        storable.type_name = Some(<Self as Storable>::type_name().to_string());
    }

    fn generate_id(&mut self) {
        <Self as Storable>::set_id(
            self,
            format!("{}:{}", <Self as Storable>::type_name(), uuid_string(),),
        );
    }

    fn validate(&self) -> DataResult<()> {
        match <Self as Entity>::validate(&self) {
            Ok(()) => Ok(()),
            Err(e) => Err(DataError::ValidationError(e.to_string())),
        }
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

    fn referential_fields(&self) -> Vec<Reference> {
        const NO_COMPONENT_ID: &str = "<NO_COMPONENT_ID_HERE_GERALDINE>";
        const NO_INTEGRATION_ID: &str = "<NO_INTEGRATION_ID_HERE_GERALDINE>";
        const NO_INTEGRATION_SERVICE_ID: &str = "<NO_INTEGRATION_SERVICE_ID_HERE_GERALDINE>";
        const NO_WORKSPACE_ID: &str = "<NO_WORKSPACE_ID_HERE_GERALDINE>";
        const NO_ORGANIZATION_ID: &str = "<NO_ORGANIZATION_ID_HERE_GERALDINE>";
        const NO_BILLING_ACCOUNT_ID: &str = "<NO_BILLING_ACCOUNT_ID_HERE_GERALDINE>";

        let component_id = match &self.si_properties {
            Some(cip) => cip
                .component_id
                .as_ref()
                .map(String::as_ref)
                .unwrap_or(NO_COMPONENT_ID),
            None => NO_COMPONENT_ID,
        };
        let integration_id = match &self.si_properties {
            Some(cip) => cip
                .integration_id
                .as_ref()
                .map(String::as_ref)
                .unwrap_or(NO_INTEGRATION_ID),
            None => NO_INTEGRATION_ID,
        };
        let integration_service_id = match &self.si_properties {
            Some(cip) => cip
                .integration_service_id
                .as_ref()
                .map(String::as_ref)
                .unwrap_or(NO_INTEGRATION_SERVICE_ID),
            None => NO_INTEGRATION_SERVICE_ID,
        };
        let workspace_id = match &self.si_properties {
            Some(cip) => cip
                .workspace_id
                .as_ref()
                .map(String::as_ref)
                .unwrap_or(NO_WORKSPACE_ID),
            None => NO_WORKSPACE_ID,
        };
        let organization_id = match &self.si_properties {
            Some(cip) => cip
                .organization_id
                .as_ref()
                .map(String::as_ref)
                .unwrap_or(NO_ORGANIZATION_ID),
            None => NO_ORGANIZATION_ID,
        };
        let billing_account_id = match &self.si_properties {
            Some(cip) => cip
                .billing_account_id
                .as_ref()
                .map(String::as_ref)
                .unwrap_or(NO_BILLING_ACCOUNT_ID),
            None => NO_BILLING_ACCOUNT_ID,
        };

        vec![
            Reference::HasOne("component_id", component_id),
            Reference::HasOne("integration_id", integration_id),
            Reference::HasOne("integration_service_id", integration_service_id),
            Reference::HasOne("workspace_id", workspace_id),
            Reference::HasOne("organization_id", organization_id),
            Reference::HasOne("billing_account_id", billing_account_id),
        ]
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
    /// * When a component's `tenant_ids` are not set
    /// * When a component's `name` is not set
    fn set_natural_key(&mut self) {
        if let None = self.si_storable {
            self.si_storable = Some(Default::default());
        }
        let mut tenant_ids = self.get_tenant_ids().iter();
        let natural_key = format!(
            "{}:{}:{}:{}:{}",
            // This is safe *only* after the object has been created.
            tenant_ids.next().expect(
                "KubernetesDeploymentEntity's tenant_ids must be set with KubernetesDeploymentEntity.set_natural_key() is called (n=0)"
            ),
            tenant_ids.next().expect(
                "KubernetesDeploymentEntity's tenant_ids must be set with KubernetesDeploymentEntity.set_natural_key() is called (n=1)"
            ),
            tenant_ids.next().expect(
                "KubernetesDeploymentEntity's tenant_ids must be set with KubernetesDeploymentEntity.set_natural_key() is called (n=2)"
            ),
            <Self as Storable>::type_name(),
            self.name
                .as_ref()
                .expect("KubernetesDeploymentEntity.name must be set when KubernetesDeploymentEntity.set_natural_key() is called")
        );

        let mut storable = self.si_storable.as_mut().unwrap();
        storable.natural_key = Some(natural_key);
    }

    fn order_by_fields() -> Vec<&'static str> {
        vec![
            "id",
            "naturalKey",
            "typeName",
            "displayName",
            "name",
            "description",
            "kubernetesVersion",
            "state",
        ]
    }
}

impl ListRequest for KubernetesDeploymentEntityListEntitiesRequest {
    fn query(&self) -> &Option<DataQuery> {
        &self.query
    }

    fn set_query(&mut self, query: Option<DataQuery>) {
        self.query = query;
    }

    fn page_size(&self) -> u32 {
        self.page_size.unwrap_or(0)
    }

    fn set_page_size(&mut self, page_size: u32) {
        self.page_size = Some(page_size);
    }

    fn order_by(&self) -> &str {
        self.order_by.as_ref().map(String::as_ref).unwrap_or("ASC")
    }

    fn set_order_by(&mut self, order_by: impl Into<String>) {
        self.order_by = Some(order_by.into());
    }

    fn order_by_direction(&self) -> i32 {
        self.order_by_direction
    }

    fn set_order_by_direction(&mut self, order_by_direction: i32) {
        self.order_by_direction = order_by_direction;
    }

    fn page_token(&self) -> Option<&str> {
        self.page_token.as_ref().map(String::as_ref)
    }

    fn set_page_token(&mut self, page_token: Option<impl Into<String>>) {
        self.page_token = page_token.map(|s| s.into());
    }

    fn scope_by_tenant_id(&self) -> &str {
        self.scope_by_tenant_id
            .as_ref()
            .map(String::as_ref)
            .unwrap_or("")
    }

    fn set_scope_by_tenant_id(&mut self, scope_by_tenant_id: impl Into<String>) {
        self.scope_by_tenant_id = Some(scope_by_tenant_id.into());
    }
}

impl ListReply for KubernetesDeploymentEntityListEntitiesReply {
    type Reply = KubernetesDeploymentEntity;

    fn items(&self) -> &Vec<Self::Reply> {
        &self.items
    }

    fn set_items(&mut self, items: Vec<Self::Reply>) {
        self.items = items;
    }

    fn total_count(&self) -> u32 {
        self.total_count.unwrap_or(0)
    }

    fn set_total_count(&mut self, total_count: u32) {
        self.total_count = Some(total_count);
    }

    fn next_page_token(&self) -> Option<&str> {
        self.next_page_token.as_ref().map(String::as_ref)
    }

    fn set_next_page_token(&mut self, page_token: Option<impl Into<String>>) {
        self.next_page_token = page_token.map(|s| s.into());
    }
}

impl From<ListResult<KubernetesDeploymentEntity>> for KubernetesDeploymentEntityListEntitiesReply {
    fn from(list_result: ListResult<KubernetesDeploymentEntity>) -> Self {
        if list_result.items.len() == 0 {
            Self::default()
        } else {
            let next_page_token = if list_result.page_token().is_empty() {
                None
            } else {
                Some(list_result.page_token().to_string())
            };

            Self {
                total_count: Some(list_result.total_count()),
                next_page_token,
                items: list_result.items,
            }
        }
    }
}
