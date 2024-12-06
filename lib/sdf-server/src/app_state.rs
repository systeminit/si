use std::{ops::Deref, sync::Arc};

use asset_sprayer::AssetSprayer;
use audit_database::AuditDatabaseContext;
use axum::extract::FromRef;
use dal::JwtPublicSigningKey;
use nats_multiplexer_client::MultiplexerClient;
use si_data_spicedb::SpiceDbClient;
use std::fmt;
use tokio::sync::{Mutex, RwLock};
use tokio_util::sync::CancellationToken;

use crate::{
    nats_multiplexer::NatsMultiplexerClients, service::ws::crdt::BroadcastGroups,
    WorkspacePermissions, WorkspacePermissionsMode,
};

#[remain::sorted]
#[derive(Debug, Clone, Copy)]
pub enum ApplicationRuntimeMode {
    Maintenance,
    Running,
}

#[derive(Clone, FromRef)]
pub struct AppState {
    services_context: ServicesContext,
    broadcast_groups: BroadcastGroups,
    jwt_public_signing_key: JwtPublicSigningKey,
    posthog_client: PosthogClient,
    auth_api_url: String, // TODO(victor) store the auth client on state instead of just the URL
    asset_sprayer: Option<AssetSprayer>,
    for_tests: bool,
    nats_multiplexer_clients: NatsMultiplexerClients,
    create_workspace_permissions: WorkspacePermissionsMode,
    create_workspace_allowlist: Vec<WorkspacePermissions>,
    pub application_runtime_mode: Arc<RwLock<ApplicationRuntimeMode>>,
    shutdown_token: CancellationToken,
    spicedb_client: Option<SpiceDbClient>,
    audit_database_context: AuditDatabaseContext,
}

impl AppState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        services_context: impl Into<ServicesContext>,
        jwt_public_signing_key: impl Into<JwtPublicSigningKey>,
        posthog_client: impl Into<PosthogClient>,
        auth_api_url: impl AsRef<str>,
        asset_sprayer: Option<AssetSprayer>,
        for_tests: bool,
        ws_multiplexer_client: MultiplexerClient,
        crdt_multiplexer_client: MultiplexerClient,
        create_workspace_permissions: WorkspacePermissionsMode,
        create_workspace_allowlist: Vec<WorkspacePermissions>,
        application_runtime_mode: Arc<RwLock<ApplicationRuntimeMode>>,
        shutdown_token: CancellationToken,
        spicedb_client: Option<SpiceDbClient>,
        audit_database_context: AuditDatabaseContext,
    ) -> Self {
        let nats_multiplexer_clients = NatsMultiplexerClients {
            ws: Arc::new(Mutex::new(ws_multiplexer_client)),
            crdt: Arc::new(Mutex::new(crdt_multiplexer_client)),
        };

        Self {
            services_context: services_context.into(),
            jwt_public_signing_key: jwt_public_signing_key.into(),
            broadcast_groups: Default::default(),
            posthog_client: posthog_client.into(),
            auth_api_url: auth_api_url.as_ref().to_string(),
            asset_sprayer,
            for_tests,
            nats_multiplexer_clients,
            create_workspace_permissions,
            create_workspace_allowlist,
            application_runtime_mode,
            shutdown_token,
            spicedb_client,
            audit_database_context,
        }
    }

    pub fn services_context(&self) -> &ServicesContext {
        &self.services_context
    }

    pub fn posthog_client(&self) -> &PosthogClient {
        &self.posthog_client
    }

    pub fn auth_api_url(&self) -> &String {
        &self.auth_api_url
    }

    pub fn asset_sprayer(&self) -> Option<&AssetSprayer> {
        self.asset_sprayer.as_ref()
    }

    pub fn jwt_public_signing_key(&self) -> &JwtPublicSigningKey {
        &self.jwt_public_signing_key
    }

    pub fn for_tests(&self) -> bool {
        self.for_tests
    }

    pub fn create_workspace_permissions(&self) -> WorkspacePermissionsMode {
        self.create_workspace_permissions
    }

    pub fn create_workspace_allowlist(&self) -> &[String] {
        &self.create_workspace_allowlist
    }

    pub fn shutdown_token(&self) -> &CancellationToken {
        &self.shutdown_token
    }

    pub fn spicedb_client(&mut self) -> Option<&mut SpiceDbClient> {
        self.spicedb_client.as_mut()
    }

    pub fn spicedb_client_clone(&self) -> Option<SpiceDbClient> {
        self.spicedb_client.clone()
    }

    pub fn audit_database_context(&self) -> &AuditDatabaseContext {
        &self.audit_database_context
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

#[derive(Clone, FromRef)]
pub struct ServicesContext(dal::ServicesContext);

impl fmt::Debug for ServicesContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ServicesContext").finish_non_exhaustive()
    }
}

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
