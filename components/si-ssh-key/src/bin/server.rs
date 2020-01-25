use anyhow::{Context, Result};
use si_data::Db;
use si_settings::Settings;
use tokio;
use tonic::transport::Server;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use si_ssh_key::{migrate, Service, SshKeyServer};

async fn run() -> Result<()> {
    let settings = Settings::new()?;

    let db = Db::new(&settings).context("Cannot connect to the database")?;

    println!("*** Migrating so much right now ***");
    migrate(&db).await?;

    let service = Service::new(db);

    let listen_string = format!("0.0.0.0:{}", settings.service.port);

    let addr = listen_string.parse().unwrap();

    println!("--> SSH Key service listening on {} <--", addr);
    println!("--> Let us make ssh keys and stuff <--");

    Server::builder()
        .add_service(SshKeyServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .context("cannot set the global tracing defalt")?;

    let handle = tokio::spawn(async move { run().await });

    handle.await??;
    Ok(())
}
