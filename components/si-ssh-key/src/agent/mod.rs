use futures::compat::Future01CompatExt;
use paho_mqtt as mqtt;
use prost::Message;

use si_data::Db;

use std::fmt;
use std::ops::Deref;

use crate::error::{Result, SshKeyError};
use crate::model::entity::EntityEvent;

mod server {}

#[derive(Debug)]
pub struct AgentClient {
    mqtt: MqttAsyncClientInternal,
}

struct MqttAsyncClientInternal {
    mqtt: mqtt::AsyncClient,
}

impl std::fmt::Debug for MqttAsyncClientInternal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MQTT Async Client")
    }
}

impl Deref for MqttAsyncClientInternal {
    type Target = mqtt::AsyncClient;

    fn deref(&self) -> &Self::Target {
        &self.mqtt
    }
}

impl AgentClient {
    pub async fn new() -> Result<AgentClient> {
        // Create a client & define connect options
        let cli = mqtt::AsyncClient::new("tcp://localhost:1883")?;

        cli.connect(mqtt::ConnectOptions::new()).compat().await?;

        Ok(AgentClient {
            mqtt: MqttAsyncClientInternal { mqtt: cli },
        })
    }

    pub async fn dispatch(&self, entity_event: &EntityEvent) -> Result<()> {
        match &entity_event.action_name[..] {
            "create" => self.send(entity_event).await,
            _ => Err(SshKeyError::InvalidEntityEventInvalidActionName),
        }
    }

    // Eventually, we need to be able to dispatch to an individual agent id, so
    // that people can run specific agents for their billing account. We can
    // do that by just putting it in the EntityEvent stuct, and if it is
    // filled in, we use it.
    fn generate_topic(&self, entity_event: &EntityEvent) -> String {
        let topic = format!(
            "{}/{}/{}/{}/{}/{}/{}/{}/{}",
            entity_event.billing_account_id,
            entity_event.organization_id,
            entity_event.workspace_id,
            entity_event.integration_id,
            entity_event.integration_service_id,
            entity_event.entity_id,
            "action",
            entity_event.action_name,
            entity_event.id,
        );
        topic
    }

    pub async fn send(&self, entity_event: &EntityEvent) -> Result<()> {
        let mut payload = Vec::new();
        entity_event.encode(&mut payload)?;
        // We are very close to the broker - so no need to pretend that we are at
        // risk of not receiving our messages. Right?
        let msg = mqtt::Message::new(self.generate_topic(&entity_event), payload, 0);
        self.mqtt.publish(msg).compat().await?;
        Ok(())
    }
}
