use std::{env, path::Path, sync::Arc};

use buck2_resources::Buck2Resources;
use dal::generate_unique_id;
use dal::{
    pkg::import_pkg_from_pkg, ChangeSet, DalContext, JobQueueProcessor, NatsProcessor,
    ServicesContext, Tenancy, Workspace,
};
use si_data_nats::{NatsClient, NatsConfig};
use si_data_pg::{PgPool, PgPoolConfig};
use si_pkg::SiPkg;
use veritech_client::{Client as VeritechClient, EncryptionKey};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + 'static>>;

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = env::args();
    let tar_file = args.nth(1).expect("usage: program <PKG_FILE>");

    let mut ctx = ctx().await?;

    let workspace = match Workspace::find_first_user_workspace(&ctx).await? {
        Some(workspace) => workspace,
        None => Workspace::builtin(&ctx).await?,
    };

    ctx.update_tenancy(Tenancy::new(*workspace.pk()));

    let pkg = SiPkg::load_from_file(Path::new(&tar_file)).await?;
    let metadata = pkg.metadata()?;
    let change_set_name = format!(
        "pkg - {} ({}) {}",
        metadata.name(),
        metadata.version(),
        generate_unique_id(4)
    );
    let change_set = ChangeSet::new(&ctx, &change_set_name, None).await?;
    let ctx = ctx.clone_with_new_visibility(ctx.visibility().to_change_set(change_set.pk));

    println!(
        "--- Importing pkg: {tar_file} into change set \"{change_set_name}\" in workspace \"{}\"",
        workspace.name()
    );
    import_pkg_from_pkg(&ctx, &pkg, None).await?;

    println!("--- Committing database transaction");
    ctx.commit().await?;
    println!("  - Committed.");

    println!("--- Import complete.");
    Ok(())
}

async fn ctx() -> Result<DalContext> {
    let encryption_key = Arc::new(load_encryption_key().await?);
    let pg_pool = create_pg_pool().await?;
    let nats_conn = connect_to_nats().await?;
    let veritech = create_veritech_client(nats_conn.clone());

    let job_processor = connect_processor(nats_conn.clone()).await?;

    let services_context = ServicesContext::new(
        pg_pool,
        nats_conn,
        job_processor,
        veritech,
        encryption_key,
        None,
        None,
    );

    Ok(DalContext::builder(services_context, false)
        .build_default()
        .await?)
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

#[allow(clippy::disallowed_methods)] // Used to determine if running in development
async fn load_encryption_key() -> Result<EncryptionKey> {
    let path = if env::var("BUCK_RUN_BUILD_ID").is_ok() || env::var("BUCK_BUILD_ID").is_ok() {
        Buck2Resources::read()?.get_ends_with("dev.encryption.key")?
    } else if let Ok(dir) = env::var("CARGO_MANIFEST_DIR") {
        Path::new(&dir).join("../../lib/cyclone-server/src/dev.encryption.key")
    } else {
        unimplemented!("not running with Buck2 or Cargo, unsupported")
    };

    EncryptionKey::load(path).await.map_err(Into::into)
}

async fn connect_processor(
    job_client: NatsClient,
) -> Result<Box<dyn JobQueueProcessor + Send + Sync>> {
    let job_processor =
        Box::new(NatsProcessor::new(job_client)) as Box<dyn JobQueueProcessor + Send + Sync>;
    Ok(job_processor)
}
