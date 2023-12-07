use std::{ops::Deref, sync::Arc};

use axum::extract::FromRef;
use dal::JwtPublicSigningKey;
use si_std::SensitiveString;
use tokio::sync::{broadcast, mpsc};

use super::server::ShutdownSource;

#[derive(Clone, FromRef)]
pub struct AppState {
    services_context: ServicesContext,
    signup_secret: SignupSecret,
    jwt_public_signing_key: JwtPublicSigningKey,
    posthog_client: PosthogClient,
    shutdown_broadcast: ShutdownBroadcast,
    for_tests: bool,

    // TODO(fnichol): we're likely going to use this, but we can't allow it to be dropped because
    // that will trigger the read side and... shutdown. Cool, no?
    #[from_ref(skip)]
    _tmp_shutdown_tx: Arc<mpsc::Sender<ShutdownSource>>,
}

impl AppState {
    pub fn new(
        services_context: impl Into<ServicesContext>,
        signup_secret: impl Into<SignupSecret>,
        jwt_public_signing_key: impl Into<JwtPublicSigningKey>,
        posthog_client: impl Into<PosthogClient>,
        shutdown_broadcast_tx: broadcast::Sender<()>,
        tmp_shutdown_tx: mpsc::Sender<ShutdownSource>,
        for_tests: bool,
    ) -> Self {
        Self {
            services_context: services_context.into(),
            signup_secret: signup_secret.into(),
            jwt_public_signing_key: jwt_public_signing_key.into(),
            posthog_client: posthog_client.into(),
            shutdown_broadcast: ShutdownBroadcast(shutdown_broadcast_tx),
            for_tests,
            _tmp_shutdown_tx: Arc::new(tmp_shutdown_tx),
        }
    }

    pub fn services_context(&self) -> &ServicesContext {
        &self.services_context
    }

    pub fn posthog_client(&self) -> &PosthogClient {
        &self.posthog_client
    }

    pub fn jwt_public_signing_key(&self) -> &JwtPublicSigningKey {
        &self.jwt_public_signing_key
    }

    pub fn for_tests(&self) -> bool {
        self.for_tests
    }
}

#[derive(Clone, Debug, FromRef)]
pub struct PosthogClient(si_posthog::PosthogClient);

impl PosthogClient {
    pub fn into_inner(self) -> si_posthog::PosthogClient {
        self.into()
    }
}

impl Deref for PosthogClient {
    type Target = si_posthog::PosthogClient;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<si_posthog::PosthogClient> for PosthogClient {
    fn from(value: si_posthog::PosthogClient) -> Self {
        Self(value)
    }
}

impl From<PosthogClient> for si_posthog::PosthogClient {
    fn from(value: PosthogClient) -> Self {
        value.0
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

#[derive(Clone, Debug)]
pub struct ShutdownBroadcast(broadcast::Sender<()>);

impl ShutdownBroadcast {
    #[allow(dead_code)]
    pub fn subscribe(&self) -> broadcast::Receiver<()> {
        self.0.subscribe()
    }
}

impl From<broadcast::Sender<()>> for ShutdownBroadcast {
    fn from(value: broadcast::Sender<()>) -> Self {
        Self(value)
    }
}
