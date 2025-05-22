use std::{
    path::PathBuf,
    sync::Arc,
};

use dal::{
    DalLayerDb,
    DedicatedExecutor,
    JetstreamStreams,
    JobQueueProcessor,
    NatsProcessor,
    ServicesContext,
    feature_flags::FeatureFlagService,
};
use rebaser_client::RebaserClient;
use si_crypto::{
    SymmetricCryptoService,
    SymmetricCryptoServiceConfig,
    VeritechCryptoConfig,
    VeritechEncryptionKey,
};
use si_data_nats::{
    NatsClient,
    NatsConfig,
};
use si_data_pg::{
    PgPool,
    PgPoolConfig,
};
use si_jwt_public_key::{
    JwtConfig,
    JwtPublicSigningKeyChain,
    JwtPublicSigningKeyError,
};
use si_layer_cache::{
    LayerDb,
    db::{
        LayerDbConfig,
        LayerDbGracefulShutdown,
    },
};
use si_posthog::{
    PosthogClient,
    PosthogConfig,
    PosthogSender,
};
use telemetry::prelude::*;
use thiserror::Error;
use tokio_util::sync::CancellationToken;

use crate::Config;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum InitError {
    #[error("compute executor initialization error: {0}")]
    ComputeExecutor(#[from] dal::DedicatedExecutorInitializeError),
    #[error("initialization error: {0}")]
    DalInitialization(#[from] dal::InitializationError),
    #[error("failed to initialize a dal jetstream streams: {0}")]
    DalJetstreamStreams(#[source] dal::JetstreamStreamsError),
    #[error("job queue processor error: {0}")]
    DalJobQueueProcessor(#[from] dal::job::processor::JobQueueProcessorError),
    #[error("jwt key error")]
    JwtKey(#[from] JwtPublicSigningKeyError),
    #[error("layer cache error: {0}")]
    LayerCache(#[from] si_layer_cache::LayerDbError),
    #[error("failed to initialize a nats client: {0}")]
    NatsClient(#[source] si_data_nats::NatsError),
    #[error("pg pool error: {0}")]
    PgPool(#[from] Box<si_data_pg::PgPoolError>),
    #[error("posthog client error: {0}")]
    Posthog(#[from] si_posthog::PosthogError),
    #[error("rebaser client error: {0}")]
    Rebaser(#[from] rebaser_client::ClientError),
    #[error("symmetric crypto error: {0}")]
    SymmetricCryptoService(#[from] si_crypto::SymmetricCryptoError),
    #[error("error when loading cyclone encryption key: {0}")]
    VeritechEncryptionKey(#[from] si_crypto::VeritechEncryptionKeyError),
}

impl From<si_data_pg::PgPoolError> for InitError {
    fn from(e: si_data_pg::PgPoolError) -> Self {
        Self::PgPool(Box::new(e))
    }
}
type InitResult<T> = std::result::Result<T, InitError>;

pub(crate) async fn services_context_from_config(
    config: &Config,
    helping_tasks_token: CancellationToken,
) -> InitResult<(ServicesContext, LayerDbGracefulShutdown)> {
    dal::init()?;

    let encryption_key = load_encryption_key(config.crypto().clone()).await?;
    let nats = connect_to_nats(config.nats()).await?;
    let jetstream_streams = get_or_create_jetstream_streams(nats.clone()).await?;
    let pg_pool = create_pg_pool(config.pg_pool()).await?;
    let rebaser = create_rebaser_client(nats.clone()).await?;
    let veritech = create_veritech_client(nats.clone());
    let job_processor = create_job_processor(nats.clone()).await?;
    let symmetric_crypto_service =
        create_symmetric_crypto_service(config.symmetric_crypto_service()).await?;

    let pkgs_path: PathBuf = config.pkgs_path().into();
    let module_index_url = Some(config.module_index_url().to_string());
    let feature_flags_service = FeatureFlagService::new(config.boot_feature_flags().clone());

    let compute_executor = create_compute_executor()?;

    let (layer_db, layer_db_graceful_shutdown) = initialize_layer_db(
        config.layer_db_config().clone(),
        compute_executor.clone(),
        helping_tasks_token.clone(),
    )
    .await?;

    let services_context = ServicesContext::new(
        pg_pool,
        nats.clone(),
        jetstream_streams,
        job_processor,
        rebaser,
        veritech,
        encryption_key,
        Some(pkgs_path),
        module_index_url,
        symmetric_crypto_service,
        layer_db,
        feature_flags_service,
        compute_executor,
    );

    Ok((services_context, layer_db_graceful_shutdown))
}

#[instrument(name = "luminork.init.load_encryption_key", level = "info", skip_all)]
pub(crate) async fn load_encryption_key(
    crypto_config: VeritechCryptoConfig,
) -> InitResult<Arc<VeritechEncryptionKey>> {
    Ok(Arc::new(
        VeritechEncryptionKey::from_config(crypto_config).await?,
    ))
}

#[instrument(name = "luminork.init.connect_to_nats", level = "info", skip_all)]
pub(crate) async fn connect_to_nats(nats_config: &NatsConfig) -> InitResult<NatsClient> {
    let client = NatsClient::new(nats_config)
        .await
        .map_err(InitError::NatsClient)?;
    debug!("successfully connected nats client");
    Ok(client)
}

#[instrument(
    name = "luminork.init.get_or_create_jetstream_streams",
    level = "info",
    skip_all
)]
pub(crate) async fn get_or_create_jetstream_streams(
    client: NatsClient,
) -> InitResult<JetstreamStreams> {
    let streams = JetstreamStreams::new(client)
        .await
        .map_err(InitError::DalJetstreamStreams)?;
    debug!("created jetstream streams");
    Ok(streams)
}

#[instrument(name = "luminork.init.create_pg_pool", level = "info", skip_all)]
pub(crate) async fn create_pg_pool(pg_pool_config: &PgPoolConfig) -> InitResult<PgPool> {
    let pool = PgPool::new(pg_pool_config).await?;
    debug!("successfully started pg pool (note that not all connections may be healthy)");
    Ok(pool)
}

#[instrument(name = "luminork.init.create_rebaser_client", level = "info", skip_all)]
async fn create_rebaser_client(nats: NatsClient) -> InitResult<RebaserClient> {
    let client = RebaserClient::new(nats).await?;
    debug!("successfully initialized the rebaser client");
    Ok(client)
}

pub(crate) fn create_veritech_client(nats: NatsClient) -> veritech_client::Client {
    veritech_client::Client::new(nats)
}

#[instrument(
    name = "luminork.init.create_compute_executor",
    level = "info",
    skip_all
)]
pub(crate) fn create_compute_executor() -> InitResult<DedicatedExecutor> {
    dal::compute_executor("luminork").map_err(Into::into)
}

#[instrument(name = "luminork.init.create_job_processor", level = "info", skip_all)]
pub(crate) async fn create_job_processor(
    nats: NatsClient,
) -> InitResult<Box<dyn JobQueueProcessor + Send + Sync>> {
    Ok(Box::new(NatsProcessor::new(nats).await?) as Box<dyn JobQueueProcessor + Send + Sync>)
}

#[instrument(
    name = "luminork.init.create_symmetric_crypto_service",
    level = "info",
    skip_all
)]
pub(crate) async fn create_symmetric_crypto_service(
    config: &SymmetricCryptoServiceConfig,
) -> InitResult<SymmetricCryptoService> {
    SymmetricCryptoService::from_config(config)
        .await
        .map_err(Into::into)
}

#[instrument(name = "luminork.init.initialize_layer_db", level = "info", skip_all)]
pub(crate) async fn initialize_layer_db(
    config: LayerDbConfig,
    compute_executor: DedicatedExecutor,
    token: CancellationToken,
) -> InitResult<(DalLayerDb, LayerDbGracefulShutdown)> {
    LayerDb::from_config(config, compute_executor, token)
        .await
        .map_err(Into::into)
}

#[instrument(
    name = "luminork.init.load_jwt_public_signing_key",
    level = "info",
    skip_all
)]
pub(crate) async fn load_jwt_public_signing_key(
    primary: JwtConfig,
    secondary: Option<JwtConfig>,
) -> InitResult<JwtPublicSigningKeyChain> {
    Ok(JwtPublicSigningKeyChain::from_config(primary, secondary).await?)
}

pub(crate) fn initialize_posthog(
    config: &PosthogConfig,
    token: CancellationToken,
) -> InitResult<(PosthogSender, PosthogClient)> {
    si_posthog::from_config(config, token).map_err(Into::into)
}
