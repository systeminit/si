use anyhow::Context;
use opentelemetry::{api::Provider, sdk};
use si_cea::binary::server::prelude::*;
use si_core::agent::global_core_service;
use si_core::gen::service::{Server, Service};
use si_core::model::{ApplicationEntityEvent, ServiceEntityEvent, SystemEntityEvent};
use tracing_opentelemetry::layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{self, fmt, EnvFilter, Registry};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let exporter = opentelemetry_jaeger::Exporter::builder()
        .with_process(opentelemetry_jaeger::Process {
            service_name: "si-core".into(),
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

    let tracer = provider.get_tracer("si-core");

    let fmt_layer = fmt::Layer::default();
    let opentelemetry_layer = layer().with_tracer(tracer);
    let env_filter_layer = EnvFilter::from_default_env();

    let subscriber = Registry::default()
        .with(env_filter_layer)
        .with(fmt_layer)
        .with(opentelemetry_layer);

    tracing::subscriber::set_global_default(subscriber)
        .context("cannot set the global tracing default")?;

    let server_name = "core";

    println!("*** Starting {} ***", server_name);

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
        ServiceEntityEvent::type_name()
    );
    let mut agent_dispatcher = Dispatcher::default();
    agent_dispatcher
        .add(&db, global_core_service::dispatcher())
        .await?;
    let mut agent_server = AgentServer::new(server_name, agent_dispatcher, &settings)?;
    tokio::spawn(async move { agent_server.run().await });

    // TODO(fnichol): We need to add an envelope to the payload before activating this code,
    // otherwise both Deployments and Services will be consumed by both Agents
    //
    println!(
        "*** (NOT YET) Spawning the {} Agent Server ***",
        ApplicationEntityEvent::type_name()
    );
    // let mut agent_dispatcher = Dispatcher::default();
    // agent_dispatcher
    //     .add(&db, global_core_application::dispatcher())
    //     .await?;
    // let mut agent_server = AgentServer::new(server_name, agent_dispatcher, &settings);
    // tokio::spawn(async move { agent_server.run().await });

    // TODO(fnichol): We need to add an envelope to the payload before activating this code,
    // otherwise both Deployments and Services will be consumed by both Agents
    //
    println!(
        "*** (NOT YET) Spawning the {} Agent Server ***",
        SystemEntityEvent::type_name()
    );
    // let mut agent_dispatcher = Dispatcher::default();
    // agent_dispatcher
    //     .add(&db, global_core_system::dispatcher())
    //     .await?;
    // let mut agent_server = AgentServer::new(server_name, agent_dispatcher, &settings);
    // tokio::spawn(async move { agent_server.run().await });

    println!(
        "*** Spawning the {} Agent Finalizer ***",
        ServiceEntityEvent::type_name()
    );
    let mut finalizer =
        AgentFinalizer::new(db.clone(), ServiceEntityEvent::type_name(), &settings)?;
    tokio::spawn(async move { finalizer.run::<ServiceEntityEvent>().await });

    // TODO(fnichol): We need to add an envelope to the payload before activating this code,
    // otherwise both Deployments and Services will be consumed by both Finalizers
    //
    println!(
        "*** (NOT YET) Spawning the {} Agent Finalizer ***",
        ApplicationEntityEvent::type_name()
    );
    // let mut finalizer = AgentFinalizer::new(
    //     db.clone(),
    //     ApplicationEntityEvent::type_name(),
    //     &settings,
    // );
    // tokio::spawn(async move { finalizer.run::<ApplicationEntityEvent>().await });

    // TODO(fnichol): We need to add an envelope to the payload before activating this code,
    // otherwise both Deployments and Services will be consumed by both Finalizers
    //
    println!(
        "*** (NOT YET) Spawning the {} Agent Finalizer ***",
        SystemEntityEvent::type_name()
    );
    // let mut finalizer = AgentFinalizer::new(
    //     db.clone(),
    //     SystemEntityEvent::type_name(),
    //     &settings,
    // );
    // tokio::spawn(async move { finalizer.run::<SystemEntityEvent>().await });

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
