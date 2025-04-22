use std::time::Duration;

use axum::{
    Router,
    extract::State,
    http::{
        Request,
        StatusCode,
    },
    middleware::{
        self,
        Next,
    },
    response::{
        IntoResponse,
        Json,
        Response,
    },
    routing::get,
};
use hyper::{
    Method,
    header,
};
use serde_json::{
    Value,
    json,
};
use si_data_nats::NatsError;
use si_data_pg::PgError;
use telemetry::prelude::*;
use thiserror::Error;
use tower_http::{
    compression::CompressionLayer,
    cors::{
        AllowOrigin,
        CorsLayer,
    },
};
use utoipa::{
    OpenApi,
    ToSchema,
};

use crate::{
    ServerError,
    app_state::{
        AppState,
        ApplicationRuntimeMode,
    },
    service::v1,
};

const MAINTENANCE_MODE_MESSAGE: &str = concat!(
    " SI is currently in maintenance mode. ",
    "Please refresh & try again later. ",
    "Reach out to support@systeminit.com ",
    "or in the SI Discord for more information if this problem persists",
);

async fn app_state_middeware<B>(
    State(state): State<AppState>,
    request: Request<B>,
    next: Next<B>,
) -> Response {
    match *state.application_runtime_mode.read().await {
        ApplicationRuntimeMode::Maintenance => {
            // Return a 503 when the server is in maintenance/offline
            (StatusCode::SERVICE_UNAVAILABLE, MAINTENANCE_MODE_MESSAGE).into_response()
        }
        ApplicationRuntimeMode::Running => next.run(request).await,
    }
}

// Define the OpenAPI document for your API
#[derive(OpenApi)]
#[openapi(
    paths(
        system_status_route,
        crate::service::whoami::whoami,
    ),
    components(
        schemas(
            SystemStatusResponse,
            ErrorResponse,
            ErrorDetail,
            crate::service::whoami::WhoamiResponse,
            crate::service::v1::common::ApiSuccess<String>,
            crate::service::v1::common::ApiError,
        )
    ),
    tags(
        (name = "root", description = "Root API endpoints"),
        (name = "whoami", description = "User identity endpoints"),
        (name = "change_sets", description = "Change set management endpoints"),
        (name = "components", description = "Component management endpoints"),
        (name = "management", description = "Management function endpoints")
    ),
    info(
        title = "Luminork API",
        description = "System Initiative External API server",
        version = "1.0.0"
    ),
    servers(
        (url = "/", description = "Public API Server for System Initiative")
    )
)]
struct ApiDoc;

// Define the schema for the system status response
#[derive(serde::Serialize, ToSchema)]
struct SystemStatusResponse {
    #[schema(example = "I am luminork, the new System Initiative External API server")]
    #[serde(rename = "What is this?")]
    what_is_this: String,

    #[schema(example = "Available at /swagger-ui")]
    #[serde(rename = "API Documentation")]
    api_documentation: String,
}

// Define the schema for error responses
#[derive(serde::Serialize, ToSchema)]
struct ErrorResponse {
    error: ErrorDetail,
}

#[derive(serde::Serialize, ToSchema)]
struct ErrorDetail {
    message: String,
    code: i32,
    status_code: u16,
}

#[allow(clippy::too_many_arguments)]
pub fn routes(state: AppState) -> Router {
    async fn openapi_handler() -> Result<Json<utoipa::openapi::OpenApi>, (StatusCode, String)> {
        let mut openapi = ApiDoc::openapi();
        let v1_openapi = v1::get_openapi();

        for (path, path_item) in v1_openapi.paths.paths {
            openapi.paths.paths.insert(path, path_item);
        }

        if let Some(openapi_components) = openapi.components.as_mut() {
            if let Some(v1_components) = v1_openapi.components {
                for (name, schema) in v1_components.schemas {
                    openapi_components.schemas.insert(name, schema);
                }
            }
        }

        Ok(Json(openapi))
    }

    Router::new()
        .nest("/whoami", crate::service::whoami::routes())
        .nest("/v1", crate::service::v1::routes(state.clone()))
        .route("/openapi.json", get(openapi_handler))
        // Add a route for Swagger UI HTML
        .route("/swagger-ui", get(serve_swagger_ui))
        .layer(CompressionLayer::new())
        .layer(
            // CORS configuration for public API
            CorsLayer::new()
                // Allow any origin since this is a public API
                .allow_origin(AllowOrigin::any())
                // Allow common headers needed for API operations
                .allow_headers([
                    header::ACCEPT,
                    header::ACCEPT_LANGUAGE,
                    header::AUTHORIZATION,
                    header::CONTENT_TYPE,
                    header::ORIGIN,
                    header::CACHE_CONTROL,
                    header::CONTENT_LANGUAGE,
                    header::PRAGMA,
                ])
                // Expose headers that clients might need to access
                .expose_headers([header::CONTENT_LENGTH, header::CONTENT_TYPE])
                // Allow standard methods
                .allow_methods([
                    Method::GET,
                    Method::POST,
                    Method::PUT,
                    Method::DELETE,
                    Method::OPTIONS,
                    Method::PATCH,
                ])
                // Cache preflight requests for 1 hour (3600 seconds)
                .max_age(Duration::from_secs(3600)),
        )
        .layer(middleware::from_fn_with_state(
            state.clone(),
            app_state_middeware,
        ))
        .nest(
            "/",
            Router::new().route("/", get(system_status_route).layer(CorsLayer::permissive())),
        )
        .with_state(state)
}

