use crate::agent::client::MqttAsyncClientInternal;
use crate::entity::{Entity, EntityState};
use crate::error::{CeaError, CeaResult};
use crate::list::ListRequest;
use async_trait::async_trait;
use chrono::prelude::{DateTime, Utc};
use futures::compat::Future01CompatExt;
use paho_mqtt as mqtt;
use prost::Message;
use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use si_data::{Db, Storable};

#[async_trait]
pub trait EntityEvent:
    Message + Storable + Serialize + DeserializeOwned + std::fmt::Debug + Default
{
    type Entity: Entity;

    fn new(
        user_id: impl Into<String>,
        action_name: impl Into<String>,
        entity: &Self::Entity,
    ) -> Self {
        let create_time: DateTime<Utc> = Utc::now();
        let mut entity_event: Self = Default::default();
        entity_event.set_user_id(user_id);
        entity_event.set_action_name(action_name);
        entity_event.set_entity_id(entity.id());
        entity_event.set_create_time(create_time.to_string());
        entity_event.set_component_id(entity.component_id());
        entity_event.set_integration_id(entity.integration_id());
        entity_event.set_integration_service_id(entity.integration_service_id());
        entity_event.set_workspace_id(entity.workspace_id());
        entity_event.set_organization_id(entity.organization_id());
        entity_event.set_billing_account_id(entity.billing_account_id());
        entity_event.set_input_entity(Some(entity.clone()));
        <Self as EntityEvent>::add_to_tenant_ids(&mut entity_event, entity.billing_account_id());
        <Self as EntityEvent>::add_to_tenant_ids(&mut entity_event, entity.organization_id());
        <Self as EntityEvent>::add_to_tenant_ids(&mut entity_event, entity.workspace_id());
        <Self as EntityEvent>::add_to_tenant_ids(&mut entity_event, entity.id());
        entity_event
    }

    fn action_names() -> &'static [&'static str];
    fn action_name(&self) -> &str;
    fn set_action_name(&mut self, action_name: impl Into<String>);
    fn billing_account_id(&self) -> &str;
    fn set_billing_account_id(&mut self, billing_account_id: impl Into<String>);
    fn organization_id(&self) -> &str;
    fn set_organization_id(&mut self, organization_id: impl Into<String>);
    fn workspace_id(&self) -> &str;
    fn set_workspace_id(&mut self, workspace_id: impl Into<String>);
    fn integration_id(&self) -> &str;
    fn set_integration_id(&mut self, integration_id: impl Into<String>);
    fn integration_service_id(&self) -> &str;
    fn set_integration_service_id(&mut self, integration_service_id: impl Into<String>);
    fn component_id(&self) -> &str;
    fn set_component_id(&mut self, component_id: impl Into<String>);
    fn entity_id(&self) -> &str;
    fn set_entity_id(&mut self, entity_id: impl Into<String>);
    fn id(&self) -> &str;
    fn set_id(&mut self, id: impl Into<String>);
    fn type_name() -> &'static str;
    fn get_type_name(&self) -> &str;
    fn set_type_name(&mut self, type_name: impl Into<String>);
    fn tenant_ids(&self) -> &[String];
    fn add_to_tenant_ids(&mut self, id: impl Into<String>);
    fn natural_key(&self) -> Option<&str>;
    fn set_natural_key(&mut self, natural_key: impl Into<String>);
    fn order_by_fields() -> Vec<&'static str>;
    fn output_lines(&self) -> &[String];
    fn add_to_output_lines(&mut self, line: impl Into<String>);
    fn error_lines(&self) -> &[String];
    fn add_to_error_lines(&mut self, line: impl Into<String>);
    fn success(&self) -> bool;
    fn set_success(&mut self, success: bool);
    fn updated_time(&self) -> &str;
    fn set_updated_time(&mut self, time_string: impl Into<String>);
    fn final_time(&self) -> &str;
    fn set_final_time(&mut self, time_string: impl Into<String>);
    fn finalized(&self) -> bool;
    fn set_finalized(&mut self, finalized: bool);
    fn error_message(&self) -> &str;
    fn set_error_message(&mut self, error_message: impl Into<String>);
    fn output_entity(&self) -> Option<&Self::Entity>;
    fn mut_output_entity(&mut self) -> Option<&mut Self::Entity>;
    fn input_entity(&self) -> Option<&Self::Entity>;
    fn set_input_entity(&mut self, entity: Option<Self::Entity>);
    fn set_output_entity(&mut self, entity: Option<Self::Entity>);
    fn user_id(&self) -> &str;
    fn set_user_id(&mut self, user_id: impl Into<String>);
    fn create_time(&self) -> &str;
    fn set_create_time(&mut self, create_time: impl Into<String>);

    async fn create(
        db: &Db,
        user_id: &str,
        action_name: &str,
        entity: &Self::Entity,
    ) -> CeaResult<Self> {
        let mut entity_event = Self::new(user_id, action_name, entity);
        db.validate_and_insert_as_new(&mut entity_event).await?;
        Ok(entity_event)
    }

    async fn list<T: ListRequest>(
        db: &Db,
        list_request: &T,
    ) -> CeaResult<si_data::ListResult<Self>> {
        let result = if list_request.has_page_token() {
            db.list_by_page_token(list_request.page_token()).await?
        } else {
            db.list(
                list_request.query(),
                list_request.page_size(),
                list_request.order_by(),
                list_request.order_by_direction(),
                list_request.scope_by_tenant_id(),
                "",
            )
            .await?
        };
        Ok(result)
    }

    fn validate_action_name(&self) -> CeaResult<()> {
        if Self::action_names()
            .iter()
            .find(|&&x| x == self.action_name())
            .is_none()
        {
            return Err(CeaError::ValidationError(format!(
                "Action name is invalid: {}",
                self.action_name()
            )));
        }
        Ok(())
    }

    fn validate_input_entity(&self) -> CeaResult<()> {
        if self.input_entity().is_none() {
            return Err(CeaError::ValidationError(format!(
                "Input entity must not be empty"
            )));
        }
        Ok(())
    }

    fn validate(&self) -> CeaResult<()> {
        self.validate_action_name()?;
        self.validate_input_entity()?;
        Ok(())
    }

    fn log(&mut self, line: impl Into<String>) {
        self.add_to_output_lines(line);
    }

    fn error_log(&mut self, line: impl Into<String>) {
        self.add_to_error_lines(line);
    }

    fn failed(&mut self, err: impl std::error::Error) {
        let time: DateTime<Utc> = Utc::now();
        let time_string = format!("{}", time);
        self.set_success(false);
        self.set_updated_time(time_string.clone());
        self.set_final_time(time_string);
        self.set_finalized(true);
        self.set_error_message(err.to_string());
        self.add_to_error_lines(format!("*** ERROR STRING ***\n{}", err));
        if self.output_entity().is_none() {
            if self.input_entity().is_some() {
                let input_entity = self.input_entity().unwrap().clone();
                let mut output_entity = input_entity;
                output_entity.set_state(EntityState::Error as i32);
                self.set_output_entity(Some(output_entity));
            }
        }
        self.log("*** Task failed ***");
    }

    fn succeeded(&mut self) {
        let time: DateTime<Utc> = Utc::now();
        let time_string = format!("{}", time);
        self.set_success(true);
        self.set_updated_time(time_string.clone());
        self.set_final_time(time_string);
        self.set_finalized(true);
        if self.output_entity().is_none() {
            // What happens if there is no input entity, and no output entity?
            if self.input_entity().is_some() {
                let mut output_entity = self.input_entity().unwrap().clone();
                // You're safe, because we just checked... twice!
                output_entity.set_state(EntityState::Ok as i32);
                self.set_output_entity(Some(output_entity));
            }
        } else {
            self.mut_output_entity()
                .unwrap()
                .set_state(EntityState::Ok as i32);
        }
        self.log("*** Task Succeeded ***");
    }

    async fn send_via_mqtt(&self, mqtt_client: &MqttAsyncClientInternal) -> CeaResult<()> {
        let mut payload = Vec::new();
        let finalized = {
            self.encode(&mut payload)?;
            self.finalized()
        };
        if finalized {
            let msg = mqtt::Message::new(self.result_topic(), payload.clone(), 0);
            mqtt_client.publish(msg).compat().await?;
            let msg = mqtt::Message::new(self.finalized_topic(), payload, 2);
            mqtt_client.publish(msg).compat().await?;
        } else {
            let msg = mqtt::Message::new(self.result_topic(), payload, 0);
            mqtt_client.publish(msg).compat().await?;
        }
        Ok(())
    }

    fn result_topic(&self) -> String {
        let topic = format!(
            "{}/{}/{}/{}/{}/{}/{}/{}/{}/result",
            self.billing_account_id(),
            self.organization_id(),
            self.workspace_id(),
            self.integration_id(),
            self.integration_service_id(),
            self.entity_id(),
            "action",
            self.action_name(),
            self.id(),
        );
        topic
    }

    fn finalized_topic(&self) -> String {
        let topic = format!(
            "{}/{}/{}/{}/{}/{}/{}/{}/{}/{}/finalized",
            self.billing_account_id(),
            self.organization_id(),
            self.workspace_id(),
            self.integration_id(),
            self.integration_service_id(),
            self.get_type_name(),
            self.entity_id(),
            "action",
            self.action_name(),
            self.id(),
        );
        topic
    }
}

