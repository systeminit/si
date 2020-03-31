use si_cea::binary::server::prelude::*;
use si_kubernetes::model::{Component, EntityEvent};
use si_kubernetes::{Dispatcher, KubernetesDeploymentServer, Service};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    gen_server_binary!(
        name: "Kubernetes",
        dispatcher: Dispatcher,
        component: Component,
        entity_event: EntityEvent,
        service: Service,
        server: KubernetesDeploymentServer
    );
    Ok(())
}
