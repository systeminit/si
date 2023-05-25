use std::sync::Arc;

use axum::extract::FromRef;
use sea_orm::DatabaseConnection;
pub use si_posthog::PosthogClient;

use tokio::sync::{broadcast, mpsc};

use crate::jwt_key::JwtPublicSigningKey;

#[remain::sorted]
#[derive(Debug, Eq, PartialEq)]
pub enum ShutdownSource {}

#[derive(Clone, Debug)]
pub struct ShutdownBroadcast(broadcast::Sender<()>);

#[derive(Clone, Debug, FromRef)]
pub struct AppState {
    /// A PostgreSQL connection pool.
    pg_pool: DatabaseConnection,
    jwt_public_signing_key: JwtPublicSigningKey,
    posthog_client: PosthogClient,

    shutdown_broadcast: ShutdownBroadcast,

    // see notes in sdf AppState
    #[from_ref(skip)]
    _tmp_shutdown_tx: Arc<mpsc::Sender<ShutdownSource>>,
}

impl AppState {
    /// Constructs a new instance of a `AppState`.
    pub fn new(
        pg_pool: DatabaseConnection,
        jwt_public_signing_key: JwtPublicSigningKey,
        posthog_client: PosthogClient,
        shutdown_broadcast_tx: broadcast::Sender<()>,
        tmp_shutdown_tx: mpsc::Sender<ShutdownSource>,
    ) -> Self {
        Self {
            pg_pool,
            jwt_public_signing_key,
            posthog_client,
            shutdown_broadcast: ShutdownBroadcast(shutdown_broadcast_tx),
            _tmp_shutdown_tx: Arc::new(tmp_shutdown_tx),
        }
    }

    /// Gets a reference to the Postgres pool.
    pub fn pg_pool(&self) -> &DatabaseConnection {
        &self.pg_pool
    }

    pub fn jwt_public_signing_key(&self) -> &JwtPublicSigningKey {
        &self.jwt_public_signing_key
    }

    /// Gets a reference to the Posthog client.
    pub fn posthog_client(&self) -> &PosthogClient {
        &self.posthog_client
    }
}
