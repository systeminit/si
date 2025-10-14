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
mod search;
mod secrets;
mod user;
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
    ComponentPropViewV1,
    ComponentReference,
    ComponentV1RequestPath,
    ComponentViewV1,
    ComponentsError,
    ComponentsResult,
    ConnectionViewV1,
    DomainPropPath,
    SourceViewV1,
    add_action::{
        ActionReference,
        AddActionV1Request,
        AddActionV1Response,
    },
    create_component::{
        CreateComponentV1Request,
        CreateComponentV1Response,
    },
    delete_component::DeleteComponentV1Response,
    duplicate_components::{
        DuplicateComponentsV1Request,
        DuplicateComponentsV1Response,
    },
    erase_component::EraseComponentV1Response,
    execute_management_function::{
        ExecuteManagementFunctionV1Request,
        ExecuteManagementFunctionV1Response,
        ManagementFunctionReference,
    },
    find_component::FindComponentV1Params,
    generate_template::{
        GenerateTemplateV1Request,
        GenerateTemplateV1Response,
    },
    get_component::{
        GetComponentV1Response,
        GetComponentV1ResponseManagementFunction,
    },
    list_components::{
        ComponentDetailsV1,
        ListComponentsV1Response,
    },
    manage_component::{
        ManageComponentV1Request,
        ManageComponentV1Response,
    },
    restore_component::RestoreComponentV1Response,
    search_components::{
        SearchComponentsV1Request,
        SearchComponentsV1Response,
    },
    update_component::{
        SecretPropKey,
        SecretPropPath,
        UpdateComponentV1Request,
        UpdateComponentV1Response,
    },
    upgrade_component::UpgradeComponentV1Response,
};
pub use funcs::{
    FuncRunV1RequestPath,
    FuncV1RequestPath,
    FuncsResult,
    get_func::GetFuncV1Response,
    get_func_run::GetFuncRunV1Response,
    unlock_func::{
        UnlockFuncV1Request,
        UnlockFuncV1Response,
    },
    update_func::{
        UpdateFuncV1Request,
        UpdateFuncV1Response,
    },
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
    create_action::{
        CreateVariantActionFuncV1Request,
        CreateVariantActionFuncV1Response,
    },
    create_authentication::{
        CreateVariantAuthenticationFuncV1Request,
        CreateVariantAuthenticationFuncV1Response,
    },
    create_codegen::{
        CreateVariantCodegenFuncV1Request,
        CreateVariantCodegenFuncV1Response,
    },
    create_management::{
        CreateVariantManagementFuncV1Request,
        CreateVariantManagementFuncV1Response,
    },
    create_qualification::{
        CreateVariantQualificationFuncV1Request,
        CreateVariantQualificationFuncV1Response,
    },
    create_schema::CreateSchemaV1Request,
    find_schema::{
        FindSchemaV1Params,
        FindSchemaV1Response,
    },
    list_schemas::ListSchemaV1Response,
    search_schemas::{
        SearchSchemasV1Request,
        SearchSchemasV1Response,
    },
    unlock_schema::UnlockedSchemaV1Response,
    update_schema_variant::UpdateSchemaVariantV1Request,
    upgrade_schema::UpgradeSchemaResponse,
};
pub use search::{
    SearchV1Request,
    SearchV1Response,
};
pub use workspaces::WorkspaceError;

pub use crate::api_types::func_run::v1::{
    FuncRunLogViewV1,
    FuncRunViewV1,
    OutputLineViewV1,
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
        components::manage_component::manage_component,
        components::duplicate_components::duplicate_components,
        components::upgrade_component::upgrade_component,
        components::generate_template::generate_template,
        components::erase_component::erase_component,
        components::restore_component::restore_component,
        schemas::list_schemas::list_schemas,
        schemas::find_schema::find_schema,
        schemas::get_schema::get_schema,
        schemas::get_variant::get_variant,
        schemas::get_default_variant::get_default_variant,
        schemas::create_schema::create_schema,
        schemas::unlock_schema::unlock_schema,
        schemas::create_action::create_variant_action,
        schemas::search_schemas::search_schemas,
        schemas::create_authentication::create_variant_authentication,
        schemas::create_qualification::create_variant_qualification,
        schemas::create_codegen::create_variant_codegen,
        schemas::create_management::create_variant_management,
        schemas::update_schema_variant::update_schema_variant,
        schemas::upgrade_schema::upgrade_schema,
        funcs::get_func_run::get_func_run,
        funcs::get_func::get_func,
        funcs::update_func::update_func,
        funcs::unlock_func::unlock_func,
        management_funcs::get_management_func_run_state::get_management_func_run_state,
        actions::cancel_action::cancel_action,
        actions::retry_action::retry_action,
        actions::get_actions::get_actions,
        actions::put_on_hold::put_on_hold,
        secrets::create_secret::create_secret,
        secrets::delete_secret::delete_secret,
        secrets::update_secret::update_secret,
        secrets::get_secrets::get_secrets,
        search::search,
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
            ManageComponentV1Request,
            ManageComponentV1Response,
            UpgradeComponentV1Response,
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
            EraseComponentV1Response,
            RestoreComponentV1Response,
            DuplicateComponentsV1Request,
            DuplicateComponentsV1Response,
            GenerateTemplateV1Request,
            GenerateTemplateV1Response,
            ExecuteManagementFunctionV1Request,
            ExecuteManagementFunctionV1Response,
            ComponentPropKey,
            SecretPropKey,
            DomainPropPath,
            SecretPropPath,
            ComponentReference,
            ComponentViewV1,
            ComponentPropViewV1,
            ConnectionViewV1,
            SourceViewV1,
            ComponentDetailsV1,
            ListComponentsV1Response,
            FindComponentV1Params,
            FuncRunV1RequestPath,
            FuncRunLogViewV1,
            FuncRunViewV1,
            OutputLineViewV1,
            GetFuncV1Response,
            GetFuncRunV1Response,
            UpdateFuncV1Request,
            UpdateFuncV1Response,
            UnlockFuncV1Request,
            UnlockFuncV1Response,
            PropSchemaV1,
            CancelActionV1Response,
            RetryActionV1Response,
            GetActionsV1Response,
            PutOnHoldActionV1Response,
            ActionV1RequestPath,
            FindSchemaV1Params,
            FindSchemaV1Response,
            SearchSchemasV1Request,
            SearchSchemasV1Response,
            GetManagementFuncJobStateV1Response,
            ManagementFuncJobStateV1RequestPath,
            CreateVariantActionFuncV1Request,
            CreateVariantActionFuncV1Response,
            CreateVariantAuthenticationFuncV1Request,
            CreateVariantAuthenticationFuncV1Response,
            CreateVariantQualificationFuncV1Request,
            CreateVariantQualificationFuncV1Response,
            CreateSchemaV1Request,
            UnlockedSchemaV1Response,
            CreateVariantCodegenFuncV1Request,
            CreateVariantCodegenFuncV1Response,
            CreateVariantManagementFuncV1Request,
            CreateVariantManagementFuncV1Response,
            UpdateSchemaVariantV1Request,
            SearchV1Request,
            SearchV1Response,
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
