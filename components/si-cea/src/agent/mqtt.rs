use futures::compat::Future01CompatExt;
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
