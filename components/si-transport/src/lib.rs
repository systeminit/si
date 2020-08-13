mod error;
mod message;
mod metadata;
mod qos;
mod transport;

pub use error::{Error, Result};
pub use message::{Message, TypeHint, WireMessage};
pub use metadata::{
    AgentCommand, AgentCommandHeader, AgentCommandTopic, AgentData, AgentDataHeader,
    AgentDataTopic, Header, Topic,
};
pub use qos::QoS;
pub use transport::{SubscribedTransport, Transport};

pub const TEMP_AGENT_ID: &str = "placeholder_agent_id";
pub const TEMP_AGENT_INSTALLATION_ID: &str = "si";
