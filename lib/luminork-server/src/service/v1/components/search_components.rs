use std::collections::HashSet;

use axum::response::Json;
use dal::{
    Component,
    ComponentId,
    DalContext,
    Schema,
    SchemaVariant,
};
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::json;
use utoipa::{
    self,
    ToSchema,
};

use super::ComponentsResult;
use crate::{
    extract::{
        PosthogEventTracker,
        change_set::ChangeSetDalContext,
    },
    service::v1::ComponentsError,
};

#[utoipa::path(
    post,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/components/search",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
    ),
    tag = "components",
    request_body = SearchComponentsV1Request,
    summary = "Complex search for components",
    responses(
        (status = 200, description = "Components retrieved successfully", body = SearchComponentsV1Response),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 404, description = "Component not found"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn search_components(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    payload: Result<Json<SearchComponentsV1Request>, axum::extract::rejection::JsonRejection>,
) -> Result<Json<SearchComponentsV1Response>, ComponentsError> {
    let Json(payload) = payload?;

    let mut component_ids = Component::list_ids(ctx).await?;

    if let Some(schema_name) = payload.schema_name.clone() {
        component_ids = apply_schema_filter(ctx, component_ids, schema_name).await?;
    }

    tracker.track(
        ctx,
        "api_search_components",
        json!({
            "schema_name": payload.schema_name,
        }),
    );

    Ok(Json(SearchComponentsV1Response {
        components: component_ids,
    }))
}

async fn apply_schema_filter(
    ctx: &DalContext,
    component_ids: Vec<ComponentId>,
    schema_name: String,
) -> ComponentsResult<Vec<ComponentId>> {
    let schema = Schema::get_by_name(ctx, schema_name.clone())
        .await
        .map_err(|_| ComponentsError::SchemaNameNotFound(schema_name))?;

    let current_ids_set: HashSet<_> = component_ids.iter().collect();

    let variant_ids = Schema::list_schema_variant_ids(ctx, schema.id()).await?;
    let mut schema_filtered_components = Vec::new();

    for variant_id in variant_ids {
        let matching_comps = SchemaVariant::list_component_ids(ctx, variant_id).await?;

        schema_filtered_components.extend(
            matching_comps
                .into_iter()
                .filter(|comp_id| current_ids_set.contains(comp_id)),
        );
    }

    schema_filtered_components.sort();
    schema_filtered_components.dedup();

    Ok(schema_filtered_components)
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SearchComponentsV1Request {
    #[schema(example = "AWS::EC2::Instance", required = false)]
    pub schema_name: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SearchComponentsV1Response {
    #[schema(value_type = Vec<Vec<String>>, example = json!(["01H9ZQD35JPMBGHH69BT0Q79AA", "01H9ZQD35JPMBGHH69BT0Q79BB", "01H9ZQD35JPMBGHH69BT0Q79CC"]))]
    pub components: Vec<ComponentId>,
}
