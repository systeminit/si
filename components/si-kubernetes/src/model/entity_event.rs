use crate::model::KubernetesDeploymentEntity;
pub use crate::protobuf::{
    KubernetesDeploymentEntityEvent, KubernetesDeploymentEntityEventListEntityEventsReply,
    KubernetesDeploymentEntityEventListEntityEventsRequest,
};
use si_data::DataQuery;

type Entity = KubernetesDeploymentEntity;
type EntityEvent = KubernetesDeploymentEntityEvent;

impl si_cea::EntityEvent for EntityEvent {
    type Entity = Entity;

    fn set_action_name(&mut self, action_name: impl Into<String>) {
        self.action_name = Some(action_name.into());
    }

    fn set_billing_account_id(&mut self, billing_account_id: impl Into<String>) {
        self.si_properties
            .as_mut()
            .expect("TODO: fix")
            .billing_account_id = Some(billing_account_id.into());
    }

    fn set_organization_id(&mut self, organization_id: impl Into<String>) {
        self.si_properties
            .as_mut()
            .expect("TODO: fix")
            .organization_id = Some(organization_id.into());
    }

    fn set_workspace_id(&mut self, workspace_id: impl Into<String>) {
        self.si_properties.as_mut().expect("TODO: fix").workspace_id = Some(workspace_id.into());
    }

    fn set_integration_id(&mut self, integration_id: impl Into<String>) {
        self.si_properties
            .as_mut()
            .expect("TODO: fix")
            .integration_id = Some(integration_id.into());
    }

    fn set_integration_service_id(&mut self, integration_service_id: impl Into<String>) {
        self.si_properties
            .as_mut()
            .expect("TODO: fix")
            .integration_service_id = Some(integration_service_id.into());
    }

    fn set_component_id(&mut self, component_id: impl Into<String>) {
        self.si_properties.as_mut().expect("TODO: fix").component_id = Some(component_id.into());
    }

    fn set_entity_id(&mut self, entity_id: impl Into<String>) {
        self.si_properties.as_mut().expect("TODO: fix").entity_id = Some(entity_id.into());
    }

    fn success(&self) -> bool {
        self.success.expect("TODO: fix")
    }

    fn set_success(&mut self, success: bool) {
        self.success = Some(success);
    }

    fn updated_time(&self) -> &str {
        self.updated_time.as_ref().expect("TODO: fix")
    }

    fn set_updated_time(&mut self, updated_time: impl Into<String>) {
        self.updated_time = Some(updated_time.into());
    }

    fn final_time(&self) -> &str {
        self.final_time.as_ref().expect("TODO: fix")
    }

    fn set_final_time(&mut self, final_time: impl Into<String>) {
        self.final_time = Some(final_time.into());
    }

    fn finalized(&self) -> bool {
        self.finalized.expect("TODO: fix")
    }

    fn set_finalized(&mut self, finalized: bool) {
        self.finalized = Some(finalized);
    }

    fn error_message(&self) -> &str {
        self.error_message.as_ref().expect("TODO: fix")
    }

    fn set_error_message(&mut self, error_message: impl Into<String>) {
        self.error_message = Some(error_message.into());
    }

    fn output_entity(&self) -> Option<&Self::Entity> {
        self.output_entity.as_ref()
    }

    fn mut_output_entity(&mut self) -> Option<&mut Self::Entity> {
        self.output_entity.as_mut()
    }

    fn set_input_entity(&mut self, input_entity: Option<Self::Entity>) {
        self.input_entity = input_entity;
    }

    fn set_output_entity(&mut self, output_entity: Option<Self::Entity>) {
        self.output_entity = output_entity;
    }

    fn set_previous_entity(&mut self, previous_entity: Option<Self::Entity>) {
        self.previous_entity = previous_entity;
    }

    fn user_id(&self) -> &str {
        self.user_id.as_ref().expect("TODO: fix")
    }

