use anyhow::{Context, Result};
use si_data::Db;
use si_settings::Settings;
use tokio;
use tonic::transport::Server;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use si_account::{Integration, IntegrationService};
use si_aws_eks_cluster_runtime::{
    migrate, AgentClient, AgentFinalizer, AgentServer, AwsEksClusterRuntimeServer, Service,
};

async fn run() -> Result<()> {
    let settings = Settings::new()?;

    let db = Db::new(&settings).context("Cannot connect to the database")?;

    println!("*** Migrating so much right now ***");
    migrate(&db).await?;

    println!("*** Starting the Agent Client ***");
    let agent_client = AgentClient::new().await?;

    let integration: Integration = db.lookup_by_natural_key("global:integration:aws").await?;

    let integration_service_lookup_id =
        format!("global:{}:integration_service:eks", integration.id);
    let integration_service: IntegrationService = db
        .lookup_by_natural_key(integration_service_lookup_id)
        .await?;

    // I bet you want to actually be smarter than this, if this errors - but life goes
    // on.
    let mut agent_server = AgentServer::new("aws", integration.id, integration_service.id);
    tokio::spawn(async move { agent_server.run().await });

    let mut agent_finalizer = AgentFinalizer::new(db.clone());
    tokio::spawn(async move { agent_finalizer.run().await });

    let service = Service::new(db, agent_client);

    let listen_string = format!("0.0.0.0:{}", settings.service.port);

    let addr = listen_string.parse().unwrap();

    println!(
        "--> AWS EKS Cluster Runtime service listening on {} <--",
        addr
    );
    println!("--> Let us make ssh keys and stuff <--");

    Server::builder()
        .add_service(AwsEksClusterRuntimeServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .without_time()
        .with_ansi(true)
        .compact()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .context("cannot set the global tracing defalt")?;

    let handle = tokio::spawn(async move { run().await });

    handle.await??;
    Ok(())
}
