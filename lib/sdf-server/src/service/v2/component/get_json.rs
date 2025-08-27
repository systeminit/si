use axum::{
    Json,
    extract::Path,
};
use dal::{
    Component,
    component::properties::ComponentProperties,
};
use sdf_extract::{
    PosthogEventTracker,
    change_set::ChangeSetDalContext,
};
use serde::{
    Deserialize,
    Serialize,
};

use super::Result;
use crate::service::v2::component::ComponentIdFromPath;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JsonResponse {
    pub json: ComponentProperties,
}

pub(crate) async fn get_json(
    ChangeSetDalContext(ref mut ctx): ChangeSetDalContext,
    _tracker: PosthogEventTracker,
    Path(ComponentIdFromPath { component_id }): Path<ComponentIdFromPath>,
) -> Result<Json<JsonResponse>> {
    let json = Component::get_json_representation(ctx, component_id).await?;

    Ok(Json(JsonResponse { json }))
}
