use axum::{
    http::StatusCode,
    response::Json,
};
use dal::{
    Component,
    ComponentError,
    ComponentId,
    diagram::{
        geometry::RawGeometry,
        view::View,
    },
};
use serde::Deserialize;
use utoipa::ToSchema;

use super::ComponentsError;
use crate::extract::{
    PosthogEventTracker,
    change_set::ChangeSetDalContext,
};

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AddToViewV1Request {
    pub view_name: String,
    #[schema(value_type = Vec<String>)]
    pub component_ids: Vec<ComponentId>,
}

#[utoipa::path(
    post,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/components/add_to_view",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
    ),
    tag = "components",
    request_body = AddToViewV1Request,
    summary = "Add components to a view",
    description = "Adds multiple components to a view by name. If the view doesn't exist, it will be created automatically.",
    responses(
        (status = 204, description = "Components added to view successfully"),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 409, description = "Conflict - Changes not permitted on HEAD change set", body = crate::service::v1::common::ApiError),
        (status = 422, description = "Validation error - Invalid request data", body = crate::service::v1::common::ApiError),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn add_to_view(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    payload: Result<Json<AddToViewV1Request>, axum::extract::rejection::JsonRejection>,
) -> Result<StatusCode, ComponentsError> {
    let Json(payload) = payload?;

    // Prevent mutations on HEAD change set
    if ctx.change_set_id() == ctx.get_workspace_default_change_set_id().await? {
        return Err(ComponentsError::NotPermittedOnHead);
    }

    // Look up view by name, create if it doesn't exist
    let view = match View::find_by_name(ctx, &payload.view_name).await? {
        Some(view) => view,
        None => View::new(ctx, &payload.view_name).await?,
    };
    let view_id = view.id();

    // Add components to view - match sdf behavior:
    // If at least one component succeeds, don't blow up if errors happen
    let mut at_least_one_succeeded = false;
    let mut latest_error = None;

    for component_id in payload.component_ids.iter() {
        match Component::add_to_view(ctx, *component_id, view_id, RawGeometry::default()).await {
            Ok(_) => {}
            Err(err @ ComponentError::ComponentAlreadyInView(_, _)) => {
                latest_error = Some(err);
                continue;
            }
            Err(err) => return Err(err)?,
        };

        at_least_one_succeeded = true;
    }

    // If all components failed with ComponentAlreadyInView, return error
    if let Some(err) = latest_error {
        if !at_least_one_succeeded {
            return Err(err)?;
        }
    }

    // Track event
    tracker.track(
        ctx,
        "component_added_to_view",
        serde_json::json!({
            "view_id": view_id,
            "view_name": payload.view_name,
            "change_set_id": ctx.change_set_id(),
            "component_count": payload.component_ids.len(),
        }),
    );

    ctx.commit().await?;

    Ok(StatusCode::NO_CONTENT)
}
