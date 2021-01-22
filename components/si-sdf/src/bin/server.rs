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

    println!("*** Connecting to postgres ***");
    let pg = si_sdf::data::PgPool::new(&settings.pg).await?;

    println!("*** Connecting to NATS ***");
    let nats = si_sdf::data::NatsConn::new(&settings.nats).await?;

    println!("*** Initializing EventLogFs ***");
    let event_log_fs = si_sdf::data::EventLogFS::init(&settings.event_log_fs).await?;

    println!("*** Initializing Veritech ***");
    let veritech = si_sdf::veritech::Veritech::new(&settings.veritech, event_log_fs.clone());

    println!("*** Checking for JWT keys ***");
    let mut conn = pg.pool.get().await?;
    let txn = conn.transaction().await?;
    si_sdf::models::jwt_key::create_jwt_key_if_missing(
        &txn,
        "config/public.pem",
        "config/private.pem",
        &settings.jwt_encrypt.key,
    )
    .await?;
    txn.commit().await?;

    println!("*** Starting service ***");
    start(pg, nats, veritech, event_log_fs, settings).await;

    Ok(())
}
