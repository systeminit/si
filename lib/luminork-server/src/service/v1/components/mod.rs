use axum::{
    Router,
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::IntoResponse,
    routing::{
        delete,
        get,
        post,
        put,
    },
};
use dal::{
    ActionPrototypeId,
    Component,
    ComponentId,
    Func,
    PropId,
    SchemaVariantId,
    action::{
        ActionError,
        prototype::{
            ActionPrototype,
            ActionPrototypeError,
        },
    },
    management::prototype::ManagementPrototype,
    prop::{
        PROP_PATH_SEPARATOR,
        PropPath,
        PropResult,
    },
};
use serde::{
    Deserialize,
    Serialize,
};
use thiserror::Error;
use utoipa::ToSchema;

use crate::AppState;

pub mod add_action;
pub mod connections;
pub mod create_component;
pub mod delete_component;
pub mod execute_management_function;
pub mod find_component;
pub mod get_component;
pub mod list_components;
pub mod search_components;
pub mod update_component;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ComponentsError {
    #[error("action error: {0}")]
    Action(#[from] ActionError),
    #[error("action already enqueued: {0}")]
    ActionAlreadyEnqueued(ActionPrototypeId),
    #[error("action function not found: {0}")]
    ActionFunctionNotFound(String),
    #[error("component error: {0}")]
    ActionPrototype(#[from] ActionPrototypeError),
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] dal::attribute::value::AttributeValueError),
    #[error("cached module error: {0}")]
    CachedModule(#[from] dal::cached_module::CachedModuleError),
    #[error("component error: {0}")]
    Component(#[from] dal::ComponentError),
    #[error("component not found: {0}")]
    ComponentNotFound(String),
    #[error("dal change set error: {0}")]
    DalChangeSet(#[from] dal::ChangeSetError),
    #[error("diagram error: {0}")]
    Diagram(#[from] dal::diagram::DiagramError),
    #[error(
        "ambiguous action function name reference: {0} (found multiple action functions with this name)"
    )]
    DuplicateActionFunctionName(String),
    #[error("ambiguous component name reference: {0} (found multiple components with this name)")]
    DuplicateComponentName(String),
    #[error(
        "ambiguous management function name reference: {0} (found multiple management functions with this name)"
    )]
    DuplicateManagementFunctionName(String),
    #[error("func error: {0}")]
    Func(#[from] dal::FuncError),
    #[error("input socket error: {0}")]
    InputSocket(#[from] dal::socket::input::InputSocketError),
    #[error("invalid secret value: {0}")]
    InvalidSecretValue(String),
    #[error("management func error: {0}")]
    ManagementFuncExecution(#[from] si_db::ManagementFuncExecutionError),
    #[error("management function already running for this component")]
    ManagementFunctionAlreadyRunning,
    #[error("management function execution error: {0}")]
    ManagementFunctionExecutionFailed(si_db::ManagementFuncExecutionError),
    #[error("management function not found: {0}")]
    ManagementFunctionNotFound(String),
    #[error("prop error: {0}")]
    ManagementPrototype(#[from] dal::management::prototype::ManagementPrototypeError),
    #[error("output socket error: {0}")]
    OutputSocket(#[from] dal::socket::output::OutputSocketError),
    #[error("prop error: {0}")]
    Prop(#[from] dal::prop::PropError),
    #[error("schema error: {0}")]
    Schema(#[from] dal::SchemaError),
    #[error("schema not found by name error: {0}")]
    SchemaNameNotFound(String),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] dal::SchemaVariantError),
    #[error("secret error: {0}")]
    Secret(#[from] dal::SecretError),
    #[error("secret not found: {0}")]
    SecretNotFound(String),
    #[error("serde_json error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("transactions error: {0}")]
    Transactions(#[from] dal::TransactionsError),
    #[error("Ulid Decode Error: {0}")]
    UlidDecode(#[from] ulid::DecodeError),
    #[error("validation error: {0}")]
    Validation(String),
    #[error("view not found: {0}")]
    ViewNotFound(String),
    #[error("ws event error: {0}")]
    WsEvent(#[from] dal::WsEventError),
}

pub type ComponentsResult<T> = Result<T, ComponentsError>;

/// Resolves a secret value (ID or name) to a SecretId
pub async fn resolve_secret_id(
    ctx: &dal::DalContext,
    value: &serde_json::Value,
) -> ComponentsResult<dal::SecretId> {
    match value {
        serde_json::Value::String(value_str) => {
            if let Ok(id) = value_str.parse() {
                if dal::Secret::get_by_id(ctx, id).await.is_ok() {
                    Ok(id)
                } else {
                    let secrets = dal::Secret::list(ctx).await?;
                    let found_secret = secrets
                        .into_iter()
                        .find(|s| s.name() == value_str)
                        .ok_or_else(|| {
                            ComponentsError::SecretNotFound(format!(
                                "Secret '{}' not found",
                                value_str
                            ))
                        })?;
                    Ok(found_secret.id())
                }
            } else {
                let secrets = dal::Secret::list(ctx).await?;
                let found_secret = secrets
                    .into_iter()
                    .find(|s| s.name() == value_str)
                    .ok_or_else(|| {
                        ComponentsError::SecretNotFound(format!("Secret '{}' not found", value_str))
                    })?;
                Ok(found_secret.id())
            }
        }
        _ => Err(ComponentsError::InvalidSecretValue(format!(
            "Secret value must be a string containing ID or name, got: {}",
            value
        ))),
    }
}

#[derive(Deserialize, ToSchema)]
pub struct ComponentV1RequestPath {
    #[schema(value_type = String)]
    pub component_id: ComponentId,
}

impl IntoResponse for ComponentsError {
    fn into_response(self) -> axum::response::Response {
        use crate::service::v1::common::ErrorIntoResponse;
        self.to_api_response()
    }
}

impl From<JsonRejection> for ComponentsError {
    fn from(rejection: JsonRejection) -> Self {
        match rejection {
            JsonRejection::JsonDataError(_) => {
                ComponentsError::Validation(format!("Invalid JSON data format: {}", rejection))
            }
            JsonRejection::JsonSyntaxError(_) => {
                ComponentsError::Validation(format!("Invalid JSON syntax: {}", rejection))
            }
            JsonRejection::MissingJsonContentType(_) => ComponentsError::Validation(
                "Request must have Content-Type: application/json header".to_string(),
            ),
            _ => ComponentsError::Validation(format!("JSON validation error: {}", rejection)),
        }
    }
}

impl crate::service::v1::common::ErrorIntoResponse for ComponentsError {
    fn status_and_message(&self) -> (StatusCode, String) {
        match self {
            ComponentsError::Component(dal::ComponentError::NotFound(_)) => {
                (StatusCode::NOT_FOUND, self.to_string())
            }
            ComponentsError::ComponentNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            ComponentsError::SchemaNameNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            ComponentsError::ActionFunctionNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            ComponentsError::ManagementFunctionNotFound(_) => {
                (StatusCode::NOT_FOUND, self.to_string())
            }
            ComponentsError::ManagementFunctionAlreadyRunning => {
                (StatusCode::CONFLICT, self.to_string())
            }
            ComponentsError::SecretNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            ComponentsError::Secret(dal::SecretError::SecretNotFound(_)) => {
                (StatusCode::NOT_FOUND, self.to_string())
            }
            ComponentsError::ActionAlreadyEnqueued(_) => (StatusCode::CONFLICT, self.to_string()),
            ComponentsError::DuplicateComponentName(_) => {
                (StatusCode::PRECONDITION_FAILED, self.to_string())
            }
            ComponentsError::DuplicateManagementFunctionName(_) => {
                (StatusCode::PRECONDITION_FAILED, self.to_string())
            }
            ComponentsError::DuplicateActionFunctionName(_) => {
                (StatusCode::PRECONDITION_FAILED, self.to_string())
            }
            ComponentsError::ViewNotFound(_) => (StatusCode::PRECONDITION_FAILED, self.to_string()),
            ComponentsError::Validation(_) => (StatusCode::UNPROCESSABLE_ENTITY, self.to_string()),
            ComponentsError::InvalidSecretValue(_) => {
                (StatusCode::UNPROCESSABLE_ENTITY, self.to_string())
            }
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        }
    }
}

use get_component::{
    GetComponentV1ResponseActionFunction,
    GetComponentV1ResponseManagementFunction,
};

pub async fn get_component_functions(
    ctx: &dal::DalContext,
    component_id: ComponentId,
) -> Result<
    (
        Vec<GetComponentV1ResponseManagementFunction>,
        Vec<GetComponentV1ResponseActionFunction>,
    ),
    ComponentsError,
> {
    let schema_variant_id = Component::schema_variant_id(ctx, component_id).await?;

    let mut action_functions = Vec::new();
    for action_prototype in ActionPrototype::for_variant(ctx, schema_variant_id).await? {
        let func_id = ActionPrototype::func_id(ctx, action_prototype.id).await?;
        let func = Func::get_by_id(ctx, func_id).await?;

        action_functions.push(GetComponentV1ResponseActionFunction {
            prototype_id: action_prototype.id,
            func_name: func.display_name.unwrap_or_else(|| func.name.clone()),
        });
    }

    let mut management_functions = Vec::new();
    for prototype in ManagementPrototype::list_for_variant_id(ctx, schema_variant_id).await? {
        let func_id = ManagementPrototype::func_id(ctx, prototype.id).await?;
        let func = Func::get_by_id(ctx, func_id).await?;

        management_functions.push(GetComponentV1ResponseManagementFunction {
            management_prototype_id: prototype.id,
            func_name: func.display_name.unwrap_or_else(|| func.name.clone()),
        });
    }

    Ok((management_functions, action_functions))
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash, ToSchema)]
#[serde(untagged)]
pub enum ComponentPropKey {
    #[schema(value_type = String)]
    PropId(PropId),
    PropPath(DomainPropPath),
}

impl ComponentPropKey {
    pub async fn prop_id(
        &self,
        ctx: &dal::DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> PropResult<PropId> {
        match self {
            ComponentPropKey::PropId(prop_id) => Ok(*prop_id),
            ComponentPropKey::PropPath(path) => {
                dal::Prop::find_prop_id_by_path(ctx, schema_variant_id, &path.to_prop_path()).await
            }
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash, ToSchema)]
#[serde(untagged)]
pub enum SecretPropKey {
    #[schema(value_type = String)]
    PropId(PropId),
    PropPath(SecretPropPath),
}

impl SecretPropKey {
    pub async fn prop_id(
        &self,
        ctx: &dal::DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> PropResult<PropId> {
        match self {
            SecretPropKey::PropId(prop_id) => Ok(*prop_id),
            SecretPropKey::PropPath(path) => {
                dal::Prop::find_prop_id_by_path(ctx, schema_variant_id, &path.to_prop_path()).await
            }
        }
    }
}

/// A prop path, starting from root/domain, with / instead of PROP_PATH_SEPARATOR as its separator
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash, ToSchema)]
pub struct DomainPropPath(pub String);

impl DomainPropPath {
    pub fn to_prop_path(&self) -> PropPath {
        PropPath::new(["root", "domain"]).join(&self.0.replace("/", PROP_PATH_SEPARATOR).into())
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash, ToSchema)]
pub struct SecretPropPath(pub String);

impl SecretPropPath {
    pub fn to_prop_path(&self) -> PropPath {
        PropPath::new(["root", "secrets"]).join(&self.0.replace("/", PROP_PATH_SEPARATOR).into())
    }
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", post(create_component::create_component))
        .route("/", get(list_components::list_components))
        .route("/find", get(find_component::find_component))
        .route("/search", post(search_components::search_components))
        .nest(
            "/:component_id",
            Router::new()
                .route("/", get(get_component::get_component))
                .route("/", put(update_component::update_component))
                .route("/", delete(delete_component::delete_component))
                .route(
                    "/execute-management-function",
                    post(execute_management_function::execute_management_function),
                )
                .route("/action", post(add_action::add_action)),
        )
}
