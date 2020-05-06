//! This is the Component/Entity/Actor (CEA) framework for SI.
//!
//! It makes it easy to produce new components, entities, and actors!
//!

pub mod protobuf {
    tonic::include_proto!("si.cea");
}

pub mod agent;
pub mod error;
pub mod service;

pub mod binary;
pub mod component;
pub mod entity;
pub mod entity_event;
pub mod list;

pub use crate::agent::client::AgentClient;
pub use crate::agent::mqtt::MqttClient;
pub use crate::entity::{Entity, EntitySiPropertiesEntityState};
pub use crate::entity_event::EntityEvent;
pub use crate::error::{CeaError, CeaResult};

// pub use crate::list::{ListReply, ListRequest};
// pub use crate::agent::dispatch::Dispatcher;
// pub use crate::agent::finalizer::AgentFinalizer;
// pub use crate::agent::server::AgentServer;
// pub use crate::agent::utility::spawn_command::{spawn_command, CaptureOutput, CommandResult};
// pub use crate::binary::server::setup_tracing;
// pub use crate::component::{Component, MigrateComponent};
// pub use crate::entity::{Entity, EntityState};
// pub use crate::entity_event::EntityEvent;
// pub use crate::service::auth::Authentication;
