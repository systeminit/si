use paho_mqtt::{AsyncClient, ConnectOptions, Error, ServerResponse};
use std::fmt;
use std::ops::{Deref, DerefMut};
use std::result;

pub use paho_mqtt::{ClientPersistence, Message, PersistenceType, UserData};

#[derive(Clone)]
pub struct MqttClient {
    inner: AsyncClient,
}

impl MqttClient {
    pub fn new() -> CreateOptionsBuilder {
        CreateOptionsBuilder {
            inner: paho_mqtt::CreateOptionsBuilder::new(),
        }
    }

    pub async fn default_connect(&self) -> result::Result<ServerResponse, Error> {
        self.inner.connect(ConnectOptions::new()).await
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

pub struct CreateOptionsBuilder {
    inner: paho_mqtt::CreateOptionsBuilder,
}

impl CreateOptionsBuilder {
    /// Sets the the URI to the MQTT broker.
    /// Alternately, the application can specify multiple servers via the
    /// connect options.
    ///
    /// # Arguments
    ///
    /// `server_uri` The URI string to specify the server in the form
    ///              _protocol://host:port_, where the protocol can be
    ///              _tcp_ or _ssl_, and the host can be an IP address
    ///              or domain name.
    pub fn server_uri(mut self, server_uri: impl Into<String>) -> Self {
        self.inner = self.inner.server_uri(server_uri);
        self
    }

    /// Sets the client identifier string that is sent to the server.
    /// The client ID is a unique name to identify the client to the server,
    /// which can be used if the client desires the server to hold state
    /// about the session. If the client requests a clean sesstion, this can
    /// be an empty string.
    ///
    /// The broker is required to honor a client ID of up to 23 bytes, but
    /// could honor longer ones, depending on the broker.
    ///
    /// Note that if this is an empty string, the clean session parameter
    /// *must* be set to _true_.
    ///
    /// # Arguments
    ///
    /// `client_id` A UTF-8 string identifying the client to the server.
    pub fn client_id(mut self, client_id: impl Into<String>) -> Self {
        self.inner = self.inner.client_id(client_id);
        self
    }

    /// Sets the type of persistence used by the client.
    /// The default is for the library to automatically use file persistence,
    /// although this can be turned off by specify `None` for a more
    /// performant, though possibly less reliable system.
    ///
    /// # Arguments
    ///
    /// `persist` The type of persistence to use.
    pub fn persistence(mut self, persist: PersistenceType) -> Self {
        self.inner = self.inner.persistence(persist);
        self
    }

    /// Sets a user-defined persistence store.
    /// This sets the persistence to use a custom one defined by the
    /// application. This can be anything that implements the
    /// `ClientPersistence` trait.
    ///
    /// # Arguments
    ///
    /// `persist` An application-defined custom persistence store.
    pub fn user_persistence<T>(mut self, persistence: T) -> Self
    where
        T: ClientPersistence + 'static,
    {
        self.inner = self.inner.user_persistence(persistence);
        self
    }

    /// Sets the maximum number of messages that can be buffered for delivery
    /// when the client is off-line.
    /// The client has limited support for bufferering messages when the
    /// client is temporarily disconnected. This specifies the maximum number
    /// of messages that can be buffered.
    ///
    /// # Arguments
    ///
    /// `n` The maximum number of messages that can be buffered. Setting this
    ///     to zero disables off-line buffering.
    pub fn max_buffered_messages(mut self, n: i32) -> Self {
        self.inner = self.inner.max_buffered_messages(n);
        self
    }

    /// Sets the version of MQTT to use on the connect.
    ///
    /// # Arguments
    ///
    /// `ver` The version of MQTT to use when connecting to the broker.
    ///       * (0) try the latest version (3.1.1) and work backwards
    ///       * (3) only try v3.1
    ///       * (4) only try v3.1.1
    ///       * (5) only try v5
    ///
    pub fn mqtt_version(mut self, ver: u32) -> Self {
        self.inner = self.inner.mqtt_version(ver);
        self
    }

    /// Sets the uer-defined data structure for the client.
    pub fn user_data(mut self, data: UserData) -> Self {
        self.inner = self.inner.user_data(data);
        self
    }

    /// Finalize the builder and create an asynchronous client.
    pub fn create_client(self) -> paho_mqtt::Result<MqttClient> {
        Ok(MqttClient {
            inner: self.inner.create_client()?,
        })
    }
}