#[utoipa::path(
    get,
    path = "/",
    tag = "root",
    responses(
        (status = 200, description = "System status information", body = SystemStatusResponse),
        (status = 503, description = "Service in maintenance mode")
    )
)]
async fn system_status_route() -> Json<Value> {
    Json(json!({
        "What is this?": "I am luminork, the new System Initiative External API server",
        "API Documentation": "Available at /swagger-ui"
    }))
}

/// Serve a static HTML page that contains Swagger UI
async fn serve_swagger_ui() -> impl IntoResponse {
    const SWAGGER_UI_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8">
    <title>Luminork API Documentation</title>
    <link rel="stylesheet" type="text/css" href="https://unpkg.com/swagger-ui-dist@5.1.0/swagger-ui.css">
    <link rel="icon" type="image/png" href="https://unpkg.com/swagger-ui-dist@5.1.0/favicon-32x32.png" sizes="32x32">
    <link rel="icon" type="image/png" href="https://unpkg.com/swagger-ui-dist@5.1.0/favicon-16x16.png" sizes="16x16">
    <style>
      html { box-sizing: border-box; overflow: -moz-scrollbars-vertical; overflow-y: scroll; }
      *, *:before, *:after { box-sizing: inherit; }
      body { margin: 0; background: #fafafa; }
      .topbar { display: none; }
    </style>
  </head>
  <body>
    <div id="swagger-ui"></div>

    <script src="https://unpkg.com/swagger-ui-dist@5.1.0/swagger-ui-bundle.js" charset="UTF-8"></script>
    <script src="https://unpkg.com/swagger-ui-dist@5.1.0/swagger-ui-standalone-preset.js" charset="UTF-8"></script>
    <script>
      window.onload = function() {
        const ui = SwaggerUIBundle({
          url: "/openapi.json",
          dom_id: '#swagger-ui',
          deepLinking: true,
          presets: [
            SwaggerUIBundle.presets.apis,
            SwaggerUIStandalonePreset
          ],
          plugins: [
            SwaggerUIBundle.plugins.DownloadUrl
          ],
          layout: "StandaloneLayout",
          defaultModelsExpandDepth: 3,
          defaultModelExpandDepth: 3,
          docExpansion: "list",
          tryItOutEnabled: true,
          displayRequestDuration: true,
          filter: true,
          persistAuthorization: true, // Remember auth token between page refreshes
          // Define security scheme for JWT authentication (since OpenAPI doesn't support it)
          requestInterceptor: function(request) {
            const authToken = localStorage.getItem('swagger_ui_token');
            if (authToken) {
              request.headers.Authorization = "Bearer " + authToken;
            }
            return request;
          },
          syntaxHighlight: {
            activate: true,
            theme: "agate"
          },
          // Add a custom authorization UI
          onComplete: function() {
            // Get saved token from localStorage
            const savedToken = localStorage.getItem('swagger_ui_token') || '';

            // Create UI elements
            const authContainer = document.createElement('div');
            authContainer.id = 'swagger-ui-auth';
            authContainer.style.cssText = 'margin:20px 0; padding:20px; background:#fafafa; border:1px solid #ddd;';

            const heading = document.createElement('h3');
            heading.textContent = 'JWT Authorization';
            heading.style.margin = '0 0 10px 0';

            const tokenInput = document.createElement('input');
            tokenInput.type = 'text';
            tokenInput.id = 'jwt-token-input';
            tokenInput.placeholder = 'Enter JWT token';
            tokenInput.value = savedToken;
            tokenInput.style.cssText = 'width:100%; padding:8px; margin:10px 0; border:1px solid #ddd;';

            const buttonContainer = document.createElement('div');

            const authButton = document.createElement('button');
            authButton.textContent = 'Authorize';
            authButton.style.cssText = 'background:#4990e2; color:white; border:none; padding:8px 16px; margin-right:10px; cursor:pointer;';

            const clearButton = document.createElement('button');
            clearButton.textContent = 'Clear';
            clearButton.style.cssText = 'background:#e57373; color:white; border:none; padding:8px 16px; cursor:pointer;';

            // Event handlers
            authButton.onclick = function() {
              const token = tokenInput.value.trim();
              if (token) {
                localStorage.setItem('swagger_ui_token', token);
                alert('Token saved. The Authorization header will be added to all requests.');
              } else {
                alert('Please enter a valid token');
              }
            };

            clearButton.onclick = function() {
              tokenInput.value = '';
              localStorage.removeItem('swagger_ui_token');
              alert('Token cleared. The Authorization header will not be added to requests.');
            };

            // Assemble the DOM
            buttonContainer.appendChild(authButton);
            buttonContainer.appendChild(clearButton);

            authContainer.appendChild(heading);
            authContainer.appendChild(tokenInput);
            authContainer.appendChild(buttonContainer);

            // Insert at the top of the Swagger UI
            const swaggerUIContainer = document.getElementById('swagger-ui');
            swaggerUIContainer.insertBefore(authContainer, swaggerUIContainer.firstChild);
          }
        });
        window.ui = ui;
      };
    </script>
  </body>
</html>"#;

    (
        StatusCode::OK,
        [("Content-Type", "text/html")],
        SWAGGER_UI_HTML,
    )
}

#[allow(clippy::large_enum_variant)]
#[remain::sorted]
#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    Nats(#[from] NatsError),
    #[error(transparent)]
    Pg(#[from] PgError),
    #[error(transparent)]
    Server(#[from] ServerError),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = (StatusCode::INTERNAL_SERVER_ERROR, self.to_string());

        let body = Json(serde_json::json!({
            "error": {
                "message": error_message,
                "code": 42,
                "statusCode": status.as_u16(),
            },
        }));
        error!(si.error.message = error_message);
        (status, body).into_response()
    }
}
