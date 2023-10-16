use buck2_resources::Buck2Resources;
use std::{env, path::Path, str::FromStr, sync::Arc};
use tokio::fs;

use dal::{
    pkg::PkgExporter, ChangeSet, ChangeSetPk, DalContext, JobQueueProcessor, NatsProcessor, Schema,
    ServicesContext, StandardModel, Tenancy, Workspace,
};
use si_crypto::{SymmetricCryptoService, SymmetricCryptoServiceConfigFile};
use si_data_nats::{NatsClient, NatsConfig};
use si_data_pg::{PgPool, PgPoolConfig};
use veritech_client::{Client as VeritechClient, EncryptionKey};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + 'static>>;

const USAGE: &str =
    "usage: program <CHANGE_SET_PK> <PKG_FILE> <NAME> <VERSION> <CREATED_BY> <SCHEMA_NAME,SCHEMA_NAME[,...]>";

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = env::args();
    let change_set_pk = ChangeSetPk::from_str(args.nth(1).expect(USAGE).as_str())?;
    let tar_file = args.next().expect(USAGE);
    let name = args.next().expect(USAGE);
    let version = args.next().expect(USAGE);
    let created_by = args.next().expect(USAGE);
    let schema_names = args.next().expect(USAGE);
    let schema_names = schema_names.split(',');

    let description = format!("{name} package, created by {created_by}.");

    let mut ctx = ctx().await?;

    let workspace = match Workspace::find_first_user_workspace(&ctx).await? {
        Some(workspace) => workspace,
        None => Workspace::builtin(&ctx).await?,
    };

    ctx.update_tenancy(Tenancy::new(*workspace.pk()));
    let change_set = ChangeSet::get_by_pk(&ctx, &change_set_pk)
        .await?
        .expect("That change set could not be found");
    ctx.update_visibility(ctx.visibility().to_change_set(change_set.pk));

    let mut schema_ids = Vec::new();
    for schema_name in schema_names {
        schema_ids.push(*Schema::find_by_name(&ctx, schema_name.trim()).await?.id());
    }

    println!(
        "--- Exporting pkg: {tar_file} from head change set in workspace \"{}\"",
        workspace.name()
    );
    let mut exporter =
        PkgExporter::new_module_exporter(name, version, Some(description), created_by, schema_ids);

    fs::write(&tar_file, exporter.export_as_bytes(&ctx).await?).await?;

    println!("--- Committing database transaction");
    ctx.commit().await?;
    println!("  - Committed.");

    println!("--- Export complete.");
    Ok(())
}

async fn ctx() -> Result<DalContext> {
    let encryption_key = Arc::new(load_encryption_key().await?);
    let pg_pool = create_pg_pool().await?;
    let nats_conn = connect_to_nats().await?;
    let veritech = create_veritech_client(nats_conn.clone());
    let symmetric_crypto_service = create_symmetric_crypto_service().await?;

    let job_processor = connect_processor(nats_conn.clone()).await?;

    let services_context = ServicesContext::new(
        pg_pool,
        nats_conn,
        job_processor,
        veritech,
        encryption_key,
        None,
        None,
        symmetric_crypto_service,
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

#[allow(clippy::disallowed_methods)] // Used to determine if running in development
async fn create_symmetric_crypto_service() -> Result<SymmetricCryptoService> {
    let active_key = if env::var("BUCK_RUN_BUILD_ID").is_ok() || env::var("BUCK_BUILD_ID").is_ok() {
        Buck2Resources::read()?.get_ends_with("dev.donkey.key")?
    } else if let Ok(dir) = env::var("CARGO_MANIFEST_DIR") {
        Path::new(&dir).join("../../lib/dal/dev.donkey.key")
    } else {
        unimplemented!("not running with Buck2 or Cargo, unsupported")
    };

    SymmetricCryptoService::from_config(
        &SymmetricCryptoServiceConfigFile {
            active_key: active_key.to_string_lossy().into_owned(),
            extra_keys: Default::default(),
        }
        .try_into()?,
    )
    .await
    .map_err(Into::into)
}

async fn connect_processor(
    job_client: NatsClient,
) -> Result<Box<dyn JobQueueProcessor + Send + Sync>> {
    let job_processor =
        Box::new(NatsProcessor::new(job_client)) as Box<dyn JobQueueProcessor + Send + Sync>;
    Ok(job_processor)
}
