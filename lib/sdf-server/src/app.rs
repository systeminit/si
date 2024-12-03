use std::sync::Arc;

use asset_sprayer::AssetSprayer;
use audit_logs::database::AuditDatabaseContext;
use axum::Router;
use dal::{JwtPublicSigningKey, ServicesContext};
use nats_multiplexer_client::MultiplexerClient;
use si_data_spicedb::SpiceDbClient;
use si_posthog::PosthogClient;
use telemetry::prelude::*;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use tower_http::trace::TraceLayer;

use crate::{
    routes::routes, AppState, ApplicationRuntimeMode, WorkspacePermissions,
    WorkspacePermissionsMode,
};

#[derive(Debug)]
pub struct AxumApp(Router);

impl AxumApp {
    #[allow(clippy::too_many_arguments)]
    #[inline]
    pub fn from_services(
        services_context: ServicesContext,
        jwt_public_signing_key: JwtPublicSigningKey,
        posthog_client: PosthogClient,
        auth_api_url: impl AsRef<str>,
        asset_sprayer: Option<AssetSprayer>,
        ws_multiplexer_client: MultiplexerClient,
        crdt_multiplexer_client: MultiplexerClient,
        create_workspace_permissions: WorkspacePermissionsMode,
        create_workspace_allowlist: Vec<WorkspacePermissions>,
        application_runtime_mode: Arc<RwLock<ApplicationRuntimeMode>>,
        shutdown_token: CancellationToken,
        spicedb_client: Option<SpiceDbClient>,
        audit_database_context: AuditDatabaseContext,
    ) -> Self {
        Self::inner_from_services(
            services_context,
            jwt_public_signing_key,
            posthog_client,
            auth_api_url,
            asset_sprayer,
            false,
            ws_multiplexer_client,
            crdt_multiplexer_client,
            create_workspace_permissions,
            create_workspace_allowlist,
            application_runtime_mode,
            shutdown_token,
            spicedb_client,
            audit_database_context,
        )
    }

    // TODO(fnichol): This really smells, we should not be flipping the app behavior differently if
    // we're in a testing scenario. What is the configurable differences necessary to drive the app
    // under test? *That* is how the app should be configured--no `for_tests` bool.
    //
    // Until that is refactored though...this constructor remains.
    #[allow(clippy::too_many_arguments)]
    #[inline]
    pub fn from_services_for_tests(
        services_context: ServicesContext,
        jwt_public_signing_key: JwtPublicSigningKey,
        posthog_client: PosthogClient,
        auth_api_url: impl AsRef<str>,
        asset_sprayer: Option<AssetSprayer>,
        ws_multiplexer_client: MultiplexerClient,
        crdt_multiplexer_client: MultiplexerClient,
        create_workspace_permissions: WorkspacePermissionsMode,
        create_workspace_allowlist: Vec<WorkspacePermissions>,
        application_runtime_mode: Arc<RwLock<ApplicationRuntimeMode>>,
        shutdown_token: CancellationToken,
        spicedb_client: SpiceDbClient,
        audit_database_context: AuditDatabaseContext,
    ) -> Self {
        Self::inner_from_services(
            services_context,
            jwt_public_signing_key,
            posthog_client,
            auth_api_url,
            asset_sprayer,
            true,
            ws_multiplexer_client,
            crdt_multiplexer_client,
            create_workspace_permissions,
            create_workspace_allowlist,
            application_runtime_mode,
            shutdown_token,
            Some(spicedb_client),
            audit_database_context,
        )
    }

    pub fn into_inner(self) -> Router {
        self.0
    }

    #[allow(clippy::too_many_arguments)]
    fn inner_from_services(
        services_context: ServicesContext,
        jwt_public_signing_key: JwtPublicSigningKey,
        posthog_client: PosthogClient,
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
        let state = AppState::new(
            services_context,
            jwt_public_signing_key,
            posthog_client,
            auth_api_url,
            asset_sprayer,
            for_tests,
            ws_multiplexer_client,
            crdt_multiplexer_client,
            create_workspace_permissions,
            create_workspace_allowlist,
            application_runtime_mode,
            shutdown_token,
            spicedb_client,
            audit_database_context,
        );

        let path_filter = Box::new(|path: &str| match path {
            "/api/" => Some(Level::TRACE),
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