    fn set_user_id(&mut self, user_id: impl Into<String>) {
        self.user_id = Some(user_id.into());
    }

    fn create_time(&self) -> &str {
        self.create_time.as_ref().expect("TODO: fix")
    }

    fn set_create_time(&mut self, create_time: impl Into<String>) {
        self.create_time = Some(create_time.into());
    }

    fn action_names() -> &'static [&'static str] {
        &["create", "sync", "apply", "edit"]
    }

    fn action_name(&self) -> &str {
        self.action_name.as_ref().expect("TODO: fix")
    }

    fn billing_account_id(&self) -> &str {
        self.si_properties
            .as_ref()
            .expect("TODO: fix")
            .billing_account_id
            .as_ref()
            .expect("TODO: fix")
    }

    fn organization_id(&self) -> &str {
        self.si_properties
            .as_ref()
            .expect("TODO: fix")
            .organization_id
            .as_ref()
            .expect("TODO: fix")
    }

    fn workspace_id(&self) -> &str {
        self.si_properties
            .as_ref()
            .expect("TODO: fix")
            .workspace_id
            .as_ref()
            .expect("TODO: fix")
    }

    fn integration_id(&self) -> &str {
        self.si_properties
            .as_ref()
            .expect("TODO: fix")
            .integration_id
            .as_ref()
            .expect("TODO: fix")
    }

    fn integration_service_id(&self) -> &str {
        self.si_properties
            .as_ref()
            .expect("TODO: fix")
            .integration_service_id
            .as_ref()
            .expect("TODO: fix")
    }

    fn component_id(&self) -> &str {
        self.si_properties
            .as_ref()
            .expect("TODO: fix")
            .component_id
            .as_ref()
            .expect("TODO: fix")
    }

    fn entity_id(&self) -> &str {
        self.si_properties
            .as_ref()
            .expect("TODO: fix")
            .entity_id
            .as_ref()
            .expect("TODO: fix")
    }

    fn id(&self) -> &str {
        self.id.as_ref().expect("TODO: fix")
    }

    fn set_id(&mut self, id: impl Into<String>) {
        self.id = Some(id.into());
    }

    fn input_entity(&self) -> Option<&Self::Entity> {
        self.input_entity.as_ref()
    }

    fn type_name() -> &'static str {
        "kubernetes_deployment_entity_event"
    }

    fn get_type_name(&self) -> &str {
        <Self as si_data::Storable>::type_name()
    }

    fn set_type_name(&mut self, type_name: impl Into<String>) {
        if let None = self.si_storable {
            self.si_storable = Some(Default::default());
        }

        let storable = self.si_storable.as_mut().unwrap();
        storable.type_name = Some(type_name.into());
    }

    fn tenant_ids(&self) -> &[String] {
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

    fn natural_key(&self) -> Option<&str> {
        None
    }

    fn set_natural_key(&mut self, natural_key: impl Into<String>) {
        if let None = self.si_storable {
            self.si_storable = Some(Default::default());
        }

        let mut storable = self.si_storable.as_mut().unwrap();
        storable.natural_key = Some(natural_key.into());
    }

    fn output_lines(&self) -> &[String] {
        &self.output_lines
    }

    fn add_to_output_lines(&mut self, line: impl Into<String>) {
        self.output_lines.push(line.into());
    }

    fn error_lines(&self) -> &[String] {
        self.error_lines.as_ref()
    }

    fn add_to_error_lines(&mut self, line: impl Into<String>) {
        self.error_lines.push(line.into());
    }

    fn order_by_fields() -> Vec<&'static str> {
        vec![
            "id",
            "naturalKey",
            "typeName",
            "userId",
            "actionName",
            "createTime",
            "updatedTime",
            "finalTime",
            "finalized",
            "entityId",
            "componentId",
            "integrationId",
            "integrationServiceId",
            "workspaceId",
            "organizationId",
            "billingAccountId",
        ]
    }
}

