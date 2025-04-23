use std::{
    collections::HashMap,
    sync::Arc,
};

use axum::extract::FromRef;
use s3::creds::Credentials as AwsCredentials;
use sea_orm::DatabaseConnection;
use si_jwt_public_key::JwtPublicSigningKeyChain;
pub use si_posthog::PosthogClient;
use tokio::sync::{
    Mutex,
    mpsc,
};

use crate::s3::S3Config;

#[remain::sorted]
#[derive(Debug, Eq, PartialEq)]
pub enum ShutdownSource {}

#[derive(Clone, Debug, FromRef)]
pub struct AppState {
    /// A PostgreSQL connection pool.
    pg_pool: DatabaseConnection,
    auth_api_url: String,
    jwt_public_signing_key_chain: JwtPublicSigningKeyChain,
    posthog_client: PosthogClient,
    aws_creds: AwsCredentials,
    s3_config: S3Config,
    token_emails: Arc<Mutex<HashMap<String, String>>>,

    // see notes in sdf AppState
    #[from_ref(skip)]
    _tmp_shutdown_tx: Arc<mpsc::Sender<ShutdownSource>>,
}

impl AppState {
    /// Constructs a new instance of a `AppState`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        pg_pool: DatabaseConnection,
        auth_api_url: String,
        jwt_public_signing_key_chain: JwtPublicSigningKeyChain,
        posthog_client: PosthogClient,
        aws_creds: AwsCredentials,
        s3_config: S3Config,
        tmp_shutdown_tx: mpsc::Sender<ShutdownSource>,
    ) -> Self {
        Self {
            pg_pool,
            auth_api_url,
            jwt_public_signing_key_chain,
            posthog_client,
            aws_creds,
            s3_config,
            token_emails: Arc::new(Mutex::new(HashMap::new())),
            _tmp_shutdown_tx: Arc::new(tmp_shutdown_tx),
        }
    }

    /// Gets a reference to the Postgres pool.
    pub fn pg_pool(&self) -> &DatabaseConnection {
        &self.pg_pool
    }

    /// Gets the URL to the auth api
    pub fn auth_api_url(&self) -> &str {
        &self.auth_api_url
    }

    /// Gets a reference to the public key used to sign the JWT
    pub fn jwt_public_signing_key(&self) -> &JwtPublicSigningKeyChain {
        &self.jwt_public_signing_key_chain
    }

    /// Gets a reference to the Posthog client.
    pub fn posthog_client(&self) -> &PosthogClient {
        &self.posthog_client
    }

    /// Gets a reference to the aws creds.
    pub fn aws_creds(&self) -> &AwsCredentials {
        &self.aws_creds
    }
    /// Gets a reference to the s3 config (bucket, region, etc)
    pub fn s3_config(&self) -> &S3Config {
        &self.s3_config
    }

    /// Clones the ArcMutex that holds a hashmap between auth tokens and emails
    pub fn token_emails(&self) -> Arc<Mutex<HashMap<String, String>>> {
        self.token_emails.clone()
    }
}
