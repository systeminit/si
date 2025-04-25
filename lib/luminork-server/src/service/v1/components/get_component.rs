use std::collections::HashMap;

use axum::{
    extract::Path,
    response::Json,
};
use dal::{
    ActionPrototypeId,
    Component,
};
use serde::Serialize;
use si_id::ManagementPrototypeId;
use utoipa::{
    self,
    ToSchema,
};

use super::{
    ComponentV1RequestPath,
    ComponentsError,
};
use crate::{
    api_types::component::v1::ComponentViewV1,
    extract::change_set::ChangeSetDalContext,
};

#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GetComponentV1Response {
    pub component: ComponentViewV1,
    pub management_functions: Vec<GetComponentV1ResponseManagementFunction>,
    pub action_functions: Vec<GetComponentV1ResponseActionFunction>,
}

#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GetComponentV1ResponseManagementFunction {
    #[schema(value_type = String)]
    pub management_prototype_id: ManagementPrototypeId,
    pub func_name: String,
}

#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GetComponentV1ResponseActionFunction {
    #[schema(value_type = String)]
    pub prototype_id: ActionPrototypeId,
    pub func_name: String,
}

#[utoipa::path(
    get,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/components/{component_id}",
    params(
        ("workspace_id", description = "Workspace identifier"),
        ("change_set_id", description = "Change set identifier"),
        ("component_id", description = "Component identifier")
    ),
    tag = "components",
    responses(
        (status = 200, description = "Component retrieved successfully", body = GetComponentV1Response),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 404, description = "Component not found"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn get_component(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    Path(ComponentV1RequestPath { component_id }): Path<ComponentV1RequestPath>,
) -> Result<Json<GetComponentV1Response>, ComponentsError> {
    let (management_functions, action_functions) =
        super::get_component_functions(ctx, component_id).await?;

    Ok(Json(GetComponentV1Response {
        component: ComponentViewV1::assemble(ctx, component_id).await?,
        management_functions,
        action_functions,
    }))
}

pub async fn into_front_end_type(
    ctx: &dal::DalContext,
    component: Component,
) -> Result<si_frontend_types::DiagramComponentView, ComponentsError> {
    let mut socket_map = HashMap::new();
    Ok(component
        .into_frontend_type(
            ctx,
            None,
            component.change_status(ctx).await?,
            &mut socket_map,
        )
        .await?)
}
