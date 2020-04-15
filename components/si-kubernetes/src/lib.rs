pub mod protobuf {
    pub mod kubernetes {
        tonic::include_proto!("si.kubernetes");
    }

    pub mod kubernetes_deployment {
        tonic::include_proto!("si.kubernetes_deployment");
    }
}

// pub mod agent;
mod gen;
pub mod model;
// pub mod service;
//
// pub use agent::Dispatcher;
// pub use protobuf::deployment::kubernetes_deployment_server::KubernetesDeploymentServer;
// pub use service::Service;
