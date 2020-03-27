use crate::agent::client::MqttAsyncClientInternal;
use crate::entity_event::EntityEvent;
use crate::error::CeaResult;
use futures::compat::{Future01CompatExt, Stream01CompatExt};
use futures::StreamExt;
use paho_mqtt as mqtt;
use prost::Message;
use si_data::Db;
use si_settings::Settings;
use tracing::debug;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AgentFinalizer {
    mqtt: MqttAsyncClientInternal,
    entity_event_type_name: String,
    db: Db,
}

impl AgentFinalizer {
    pub fn new(
        db: Db,
        entity_event_type_name: impl Into<String>,
        settings: &Settings,
    ) -> AgentFinalizer {
        let entity_event_name = entity_event_type_name.into();
        let client_id = format!("agent_finalizer:{}:{}", entity_event_name, Uuid::new_v4());

        let cli = mqtt::AsyncClientBuilder::new()
            .server_uri(settings.vernemq_server_uri().as_ref())
            .client_id(client_id.as_ref())
            .persistence(false)
            .finalize();

        AgentFinalizer {
            mqtt: MqttAsyncClientInternal { mqtt: cli },
            db,
            entity_event_type_name: entity_event_name,
        }
    }

    fn subscribe_topics(&self) -> (Vec<String>, Vec<i32>) {
        let finalized_topic = format!(
            "$share/finalizers/+/+/+/+/+/{}/+/action/+/+/finalized",
            self.entity_event_type_name
        );
        (vec![finalized_topic], vec![2])
    }

    pub async fn dispatch(&mut self, entity_event: impl EntityEvent) -> CeaResult<()> {
        debug!("updating_entity_event");
        self.db.upsert(&entity_event).await?;
        // No output entity means we didn't mutate anything
        if let Some(entity) = entity_event.output_entity() {
            debug!("updating_entity");
            self.db.upsert(entity).await?;
        }
        Ok(())
    }

    pub async fn run<T: EntityEvent + 'static>(&mut self) -> CeaResult<()> {
        // Whats the right value? Who knows? God only knows. Ask the Beach Boys.
        let mut rx = self.mqtt.get_stream(1000).compat();
        println!("Finalizer connecting to the MQTT server...");
        let (server_uri, ver, session_present) = self
            .mqtt
            .connect(mqtt::ConnectOptions::new())
            .compat()
            .await?;
        // Make the connection to the broker
        println!("Connected to: '{}' with MQTT version {}", server_uri, ver);
        if !session_present {
            let (subscribe_channels, subscribe_qos) = self.subscribe_topics();
            self.mqtt
                .subscribe(&subscribe_channels[0], subscribe_qos[0])
                .compat()
                .await?;
        }

        // Just wait for incoming messages by running the receiver stream
        // in this thread.
        println!("Waiting for messages...");
        while let Some(stream_msg) = rx.next().await {
            let msg = match stream_msg {
                Ok(maybe_msg) => match maybe_msg {
                    Some(msg) => msg,
                    None => {
                        debug!("you don't have a message, eh?");
                        continue;
                    }
                },
                Err(_) => {
                    debug!("whats up?");
                    continue;
                }
            };
            let entity_event = match <T as Message>::decode(msg.payload()) {
                Ok(e) => e,
                Err(err) => {
                    debug!(?err, "deserialzing error - bad message");
                    continue;
                }
            };
            let mut self_ref: AgentFinalizer = self.clone();
            tokio::spawn(async move { self_ref.dispatch(entity_event).await });
        }
        Ok(())
    }
}
