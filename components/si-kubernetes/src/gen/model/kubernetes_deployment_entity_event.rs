// Auth-generated code!
// No touchy!

impl crate::protobuf::KubernetesDeploymentEntityEvent {
    pub async fn get(
        db: &si_data::Db,
        id: &str,
    ) -> si_data::Result<crate::protobuf::KubernetesDeploymentEntityEvent> {
        let obj = db.get(id).await?;
        Ok(obj)
    }

    pub async fn get_by_natural_key(
        db: &si_data::Db,
        natural_key: &str,
    ) -> si_data::Result<crate::protobuf::KubernetesDeploymentEntityEvent> {
        let obj = db.lookup_by_natural_key(natural_key).await?;
        Ok(obj)
    }

    pub async fn save(&self, db: &si_data::Db) -> si_data::Result<()> {
        db.upsert(self).await?;
        Ok(())
    }

    pub async fn list(
        db: &si_data::Db,
        list_request: crate::protobuf::KubernetesDeploymentEntityEventListRequest,
    ) -> si_data::Result<si_data::ListResult<crate::protobuf::KubernetesDeploymentEntityEvent>>
    {
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

impl si_cea::EntityEvent for crate::protobuf::KubernetesDeploymentEntityEvent {
    type Entity = crate::protobuf::KubernetesDeploymentEntity;

    fn action_names() -> &'static [&'static str] {
        &[
            "create",
            "apply",
            "edit_kubernetes_object",
            "edit_kubernetes_object_yaml",
            "sync",
        ]
    }

    fn action_name(&self) -> si_data::Result<&str> {
        self.action_name
            .as_ref()
            .map(String::as_str)
            .ok_or_else(|| si_data::DataError::RequiredField("action_name".to_string()))
    }

    fn set_action_name(&mut self, action_name: impl Into<String>) {
        self.action_name = Some(action_name.into());
    }

    fn create_time(&self) -> si_data::Result<&str> {
        self.create_time
            .as_ref()
            .map(String::as_str)
            .ok_or_else(|| si_data::DataError::RequiredField("create_time".to_string()))
    }

    fn set_create_time(&mut self, create_time: impl Into<String>) {
        self.create_time = Some(create_time.into());
    }

    fn updated_time(&self) -> Option<&str> {
        self.updated_time.as_ref().map(String::as_str)
    }

    fn set_updated_time(&mut self, updated_time: impl Into<String>) {
        self.updated_time = Some(updated_time.into());
    }

    fn final_time(&self) -> Option<&str> {
        self.final_time.as_ref().map(String::as_str)
    }

    fn set_final_time(&mut self, final_time: impl Into<String>) {
        self.final_time = Some(final_time.into());
    }

    fn success(&self) -> Option<bool> {
        self.success
    }

    fn set_success(&mut self, success: bool) {
        self.success = Some(success);
    }

    fn finalized(&self) -> Option<bool> {
        self.finalized
    }

    fn set_finalized(&mut self, finalized: bool) {
        self.finalized = Some(finalized);
    }

    fn user_id(&self) -> si_data::Result<&str> {
        self.user_id
            .as_ref()
            .map(String::as_str)
            .ok_or_else(|| si_data::DataError::RequiredField("user_id".to_string()))
    }

    fn set_user_id(&mut self, user_id: impl Into<String>) {
        self.user_id = Some(user_id.into());
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

    fn error_message(&self) -> Option<&str> {
        self.error_message.as_ref().map(String::as_str)
    }

    fn set_error_message(&mut self, error_message: impl Into<String>) {
        self.error_message = Some(error_message.into());
    }

    fn previous_entity(&self) -> Option<&Self::Entity> {
        self.previous_entity.as_ref()
    }

    fn set_previous_entity(&mut self, previous_entity: Self::Entity) {
        self.previous_entity = Some(previous_entity);
    }

    fn input_entity(&self) -> si_data::Result<&Self::Entity> {
        self.input_entity
            .as_ref()
            .ok_or_else(|| si_data::DataError::RequiredField("input_entity".to_string()))
    }

    fn set_input_entity(&mut self, input_entity: Self::Entity) {
        self.input_entity = Some(input_entity);
    }

    fn output_entity(&self) -> Option<&Self::Entity> {
        self.output_entity.as_ref()
    }

    fn set_output_entity(&mut self, output_entity: Self::Entity) {
        self.output_entity = Some(output_entity);
    }

    fn mut_output_entity(&mut self) -> si_data::Result<&mut Self::Entity> {
        if self.output_entity.is_none() {
            self.init_output_entity()?;
        }

        Ok(self
            .output_entity
            .as_mut()
            .expect("output_entity has been set or initialized"))
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
            "crate::protobuf::KubernetesDeploymentEntityEvent.si_properties \
                has been set or initialized",
        );
        si_properties.billing_account_id = Some(billing_account_id.into());
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
            "crate::protobuf::KubernetesDeploymentEntityEvent.si_properties \
                has been set or initialized",
        );
        si_properties.organization_id = Some(organization_id.into());
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
            "crate::protobuf::KubernetesDeploymentEntityEvent.si_properties \
                has been set or initialized",
        );
        si_properties.workspace_id = Some(workspace_id.into());
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
            "crate::protobuf::KubernetesDeploymentEntityEvent.si_properties \
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
            "crate::protobuf::KubernetesDeploymentEntityEvent.si_properties \
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
            "crate::protobuf::KubernetesDeploymentEntityEvent.si_properties \
                has been set or initialized",
        );
        si_properties.component_id = Some(component_id.into());
    }

    fn entity_id(&self) -> si_data::Result<&str> {
        self.si_properties
            .as_ref()
            .ok_or_else(|| si_data::DataError::RequiredField("si_properties".to_string()))?
            .entity_id
            .as_ref()
            .map(String::as_str)
            .ok_or_else(|| si_data::DataError::RequiredField("entity_id".to_string()))
    }

    fn set_entity_id(&mut self, entity_id: impl Into<String>) {
        if self.si_properties.is_none() {
            self.si_properties = Some(Default::default());
        }

        let si_properties = self.si_properties.as_mut().expect(
            "crate::protobuf::KubernetesDeploymentEntityEvent.si_properties \
                has been set or initialized",
        );
        si_properties.entity_id = Some(entity_id.into());
    }
}

impl si_data::Storable for crate::protobuf::KubernetesDeploymentEntityEvent {
    fn type_name() -> &'static str {
        "kubernetes_deployment_entity_event"
    }

    fn set_type_name(&mut self) {
        if self.si_storable.is_none() {
            self.si_storable = Some(Default::default());
        }

        let si_storable = self.si_storable.as_mut().expect(
            "crate::protobuf::KubernetesDeploymentEntityEvent.si_storable \
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
            "crate::protobuf::KubernetesDeploymentEntityEvent.si_storable \
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
        if self.si_storable.is_none() {
            return Err(si_data::DataError::ValidationError(
                "missing required si_storable value".into(),
            ));
        }
        if self.action_name.is_none() {
            return Err(si_data::DataError::ValidationError(
                "missing required action_name value".into(),
            ));
        }
        if self.input_entity.is_none() {
            return Err(si_data::DataError::ValidationError(
                "missing required input_entity value".into(),
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
            "actionName",
            "createTime",
            "updatedTime",
            "finalTime",
            "success",
            "finalized",
            "userId",
            "outputLines",
            "errorLines",
            "errorMessage",
        ]
    }
}
