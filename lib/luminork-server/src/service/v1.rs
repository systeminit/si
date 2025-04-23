use crate::AppState;
use axum::Router;
use utoipa::OpenApi;

mod change_sets;
pub mod common;
mod components;
mod management;
mod workspaces;

pub use change_sets::ChangeSetError;
pub use components::ComponentsError;
pub use management::ManagementApiError;
pub use workspaces::WorkspaceError;

pub use change_sets::{
    create::{CreateChangeSetV1Request, CreateChangeSetV1Response},
    delete::DeleteChangeSetV1Response,
    get::GetChangeSetV1Response,
    list::ListChangeSetV1Response,
    merge_status::{
        MergeStatusV1Response, MergeStatusV1ResponseAction, MergeStatusV1ResponseActionComponent,
    },
};
pub use components::{
    ComponentV1RequestPath,
    connections::{ComponentReference, Connection, ConnectionPoint},
    create_component::{CreateComponentV1Request, CreateComponentV1Response},
    delete_component::DeleteComponentV1Response,
    get_component::{
        GeometryAndViewAndName, GetComponentV1Response, GetComponentV1ResponseManagementFunction,
    },
    update_component::{
        ComponentPropKey, DomainPropPath, UpdateComponentV1Request, UpdateComponentV1Response,
    },
};
pub use management::run_prototype::{
    RunPrototypePath, RunPrototypeV1Request, RunPrototypeV1Response,
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
        components::update_component::update_component,
        components::delete_component::delete_component,
        management::run_prototype::run_prototype
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
            GeometryAndViewAndName,
            UpdateComponentV1Request,
            UpdateComponentV1Response,
            DeleteComponentV1Response,
            RunPrototypeV1Request,
            RunPrototypeV1Response,
            RunPrototypePath,
            ComponentPropKey,
            DomainPropPath,
            ConnectionPoint,
            ComponentReference
        )
    ),
    tags(
        (name = "workspaces", description = "Workspace management endpoints"),
        (name = "change_sets", description = "Change set management endpoints"),
        (name = "components", description = "Component management endpoints"),
        (name = "management", description = "Management function endpoints")
    )
)]
pub struct V1ApiDoc;

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new().nest("/w", workspaces::routes(state))
}

pub fn get_openapi() -> utoipa::openapi::OpenApi {
    V1ApiDoc::openapi()
}
