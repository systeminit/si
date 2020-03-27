use si_aws_eks_cluster_runtime::model::{Component, EntityEvent};
use si_aws_eks_cluster_runtime::{AwsEksClusterRuntimeServer, Dispatcher, Service};
use si_cea::binary::server::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    gen_server_binary!(
        name: "AWS EKS Cluster Runtime",
        dispatcher: Dispatcher,
        component: Component,
        entity_event: EntityEvent,
        service: Service,
        server: AwsEksClusterRuntimeServer
    );
    Ok(())
}
