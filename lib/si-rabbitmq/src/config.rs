use serde::{Deserialize, Serialize};

/// The configuration settings for the si-rabbitmq [`Environment`](`crate::Environment`)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    host: String,
    username: String,
    password: String,
    port: u16,
    stream_prefix: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: "localhost".into(),
            username: "guest".into(),
            password: "guest".into(),
            port: 5552,
            stream_prefix: None,
        }
    }
}

impl Config {
    /// Create a new config for the rabbitmq [`Environment`](`crate::Environment`)
    pub fn new(
        host: String,
        username: String,
        password: String,
        port: u16,
        stream_prefix: Option<String>,
    ) -> Self {
        Self {
            host,
            username,
            password,
            port,
            stream_prefix,
        }
    }

    /// The hostname of the rabbitmq stream server we will connect to
    pub fn host(&self) -> &str {
        self.host.as_str()
    }

    /// The rabbitmq username
    pub fn username(&self) -> &str {
        self.username.as_str()
    }

    /// The rabbitmq password
    pub fn password(&self) -> &str {
        self.password.as_str()
    }

    /// The port of the rabbitmq stream server we will connect to (usually 5552)
    pub fn port(&self) -> u16 {
        self.port
    }

    /// The stream prefix to be used when creating, using and deleting rabbitmq streams
    pub fn stream_prefix(&self) -> Option<&str> {
        self.stream_prefix.as_deref()
    }

    /// Set the stream prefix on the config
    pub fn set_stream_prefix(&mut self, stream_prefix: impl Into<String>) -> &mut Self {
        self.stream_prefix = Some(stream_prefix.into());
        self
    }
}
