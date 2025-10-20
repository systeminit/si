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
    service::{
        v1,
        workspace_management,
    },
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
            (StatusCode::SERVICE_UNAVAILABLE, MAINTENANCE_MODE_MESSAGE).into_response()
        }
        ApplicationRuntimeMode::Running => next.run(request).await,
    }
}

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
        (name = "change_sets", description = "Change Set management endpoints"),
        (name = "components", description = "Components management endpoints"),
        (name = "schemas", description = "Schemas management endpoints"),
        (name = "actions", description = "Actions management endpoints"),
        (name = "secrets", description = "Secrets management endpoints"),
        (name = "funcs", description = "Functions management endpoints"),
        (name = "management_funcs", description = "Management functions endpoints"),
        (name = "workspace_management", description = "Workspace management endpoints"),
    ),
    info(
        title = "System Initiative API",
        description = "The API Server for interacting with a System Initiative workspace",
        version = "1.0.0"
    ),
    servers(
        (url = "/", description = "This API Server")
    )
)]
struct ApiDoc;

#[derive(serde::Serialize, ToSchema)]
struct SystemStatusResponse {
    #[schema(example = "Available at /swagger-ui")]
    #[serde(rename = "API Documentation")]
    api_documentation: String,
}

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

pub async fn openapi_handler() -> Result<Json<utoipa::openapi::OpenApi>, (StatusCode, String)> {
    let mut openapi = ApiDoc::openapi();
    let v1_openapi = v1::get_openapi();
    let workspace_management_openapi = workspace_management::get_openapi();

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

    for (path, path_item) in workspace_management_openapi.paths.paths {
        openapi.paths.paths.insert(path, path_item);
    }

    if let Some(openapi_components) = openapi.components.as_mut() {
        if let Some(components) = workspace_management_openapi.components {
            for (name, schema) in components.schemas {
                openapi_components.schemas.insert(name, schema);
            }
        }
    }

    Ok(Json(openapi))
}

