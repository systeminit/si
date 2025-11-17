use std::{
    env,
    path::Path,
};

use buck2_resources::Buck2Resources;
use si_data_nats::{
    NatsClient,
    NatsConfig,
};
use si_data_pg::{
    PgPool,
    PgPoolConfig,
};
use si_tls::CertificateSource;

use crate::TEST_PG_DBNAME;

mod activities;
mod db;
mod layer_cache;

const DEFAULT_TEST_PG_USER: &str = "si_test";
const DEFAULT_TEST_PG_PORT_STR: &str = "6432";

const ENV_VAR_PG_HOSTNAME: &str = "SI_TEST_PG_HOSTNAME";
const ENV_VAR_PG_DBNAME: &str = "SI_TEST_PG_DBNAME";
const ENV_VAR_PG_USER: &str = "SI_TEST_PG_USER";
const ENV_VAR_PG_PORT: &str = "SI_TEST_PG_PORT";
const ENV_VAR_PERSISTER_MODE: &str = "SI_TEST_PERSISTER_MODE";

const ENV_VAR_NATS_URL: &str = "SI_TEST_NATS_URL";

#[allow(clippy::disallowed_methods)] // Environment variables are used exclusively in test
pub async fn setup_pg_db(test_specific_db_name: &str) -> PgPool {
    // PG pool config to setup tests
    let setup_pg_pool_config = {
        let mut pg = PgPoolConfig {
            application_name: "si-layer-cache-db-tests".into(),
            certificate: Some(CertificateSource::Path(
                detect_and_configure_development()
                    .try_into()
                    .expect("should get a certifcate cache"),
            )),
            ..Default::default()
        };
        if let Ok(value) = env::var(ENV_VAR_PG_HOSTNAME) {
            pg.hostname = value;
        }
        pg.dbname = env::var(ENV_VAR_PG_DBNAME).unwrap_or_else(|_| TEST_PG_DBNAME.to_string());
        pg.user = env::var(ENV_VAR_PG_USER).unwrap_or_else(|_| DEFAULT_TEST_PG_USER.to_string());
        pg.port = env::var(ENV_VAR_PG_PORT)
            .unwrap_or_else(|_| DEFAULT_TEST_PG_PORT_STR.to_string())
            .parse()
            .expect("port should parse as an integer");
        pg
    };

    // Create the setup PG pool
    let setup_pg_pool = PgPool::new(&setup_pg_pool_config)
        .await
        .expect("cannot create pg pool for tests");

    // A test-specific PG pool config which is virtually identical to the setup pool (aside from the
    // test-specific DB name)
    let test_specific_pg_pool_config = {
        let mut pg = setup_pg_pool_config.clone();
        pg.dbname = test_specific_db_name.into();
        pg
    };

    let db_drop_query = format!(
        "DROP DATABASE IF EXISTS {}",
        test_specific_pg_pool_config.dbname
    );

    let db_create_query = format!(
        "CREATE DATABASE {} OWNER {}",
        test_specific_pg_pool_config.dbname, test_specific_pg_pool_config.user
    );

    let client = setup_pg_pool
        .get()
        .await
        .expect("unable to get pg_pool client");

    client
        .execute(&db_drop_query, &[])
        .await
        .expect("able to drop database for tests");

    client
        .execute(&db_create_query, &[])
        .await
        .expect("able to create database for tests");

    // Build and return the test-specific PG pool
    PgPool::new(&test_specific_pg_pool_config)
        .await
        .expect("cannot create pg pool for tests")
}

/// This function is used to determine the development environment and update the [`ConfigFile`]
/// accordingly.
#[allow(clippy::disallowed_methods)]
pub fn detect_and_configure_development() -> String {
    if env::var("BUCK_RUN_BUILD_ID").is_ok() || env::var("BUCK_BUILD_ID").is_ok() {
        buck2_development()
    } else if let Ok(dir) = env::var("CARGO_MANIFEST_DIR") {
        cargo_development(dir)
    } else {
        "".to_string()
    }
}

pub fn buck2_development() -> String {
    let resources = Buck2Resources::read().expect("should be able to read buck2 resources");

    resources
        .get_ends_with("dev.postgres.root.crt")
        .expect("should be able to get cert")
        .to_string_lossy()
        .to_string()
}

pub fn cargo_development(dir: String) -> String {
    Path::new(&dir)
        .join("../../config/keys/dev.postgres.root.crt")
        .to_string_lossy()
        .to_string()
}

pub async fn setup_nats_client(subject_prefix: Option<String>) -> NatsClient {
    let mut nats_config = NatsConfig {
        subject_prefix,
        ..Default::default()
    };
    #[allow(clippy::disallowed_methods)] // Environment variables are used exclusively in test
    if let Ok(value) = env::var(ENV_VAR_NATS_URL) {
        nats_config.url = value;
    }

    NatsClient::new(&nats_config)
        .await
        .expect("failed to connect to nats")
}

pub fn setup_compute_executor() -> si_runtime::DedicatedExecutor {
    si_runtime::compute_executor("test").expect("failed to create executor")
}

#[allow(clippy::disallowed_methods)] // Environment variables are used exclusively in test
pub fn make_test_layerdb_config() -> si_layer_cache::db::LayerDbConfig {
    let persister_mode = env::var(ENV_VAR_PERSISTER_MODE)
        .ok()
        .and_then(|s| serde_json::from_str(&format!("\"{s}\"")).ok())
        .unwrap_or_default();

    si_layer_cache::db::LayerDbConfig {
        pg_pool_config: PgPoolConfig {
            application_name: "si-layer-cache-test".into(),
            ..Default::default()
        },
        nats_config: NatsConfig::default(),
        cache_config: si_layer_cache::hybrid_cache::CacheConfig::default(),
        object_storage_config: si_layer_cache::ObjectStorageConfig::default(),
        persister_mode,
    }
}