#[macro_export]
macro_rules! gen_entity_event {
    (
       type_name: $type_name:tt
       ,
       action_names: [ $( $action:tt ),* ]
    ) => {
        impl si_cea::EntityEvent for EntityEvent {
            type Entity = Entity;

            fn set_action_name(&mut self, action_name: impl Into<String>) {
                self.action_name = action_name.into();
            }

            fn set_billing_account_id(&mut self, billing_account_id: impl Into<String>) {
                self.billing_account_id = billing_account_id.into();
            }

            fn set_organization_id(&mut self, organization_id: impl Into<String>) {
                self.organization_id = organization_id.into();
            }

            fn set_workspace_id(&mut self, workspace_id: impl Into<String>) {
                self.workspace_id = workspace_id.into();
            }

            fn set_integration_id(&mut self, integration_id: impl Into<String>) {
                self.integration_id = integration_id.into();
            }

            fn set_integration_service_id(&mut self, integration_service_id: impl Into<String>) {
                self.integration_service_id = integration_service_id.into();
            }

            fn set_component_id(&mut self, component_id: impl Into<String>) {
                self.component_id = component_id.into();
            }

            fn set_entity_id(&mut self, entity_id: impl Into<String>) {
                self.entity_id = entity_id.into();
            }

            fn success(&self) -> bool {
                self.success
            }

            fn set_success(&mut self, success: bool) {
                self.success = success;
            }

            fn updated_time(&self) -> &str {
                &self.updated_time
            }

            fn set_updated_time(&mut self, updated_time: impl Into<String>) {
                self.updated_time = updated_time.into();
            }

            fn final_time(&self) -> &str {
                &self.final_time
            }

            fn set_final_time(&mut self, final_time: impl Into<String>) {
                self.final_time = final_time.into();
            }

            fn finalized(&self) -> bool {
                self.finalized
            }

            fn set_finalized(&mut self, finalized: bool) {
                self.finalized = finalized;
            }

            fn error_message(&self) -> &str {
                &self.error_message
            }

            fn set_error_message(&mut self, error_message: impl Into<String>) {
                self.error_message = error_message.into();
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

            fn user_id(&self) -> &str {
                self.user_id.as_ref()
            }

            fn set_user_id(&mut self, user_id: impl Into<String>) {
                self.user_id = user_id.into();
            }

            fn create_time(&self) -> &str {
                self.create_time.as_ref()
            }

            fn set_create_time(&mut self, create_time: impl Into<String>) {
                self.create_time = create_time.into();
            }

            fn action_names() -> &'static [&'static str] {
                &[$($action),*]
            }

            fn action_name(&self) -> &str {
                self.action_name.as_ref()
            }

            fn billing_account_id(&self) -> &str {
                self.billing_account_id.as_ref()
            }

            fn organization_id(&self) -> &str {
                self.organization_id.as_ref()
            }

            fn workspace_id(&self) -> &str {
                self.workspace_id.as_ref()
            }

            fn integration_id(&self) -> &str {
                self.integration_id.as_ref()
            }

            fn integration_service_id(&self) -> &str {
                self.integration_service_id.as_ref()
            }

            fn component_id(&self) -> &str {
                self.component_id.as_ref()
            }

            fn entity_id(&self) -> &str {
                self.entity_id.as_ref()
            }

            fn id(&self) -> &str {
                self.id.as_ref()
            }

            fn set_id(&mut self, id: impl Into<String>) {
                self.id = id.into();
            }

            fn input_entity(&self) -> Option<&Self::Entity> {
                self.input_entity.as_ref()
            }

            fn type_name() -> &'static str {
                $type_name
            }

            fn get_type_name(&self) -> &str {
                self.type_name.as_ref()
            }

            fn set_type_name(&mut self, type_name: impl Into<String>) {
                self.type_name = type_name.into();
            }

            fn tenant_ids(&self) -> &[String] {
                self.tenant_ids.as_ref()
            }

            fn add_to_tenant_ids(&mut self, id: impl Into<String>) {
                self.tenant_ids.push(id.into());
            }

            fn natural_key(&self) -> Option<&str> {
                None
            }

            fn set_natural_key(&mut self, natural_key: impl Into<String>) {
                self.natural_key = natural_key.into();
            }

            fn output_lines(&self) -> &[String] {
                self.output_lines.as_ref()
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
                &self.id
            }

            fn set_id(&mut self, id: impl Into<String>) {
                self.id = id.into();
            }

            fn type_name() -> &'static str {
                $type_name
            }

            fn set_type_name(&mut self) {
                self.type_name = <Self as si_data::Storable>::type_name().to_string();
            }

            fn generate_id(&mut self) {
                let uuid = Uuid::new_v4();
                self.id = format!("{}:{}", <Self as si_data::Storable>::type_name(), uuid);
            }

            fn validate(&self) -> si_data::error::Result<()> {
                match <Self as si_cea::EntityEvent>::validate(&self) {
                    Ok(()) => Ok(()),
                    Err(e) => Err(si_data::error::DataError::ValidationError(e.to_string())),
                }
            }

            fn get_tenant_ids(&self) -> &[String] {
                &self.tenant_ids
            }

            fn add_to_tenant_ids(&mut self, id: impl Into<String>) {
                self.tenant_ids.push(id.into());
            }

            fn referential_fields(&self) -> Vec<si_data::Reference> {
                vec![
                    si_data::Reference::HasOne("entity_id", &self.entity_id),
                    si_data::Reference::HasOne("component_id", &self.component_id),
                    si_data::Reference::HasOne("integration_id", &self.integration_id),
                    si_data::Reference::HasOne("integration_service_id", &self.integration_service_id),
                    si_data::Reference::HasOne("workspace_id", &self.workspace_id),
                    si_data::Reference::HasOne("organization_id", &self.organization_id),
                    si_data::Reference::HasOne("billing_account_id", &self.billing_account_id),
                ]
            }

            fn get_natural_key(&self) -> Option<&str> {
                None
            }

            fn set_natural_key(&mut self) {
                self.natural_key = self.id.clone();
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

        impl si_cea::ListReply for ListEntityEventsReply {
            type Reply = EntityEvent;

            fn items(&self) -> &Vec<Self::Reply> {
                &self.items
            }

            fn set_items(&mut self, items: Vec<Self::Reply>) {
                self.items = items;
            }

            fn total_count(&self) -> i32 {
                self.total_count
            }

            fn set_total_count(&mut self, total_count: i32) {
                self.total_count = total_count;
            }

            fn next_page_token(&self) -> &str {
                self.next_page_token.as_ref()
            }

            fn set_next_page_token(&mut self, page_token: impl Into<String>) {
                self.next_page_token = page_token.into();
            }
        }

        impl From<si_data::ListResult<EntityEvent>> for ListEntityEventsReply {
            fn from(list_result: si_data::ListResult<EntityEvent>) -> ListEntityEventsReply {
                if list_result.items.len() == 0 {
                    ListEntityEventsReply::default()
                } else {
                    ListEntityEventsReply {
                        total_count: list_result.total_count(),
                        next_page_token: list_result.page_token().to_string(),
                        items: list_result.items,
                    }
                }
            }
        }

        impl si_cea::ListRequest for ListEntityEventsRequest {
            fn query(&self) -> &Option<Query> {
                &self.query
            }

            fn set_query(&mut self, query: Option<Query>) {
                self.query = query;
            }

            fn page_size(&self) -> i32 {
                self.page_size
            }

            fn set_page_size(&mut self, page_size: i32) {
                self.page_size = page_size;
            }

            fn order_by(&self) -> &str {
                self.order_by.as_ref()
            }

            fn set_order_by(&mut self, order_by: impl Into<String>) {
                self.order_by = order_by.into();
            }

            fn order_by_direction(&self) -> i32 {
                self.order_by_direction
            }

            fn set_order_by_direction(&mut self, order_by_direction: i32) {
                self.order_by_direction = order_by_direction;
            }

            fn page_token(&self) -> &str {
                self.page_token.as_ref()
            }

            fn set_page_token(&mut self, page_token: impl Into<String>) {
                self.page_token = page_token.into()
            }

            fn scope_by_tenant_id(&self) -> &str {
                self.scope_by_tenant_id.as_ref()
            }

            fn set_scope_by_tenant_id(&mut self, scope_by_tenant_id: impl Into<String>) {
                self.scope_by_tenant_id = scope_by_tenant_id.into();
            }
        }
    };
}
