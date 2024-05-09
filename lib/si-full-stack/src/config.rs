use std::{env, path::{Path, PathBuf}};

use derive_builder::Builder;
use si_crypto::{SymmetricCryptoServiceConfig, SymmetricCryptoServiceConfigFile};
use si_data_nats::NatsConfig;
use si_data_pg::PgPoolConfig;
use telemetry::prelude::*;
use buck2_resources::Buck2Resources;

use crate::error::SiFullStackResult;

const DEFAULT_TEST_PG_USER: &str = "si_test";
const DEFAULT_TEST_PG_PORT_STR: &str = "6432";

const ENV_VAR_NATS_URL: &str = "SI_TEST_NATS_URL";
const ENV_VAR_MODULE_INDEX_URL: &str = "SI_TEST_MODULE_INDEX_URL";
const ENV_VAR_PG_HOSTNAME: &str = "SI_TEST_PG_HOSTNAME";
const ENV_VAR_PG_DBNAME: &str = "SI_TEST_PG_DBNAME";
const ENV_VAR_LAYER_CACHE_PG_DBNAME: &str = "SI_TEST_LAYER_CACHE_PG_DBNAME";
const ENV_VAR_PG_USER: &str = "SI_TEST_PG_USER";
const ENV_VAR_PG_PORT: &str = "SI_TEST_PG_PORT";

#[allow(missing_docs)]
#[derive(Builder, Clone, Debug)]
pub struct Config {
    #[builder(default = "PgPoolConfig::default()")]
    pg: PgPoolConfig,
    #[builder(default = "NatsConfig::default()")]
    nats: NatsConfig,
    #[builder(default = "module_index_client::DEFAULT_URL.to_string()")]
    module_index_url: String,
    cyclone_encryption_key_path: String,
    jwt_signing_public_key_path: String,
    jwt_signing_private_key_path: String,
    postgres_key_path: String,
    #[builder(default)]
    pkgs_path: Option<PathBuf>,
    symmetric_crypto_service_config: SymmetricCryptoServiceConfig,
    // TODO(nick): determine why this is unused.
    #[allow(dead_code)]
    #[builder(default = "si_layer_cache::default_pg_pool_config()")]
    layer_cache_pg_pool: PgPoolConfig,
}

impl Config {
    pub fn nats(&self) -> NatsConfig {
        self.nats
    }

    pub fn pg(&self) -> PgPoolConfig {
        self.pg
    }

    pub fn layer_cache_pg_pool(&self) -> PgPoolConfig {
        self.layer_cache_pg_pool
    }

    #[allow(clippy::disallowed_methods)] // Environment variables are used exclusively in test and
    pub fn create_default(
        pg_dbname: &'static str,
        layer_cache_pg_dbname: &'static str,
    ) -> SiFullStackResult<Self> {
        let mut config = {
            let mut builder = ConfigBuilder::default();
            detect_and_configure_testing(&mut builder)?;
            builder.build()?
        };

        if let Ok(value) = env::var(ENV_VAR_NATS_URL) {
            config.nats.url = value;
        }

        if let Ok(value) = env::var(ENV_VAR_PG_HOSTNAME) {
            config.pg.hostname = value;
        }
        config.pg.dbname = env::var(ENV_VAR_PG_DBNAME).unwrap_or_else(|_| pg_dbname.to_string());
        config.pg.user =
            env::var(ENV_VAR_PG_USER).unwrap_or_else(|_| DEFAULT_TEST_PG_USER.to_string());
        config.pg.port = env::var(ENV_VAR_PG_PORT)
            .unwrap_or_else(|_| DEFAULT_TEST_PG_PORT_STR.to_string())
            .parse()?;
        //config.pg.pool_max_size *= 32;
        config.pg.pool_max_size = 8;
        config.pg.certificate_path = Some(config.postgres_key_path.clone().try_into()?);

        if let Ok(value) = env::var(ENV_VAR_PG_HOSTNAME) {
            config.layer_cache_pg_pool.hostname = value;
        }
        config.layer_cache_pg_pool.dbname = env::var(ENV_VAR_LAYER_CACHE_PG_DBNAME)
            .unwrap_or_else(|_| layer_cache_pg_dbname.to_string());
        config.layer_cache_pg_pool.user =
            env::var(ENV_VAR_PG_USER).unwrap_or_else(|_| DEFAULT_TEST_PG_USER.to_string());
        config.layer_cache_pg_pool.port = env::var(ENV_VAR_PG_PORT)
            .unwrap_or_else(|_| DEFAULT_TEST_PG_PORT_STR.to_string())
            .parse()?;
        //config.layer_cache_pg_pool.pool_max_size *= 32;
        config.layer_cache_pg_pool.pool_max_size = 8;
        config.layer_cache_pg_pool.certificate_path =
            Some(config.postgres_key_path.clone().try_into()?);

        if let Ok(value) = env::var(ENV_VAR_MODULE_INDEX_URL) {
            config.module_index_url = value;
        }

        debug!(?config, "test config");

        Ok(config)
    }
}

