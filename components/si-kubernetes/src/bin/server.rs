// use si_kubernetes::agent::aws;
// use si_kubernetes::model::{KubernetesDeploymentComponent, KubernetesDeploymentEntityEvent};
// use si_kubernetes::togen::service;

use anyhow::Context;
use si_cea::binary::server::prelude::*;
use si_kubernetes::gen::service::{Server, Service};
use si_kubernetes::model::KubernetesDeploymentEntityEvent;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server_name = "kubernetes";

    println!("*** Starting {} ***", server_name);
    si_cea::binary::server::setup_tracing()?;

    println!("*** Loading settings ***");
    let settings = si_settings::Settings::new()?;

    println!("*** Connecting to the database ***");
    let db = si_data::Db::new(&settings).context("Cannot connect to the database")?;

    let agent_client = si_cea::AgentClient::new(server_name, &settings).await?;
    let service = Service::new(db.clone(), agent_client);
    println!("*** Migrating so much right now ***");
    service.migrate().await?;

    // TODO: fix
    // Add in agent server dispatcher

    // println!(
    //     "*** Spawning the {} Agent Server ***",
    //     KubernetesDeploymentEntityEvent::type_name()
    // );
    // let mut agent_dispatcher = Dispatcher::default();
    // agent_dispatcher.add(&db, aws::dispatcher()).await?;
    // let mut agent_server = AgentServer::new(name, agent_dispatcher, &settings);
    // tokio::spawn(async move { agent_server.run().await });

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

    println!("--> {} service listening on {} <--", server_name, addr);
    tonic::transport::Server::builder()
        .add_service(Server::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
