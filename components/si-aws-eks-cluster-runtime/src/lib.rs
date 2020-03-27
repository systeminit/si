pub mod protobuf {
    tonic::include_proto!("si.aws_eks_cluster_runtime");
}

pub mod agent;
pub mod model;
pub mod service;

pub use agent::Dispatcher;
pub use protobuf::aws_eks_cluster_runtime_server::AwsEksClusterRuntimeServer;
pub use service::Service;
