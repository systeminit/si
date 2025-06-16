use std::str::FromStr;

use axum::{
    extract::Query,
    response::Json,
};
use dal::{
    Component,
    ComponentId,
};
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::json;
use utoipa::{
    self,
    IntoParams,
    ToSchema,
};

use super::{
    ComponentReference,
    ComponentsError,
    get_component::GetComponentV1Response,
    resolve_component_reference,
};
use crate::{
    extract::{
        PosthogEventTracker,
        change_set::ChangeSetDalContext,
    },
    service::v1::ComponentViewV1,
};

#[derive(Deserialize, Serialize, Debug, IntoParams, ToSchema)]
#[serde(rename_all = "camelCase")]
#[into_params(style = Form, parameter_in = Query)]
pub struct FindComponentV1Params {
    #[param(required = false, nullable = true)]
    pub component: Option<String>,

    #[serde(rename = "componentId")]
    #[param(value_type = String, required = false, nullable = true)]
    #[schema(value_type = String)]
    pub component_id: Option<String>,
}

#[utoipa::path(
    get,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/components/find",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
        FindComponentV1Params,
    ),
    tag = "components",
    summary = "Find a component by name or component Id",
    responses(
        (status = 200, description = "Component retrieved successfully", body = GetComponentV1Response),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 404, description = "Component not found"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn find_component(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    Query(params): Query<FindComponentV1Params>,
) -> Result<Json<GetComponentV1Response>, ComponentsError> {
    let component_list = Component::list_ids(ctx).await?;
    let component_ref = if let Some(component_id_str) = &params.component_id {
        let component_id = ComponentId::from_str(component_id_str)?;
        ComponentReference::ById { component_id }
    } else if let Some(component_name) = &params.component {
        ComponentReference::ByName {
            component: component_name.clone(),
        }
    } else {
        return Err(ComponentsError::Validation(
            "Either component or componentId must be provided".to_string(),
        ));
    };

    let component_id = resolve_component_reference(ctx, &component_ref, &component_list).await?;

    tracker.track(
        ctx,
        "api_find_component",
        json!({
            "component_id": component_id
        }),
    );

    let (management_functions, action_functions) =
        super::get_component_functions(ctx, component_id).await?;

    Ok(Json(GetComponentV1Response {
        component: ComponentViewV1::assemble(ctx, component_id).await?,
        management_functions,
        action_functions,
    }))
}
