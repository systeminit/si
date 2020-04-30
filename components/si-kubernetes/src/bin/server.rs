use si_cea::binary::server::prelude::*;
use si_kubernetes::agent::aws;
use si_kubernetes::model::{KubernetesDeploymentComponent, KubernetesDeploymentEntityEvent};
use si_kubernetes::togen::service;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let name = "Kubernetes";

    println!("*** Starting {} ***", name);
    si_cea::binary::server::setup_tracing()?;

    println!("*** Loading settings ***");
    let settings = Settings::new()?;

    println!("*** Connecting to the database ***");
    let db = Db::new(&settings)?;

    println!(
        "*** Migrating {} ***",
        KubernetesDeploymentComponent::type_name()
    );
    KubernetesDeploymentComponent::migrate(&db).await?;

    println!(
        "*** Spawning the {} Agent Server ***",
        KubernetesDeploymentEntityEvent::type_name()
    );
    let mut agent_dispatcher = Dispatcher::default();
    agent_dispatcher.add(&db, aws::dispatcher()).await?;
    let mut agent_server = AgentServer::new(name, agent_dispatcher, &settings);
    tokio::spawn(async move { agent_server.run().await });

    println!(
        "*** Spawning the {} Agent Finalizer ***",
        KubernetesDeploymentEntityEvent::type_name()
    );
    let mut finalizer = AgentFinalizer::new(
        db.clone(),
        KubernetesDeploymentEntityEvent::type_name(),
        &settings,
    );
    tokio::spawn(async move { finalizer.run::<KubernetesDeploymentEntityEvent>().await });

    let addr = format!("0.0.0.0:{}", settings.service.port)
        .parse()
        .unwrap();

    println!("--> {} service listening on {} <--", name, addr);
    tonic::transport::Server::builder()
        .add_service(service::kubernetes(db, &settings).await?)
        .serve(addr)
        .await?;

    Ok(())
}
