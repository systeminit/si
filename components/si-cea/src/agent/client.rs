use futures::compat::Future01CompatExt;
use paho_mqtt as mqtt;
use uuid::Uuid;

use std::fmt;
use std::ops::{Deref, DerefMut};

use crate::entity_event::EntityEvent;
use crate::error::Result;

pub use tracing::{debug, debug_span};
pub use tracing_futures::Instrument as _;

#[derive(Debug, Clone)]
pub struct AgentClient {
    pub mqtt: MqttAsyncClientInternal,
}

#[derive(Clone)]
pub struct MqttAsyncClientInternal {
    pub mqtt: mqtt::AsyncClient,
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

impl DerefMut for MqttAsyncClientInternal {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.mqtt
    }
}

impl AgentClient {
    pub async fn new() -> Result<AgentClient> {
        // Create a client & define connect options
        let client_id = format!("agent_client:{}", Uuid::new_v4());

        let cli = mqtt::AsyncClientBuilder::new()
            .server_uri("tcp://localhost:1883")
            .client_id(client_id.as_ref())
            .persistence(false)
            .finalize();

        cli.connect(mqtt::ConnectOptions::new()).compat().await?;

        Ok(AgentClient {
            mqtt: MqttAsyncClientInternal { mqtt: cli },
        })
    }

    pub async fn dispatch(&self, entity_event: &impl EntityEvent) -> Result<()> {
        async {
            entity_event.validate_input_entity()?;
            entity_event.validate_action_name()?;
            self.send(entity_event).await?;
            Ok(())
        }
        .instrument(debug_span!("async_client_dispatch", ?entity_event))
        .await
    }

    // Eventually, we need to be able to dispatch to an individual agent id, so
    // that people can run specific agents for their billing account. We can
    // do that by just putting it in the EntityEvent stuct, and if it is
    // filled in, we use it.
    fn generate_topic(&self, entity_event: &impl EntityEvent) -> String {
        let topic = format!(
            "{}/{}/{}/{}/{}/{}/{}/{}/{}",
            entity_event.billing_account_id(),
            entity_event.organization_id(),
            entity_event.workspace_id(),
            entity_event.integration_id(),
            entity_event.integration_service_id(),
            entity_event.entity_id(),
            "action",
            entity_event.action_name(),
            entity_event.id(),
        );
        topic
    }

    pub async fn send(&self, entity_event: &impl EntityEvent) -> Result<()> {
        async {
            let mut payload = Vec::new();
            entity_event.encode(&mut payload)?;
            // We are very close to the broker - so no need to pretend that we are at
            // risk of not receiving our messages. Right?
            let topic = self.generate_topic(entity_event);
            debug!(?topic, "topic");
            let msg = mqtt::Message::new(self.generate_topic(entity_event), payload, 0);
            self.mqtt.publish(msg).compat().await?;
            Ok(())
        }
        .instrument(debug_span!("async_client_send", ?entity_event))
        .await
    }
}
