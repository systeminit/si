use crate::entity_event::EntityEvent;
use crate::{agent::mqtt::PersistenceType, CeaResult, MqttClient};
use futures::StreamExt;
use si_data::{uuid_string, Db};
use si_settings::Settings;
use tracing::{debug, info, warn};

#[derive(Debug, Clone)]
pub struct AgentFinalizer {
    mqtt: MqttClient,
    entity_event_type_name: String,
    db: Db,
}

impl AgentFinalizer {
    pub fn new(
        db: Db,
        entity_event_type_name: impl Into<String>,
        settings: &Settings,
    ) -> CeaResult<AgentFinalizer> {
        let entity_event_name = entity_event_type_name.into();
        let client_id = format!("agent_finalizer:{}:{}", entity_event_name, uuid_string());

        let mqtt = MqttClient::new()
            .server_uri(settings.vernemq_server_uri())
            .client_id(client_id)
            .persistence(PersistenceType::None)
            .create_client()?;

        Ok(AgentFinalizer {
            mqtt,
            db,
            entity_event_type_name: entity_event_name,
        })
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
        let mut rx = self.mqtt.get_stream(1000);
        println!("Finalizer connecting to the MQTT server...");
        let response = self
            .mqtt
            .default_connect()
            .await?
            .connect_response()
            .expect("should contain a connection response");
        let server_uri = response.server_uri;
        let ver = response.mqtt_version;
        let session_present = response.session_present;
        // Make the connection to the broker
        println!("Connected to: '{}' with MQTT version {}", server_uri, ver);
        if !session_present {
            let (subscribe_channels, subscribe_qos) = self.subscribe_topics();
            self.mqtt
                .subscribe(&subscribe_channels[0], subscribe_qos[0])
                .await?;
        }

        // Just wait for incoming messages by running the receiver stream
        // in this thread.
        println!("Waiting for messages...");
        while let Some(stream_msg) = rx.next().await {
            let msg = match stream_msg {
                Some(msg) => msg,
                None => {
                    info!("lost connection to mqtt, attempting to reconnect");
                    while let Err(err) = self.mqtt.reconnect().await {
                        warn!(?err, "error reconnecting to MQTT");
                        tokio::time::delay_for(std::time::Duration::from_millis(1000)).await;
                    }
                    continue;
                }
            };
            let payload_str = match std::str::from_utf8(msg.payload()) {
                Ok(payload_str) => payload_str,
                Err(err) => {
                    debug!(?err, "utf8 error deserializing message");
                    continue;
                }
            };
            let entity_event: T = match serde_json::from_str(payload_str) {
                Ok(entity_event) => entity_event,
                Err(err) => {
                    debug!(?err, "error deserializing json from payload buffer");
                    continue;
                }
            };

            // Alas, poor protocol buffer - I knew him well.
            //
            //let entity_event = match <T as Message>::decode(msg.payload()) {
            //    Ok(e) => e,
            //    Err(err) => {
            //        debug!(?err, "deserialzing error - bad message");
            //        continue;
            //    }
            //};
            let mut self_ref: AgentFinalizer = self.clone();
            tokio::spawn(async move { self_ref.dispatch(entity_event).await });
        }
        Ok(())
    }
}
