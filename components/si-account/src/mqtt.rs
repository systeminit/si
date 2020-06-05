use futures::compat::Future01CompatExt;
use si_data::{uuid_string, Storable};
use si_settings::Settings;

use serde::{Deserialize, Serialize};
use tracing::{debug, debug_span};
use tracing_futures::Instrument as _;

use crate::error::{AccountError, Result};

enum ChangeSetAction {
    Create,
    Edit,
    Action(String),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChangeSetExecuteEnvelope {
    change_set_action: ChangeSetAction,
    concrete_type: String,
    json: serde_json::Value,
}

impl ChangeSetExecuteEnvelope {
    fn new<T: Serialize + Storable>(
        change_set_action: ChangeSetAction,
        concrete_type: impl Into<String>,
        data: T,
    ) -> ChangeSetExecuteEnvelope {
        ChangeSetExecuteEnvelope {
            change_set_action,
            concrete_type: concrete_type.into(),
            json: serde_json::to_string(data),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChangeSetAgent {
    pub mqtt: MqttClient,
}

impl ChangeSetAgent {
    pub async fn new(name: &str, settings: &Settings) -> Result<ChangeSetAgent> {
        // Create a client & define connect options
        let client_id = format!("agent_change_set:{}:{}", name, uuid_string());

        let mqtt = MqttClient::new()
            .server_uri(settings.vernemq_server_uri().as_ref())
            .client_id(client_id.as_ref())
            .persistence(false)
            .finalize();
        mqtt.default_connect().await?;

        Ok(AgentClient { mqtt })
    }

    pub async fn dispatch(&self, entity_event: &impl EntityEvent) -> CeaResult<()> {
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
    fn generate_topic(&self, entity_event: &impl EntityEvent) -> CeaResult<String> {
        let topic = format!(
            "{}/{}/{}/{}/{}/{}/{}/{}/{}",
            entity_event.billing_account_id()?,
            entity_event.organization_id()?,
            entity_event.workspace_id()?,
            entity_event.integration_id()?,
            entity_event.integration_service_id()?,
            entity_event.entity_id()?,
            "action",
            entity_event.action_name()?,
            entity_event.id()?,
        );

        Ok(topic)
    }

    pub async fn send(&self, entity_event: &impl EntityEvent) -> CeaResult<()> {
        async {
            let mut payload = Vec::new();
            entity_event.encode(&mut payload)?;
            // We are very close to the broker - so no need to pretend that we are at
            // risk of not receiving our messages. Right?
            let topic = self.generate_topic(entity_event)?;
            debug!(?topic, "topic");
            let msg = Message::new(self.generate_topic(entity_event)?, payload, 0);
            self.mqtt.publish(msg).compat().await?;
            Ok(())
        }
        .instrument(debug_span!("async_client_send", ?entity_event))
        .await
    }
}

use paho_mqtt::{AsyncClient, AsyncClientBuilder, ConnectOptions, MqttError};
use std::fmt;
use std::ops::{Deref, DerefMut};
use std::result;

pub use paho_mqtt::Message;

#[derive(Clone)]
pub struct MqttClient {
    inner: AsyncClient,
}

impl MqttClient {
    pub fn new() -> ClientBuilder {
        ClientBuilder {
            inner: AsyncClientBuilder::new(),
        }
    }

    pub async fn default_connect(&self) -> result::Result<(String, i32, bool), MqttError> {
        self.inner.connect(ConnectOptions::new()).compat().await
    }
}

impl std::fmt::Debug for MqttClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MqttClient")
            .field("inner", &"mqtt::AsyncClient")
            .finish()
    }
}

impl Deref for MqttClient {
    type Target = AsyncClient;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for MqttClient {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub struct ClientBuilder {
    inner: AsyncClientBuilder,
}

impl ClientBuilder {
    /// Sets the address for the MQTT broker/server.
    ///
    /// # Arguments
    ///
    /// `server_uri` The address of the MQTT broker. It takes the form
    ///              <i>protocol://host:port</i>, where <i>protocol</i> must
    ///              be <i>tcp</i> or <i>ssl</i>. For <i>host</i>, you can
    ///              specify either an IP address or a host name. For instance,
    ///              to connect to a server running on the local machines with
    ///              the default MQTT port, specify <i>tcp://localhost:1883</i>.
    pub fn server_uri(&mut self, server_uri: &str) -> &mut ClientBuilder {
        self.inner.server_uri(server_uri);
        self
    }

    /// Sets the client identifier for connection to the broker.
    ///
    /// # Arguments
    ///
    /// `client_id` A unique identifier string to be passed to the broker
    ///             when the connection is made. This must be a UTF-8 encoded
    ///             string. If it is empty, the broker will create and assign
    ///             a unique name for the client.
    pub fn client_id(&mut self, client_id: &str) -> &mut ClientBuilder {
        self.inner.client_id(client_id);
        self
    }

    /// Turns default file persistence on or off.
    /// When turned on, the client will use the default, file-based,
    /// persistence mechanism. This stores information about in-flight
    /// messages in persistent storage on the file system, and provides
    /// some protection against message loss in the case of unexpected
    /// failure.
    /// When turned off, the client uses in-memory persistence. If the
    /// client crashes or system power fails, the client could lose
    /// messages.
    ///
    /// # Arguments
    ///
    /// `on` Whether to turn on file-based message persistence.
    pub fn persistence(&mut self, on: bool) -> &mut ClientBuilder {
        self.inner.persistence(on);
        self
    }

    /// Enables or disables off-line buffering of out-going messages when
    /// the client is disconnected.
    ///
    /// # Arguments
    ///
    /// `on` Whether or not the application is allowed to publish messages
    ///      if the client is off-line.
    pub fn offline_buffering(&mut self, on: bool) -> &mut ClientBuilder {
        self.inner.offline_buffering(on);
        self
    }

    /// Enables off-line buffering of out-going messages when the client is
    /// disconnected and sets the maximum number of messages that can be
    /// buffered.
    ///
    /// # Arguments
    ///
    /// `max_buffered_msgs` The maximum number of messages that the client
    ///                     will buffer while off-line.
    pub fn max_buffered_messages(&mut self, max_buffered_messages: i32) -> &mut ClientBuilder {
        self.inner.max_buffered_messages(max_buffered_messages);
        self
    }

    /// Finalize the builder and create an asynchronous client.
    pub fn finalize(&self) -> MqttClient {
        MqttClient {
            inner: self.inner.finalize(),
        }
    }
}
