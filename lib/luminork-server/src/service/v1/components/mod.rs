use std::collections::VecDeque;

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
    AttributeValue,
    Component,
    ComponentId,
    DalContext,
    Func,
    Prop,
    SchemaId,
    SchemaVariant,
    SchemaVariantId,
    action::{
        ActionError,
        prototype::{
            ActionPrototype,
            ActionPrototypeError,
        },
    },
    attribute::attributes::AttributeSources,
    diagram::{
        geometry::Geometry,
        view::View,
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
use serde_json::Value;
use si_id::{
    AttributeValueId,
    PropId,
    ViewId,
};
use strum::{
    AsRefStr,
    Display,
};
use thiserror::Error;
use utoipa::ToSchema;

use crate::AppState;

pub mod add_action;
pub mod create_component;
pub mod delete_component;
pub mod duplicate_components;
pub mod erase_component;
pub mod execute_management_function;
pub mod find_component;
pub mod generate_template;
pub mod get_component;
pub mod list_components;
pub mod manage_component;
pub mod restore_component;
pub mod search_components;
pub mod update_component;
pub mod upgrade_component;

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
    #[error("attribute error: {0}")]
    Attribute(#[from] dal::attribute::attributes::AttributesError),
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] dal::attribute::value::AttributeValueError),
    #[error("attribute value {0} not from component {1}")]
    AttributeValueNotFromComponent(AttributeValueId, ComponentId),
    #[error("cached module error: {0}")]
    CachedModule(#[from] dal::cached_module::CachedModuleError),
    #[error("component error: {0}")]
    Component(#[from] dal::ComponentError),
    #[error("component not found: {0}")]
    ComponentNotFound(String),
    #[error("component not marked for deletion: {0}")]
    ComponentNotRestorable(ComponentId),
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
    #[error("generate template error: {0}")]
    GenerateTemplate(#[from] sdf_core::generate_template::GenerateTemplateError),
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
    #[error("changes not permitted on HEAD change set")]
    NotPermittedOnHead,
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
    #[error("schema variant upgrade not required")]
    SchemaVariantUpgradeSkipped,
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
    #[error("component upgrade skipped due to running or dispatched actions")]
    UpgradeSkippedDueToActions,
    #[error("validation error: {0}")]
    Validation(String),
    #[error("view not found: {0}")]
    ViewNotFound(String),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] dal::WorkspaceSnapshotError),
    #[error("ws event error: {0}")]
    WsEvent(#[from] dal::WsEventError),
}

pub type ComponentsResult<T> = Result<T, ComponentsError>;

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
                ComponentsError::Validation(format!("Invalid JSON data format: {rejection}"))
            }
            JsonRejection::JsonSyntaxError(_) => {
                ComponentsError::Validation(format!("Invalid JSON syntax: {rejection}"))
            }
            JsonRejection::MissingJsonContentType(_) => ComponentsError::Validation(
                "Request must have Content-Type: application/json header".to_string(),
            ),
            _ => ComponentsError::Validation(format!("JSON validation error: {rejection}")),
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
            ComponentsError::ComponentNotRestorable(_) => {
                (StatusCode::PRECONDITION_FAILED, self.to_string())
            }
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
            ComponentsError::NotPermittedOnHead => (StatusCode::BAD_REQUEST, self.to_string()),
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
#[schema(example = json!({"component": "ComponentName"}))]
pub enum ComponentReference {
    ByName {
        component: String,
    },
    #[serde(rename_all = "camelCase")]
    ById {
        #[schema(value_type = String, example = "01H9ZQD35JPMBGHH69BT0Q79VY")]
        component_id: ComponentId,
    },
}

impl Default for ComponentReference {
    fn default() -> Self {
        ComponentReference::ByName {
            component: String::new(),
        }
    }
}

impl ComponentReference {
    pub fn is_empty(&self) -> bool {
        match self {
            ComponentReference::ByName { component } => component.is_empty(),
            ComponentReference::ById { component_id: _ } => false, // IDs are never considered "empty"
        }
    }
}

