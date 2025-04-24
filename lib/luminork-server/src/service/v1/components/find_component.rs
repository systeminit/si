use axum::response::Json;
use dal::{
    Component,
    management::prototype::ManagementPrototype,
};
use serde::{
    Deserialize,
    Serialize,
};
use utoipa::{
    self,
    ToSchema,
};

use super::{
    ComponentsError,
    connections::{
        ComponentReference,
        resolve_component_reference,
    },
    get_component::{
        GetComponentV1Response,
        GetComponentV1ResponseManagementFunction,
    },
};
use crate::{
    api_types::component::v1::ComponentViewV1,
    extract::change_set::ChangeSetDalContext,
};

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FindComponentV1Request {
    #[serde(flatten)]
    pub component_ref: ComponentReference,
}

#[utoipa::path(
    post,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/components/find",
    params(
        ("workspace_id", description = "Workspace identifier"),
        ("change_set_id", description = "Change set identifier"),
    ),
    request_body = FindComponentV1Request,
    tag = "components",
    responses(
        (status = 200, description = "Component retrieved successfully", body = GetComponentV1Response),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 404, description = "Component not found"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn find_component(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    payload: Result<Json<FindComponentV1Request>, axum::extract::rejection::JsonRejection>,
) -> Result<Json<GetComponentV1Response>, ComponentsError> {
    let Json(payload) = payload?;
    let component_list = Component::list_ids(ctx).await?;
    let component_id =
        resolve_component_reference(ctx, &payload.component_ref, &component_list).await?;
    let schema_variant_id = Component::schema_variant_id(ctx, component_id).await?;
    let management_functions = ManagementPrototype::list_for_variant_id(ctx, schema_variant_id)
        .await?
        .into_iter()
        .map(|prototype| GetComponentV1ResponseManagementFunction {
            management_prototype_id: prototype.id,
            name: prototype.name,
        })
        .collect();

    Ok(Json(GetComponentV1Response {
        component: ComponentViewV1::assemble(ctx, component_id).await?,
        management_functions,
    }))
}
