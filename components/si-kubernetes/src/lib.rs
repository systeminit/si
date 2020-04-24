pub mod protobuf {
    tonic::include_proto!("si.kubernetes");
}

// pub mod agent;
pub mod agent_v2;
mod gen;
pub mod model;
pub mod service;

// pub use agent::Dispatcher;
pub use protobuf::kubernetes_server::KubernetesServer;
pub use service::Service;