#[allow(clippy::too_many_arguments)]
pub fn routes(state: AppState) -> Router {
    async fn openapi_handler_with_globals() -> Result<Json<serde_json::Value>, (StatusCode, String)>
    {
        let original_response = openapi_handler().await?;
        let mut spec = serde_json::to_value(original_response.0).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("JSON conversion error: {e}"),
            )
        })?;

        if let Some(components) = spec.get_mut("components") {
            if let Some(comp_obj) = components.as_object_mut() {
                if !comp_obj.contains_key("parameters") {
                    comp_obj.insert("parameters".to_string(), json!({}));
                }

                if let Some(parameters) = comp_obj.get_mut("parameters") {
                    if let Some(params_obj) = parameters.as_object_mut() {
                        params_obj.insert(
                            "WorkspaceId".to_string(),
                            json!({
                                "name": "workspace_id",
                                "in": "query",
                                "description": "Workspace ID - can be set globally in the UI",
                                "required": false,
                                "schema": {
                                    "type": "string",
                                    "format": "uuid"
                                }
                            }),
                        );

                        params_obj.insert(
                            "ChangeSetId".to_string(),
                            json!({
                                "name": "change_set_id",
                                "in": "query",
                                "description": "Change Set ID - can be set globally in the UI",
                                "required": false,
                                "schema": {
                                    "type": "string",
                                    "format": "uuid"
                                }
                            }),
                        );
                    }
                }
            }
        }

        Ok(Json(spec))
    }

    Router::new()
        .nest("/whoami", crate::service::whoami::routes())
        .nest(
            "/management",
            crate::service::workspace_management::routes(state.clone()),
        )
        .nest("/v1", crate::service::v1::routes(state.clone()))
        .route("/openapi.json", get(openapi_handler_with_globals))
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
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 503, description = "Service in maintenance mode")
    )
)]
async fn system_status_route() -> Json<Value> {
    Json(json!({
        "API Documentation": "Available at /swagger-ui"
    }))
}

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
        // Create a plugin to intercept executions and add global parameters
        const GlobalParametersPlugin = function() {
          return {
            statePlugins: {
              spec: {
                wrapActions: {
                  // Intercept the execute request action
                  executeRequest: (oriAction) => (req) => {
                    try {
                      // Get the global parameters from localStorage
                      const workspaceId = localStorage.getItem('swagger_ui_workspace_id');
                      const changeSetId = localStorage.getItem('swagger_ui_change_set_id');

                      // Clone the request object
                      let requestWithParams = {...req};

                      // Check if the request has parameters
                      if (!requestWithParams.parameters) {
                        requestWithParams.parameters = {};
                      }

                      // Add workspace_id if it's not already set and we have a global value
                      if (workspaceId && !requestWithParams.parameters.workspace_id) {
                        requestWithParams.parameters.workspace_id = workspaceId;
                      }

                      // Add change_set_id if it's not already set and we have a global value
                      if (changeSetId && !requestWithParams.parameters.change_set_id) {
                        requestWithParams.parameters.change_set_id = changeSetId;
                      }

                      return oriAction(requestWithParams);
                    } catch (error) {
                      console.error("Error in executeRequest interceptor:", error);
                      return oriAction(req);
                    }
                  }
                }
              }
            }
          };
        };

        const ui = SwaggerUIBundle({
          url: "/openapi.json",
          dom_id: '#swagger-ui',
          deepLinking: true,
          presets: [
            SwaggerUIBundle.presets.apis,
            SwaggerUIStandalonePreset
          ],
          plugins: [
            SwaggerUIBundle.plugins.DownloadUrl,
            GlobalParametersPlugin
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
            // Get saved values from localStorage
            const savedToken = localStorage.getItem('swagger_ui_token') || '';
            const savedWorkspaceId = localStorage.getItem('swagger_ui_workspace_id') || '';
            const savedChangeSetId = localStorage.getItem('swagger_ui_change_set_id') || '';

            // Create UI elements
            const authContainer = document.createElement('div');
            authContainer.id = 'swagger-ui-auth';
            authContainer.style.cssText = 'margin:20px 0; padding:20px; background:#fafafa; border:1px solid #ddd;';

            // Add a title for the global parameters section
            const globalTitle = document.createElement('h3');
            globalTitle.textContent = 'Global Parameters';
            globalTitle.style.margin = '0 0 15px 0';
            globalTitle.style.borderBottom = '1px solid #ddd';
            globalTitle.style.paddingBottom = '10px';

            const globalDescription = document.createElement('p');
            globalDescription.textContent = 'Set global parameters that will be automatically applied to all API requests.';
            globalDescription.style.margin = '0 0 20px 0';
            globalDescription.style.color = '#666';

            // Function to create form groups for consistency
            const createFormGroup = (title, inputId, placeholder, savedValue, description) => {
              const group = document.createElement('div');
              group.style.marginBottom = '15px';

              const label = document.createElement('label');
              label.textContent = title;
              label.style.display = 'block';
              label.style.fontWeight = 'bold';
              label.style.marginBottom = '5px';

              const input = document.createElement('input');
              input.type = 'text';
              input.id = inputId;
              input.placeholder = placeholder;
              input.value = savedValue;
              input.style.cssText = 'width:100%; padding:8px; border:1px solid #ddd;';

              group.appendChild(label);

              if (description) {
                const desc = document.createElement('p');
                desc.textContent = description;
                desc.style.margin = '0 0 5px 0';
                desc.style.fontSize = '12px';
                desc.style.color = '#666';
                group.appendChild(desc);
              }

              group.appendChild(input);

              return { group, input };
            };

            // Create form elements
            const { group: tokenGroup, input: tokenInput } = createFormGroup(
              'JWT Authorization Token',
              'jwt-token-input',
              'Enter JWT token',
              savedToken,
              'Required: Authorization token will be added as a Bearer token in the Authorization header'
            );

            const { group: workspaceGroup, input: workspaceInput } = createFormGroup(
              'Workspace ID',
              'workspace-id-input',
              'Enter workspace ID (optional)',
              savedWorkspaceId,
              'Optional: Will be added to requests that require a workspace_id parameter'
            );

            const { group: changeSetGroup, input: changeSetInput } = createFormGroup(
              'Change Set ID',
              'change-set-id-input',
              'Enter change set ID (optional)',
              savedChangeSetId,
              'Optional: Will be added to requests that require a change_set_id parameter'
            );

            const buttonContainer = document.createElement('div');
            buttonContainer.style.marginTop = '10px';

            const authButton = document.createElement('button');
            authButton.textContent = 'Save Settings';
            authButton.style.cssText = 'background:#4990e2; color:white; border:none; padding:8px 16px; margin-right:10px; cursor:pointer;';

            const clearButton = document.createElement('button');
            clearButton.textContent = 'Clear All';
            clearButton.style.cssText = 'background:#e57373; color:white; border:none; padding:8px 16px; cursor:pointer;';

            // Event handlers
            // Create a notification div for status messages
            const notificationDiv = document.createElement('div');
            notificationDiv.id = 'swagger-notification';
            notificationDiv.style.cssText = 'margin-top:10px; padding:8px; border-radius:4px; display:none;';

            // Function to show notifications without using alert()
            const showNotification = (message, isSuccess = true) => {
              notificationDiv.textContent = message;
              notificationDiv.style.display = 'block';
              notificationDiv.style.backgroundColor = isSuccess ? '#dff0d8' : '#f2dede';
              notificationDiv.style.color = isSuccess ? '#3c763d' : '#a94442';

              // Hide after 3 seconds
              setTimeout(() => {
                notificationDiv.style.display = 'none';
              }, 3000);
            };

            authButton.onclick = function() {
              // Clear any existing notifications
              notificationDiv.style.display = 'none';

              const token = tokenInput.value.trim();
              const workspaceId = workspaceInput.value.trim();
              const changeSetId = changeSetInput.value.trim();

              // Validate auth token is required
              if (!token) {
                showNotification('⚠️ Authorization token is required.', false);
                return;
              }

              // Save all values to localStorage
              localStorage.setItem('swagger_ui_token', token);
              localStorage.setItem('swagger_ui_workspace_id', workspaceId);
              localStorage.setItem('swagger_ui_change_set_id', changeSetId);

              showNotification('Settings saved. Your values will be applied to all requests.');

              // Apply the authorization token
              setTimeout(() => {
                window.ui.preauthorizeApiKey('Bearer', token);
              }, 100);
            };

            clearButton.onclick = function() {
              // Clear any existing notifications
              notificationDiv.style.display = 'none';

              // Clear input fields
              tokenInput.value = '';
              workspaceInput.value = '';
              changeSetInput.value = '';

              // Clear localStorage
              localStorage.removeItem('swagger_ui_token');
              localStorage.removeItem('swagger_ui_workspace_id');
              localStorage.removeItem('swagger_ui_change_set_id');

              showNotification('All settings cleared.');

              // Remove authorization
              setTimeout(() => {
                window.ui.preauthorizeApiKey('Bearer', '');
              }, 100);
            };

            // Add a reload the page button to force refresh the parameters
            const reloadButton = document.createElement('button');
            reloadButton.textContent = 'Apply & Reload';
            reloadButton.style.cssText = 'background:#4CAF50; color:white; border:none; padding:8px 16px; margin-right:10px; cursor:pointer;';
            reloadButton.onclick = function() {
              // Clear any existing notifications
              notificationDiv.style.display = 'none';

              // Save settings
              const token = tokenInput.value.trim();
              const workspaceId = workspaceInput.value.trim();
              const changeSetId = changeSetInput.value.trim();

              // Validate auth token is required
              if (!token) {
                showNotification('⚠️ Authorization token is required.', false);
                return;
              }

              localStorage.setItem('swagger_ui_token', token);
              localStorage.setItem('swagger_ui_workspace_id', workspaceId);
              localStorage.setItem('swagger_ui_change_set_id', changeSetId);

              // Show notification and reload
              showNotification('Settings saved. Reloading page to apply changes...');
              setTimeout(() => {
                window.location.reload();
              }, 1000);
            };

            // Assemble the DOM
            buttonContainer.appendChild(authButton);
            buttonContainer.appendChild(clearButton);
            buttonContainer.appendChild(reloadButton);

            authContainer.appendChild(globalTitle);
            authContainer.appendChild(globalDescription);
            authContainer.appendChild(tokenGroup);
            authContainer.appendChild(workspaceGroup);
            authContainer.appendChild(changeSetGroup);
            authContainer.appendChild(buttonContainer);
            authContainer.appendChild(notificationDiv);

            // Add a status indicator that shows when global params are active
            const statusDiv = document.createElement('div');
            statusDiv.style.cssText = 'margin-top:15px; padding:8px; border-left:4px solid #4990e2; background:#f8f8f8;';
            statusDiv.innerHTML = '<strong>Status:</strong> <span id="global-params-status">Loading...</span>';
            authContainer.appendChild(statusDiv);

            // Simple status update function
            const updateStatus = () => {
              const hasWorkspace = localStorage.getItem('swagger_ui_workspace_id');
              const hasChangeSet = localStorage.getItem('swagger_ui_change_set_id');
              const hasToken = localStorage.getItem('swagger_ui_token');

              const statusSpan = document.getElementById('global-params-status');
              if (statusSpan) {
                const items = [];
                if (hasToken) items.push('Auth Token');
                if (hasWorkspace) items.push('Workspace ID');
                if (hasChangeSet) items.push('Change Set ID');

                statusSpan.textContent = items.length > 0 ?
                  `Using global ${items.join(', ')}` :
                  'No global parameters set';
                statusSpan.style.color = items.length > 0 ? '#3c763d' : '#777';
              }
            };

            // Update status on load and when values change
            updateStatus();
            authButton.addEventListener('click', updateStatus);
            clearButton.addEventListener('click', updateStatus);

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
    #[error("nats error: {0}")]
    Nats(#[from] NatsError),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("server error: {0}")]
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
