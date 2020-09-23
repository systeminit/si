use anyhow::Context;
use nats;
use opentelemetry::{api::Provider, sdk};
use tracing;
use tracing_opentelemetry::layer;
use tracing_subscriber::{self, fmt, layer::SubscriberExt, EnvFilter, Registry};

use std::env;

use si_sdf::start;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if env::var_os("RUST_LOG").is_none() {
        env::set_var("RUST_LOG", "entities=info");
    }

    let server_name = "si-sdf";
    let service_name = "si-sdf";
    println!("*** Starting {} ***", server_name);
    let exporter = opentelemetry_jaeger::Exporter::builder()
        .with_process(opentelemetry_jaeger::Process {
            service_name: service_name.into(),
            tags: Vec::new(),
        })
        .init()?;
    let provider = sdk::Provider::builder()
        .with_simple_exporter(exporter)
        .with_config(sdk::Config {
            default_sampler: Box::new(sdk::Sampler::AlwaysOn),
            ..Default::default()
        })
        .build();

    let tracer = provider.get_tracer(service_name);

    let fmt_layer = fmt::Layer::default().compact();
    let opentelemetry_layer = layer().with_tracer(tracer);
    let env_filter_layer = EnvFilter::from_default_env();

    let subscriber = Registry::default()
        .with(env_filter_layer)
        .with(fmt_layer)
        .with(opentelemetry_layer);

    tracing::subscriber::set_global_default(subscriber)?;

    println!("*** Loading settings ***");
    let settings = si_settings::Settings::new()?;

    println!("*** Connecting to the database ***");
    let db = si_sdf::data::Db::new(&settings).context("failed to connect to the database")?;

    println!("*** Connecting to NATS ***");
    let nats = nats::asynk::connect("localhost").await?;

    println!("*** Creating indexes ***");
    si_sdf::data::create_indexes(&db).await?;

    println!("*** Checking for JWT keys ***");
    si_sdf::models::jwt_key::create_if_missing(
        &db,
        &nats,
        "config/public.pem",
        "config/private.pem",
        &settings.jwt_encrypt.key,
    )
    .await?;

    println!("*** Starting service ***");
    start(db, nats, settings).await;

    Ok(())
}
