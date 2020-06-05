use crate::entity::{Entity, EntitySiPropertiesEntityState};
use crate::{CeaError, CeaResult, MqttClient};
use async_trait::async_trait;
use chrono::prelude::{DateTime, Utc};
use futures::compat::Future01CompatExt;
use paho_mqtt as mqtt;
use prost::Message;
use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use si_data::{Db, Result as DataResult, Storable};

pub mod prelude {
    pub use super::EntityEvent;
    pub use crate::list::{ListReply, ListRequest};
    pub use si_data::{uuid_string, DataQuery};
}

#[async_trait]
pub trait EntityEvent:
    Clone + std::fmt::Debug + Default + DeserializeOwned + Message + Serialize + Storable
{
    type Entity: Entity;

    fn action_names() -> &'static [&'static str];
    fn action_name(&self) -> DataResult<&str>;
    fn set_action_name(&mut self, action_name: impl Into<String>);

    fn create_time(&self) -> DataResult<&str>;
    fn set_create_time(&mut self, create_time: impl Into<String>);

    fn updated_time(&self) -> Option<&str>;
    fn set_updated_time(&mut self, time_string: impl Into<String>);

    fn final_time(&self) -> Option<&str>;
    fn set_final_time(&mut self, time_string: impl Into<String>);

    fn success(&self) -> Option<bool>;
    fn set_success(&mut self, success: bool);

    fn finalized(&self) -> Option<bool>;
    fn set_finalized(&mut self, finalized: bool);

    fn user_id(&self) -> DataResult<&str>;
    fn set_user_id(&mut self, user_id: impl Into<String>);

    fn output_lines(&self) -> &[String];
    fn add_to_output_lines(&mut self, line: impl Into<String>);

    fn error_lines(&self) -> &[String];
    fn add_to_error_lines(&mut self, line: impl Into<String>);

    fn error_message(&self) -> Option<&str>;
    fn set_error_message(&mut self, error_message: impl Into<String>);

    fn previous_entity(&self) -> Option<&Self::Entity>;
    fn set_previous_entity(&mut self, entity: Self::Entity);

    fn input_entity(&self) -> DataResult<&Self::Entity>;
    fn set_input_entity(&mut self, entity: Self::Entity);

    fn output_entity(&self) -> Option<&Self::Entity>;
    fn set_output_entity(&mut self, entity: Self::Entity);
    fn mut_output_entity(&mut self) -> DataResult<&mut Self::Entity>;

    fn billing_account_id(&self) -> DataResult<&str>;
    fn set_billing_account_id(&mut self, billing_account_id: impl Into<String>);

    fn organization_id(&self) -> DataResult<&str>;
    fn set_organization_id(&mut self, organization_id: impl Into<String>);

    fn workspace_id(&self) -> DataResult<&str>;
    fn set_workspace_id(&mut self, workspace_id: impl Into<String>);

    fn integration_id(&self) -> DataResult<&str>;
    fn set_integration_id(&mut self, integration_id: impl Into<String>);

    fn integration_service_id(&self) -> DataResult<&str>;
    fn set_integration_service_id(&mut self, integration_service_id: impl Into<String>);

    fn component_id(&self) -> DataResult<&str>;
    fn set_component_id(&mut self, component_id: impl Into<String>);

    fn entity_id(&self) -> DataResult<&str>;
    fn set_entity_id(&mut self, entity_id: impl Into<String>);

    fn set_change_set_id(&mut self, change_set_id: impl Into<String>);

    fn new(
        user_id: impl Into<String>,
        action_name: impl Into<String>,
        entity: &Self::Entity,
    ) -> DataResult<Self> {
        let create_time: DateTime<Utc> = Utc::now();
        let mut entity_event: Self = Default::default();
        entity_event.add_to_tenant_ids(entity.billing_account_id()?);
        entity_event.add_to_tenant_ids(entity.organization_id()?);
        entity_event.add_to_tenant_ids(entity.workspace_id()?);
        entity_event.add_to_tenant_ids(entity.id()?);
        entity_event.set_action_name(action_name);
        entity_event.set_create_time(create_time.to_string());
        entity_event.set_user_id(user_id);
        entity_event.set_input_entity(entity.clone());
        entity_event.set_billing_account_id(entity.billing_account_id()?);
        entity_event.set_organization_id(entity.organization_id()?);
        entity_event.set_workspace_id(entity.workspace_id()?);
        entity_event.set_integration_id(entity.integration_id()?);
        entity_event.set_integration_service_id(entity.integration_service_id()?);
        entity_event.set_component_id(entity.component_id()?);
        // This can probably become an unwrap once we're all the
        // way finished, and know we won't be going back to the
        // old world without changesets.
        entity_event.set_change_set_id(entity.change_set_id()?.unwrap_or(""));
        entity_event.set_entity_id(entity.id()?);

        Ok(entity_event)
    }

    async fn create(
        db: &Db,
        user_id: &str,
        action_name: &str,
        entity: &Self::Entity,
    ) -> DataResult<Self> {
        let mut entity_event = Self::new(user_id, action_name, entity)?;

        db.validate_and_insert_as_new(&mut entity_event).await?;

        Ok(entity_event)
    }

    async fn create_with_previous_entity(
        db: &Db,
        user_id: &str,
        action_name: &str,
        entity: &Self::Entity,
        previous_entity: Self::Entity,
    ) -> DataResult<Self> {
        let mut entity_event = Self::new(user_id, action_name, entity)?;
        entity_event.set_previous_entity(previous_entity);

        db.validate_and_insert_as_new(&mut entity_event).await?;

        Ok(entity_event)
    }

    fn validate_action_name(&self) -> CeaResult<()> {
        let action_name = self.action_name()?;
        if Self::action_names()
            .iter()
            .find(|&&x| x == action_name)
            .is_none()
        {
            return Err(CeaError::ValidationError(format!(
                "Action name is invalid: {}",
                self.action_name()?
            )));
        }
        Ok(())
    }

    fn validate_input_entity(&self) -> CeaResult<()> {
        if self.input_entity().is_err() {
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

    fn init_output_entity(&mut self) -> DataResult<()> {
        self.set_output_entity(self.input_entity().map(|e| e.clone())?);
        Ok(())
    }

    fn log(&mut self, line: impl Into<String>) {
        self.add_to_output_lines(line);
    }

    fn error_log(&mut self, line: impl Into<String>) {
        self.add_to_error_lines(line);
    }

    fn failed(&mut self, err: impl std::error::Error) -> DataResult<()> {
        let time: DateTime<Utc> = Utc::now();
        let time_string = format!("{}", time);
        self.set_success(false);
        self.set_updated_time(time_string.clone());
        self.set_final_time(time_string);
        self.set_finalized(true);
        self.set_error_message(err.to_string());
        self.add_to_error_lines(format!("*** ERROR STRING ***\n{}", err));
        self.mut_output_entity()?
            .set_entity_state(EntitySiPropertiesEntityState::Error);
        self.log("*** Task failed ***");

        Ok(())
    }

    fn succeeded(&mut self) -> DataResult<()> {
        let time: DateTime<Utc> = Utc::now();
        let time_string = format!("{}", time);
        self.set_success(true);
        self.set_updated_time(time_string.clone());
        self.set_final_time(time_string);
        self.set_finalized(true);
        self.mut_output_entity()?
            .set_entity_state(EntitySiPropertiesEntityState::Ok);
        self.log("*** Task Succeeded ***");

        Ok(())
    }

    async fn send_via_mqtt(&self, mqtt_client: &MqttClient) -> CeaResult<()> {
        let payload = serde_json::to_string(self)?;
        let finalized = self.finalized();

        // When we were so young, and everything was typed.
        //
        //let mut payload = Vec::new();
        //self.encode(&mut payload)?;
        //   self.finalized()
        //};
        if finalized.unwrap_or(false) {
            let msg = mqtt::Message::new(self.result_topic()?, payload.clone(), 0);
            mqtt_client.publish(msg).compat().await?;
            let msg = mqtt::Message::new(self.finalized_topic()?, payload, 2);
            mqtt_client.publish(msg).compat().await?;
        } else {
            let msg = mqtt::Message::new(self.result_topic()?, payload, 0);
            mqtt_client.publish(msg).compat().await?;
        }
        Ok(())
    }

    fn result_topic(&self) -> DataResult<String> {
        let topic = format!(
            "{}/{}/{}/{}/{}/{}/{}/{}/{}/result",
            self.billing_account_id()?,
            self.organization_id()?,
            self.workspace_id()?,
            self.integration_id()?,
            self.integration_service_id()?,
            self.entity_id()?,
            "action",
            self.action_name()?,
            self.id()?,
        );

        Ok(topic)
    }

    fn finalized_topic(&self) -> DataResult<String> {
        let topic = format!(
            "{}/{}/{}/{}/{}/{}/{}/{}/{}/{}/finalized",
            self.billing_account_id()?,
            self.organization_id()?,
            self.workspace_id()?,
            self.integration_id()?,
            self.integration_service_id()?,
            Self::type_name(),
            self.entity_id()?,
            "action",
            self.action_name()?,
            self.id()?,
        );

        Ok(topic)
    }
}