#[allow(clippy::disallowed_methods)] // Used to determine if running in testing
fn detect_and_configure_testing(builder: &mut ConfigBuilder) -> SiFullStackResult<()> {
    if env::var("BUCK_RUN_BUILD_ID").is_ok() || env::var("BUCK_BUILD_ID").is_ok() {
        detect_and_configure_testing_for_buck2(builder)
    } else if let Ok(dir) = env::var("CARGO_MANIFEST_DIR") {
        detect_and_configure_testing_for_cargo(dir, builder)
    } else {
        unimplemented!("tests must be run either with Cargo or Buck2");
    }
}

fn detect_and_configure_testing_for_buck2(builder: &mut ConfigBuilder) -> SiFullStackResult<()> {
    let resources = Buck2Resources::read()?;

    let cyclone_encryption_key_path = resources
        .get_ends_with("dev.encryption.key")?
        .to_string_lossy()
        .to_string();
    let jwt_signing_public_key_path = resources
        .get_ends_with("dev.jwt_signing_public_key.pem")?
        .to_string_lossy()
        .to_string();
    let jwt_signing_private_key_path = resources
        .get_ends_with("dev.jwt_signing_private_key.pem")?
        .to_string_lossy()
        .to_string();
    let symmetric_crypto_service_key = resources
        .get_ends_with("dev.donkey.key")?
        .to_string_lossy()
        .to_string();
    let postgres_key = resources
        .get_ends_with("dev.postgres.root.crt")?
        .to_string_lossy()
        .to_string();
    let pkgs_path = resources
        .get_ends_with("pkgs_path")?
        .to_string_lossy()
        .to_string();

    warn!(
        cyclone_encryption_key_path = cyclone_encryption_key_path.as_str(),
        jwt_signing_private_key_path = jwt_signing_private_key_path.as_str(),
        jwt_signing_public_key_path = jwt_signing_public_key_path.as_str(),
        symmetric_crypto_service_key = symmetric_crypto_service_key.as_str(),
        postgres_key = postgres_key.as_str(),
        pkgs_path = pkgs_path.as_str(),
        "detected development run",
    );

    builder.cyclone_encryption_key_path(cyclone_encryption_key_path);
    builder.jwt_signing_public_key_path(jwt_signing_public_key_path);
    builder.jwt_signing_private_key_path(jwt_signing_private_key_path);
    builder.symmetric_crypto_service_config(
        SymmetricCryptoServiceConfigFile {
            active_key: Some(symmetric_crypto_service_key),
            active_key_base64: None,
            extra_keys: vec![],
        }
        .try_into()?,
    );
    builder.postgres_key_path(postgres_key);
    builder.pkgs_path(Some(pkgs_path.into()));

    Ok(())
}

fn detect_and_configure_testing_for_cargo(dir: String, builder: &mut ConfigBuilder) -> SiFullStackResult<()> {
    let cyclone_encryption_key_path = Path::new(&dir)
        .join("../../lib/cyclone-server/src/dev.encryption.key")
        .to_string_lossy()
        .to_string();
    let jwt_signing_public_key_path = Path::new(&dir)
        .join("../../config/keys/dev.jwt_signing_public_key.pem")
        .to_string_lossy()
        .to_string();
    let jwt_signing_private_key_path = Path::new(&dir)
        .join("../../config/keys/dev.jwt_signing_private_key.pem")
        .to_string_lossy()
        .to_string();
    let symmetric_crypto_service_key = Path::new(&dir)
        .join("../../lib/dal/dev.donkey.key")
        .to_string_lossy()
        .to_string();
    let postgres_key = Path::new(&dir)
        .join("../../config/keys/dev.postgres.root.crt")
        .to_string_lossy()
        .to_string();
    let pkgs_path = Path::new(&dir)
        .join("../../pkgs")
        .to_string_lossy()
        .to_string();

    warn!(
        cyclone_encryption_key_path = cyclone_encryption_key_path.as_str(),
        jwt_signing_private_key_path = jwt_signing_private_key_path.as_str(),
        jwt_signing_public_key_path = jwt_signing_public_key_path.as_str(),
        symmetric_crypto_service_key = symmetric_crypto_service_key.as_str(),
        postgres_key = postgres_key.as_str(),
        pkgs_path = pkgs_path.as_str(),
        "detected development run",
    );

    builder.cyclone_encryption_key_path(cyclone_encryption_key_path);
    builder.jwt_signing_public_key_path(jwt_signing_public_key_path);
    builder.jwt_signing_private_key_path(jwt_signing_private_key_path);
    builder.symmetric_crypto_service_config(
        SymmetricCryptoServiceConfigFile {
            active_key: Some(symmetric_crypto_service_key),
            active_key_base64: None,
            extra_keys: vec![],
        }
        .try_into()?,
    );
    builder.postgres_key_path(postgres_key);
    builder.pkgs_path(Some(pkgs_path.into()));

    Ok(())
}