/// Helper function to resolve a component reference to a component ID
pub async fn resolve_component_reference(
    ctx: &dal::DalContext,
    component_ref: &ComponentReference,
    component_list: &[ComponentId],
) -> Result<ComponentId, ComponentsError> {
    match component_ref {
        ComponentReference::ById { component_id } => Ok(*component_id),
        ComponentReference::ByName { component } => {
            find_component_id_by_name(ctx, component_list, component).await
        }
    }
}

/// Returns the component ID if found, or appropriate error if not found or if duplicate names exist
async fn find_component_id_by_name(
    ctx: &dal::DalContext,
    component_list: &[ComponentId],
    component_name: &str,
) -> Result<ComponentId, ComponentsError> {
    let mut matching_components = Vec::new();

    for component_id in component_list {
        let name = Component::name_by_id(ctx, *component_id).await?;
        if name == component_name {
            matching_components.push(*component_id);
        }
    }

    match matching_components.len() {
        0 => Err(ComponentsError::ComponentNotFound(
            component_name.to_string(),
        )),
        1 => Ok(matching_components[0]),
        _ => Err(ComponentsError::DuplicateComponentName(
            component_name.to_string(),
        )),
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ComponentViewV1 {
    #[schema(value_type = String)]
    pub id: ComponentId,
    #[schema(value_type = String)]
    pub schema_id: SchemaId,
    #[schema(value_type = String)]
    pub schema_variant_id: SchemaVariantId,
    // this is everything below root/domain - the whole tree! (not including root/domain itself)
    pub domain_props: Vec<ComponentPropViewV1>,
    // from root/resource_value NOT root/resource/payload
    pub resource_props: Vec<ComponentPropViewV1>,
    // maps to root/si/name
    pub name: String,
    // maps to root/si/resource_id
    pub resource_id: String,
    pub to_delete: bool,
    pub can_be_upgraded: bool,
    // current connections to/from this component (should these be separated?)
    pub connections: Vec<ConnectionViewV1>,
    // what views this component is in
    pub views: Vec<ViewV1>,

    #[schema(
        value_type = std::collections::BTreeMap<String, serde_json::Value>,
        example = json!({
            "/domain/region": "us-east-1",
            "/secrets/credential": {
                "$source": {
                    "component": "demo-credential",
                    "path": "/secrets/AWS Credential"
                }
            }
        })
    )]
    pub attributes: AttributeSources,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SourceViewV1 {
    pub component: String,
    #[serde(rename = "propPath")]
    pub prop_path: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ComponentPropViewV1 {
    #[schema(value_type = String)]
    pub id: AttributeValueId, // I know prop view with an id for an AV...
    #[schema(value_type = String)]
    pub prop_id: PropId,
    #[schema(value_type = Object)]
    pub value: Option<Value>,
    #[schema(value_type = String, example = "path/to/prop")]
    pub path: String,
    // todo: Validation
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ViewV1 {
    #[schema(value_type = String)]
    pub id: ViewId,
    pub name: String,
    pub is_default: bool,
}

#[derive(AsRefStr, Clone, Debug, Deserialize, Display, Eq, PartialEq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum ConnectionViewV1 {
    Managing(ManagingConnectionViewV1),
    ManagedBy(ManagedByConnectionViewV1),
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ManagingConnectionViewV1 {
    #[schema(value_type = String)]
    pub component_id: ComponentId,
    pub component_name: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ManagedByConnectionViewV1 {
    #[schema(value_type = String)]
    pub component_id: ComponentId,
    pub component_name: String,
}

impl ComponentViewV1 {
    pub async fn assemble(ctx: &DalContext, component_id: ComponentId) -> ComponentsResult<Self> {
        let component = Component::get_by_id(ctx, component_id).await?;
        let schema_variant = component.schema_variant(ctx).await?;

        let mut connections = Vec::new();
        // Management Connections
        // Who is managing this component?
        let managers = Component::managers_by_id(ctx, component_id).await?;
        for manager in managers {
            connections.push(ConnectionViewV1::ManagedBy(ManagedByConnectionViewV1 {
                component_id: manager,
                component_name: Component::name_by_id(ctx, manager).await?,
            }));
        }
        // Who is this component managing?
        let managing = component.get_managed(ctx).await?;
        for managed in managing {
            connections.push(ConnectionViewV1::Managing(ManagingConnectionViewV1 {
                component_id: managed,
                component_name: Component::name_by_id(ctx, managed).await?,
            }));
        }

        // Domain Props
        let mut domain_props = Vec::new();
        let domain_root_av = component.domain_prop_attribute_value(ctx).await?;
        let mut work_queue = VecDeque::new();
        let domain_values = AttributeValue::get_child_av_ids_in_order(ctx, domain_root_av).await?;
        work_queue.extend(domain_values);
        while let Some(av) = work_queue.pop_front() {
            let value = AttributeValue::view(ctx, av).await?;
            let prop_id = AttributeValue::prop_id(ctx, av).await?;
            let is_hidden_prop = Prop::get_by_id(ctx, prop_id).await?.hidden;
            if !is_hidden_prop {
                let view = ComponentPropViewV1 {
                    id: av,
                    prop_id,
                    value,
                    path: AttributeValue::get_path_for_id(ctx, av)
                        .await?
                        .unwrap_or_else(String::new),
                };
                domain_props.push(view);
                let children = AttributeValue::get_child_av_ids_in_order(ctx, av).await?;

                work_queue.extend(children);
            }
        }
        // sort alphabetically by path
        domain_props.sort_by_key(|view| view.path.to_lowercase());

        // Resource Props
        let mut resource_props = Vec::new();
        let resource_value_root_av = component.resource_value_prop_attribute_value(ctx).await?;
        let mut work_queue = VecDeque::new();
        let resource_value_values =
            AttributeValue::get_child_av_ids_in_order(ctx, resource_value_root_av).await?;
        work_queue.extend(resource_value_values);
        while let Some(av) = work_queue.pop_front() {
            let value = AttributeValue::view(ctx, av).await?;

            let view = ComponentPropViewV1 {
                id: av,
                prop_id: AttributeValue::prop_id(ctx, av).await?,
                value,
                path: AttributeValue::get_path_for_id(ctx, av)
                    .await?
                    .unwrap_or_else(String::new),
            };
            resource_props.push(view);
            let children = AttributeValue::get_child_av_ids_in_order(ctx, av).await?;

            work_queue.extend(children);
        }

        // sort alphabetically by path
        resource_props.sort_by_key(|view| view.path.to_lowercase());

        // get views
        let mut views = Vec::new();
        let geos = Geometry::by_view_for_component_id(ctx, component_id).await?;
        for view_id in geos.keys() {
            let view = View::get_by_id(ctx, *view_id).await?;
            views.push(ViewV1 {
                id: *view_id,
                name: view.name().to_string(),
                is_default: view.is_default(ctx).await?,
            });
        }

        let attributes: AttributeSources = Component::sources(ctx, component_id).await?.into();

        let result = ComponentViewV1 {
            id: component_id,
            schema_id: SchemaVariant::schema_id(ctx, schema_variant.id()).await?,
            schema_variant_id: schema_variant.id(),
            domain_props,
            resource_props,
            name: component.name(ctx).await?,
            resource_id: component.resource_id(ctx).await?,
            to_delete: component.to_delete(),
            can_be_upgraded: component.can_be_upgraded(ctx).await?,
            connections,
            views,
            attributes,
        };
        Ok(result)
    }
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", post(create_component::create_component))
        .route("/", get(list_components::list_components))
        .route("/find", get(find_component::find_component))
        .route("/search", post(search_components::search_components))
        .route(
            "/generate_template",
            post(generate_template::generate_template),
        )
        .route(
            "/duplicate",
            post(duplicate_components::duplicate_components),
        )
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
                .route("/action", post(add_action::add_action))
                .route("/manage", post(manage_component::manage_component))
                .route("/upgrade", post(upgrade_component::upgrade_component))
                .route("/erase", post(erase_component::erase_component))
                .route("/restore", post(restore_component::restore_component)),
        )
}
