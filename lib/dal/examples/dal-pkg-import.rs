use std::{env, path::Path, sync::Arc};

use dal::{
    pkg::import_schema, DalContext, JobQueueProcessor, NatsProcessor, ServicesContext, Tenancy,
    Workspace,
};
use si_data_nats::{NatsClient, NatsConfig};
use si_data_pg::{PgPool, PgPoolConfig};
use tokio::sync::mpsc;
use veritech_client::{Client as VeritechClient, EncryptionKey};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + 'static>>;

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = env::args();
    let src_dir = args.nth(1).expect("usage: program <SRC_DIR> <NAME>");
    let name = args.next().expect("usage: program <SRC_DIR> <NAME>");

    let (mut ctx, _shutdown_rx) = ctx().await?;
    let workspace = Workspace::builtin(&ctx).await?;
    ctx.update_tenancy(Tenancy::new(*workspace.pk()));

    println!("--- Importing schema '{name} from pkg prop tree in: {src_dir}");
    import_schema(&mut ctx, &name, Path::new(&src_dir)).await?;

    println!("--- Committing database transaction");
    ctx.commit().await?;
    println!("  - Committed.");

    println!("--- Import complete.");
    Ok(())
}

async fn ctx() -> Result<(DalContext, mpsc::Receiver<()>)> {
    let encryption_key = Arc::new(load_encryption_key().await?);
    let pg_pool = create_pg_pool().await?;
    let nats_conn = connect_to_nats().await?;
    let veritech = create_veritech_client(nats_conn.clone());
    let council_subject_prefix = "council".to_owned();

    let (alive_marker, job_processor_shutdown_rx) = mpsc::channel(1);
    let job_processor = connect_processor(nats_conn.clone(), alive_marker).await?;

    let services_context = ServicesContext::new(
        pg_pool,
        nats_conn,
        job_processor,
        veritech,
        encryption_key,
        council_subject_prefix,
        None,
    );

    Ok((
        DalContext::builder(services_context)
            .build_default()
            .await?,
        job_processor_shutdown_rx,
    ))
}

async fn create_pg_pool() -> Result<PgPool> {
    PgPool::new(&PgPoolConfig::default())
        .await
        .map_err(Into::into)
}

async fn connect_to_nats() -> Result<NatsClient> {
    NatsClient::new(&NatsConfig::default())
        .await
        .map_err(Into::into)
}

fn create_veritech_client(nats: NatsClient) -> VeritechClient {
    VeritechClient::new(nats)
}

async fn load_encryption_key() -> Result<EncryptionKey> {
    let path = Path::new(&env::var("CARGO_MANIFEST_DIR")?)
        .join("../../lib/cyclone-server/src/dev.encryption.key");
    EncryptionKey::load(path).await.map_err(Into::into)
}

async fn connect_processor(
    job_client: NatsClient,
    alive_marker: mpsc::Sender<()>,
) -> Result<Box<dyn JobQueueProcessor + Send + Sync>> {
    let job_processor = Box::new(NatsProcessor::new(job_client, alive_marker))
        as Box<dyn JobQueueProcessor + Send + Sync>;
    Ok(job_processor)
}
