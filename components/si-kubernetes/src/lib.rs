pub mod protobuf {
    tonic::include_proto!("si.kubernetes");
    pub mod deployment {
        tonic::include_proto!("si.kubernetes.deployment");
    }
}

pub mod agent;
pub mod model;
pub mod service;

pub use agent::Dispatcher;
pub use protobuf::deployment::kubernetes_deployment_server::KubernetesDeploymentServer;
pub use service::Service;
