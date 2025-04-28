use axum::Router;
use utoipa::OpenApi;

use crate::AppState;

mod change_sets;
pub mod common;
mod components;
mod funcs;
mod schema;
mod workspaces;

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
};
pub use components::{
    ComponentV1RequestPath,
    ComponentsError,
    ComponentsResult,
    add_action::{
        ActionReference,
        AddActionV1Request,
        AddActionV1Response,
    },
    connections::{
        ComponentReference,
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
    update_component::{
        ComponentPropKey,
        DomainPropPath,
        UpdateComponentV1Request,
        UpdateComponentV1Response,
    },
};
pub use funcs::{
    FuncRunV1RequestPath,
    FuncsResult,
};
pub use schema::{
    SchemaError,
    list_schema::ListSchemaV1Response,
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
        title = "Luminork API - V1",
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
        components::get_component::get_component,
        components::create_component::create_component,
        components::list_components::list_components,
        components::find_component::find_component,
        components::update_component::update_component,
        components::delete_component::delete_component,
        components::execute_management_function::execute_management_function,
        components::add_action::add_action,
        schema::list_schema::list_schemas,
        funcs::get_func_run::get_func_run,
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
            UpdateComponentV1Request,
            UpdateComponentV1Response,
            DeleteComponentV1Response,
            ExecuteManagementFunctionV1Request,
            ExecuteManagementFunctionV1Response,
            ComponentPropKey,
            DomainPropPath,
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
        )
    ),
    tags(
        (name = "workspaces", description = "Workspace management endpoints"),
        (name = "change_sets", description = "Change set management endpoints"),
        (name = "components", description = "Component management endpoints"),
        (name = "schemas", description = "Schema endpoints")
    )
)]
pub struct V1ApiDoc;

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new().nest("/w", workspaces::routes(state))
}

pub fn get_openapi() -> utoipa::openapi::OpenApi {
    V1ApiDoc::openapi()
}
