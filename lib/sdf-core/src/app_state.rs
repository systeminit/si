use std::{
    fmt,
    ops::Deref,
    sync::Arc,
};

use audit_database::AuditDatabaseContext;
use axum::extract::FromRef;
use edda_client::EddaClient;
use frigg::FriggStore;
use nats_multiplexer_client::MultiplexerClient;
use si_data_spicedb::SpiceDbClient;
use si_jwt_public_key::JwtPublicSigningKeyChain;
use tokio::sync::{
    Mutex,
    RwLock,
};
use tokio_util::sync::CancellationToken;

use crate::{
    BroadcastGroups,
    nats_multiplexer::{
        EddaUpdatesMultiplexerClient,
        NatsMultiplexerClients,
    },
    workspace_permissions::{
        WorkspacePermissions,
        WorkspacePermissionsMode,
    },
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
    jwt_public_signing_key_chain: JwtPublicSigningKeyChain,
    posthog_client: PosthogClient,
    auth_api_url: String, // TODO(victor) store the auth client on state instead of just the URL
    for_tests: bool,
    nats_multiplexer_clients: NatsMultiplexerClients,
    create_workspace_permissions: WorkspacePermissionsMode,
    create_workspace_allowlist: Vec<WorkspacePermissions>,
    pub application_runtime_mode: Arc<RwLock<ApplicationRuntimeMode>>,
    shutdown_token: CancellationToken,
    spicedb_client: Option<SpiceDbClient>,
    frigg: FriggStore,
    audit_database_context: AuditDatabaseContext,
    edda_client: EddaClient,
}

impl AppState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        services_context: impl Into<ServicesContext>,
        jwt_public_signing_key_chain: JwtPublicSigningKeyChain,
        posthog_client: impl Into<PosthogClient>,
        auth_api_url: impl AsRef<str>,
        for_tests: bool,
        ws_multiplexer_client: MultiplexerClient,
        crdt_multiplexer_client: MultiplexerClient,
        edda_updates_multiplexer_client: EddaUpdatesMultiplexerClient,
        create_workspace_permissions: WorkspacePermissionsMode,
        create_workspace_allowlist: Vec<WorkspacePermissions>,
        application_runtime_mode: Arc<RwLock<ApplicationRuntimeMode>>,
        shutdown_token: CancellationToken,
        spicedb_client: Option<SpiceDbClient>,
        frigg: FriggStore,
        audit_database_context: AuditDatabaseContext,
        edda_client: EddaClient,
    ) -> Self {
        let nats_multiplexer_clients = NatsMultiplexerClients {
            ws: Arc::new(Mutex::new(ws_multiplexer_client)),
            crdt: Arc::new(Mutex::new(crdt_multiplexer_client)),
            edda_updates: edda_updates_multiplexer_client,
        };

        Self {
            services_context: services_context.into(),
            jwt_public_signing_key_chain,
            broadcast_groups: Default::default(),
            posthog_client: posthog_client.into(),
            auth_api_url: auth_api_url.as_ref().to_string(),
            for_tests,
            nats_multiplexer_clients,
            create_workspace_permissions,
            create_workspace_allowlist,
            application_runtime_mode,
            shutdown_token,
            spicedb_client,
            frigg,
            audit_database_context,
            edda_client,
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

    pub fn jwt_public_signing_key_chain(&self) -> &JwtPublicSigningKeyChain {
        &self.jwt_public_signing_key_chain
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

    pub fn frigg(&self) -> &FriggStore {
        &self.frigg
    }

    pub fn edda_client(&self) -> &EddaClient {
        &self.edda_client
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
