use crate::agent::dispatch::{Dispatch, SubscribeKeys};
use crate::entity_event::EntityEvent;
use crate::{CeaResult, MqttClient};
use futures::compat::{Future01CompatExt, Stream01CompatExt};
use futures::StreamExt;
use prost::Message;
use si_data::uuid_string;
use si_settings::Settings;
use tokio;
use tracing::{debug, debug_span, warn};
use tracing_futures::Instrument as _;

pub struct AgentServer<
    EE: EntityEvent,
    D: Dispatch<EntityEvent = EE> + SubscribeKeys + Send + Sync + Clone + 'static,
> {
    pub mqtt: MqttClient,
    pub name: String,
    pub dispatch: D,
}

impl<
        EE: EntityEvent,
        D: Dispatch<EntityEvent = EE> + SubscribeKeys + Send + Sync + Clone + 'static,
    > AgentServer<EE, D>
{
    pub fn new(name: impl Into<String>, dispatch: D, settings: &Settings) -> AgentServer<EE, D> {
        let name = name.into();

        let client_id = format!("agent_server:{}:{}", name.clone(), uuid_string());

        let mqtt = MqttClient::new()
            .server_uri(settings.vernemq_server_uri().as_ref())
            .client_id(client_id.as_ref())
            .persistence(false)
            .finalize();

        let server: AgentServer<EE, D> = AgentServer {
            name,
            mqtt,
            dispatch,
        };
        server
    }

    fn subscribe_topics(&self) -> Vec<String> {
        self.dispatch
            .subscribe_keys()
            .iter()
            .map(|key| {
                format!(
                    "+/+/+/+/{}/+/action/{}/+",
                    key.integration_service_id(),
                    key.action_name()
                )
            })
            .collect()
    }

    pub async fn run(&mut self) -> CeaResult<()> {
        // Whats the right value? Who knows? God only knows. Ask the Beach Boys.
        let mut rx = self.mqtt.get_stream(1000).compat();
        println!("Connecting to the MQTT server...");
        let (server_uri, ver, session_present) = self.mqtt.default_connect().await?;
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
                    if let Err(err) = entity_event.input_entity() {
                        warn!(?err, "missing input entity on event");
                        return;
                    }
                    if let Err(err) = entity_event.init_output_entity() {
                        warn!(?err, "cannot initialize output entity");
                        return;
                    }
                    debug!(?entity_event, "dispatch");

                    // TODO: fix
                    // Setup tracing *outside* the dispatch function call

                    match dispatch.dispatch(&mqtt_client, &mut entity_event).await {
                        Ok(()) => {
                            debug!(?entity_event, "success");
                            if let Err(err) = entity_event.succeeded() {
                                warn!(?err, "error setting entity_event to succeeded");
                                return;
                            }
                        }
                        Err(e) => {
                            debug!(?entity_event, "failed");
                            if let Err(err) = entity_event.failed(e) {
                                warn!(?err, "error setting entity_event to failed");
                                return;
                            }
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
