use futures::compat::{Future01CompatExt, Stream01CompatExt};
use futures::StreamExt;
use paho_mqtt as mqtt;
use prost::Message;
use tokio;
use tracing::{debug, debug_span, warn};
use tracing_futures::Instrument as _;
use uuid::Uuid;

use si_settings::Settings;

use crate::agent::client::MqttAsyncClientInternal;
use crate::agent::dispatch::Dispatch;
use crate::entity_event::EntityEvent;
use crate::error::Result;

use std::marker::PhantomData;

pub struct AgentServer<EE: EntityEvent, D: Dispatch<EE> + Send + Sync + 'static + Clone> {
    pub mqtt: MqttAsyncClientInternal,
    pub name: String,
    pub dispatch: D,
    phantom: PhantomData<EE>,
}

impl<EE: EntityEvent, D: Dispatch<EE> + Send + Sync + 'static + Clone> AgentServer<EE, D> {
    pub fn new(name: impl Into<String>, dispatch: D, settings: &Settings) -> AgentServer<EE, D> {
        let name = name.into();

        let client_id = format!("agent_server:{}:{}", name.clone(), Uuid::new_v4());

        let cli = mqtt::AsyncClientBuilder::new()
            .server_uri(settings.vernemq_server_uri().as_ref())
            .client_id(client_id.as_ref())
            .persistence(false)
            .finalize();

        let server: AgentServer<EE, D> = AgentServer {
            name: name.into(),
            mqtt: MqttAsyncClientInternal { mqtt: cli },
            dispatch,
            phantom: PhantomData,
        };
        server
    }

    fn subscribe_topics(&self) -> Vec<String> {
        let mut topics = Vec::new();
        for (integration_service_id, action_name) in self.dispatch.keys() {
            let inbound_channel = format!(
                "+/+/+/+/{}/+/action/{}/+",
                integration_service_id, action_name
            );
            topics.push(inbound_channel);
        }
        topics
    }

    pub async fn run(&mut self) -> Result<()> {
        // Whats the right value? Who knows? God only knows. Ask the Beach Boys.
        let mut rx = self.mqtt.get_stream(1000).compat();
        println!("Connecting to the MQTT server...");
        let (server_uri, ver, session_present) = self
            .mqtt
            .connect(mqtt::ConnectOptions::new())
            .compat()
            .await?;
        // Make the connection to the broker
        println!("Connected to: '{}' with MQTT version {}", server_uri, ver);
        if !session_present {
            for topic in self.subscribe_topics() {
                println!("Subscribing to {}", topic);
                self.mqtt.subscribe(&topic, 2).compat().await?;
            }
        }

        // Just wait for incoming messages by running the receiver stream
        // in this thread.
        println!("Server is waiting for messages...");
        while let Some(stream_msg) = rx.next().await {
            let dispatch = self.dispatch.clone();
            let mqtt_client = self.mqtt.clone();
            tokio::spawn(async move {
                async {
                    let msg = match stream_msg {
                        Ok(maybe_msg) => match maybe_msg {
                            Some(msg) => msg,
                            None => {
                                warn!("no message; aborting");
                                return;
                            }
                        },
                        Err(e) => {
                            warn!(?e, "error fetching message from stream, aborting");
                            return;
                        }
                    };

                    let mut entity_event = match <EE as Message>::decode(msg.payload()) {
                        Ok(e) => e,
                        Err(err) => {
                            warn!(?err, "deserializing error - bad message");
                            return;
                        }
                    };
                    if entity_event.input_entity().is_none() {
                        warn!(?entity_event, "Missing input entity on event");
                        return;
                    }
                    let integration_service_id = entity_event.integration_service_id().to_string();
                    let action_name = entity_event.action_name().to_string();

                    debug!(
                        ?integration_service_id,
                        ?entity_event,
                        ?action_name,
                        "dispatch"
                    );

                    match dispatch
                        .dispatch(
                            &mqtt_client,
                            &mut entity_event,
                            integration_service_id,
                            action_name,
                        )
                        .await
                    {
                        Ok(()) => {
                            debug!(?entity_event, "success");
                            entity_event.succeeded();
                        }
                        Err(e) => {
                            debug!(?entity_event, "failed");
                            entity_event.failed(e);
                        }
                    }
                    match entity_event.send_via_mqtt(&mqtt_client).await {
                        Ok(()) => (),
                        Err(e) => warn!(?e, "failed to send via mqtt"),
                    }
                    ()
                }
                .instrument(debug_span!("agent_server_message_recevied"))
                .await
            });
        }
        Ok(())
    }
}
