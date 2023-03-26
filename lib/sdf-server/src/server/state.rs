use std::{ops::Deref, sync::Arc};

use axum::extract::FromRef;
use si_std::SensitiveString;
use tokio::sync::{broadcast, mpsc};

use super::server::ShutdownSource;

#[derive(Clone, FromRef)]
pub struct AppState {
    services_context: ServicesContext,
    signup_secret: SignupSecret,
    jwt_secret_key: JwtSecretKey,
    shutdown_broadcast: ShutdownBroadcast,

    // TODO(fnichol): we're likely going to use this, but we can't allow it to be dropped because
    // that will trigger the read side and... shutdown. Cool, no?
    #[from_ref(skip)]
    _tmp_shutdown_tx: Arc<mpsc::Sender<ShutdownSource>>,
}

impl AppState {
    pub fn new(
        services_context: impl Into<ServicesContext>,
        signup_secret: impl Into<SignupSecret>,
        jwt_secret_key: impl Into<JwtSecretKey>,
        shutdown_broadcast_tx: broadcast::Sender<()>,
        tmp_shutdown_tx: mpsc::Sender<ShutdownSource>,
    ) -> Self {
        Self {
            services_context: services_context.into(),
            signup_secret: signup_secret.into(),
            jwt_secret_key: jwt_secret_key.into(),
            shutdown_broadcast: ShutdownBroadcast(shutdown_broadcast_tx),
            _tmp_shutdown_tx: Arc::new(tmp_shutdown_tx),
        }
    }

    pub fn services_context(&self) -> &ServicesContext {
        &self.services_context
    }
}

#[derive(Clone, Debug, FromRef)]
pub struct ServicesContext(dal::ServicesContext);

impl ServicesContext {
    pub fn into_inner(self) -> dal::ServicesContext {
        self.into()
    }
}

impl Deref for ServicesContext {
    type Target = dal::ServicesContext;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<dal::ServicesContext> for ServicesContext {
    fn from(value: dal::ServicesContext) -> Self {
        Self(value)
    }
}

impl From<ServicesContext> for dal::ServicesContext {
    fn from(value: ServicesContext) -> Self {
        value.0
    }
}

#[derive(Clone, Debug)]
pub struct SignupSecret(Arc<SensitiveString>);

impl<S> From<S> for SignupSecret
where
    S: Into<SensitiveString>,
{
    fn from(value: S) -> Self {
        Self(Arc::new(value.into()))
    }
}

#[derive(Clone, Debug, FromRef)]
pub struct JwtSecretKey(Arc<dal::JwtSecretKey>);

impl Deref for JwtSecretKey {
    type Target = dal::JwtSecretKey;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<JwtSecretKey> for Arc<dal::JwtSecretKey> {
    fn from(value: JwtSecretKey) -> Self {
        value.0
    }
}

impl From<dal::JwtSecretKey> for JwtSecretKey {
    fn from(value: dal::JwtSecretKey) -> Self {
        Self(Arc::new(value))
    }
}

#[derive(Clone, Debug)]
pub struct ShutdownBroadcast(broadcast::Sender<()>);

impl ShutdownBroadcast {
    pub fn subscribe(&self) -> broadcast::Receiver<()> {
        self.0.subscribe()
    }
}

impl From<broadcast::Sender<()>> for ShutdownBroadcast {
    fn from(value: broadcast::Sender<()>) -> Self {
        Self(value)
    }
}
