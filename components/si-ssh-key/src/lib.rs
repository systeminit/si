pub mod protobuf {
    tonic::include_proto!("si.ssh_key");
}

pub mod agent;
pub mod error;
pub mod model;
mod serde_enum;
pub mod service;

pub use agent::Dispatcher;
pub use model::component::Component;
pub use model::entity::EntityEvent;
pub use protobuf::ssh_key_server::SshKeyServer;
pub use service::Service;
