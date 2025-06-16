use axum::Router;
use utoipa::OpenApi;

use crate::AppState;

mod actions;
mod change_sets;
pub mod common;
mod components;
mod funcs;
mod management_funcs;
mod schemas;
mod secrets;
mod workspaces;

pub use actions::{
    ActionV1RequestPath,
    cancel_action::CancelActionV1Response,
    get_actions::GetActionsV1Response,
    put_on_hold::PutOnHoldActionV1Response,
    retry_action::RetryActionV1Response,
};
pub use change_sets::{
    ChangeSetError,
    create::{
        CreateChangeSetV1Request,
        CreateChangeSetV1Response,
    },
    delete::DeleteChangeSetV1Response,
    get::GetChangeSetV1Response,
    list::ListChangeSetV1Response,
    merge_status::{
        MergeStatusV1Response,
        MergeStatusV1ResponseAction,
        MergeStatusV1ResponseActionComponent,
    },
    purge_open::PurgeOpenChangeSetsV1Response,
};
pub use components::{
    ComponentPropKey,
    ComponentReference,
    ComponentV1RequestPath,
    ComponentsError,
    ComponentsResult,
    DomainPropPath,
    SecretPropKey,
    SecretPropPath,
    add_action::{
        ActionReference,
        AddActionV1Request,
        AddActionV1Response,
    },
    connections::{
        Connection,
        ConnectionPoint,
    },
    create_component::{
        CreateComponentV1Request,
        CreateComponentV1Response,
    },
    delete_component::DeleteComponentV1Response,
    execute_management_function::{
        ExecuteManagementFunctionV1Request,
        ExecuteManagementFunctionV1Response,
        ManagementFunctionReference,
    },
    find_component::FindComponentV1Params,
    get_component::{
        GetComponentV1Response,
        GetComponentV1ResponseManagementFunction,
    },
    list_components::ListComponentsV1Response,
    search_components::{
        SearchComponentsV1Request,
        SearchComponentsV1Response,
    },
    update_component::{
        UpdateComponentV1Request,
        UpdateComponentV1Response,
    },
};
pub use funcs::{
    FuncRunV1RequestPath,
    FuncV1RequestPath,
    FuncsResult,
    get_func::GetFuncV1Response,
    get_func_run::GetFuncRunV1Response,
};
pub use management_funcs::{
    ManagementFuncJobStateV1RequestPath,
    ManagementFuncsError,
    ManagementFuncsResult,
    get_management_func_run_state::GetManagementFuncJobStateV1Response,
};
pub use schemas::{
    GetSchemaV1Response,
    GetSchemaVariantV1Response,
    PropSchemaV1,
    SchemaError,
    SchemaV1RequestPath,
    SchemaVariantV1RequestPath,
    find_schema::{
        FindSchemaV1Params,
        FindSchemaV1Response,
    },
    list_schemas::ListSchemaV1Response,
};
pub use workspaces::WorkspaceError;

pub use crate::api_types::{
    component::v1::{
        ComponentPropViewV1,
        ComponentViewV1,
        ConnectionViewV1,
    },
    func_run::v1::{
        FuncRunLogViewV1,
        FuncRunViewV1,
        OutputLineViewV1,
    },
};

/// OpenAPI documentation for v1 API
#[derive(OpenApi)]
#[openapi(
    info(
        title = "System Initiative API - V1",
        description = "System Initiative External API server - V1 Routes",
        version = "1.0.0"
    ),
    servers(
        (url = "/v1", description = "V1 API")
    ),
    paths(
        change_sets::create::create_change_set,
        change_sets::list::list_change_sets,
        change_sets::get::get_change_set,
        change_sets::delete::abandon_change_set,
        change_sets::force_apply::force_apply,
        change_sets::merge_status::merge_status,
        change_sets::request_approval::request_approval,
        change_sets::purge_open::purge_open,
        components::get_component::get_component,
        components::create_component::create_component,
        components::list_components::list_components,
        components::search_components::search_components,
        components::find_component::find_component,
        components::update_component::update_component,
        components::delete_component::delete_component,
        components::execute_management_function::execute_management_function,
        components::add_action::add_action,
        schemas::list_schemas::list_schemas,
        schemas::find_schema::find_schema,
        schemas::get_schema::get_schema,
        schemas::get_variant::get_variant,
        schemas::get_default_variant::get_default_variant,
        funcs::get_func_run::get_func_run,
        funcs::get_func::get_func,
        management_funcs::get_management_func_run_state::get_management_func_run_state,
        actions::cancel_action::cancel_action,
        actions::retry_action::retry_action,
        actions::get_actions::get_actions,
        actions::put_on_hold::put_on_hold,
        secrets::create_secret::create_secret,
        secrets::delete_secret::delete_secret,
        secrets::update_secret::update_secret,
        secrets::get_secrets::get_secrets,
    ),
    components(
        schemas(
            common::ApiError,
            common::ApiSuccess<String>,
            CreateChangeSetV1Request,
            CreateChangeSetV1Response,
            DeleteChangeSetV1Response,
            ListChangeSetV1Response,
            GetChangeSetV1Response,
            PurgeOpenChangeSetsV1Response,
            MergeStatusV1Response,
            MergeStatusV1ResponseAction,
            MergeStatusV1ResponseActionComponent,
            ComponentV1RequestPath,
            GetComponentV1Response,
            GetComponentV1ResponseManagementFunction,
            CreateComponentV1Request,
            CreateComponentV1Response,
            Connection,
            AddActionV1Response,
            AddActionV1Request,
            ActionReference,
            ListSchemaV1Response,
            SchemaV1RequestPath,
            SchemaVariantV1RequestPath,
            GetSchemaV1Response,
            GetSchemaVariantV1Response,
            UpdateComponentV1Request,
            UpdateComponentV1Response,
            DeleteComponentV1Response,
            ExecuteManagementFunctionV1Request,
            ExecuteManagementFunctionV1Response,
            ComponentPropKey,
            SecretPropKey,
            DomainPropPath,
            SecretPropPath,
            ConnectionPoint,
            ComponentReference,
            ComponentViewV1,
            ComponentPropViewV1,
            ConnectionViewV1,
            ListComponentsV1Response,
            FindComponentV1Params,
            FuncRunV1RequestPath,
            FuncRunLogViewV1,
            FuncRunViewV1,
            OutputLineViewV1,
            GetFuncV1Response,
            GetFuncRunV1Response,
            PropSchemaV1,
            CancelActionV1Response,
            RetryActionV1Response,
            GetActionsV1Response,
            PutOnHoldActionV1Response,
            ActionV1RequestPath,
            FindSchemaV1Params,
            FindSchemaV1Response,
            GetManagementFuncJobStateV1Response,
            ManagementFuncJobStateV1RequestPath,
        )
    ),
    tags(
        (name = "workspaces", description = "Workspace management endpoints"),
        (name = "change_sets", description = "Change Set management endpoints"),
        (name = "components", description = "Component management endpoints"),
        (name = "schemas", description = "Schema management endpoints"),
        (name = "actions", description = "Action management endpoints"),
        (name = "secrets", description = "Secret management endpoints"),
        (name = "funcs", description = "Functions management endpoints"),
        (name = "management_funcs", description = "Management functions endpoints")
    )
)]
pub struct V1ApiDoc;

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new().nest("/w", workspaces::routes(state))
}

pub fn get_openapi() -> utoipa::openapi::OpenApi {
    V1ApiDoc::openapi()
}
