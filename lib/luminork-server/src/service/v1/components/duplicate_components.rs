use axum::response::Json;
use dal::{
    Component,
    ComponentId,
    diagram::view::View,
};
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::json;
use si_id::ViewId;
use utoipa::{
    self,
    ToSchema,
};

use crate::{
    extract::{
        PosthogEventTracker,
        change_set::ChangeSetDalContext,
    },
    service::v1::ComponentsError,
};

#[utoipa::path(
    post,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/components/duplicate",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
    ),
    tag = "components",
    request_body = DuplicateComponentsV1Request,
    summary = "Duplicate a list of components",
    responses(
        (status = 200, description = "Components duplicated successfully", body = DuplicateComponentsV1Response),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn duplicate_components(
    ChangeSetDalContext(ref mut ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    payload: Result<Json<DuplicateComponentsV1Request>, axum::extract::rejection::JsonRejection>,
) -> Result<Json<DuplicateComponentsV1Response>, ComponentsError> {
    let Json(payload) = payload?;

    if ctx.change_set_id() == ctx.get_workspace_default_change_set_id().await? {
        return Err(ComponentsError::NotPermittedOnHead);
    }

    let prefix = payload.prefix.unwrap_or("copy of".to_string());

    let view_id: ViewId;
    if let Some(view_name) = payload.view_name {
        if let Some(view) = View::find_by_name(ctx, view_name.as_str()).await? {
            view_id = view.id();
        } else {
            let view = View::new(ctx, view_name.as_str()).await?;
            view_id = view.id()
        }
    } else {
        let default_view = View::get_id_for_default(ctx).await?;
        view_id = default_view
    };

    let duplicated_components =
        Component::duplicate(ctx, view_id, payload.components, &prefix).await?;

    tracker.track(
        ctx,
        "api_duplicate_components",
        json!({
            "duplicated_components": duplicated_components.len(),
            "prefix": prefix,
            "view": view_id,
        }),
    );

    ctx.commit().await?;

    Ok(Json(DuplicateComponentsV1Response {
        components: duplicated_components,
    }))
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateComponentsV1Request {
    #[schema(value_type = Vec<Vec<String>>, example = json!(["01H9ZQD35JPMBGHH69BT0Q79AA", "01H9ZQD35JPMBGHH69BT0Q79BB", "01H9ZQD35JPMBGHH69BT0Q79CC"]))]
    pub components: Vec<ComponentId>,
    #[schema(value_type = Option<String>, example = "copy-of-", required = false)]
    pub prefix: Option<String>,
    #[schema(value_type = Option<String>, example = "MyView")]
    pub view_name: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateComponentsV1Response {
    #[schema(value_type = Vec<Vec<String>>, example = json!(["01H9ZQD35JPMBGHH69BT0Q79AA", "01H9ZQD35JPMBGHH69BT0Q79BB", "01H9ZQD35JPMBGHH69BT0Q79CC"]))]
    pub components: Vec<ComponentId>,
}
