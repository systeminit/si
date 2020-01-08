use anyhow::{Context, Result};
use si_data::Db;
use si_settings::Settings;
use tokio::runtime::Builder;
use tonic::transport::Server;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use si_account::{protobuf::account_server::AccountServer, service::Service};

async fn run() -> Result<()> {
    let settings = Settings::new()?;

    let db = Db::new(&settings).context("Cannot connect to the database")?;

    let service = Service::new(db);

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

fn main() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .context("cannot set the global tracing defalt")?;

    let mut runtime = Builder::new()
        .enable_all()
        //.panic_handler(|err| std::panic::resume_unwind(err))
        .build()
        .context("Cannot set the tokio runtime up")?;

    runtime.block_on(async { run().await })?;
    Ok(())
}
