use axum::{
    extract::Path,
    response::Json,
};
use dal::{
    Component,
    ComponentId,
    Func,
    diagram::view::View,
    management::prototype::ManagementPrototype,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_id::{
    FuncRunId,
    ManagementPrototypeId,
};
use utoipa::{
    self,
    ToSchema,
};

use crate::{
    extract::{
        PosthogEventTracker,
        change_set::ChangeSetDalContext,
    },
    service::v1::{
        ComponentV1RequestPath,
        ComponentsError,
    },
};

#[utoipa::path(
    post,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/components/{component_id}/execute-management-function",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
        ("component_id" = String, Path, description = "Component identifier")
    ),
    tag = "components",
    request_body = ExecuteManagementFunctionV1Request,
    summary = "Execute a component's management function",
    responses(
        (status = 200, description = "Function successfully dispatched", body = ExecuteManagementFunctionV1Response),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 404, description = "Component or function not found"),
        (status = 412, description = "Precondition Failed - View not found or duplicate function name", body = crate::service::v1::common::ApiError),
        (status = 422, description = "Validation error - Invalid request data", body = crate::service::v1::common::ApiError),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn execute_management_function(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    _tracker: PosthogEventTracker,
    Path(ComponentV1RequestPath { component_id }): Path<ComponentV1RequestPath>,
    payload: Result<
        Json<ExecuteManagementFunctionV1Request>,
        axum::extract::rejection::JsonRejection,
    >,
) -> Result<Json<ExecuteManagementFunctionV1Response>, ComponentsError> {
    let Json(payload) = payload?;

    let prototype_id =
        resolve_management_function_reference(ctx, component_id, &payload.management_function)
            .await?;

    let view_id = if let Some(view_name) = payload.view_name {
        if let Some(view) = View::find_by_name(ctx, view_name.as_str()).await? {
            view.id()
        } else {
            return Err(ComponentsError::ViewNotFound(format!(
                "View '{}' not found",
                view_name
            )));
        }
    } else {
        View::get_id_for_default(ctx).await?
    };

    let func_run_id =
        ManagementPrototype::dispatch_by_id(ctx, prototype_id, component_id, view_id.into())
            .await?;

    ctx.commit().await?;

    Ok(Json(ExecuteManagementFunctionV1Response { func_run_id }))
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
/// Reference to a management function by either name or ID.
/// This allows clients to use the more human-friendly name approach
/// or the more precise ID approach when working with management functions.
// #[schema(example = json!({"function": "CreateVpc"}))]
// #[schema(example = json!({"managementPrototypeId": "01H9ZQD35JPMBGHH69BT0Q79VY"}))]
pub enum ManagementFunctionReference {
    ByName {
        #[schema(example = "CreateVpc")]
        function: String,
    },
    #[serde(rename_all = "camelCase")]
    ById {
        #[schema(value_type = String, example = "01H9ZQD35JPMBGHH69BT0Q79VY")]
        management_prototype_id: ManagementPrototypeId,
    },
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(example = json!({
    "viewName": "MyViewName",
    "managementFunction": {"managementPrototypeId": "01H9ZQD35JPMBGHH69BT0Q79VY"}
}))]
#[schema(example = json!({
    "viewName": "MyViewName",
    "managementFunction": {"function": "CreateVpc"}
}))]
pub struct ExecuteManagementFunctionV1Request {
    #[schema(example = "MyViewName")]
    pub view_name: Option<String>,

    #[schema(example = json!({"function": "CreateVpc"}), required = true)]
    pub management_function: ManagementFunctionReference,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteManagementFunctionV1Response {
    #[schema(value_type = String, example = "01H9ZQD35JPMBGHH69BT0Q79VY")]
    pub func_run_id: FuncRunId,
}

async fn resolve_management_function_reference(
    ctx: &dal::DalContext,
    component_id: ComponentId,
    function_ref: &ManagementFunctionReference,
) -> Result<ManagementPrototypeId, ComponentsError> {
    match function_ref {
        ManagementFunctionReference::ById {
            management_prototype_id,
        } => Ok(*management_prototype_id),
        ManagementFunctionReference::ByName { function } => {
            let schema_variant_id = Component::schema_variant_id(ctx, component_id).await?;
            let prototypes =
                ManagementPrototype::list_for_variant_id(ctx, schema_variant_id).await?;

            let mut matching_prototypes: Vec<_> = vec![];
            for prototype in prototypes {
                let func_id = ManagementPrototype::func_id(ctx, prototype.id).await?;
                let func = Func::get_by_id(ctx, func_id).await?;
                if func.name == *function
                    || func
                        .display_name
                        .as_ref()
                        .map_or_else(|| false, |display_name| display_name == function)
                {
                    matching_prototypes.push(prototype);
                }
            }

            match matching_prototypes.len() {
                0 => Err(ComponentsError::ManagementFunctionNotFound(
                    function.clone(),
                )),
                1 => Ok(matching_prototypes[0].id),
                _ => Err(ComponentsError::DuplicateManagementFunctionName(
                    function.clone(),
                )),
            }
        }
    }
}
