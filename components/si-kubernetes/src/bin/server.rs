use anyhow::Context;
use si_cea::binary::server::prelude::*;
use si_kubernetes::agent::aws_eks_kubernetes_deployment;
use si_kubernetes::gen::service::{Server, Service};
use si_kubernetes::model::{KubernetesDeploymentEntityEvent, KubernetesServiceEntityEvent};

use opentelemetry::{api::Provider, sdk};
use opentelemetry_jaeger;
use tracing;
use tracing_opentelemetry::layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{self, fmt, EnvFilter, Registry};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let exporter = opentelemetry_jaeger::Exporter::builder()
        .with_process(opentelemetry_jaeger::Process {
            service_name: "si-kubernetes".into(),
            tags: Vec::new(),
        })
        .init()?;
    let provider = sdk::Provider::builder()
        .with_simple_exporter(exporter)
        .with_config(sdk::Config {
            default_sampler: Box::new(sdk::Sampler::Always),
            ..Default::default()
        })
        .build();

    let tracer = provider.get_tracer("si-kubernetes");

    let fmt_layer = fmt::Layer::default();
    let opentelemetry_layer = layer().with_tracer(tracer);
    let env_filter_layer = EnvFilter::from_default_env();

    let subscriber = Registry::default()
        .with(env_filter_layer)
        .with(fmt_layer)
        .with(opentelemetry_layer);

    tracing::subscriber::set_global_default(subscriber)
        .context("cannot set the global tracing default")?;

    let server_name = "kubernetes";

    println!("*** Starting {} ***", server_name);
    //si_cea::binary::server::setup_tracing()?;

    println!("*** Loading settings ***");
    let settings = si_settings::Settings::new()?;

    println!("*** Connecting to the database ***");
    let db = si_data::Db::new(&settings).context("Cannot connect to the database")?;

    let agent_client = si_cea::AgentClient::new(server_name, &settings).await?;
    let service = Service::new(db.clone(), agent_client);
    println!("*** Migrating so much right now ***");
    service.migrate().await?;

    println!(
        "*** Spawning the {} Agent Server ***",
        KubernetesDeploymentEntityEvent::type_name()
    );
    let mut agent_dispatcher = Dispatcher::default();
    agent_dispatcher
        .add(&db, aws_eks_kubernetes_deployment::dispatcher())
        .await?;
    let mut agent_server = AgentServer::new(server_name, agent_dispatcher, &settings)?;
    tokio::spawn(async move { agent_server.run().await });

    // TODO(fnichol): We need to add an envelope to the payload before activating this code,
    // otherwise both Deployments and Services will be consumed by both Agents
    //
    println!(
        "*** (NOT YET) Spawning the {} Agent Server ***",
        KubernetesServiceEntityEvent::type_name()
    );
    // let mut agent_dispatcher = Dispatcher::default();
    // agent_dispatcher
    //     .add(&db, aws_eks_kubernetes_service::dispatcher())
    //     .await?;
    // let mut agent_server = AgentServer::new(server_name, agent_dispatcher, &settings);
    // tokio::spawn(async move { agent_server.run().await });

    println!(
        "*** Spawning the {} Agent Finalizer ***",
        KubernetesDeploymentEntityEvent::type_name()
    );
    let mut finalizer = AgentFinalizer::new(
        db.clone(),
        KubernetesDeploymentEntityEvent::type_name(),
        &settings,
    )?;
    tokio::spawn(async move { finalizer.run::<KubernetesDeploymentEntityEvent>().await });

    // TODO(fnichol): We need to add an envelope to the payload before activating this code,
    // otherwise both Deployments and Services will be consumed by both Finalizers
    //
    println!(
        "*** (NOT YET) Spawning the {} Agent Finalizer ***",
        KubernetesServiceEntityEvent::type_name()
    );
    // let mut finalizer = AgentFinalizer::new(
    //     db.clone(),
    //     KubernetesServiceEntityEvent::type_name(),
    //     &settings,
    // );
    // tokio::spawn(async move { finalizer.run::<KubernetesServiceEntityEvent>().await });

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
