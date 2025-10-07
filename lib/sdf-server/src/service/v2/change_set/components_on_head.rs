use axum::Json;
use dal::Component;
use sdf_extract::workspace::WorkspaceDalContext;
use serde::{
    Deserialize,
    Serialize,
};

use super::Result;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ComponentsOnHeadResponse {
    components_found: bool,
}

pub async fn components_on_head(
    WorkspaceDalContext(ref mut ctx): WorkspaceDalContext,
) -> Result<Json<ComponentsOnHeadResponse>> {
    let components_on_head = Component::list_ids(ctx).await?;

    Ok(Json(ComponentsOnHeadResponse {
        components_found: !components_on_head.is_empty(),
    }))
}
