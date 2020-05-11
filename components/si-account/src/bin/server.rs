use anyhow::{Context, Result};
use opentelemetry::{api::Provider, sdk};
use opentelemetry_jaeger;
use si_data::Db;
use si_settings::Settings;
use tokio;
use tonic::transport::Server;
use tracing;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{self, fmt, EnvFilter, Registry};

use si_account::{gen::service::Service, protobuf::account_server::AccountServer};

async fn run() -> Result<()> {
    let settings = Settings::new()?;

    let db = Db::new(&settings).context("Cannot connect to the database")?;
    println!("*** Creating indexes ***");
    db.create_indexes().await?;

    let service = Service::new(db);
    println!("*** Migrating so much right now ***");
    service.migrate().await?;

    let listen_string = format!("0.0.0.0:{}", settings.service.port);

    let addr = listen_string.parse().unwrap();

    println!("--> Account service listening on {} <--", addr);
    println!("--> Let us make users and stuff <--");

    Server::builder()
        .add_service(AccountServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let exporter = opentelemetry_jaeger::Exporter::builder()
        .with_process(opentelemetry_jaeger::Process {
            service_name: "si-account".into(),
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

    let tracer = provider.get_tracer("si-account");

    let fmt_layer = fmt::Layer::default();
    let opentelemetry_layer = OpenTelemetryLayer::with_tracer(tracer);
    let env_filter_layer = EnvFilter::from_default_env();

    let subscriber = Registry::default()
        .with(env_filter_layer)
        .with(fmt_layer)
        .with(opentelemetry_layer);

    tracing::subscriber::set_global_default(subscriber)
        .context("cannot set the global tracing defalt")?;

    let handle = tokio::spawn(async move { run().await });

    handle.await??;
    Ok(())
}