impl si_data::Storable for EntityEvent {
    fn get_id(&self) -> &str {
        self.id.as_ref().expect("TODO: fix")
    }

    fn set_id(&mut self, id: impl Into<String>) {
        self.id = Some(id.into());
    }

    fn type_name() -> &'static str {
        "kubernetes_deployment_entity_event"
    }

    fn set_type_name(&mut self) {
        if let None = self.si_storable {
            self.si_storable = Some(Default::default());
        }

        let storable = self.si_storable.as_mut().unwrap();
        storable.type_name = Some(<Self as si_data::Storable>::type_name().to_string());
    }

    fn generate_id(&mut self) {
        <Self as si_data::Storable>::set_id(
            self,
            format!(
                "{}:{}",
                <Self as si_data::Storable>::type_name(),
                uuid::Uuid::new_v4(),
            ),
        );
    }

    fn validate(&self) -> si_data::error::Result<()> {
        match <Self as si_cea::EntityEvent>::validate(&self) {
            Ok(()) => Ok(()),
            Err(e) => Err(si_data::error::DataError::ValidationError(e.to_string())),
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

    fn referential_fields(&self) -> Vec<si_data::Reference> {
        const NO_ENTITY_ID: &str = "<NO_ENTITY_ID_HERE_SHANNON>";
        const NO_COMPONENT_ID: &str = "<NO_COMPONENT_ID_HERE_SHANNON>";
        const NO_INTEGRATION_ID: &str = "<NO_INTEGRATION_ID_HERE_SHANNON>";
        const NO_INTEGRATION_SERVICE_ID: &str = "<NO_INTEGRATION_SERVICE_ID_HERE_SHANNON>";
        const NO_WORKSPACE_ID: &str = "<NO_WORKSPACE_ID_HERE_SHANNON>";
        const NO_ORGANIZATION_ID: &str = "<NO_ORGANIZATION_ID_HERE_SHANNON>";
        const NO_BILLING_ACCOUNT_ID: &str = "<NO_BILLING_ACCOUNT_ID_HERE_SHANNON>";

        let entity_id = match &self.si_properties {
            Some(cip) => cip
                .entity_id
                .as_ref()
                .map(String::as_ref)
                .unwrap_or(NO_ENTITY_ID),
            None => NO_ENTITY_ID,
        };
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
            si_data::Reference::HasOne("entity_id", entity_id),
            si_data::Reference::HasOne("component_id", component_id),
            si_data::Reference::HasOne("integration_id", integration_id),
            si_data::Reference::HasOne("integration_service_id", integration_service_id),
            si_data::Reference::HasOne("workspace_id", workspace_id),
            si_data::Reference::HasOne("organization_id", organization_id),
            si_data::Reference::HasOne("billing_account_id", billing_account_id),
        ]
    }

    fn get_natural_key(&self) -> Option<&str> {
        None
    }

    fn set_natural_key(&mut self) {
        if let None = self.si_storable {
            self.si_storable = Some(Default::default());
        }

        let mut storable = self.si_storable.as_mut().unwrap();
        storable.natural_key = self.id.clone();
    }

    fn order_by_fields() -> Vec<&'static str> {
        vec![
            "id",
            "naturalKey",
            "typeName",
            "userId",
            "actionName",
            "createTime",
            "updatedTime",
            "finalTime",
            "finalized",
            "entityId",
            "componentId",
            "integrationId",
            "integrationServiceId",
            "workspaceId",
            "organizationId",
            "billingAccountId",
        ]
    }
}

impl si_cea::ListReply for KubernetesDeploymentEntityEventListEntityEventsReply {
    type Reply = EntityEvent;

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

impl From<si_data::ListResult<EntityEvent>>
    for KubernetesDeploymentEntityEventListEntityEventsReply
{
    fn from(list_result: si_data::ListResult<EntityEvent>) -> Self {
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

impl si_cea::ListRequest for KubernetesDeploymentEntityEventListEntityEventsRequest {
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
