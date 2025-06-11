use std::sync::Arc;

use audit_database::AuditDatabaseContext;
use axum::Router;
use dal::ServicesContext;
use edda_client::EddaClient;
use frigg::FriggStore;
use nats_multiplexer_client::MultiplexerClient;
use sdf_core::nats_multiplexer::EddaUpdatesMultiplexerClient;
use si_data_spicedb::SpiceDbClient;
use si_jwt_public_key::JwtPublicSigningKeyChain;
use si_posthog::PosthogClient;
use telemetry::prelude::*;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use tower_http::trace::TraceLayer;

use crate::{
    AppState,
    ApplicationRuntimeMode,
    WorkspacePermissions,
    WorkspacePermissionsMode,
    routes::routes,
};

#[derive(Debug)]
pub struct AxumApp(Router);

impl AxumApp {
    #[allow(clippy::too_many_arguments)]
    #[inline]
    pub fn from_services(
        services_context: ServicesContext,
        jwt_public_signing_key_chain: JwtPublicSigningKeyChain,
        posthog_client: PosthogClient,
        auth_api_url: impl AsRef<str>,
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
        Self::inner_from_services(
            services_context,
            jwt_public_signing_key_chain,
            posthog_client,
            auth_api_url,
            false,
            ws_multiplexer_client,
            crdt_multiplexer_client,
            edda_updates_multiplexer_client,
            create_workspace_permissions,
            create_workspace_allowlist,
            application_runtime_mode,
            shutdown_token,
            spicedb_client,
            frigg,
            audit_database_context,
            edda_client,
        )
    }

    pub fn into_inner(self) -> Router {
        self.0
    }

    #[allow(clippy::too_many_arguments)]
    fn inner_from_services(
        services_context: ServicesContext,
        jwt_public_signing_key_chain: JwtPublicSigningKeyChain,
        posthog_client: PosthogClient,
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
        let state = AppState::new(
            services_context,
            jwt_public_signing_key_chain,
            posthog_client,
            auth_api_url,
            for_tests,
            ws_multiplexer_client,
            crdt_multiplexer_client,
            edda_updates_multiplexer_client,
            create_workspace_permissions,
            create_workspace_allowlist,
            application_runtime_mode,
            shutdown_token,
            spicedb_client,
            frigg,
            audit_database_context,
            edda_client,
        );

        let path_filter = Box::new(|path: &str| match path {
            "/" => Some(Level::TRACE),
            _ => None,
        });

        let app = routes(state).layer(
            TraceLayer::new_for_http()
                .make_span_with(
                    telemetry_http::HttpMakeSpan::builder()
                        .level(Level::INFO)
                        .path_filter(path_filter)
                        .build(),
                )
                .on_response(telemetry_http::HttpOnResponse::new().level(Level::DEBUG)),
        );

        Self(app)
    }
}
