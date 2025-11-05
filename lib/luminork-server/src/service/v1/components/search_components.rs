use std::{
    collections::HashSet,
    sync::Arc,
};

use axum::response::Json;
use dal::{
    Component,
    DalContext,
    Schema,
    SchemaVariant,
};
use sdf_extract::{
    FriggStore,
    change_set::ChangeSetAuthorization,
};
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::json;
use si_id::ComponentId;
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
    search::{
        self,
        SearchQuery,
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
        (status = 412, description = "Precondition Failed - missing or invalid change set index"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn search_components(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    FriggStore(ref frigg): FriggStore,
    ChangeSetAuthorization {
        workspace_id,
        change_set_id,
        ..
    }: ChangeSetAuthorization,
    tracker: PosthogEventTracker,
    payload: Result<Json<SearchComponentsV1Request>, axum::extract::rejection::JsonRejection>,
) -> Result<Json<SearchComponentsV1Response>, ComponentsError> {
    let Json(payload) = payload?;

    let query: SearchQuery = payload.query_string.as_deref().unwrap_or("").parse()?;
    let query = Arc::new(query);
    let mut component_ids = search::component::search(frigg, workspace_id, change_set_id, &query)
        .await?
        .into_iter()
        .map(|component| component.id)
        .collect();

    if let Some(schema_name) = payload.schema_name.clone() {
        component_ids = apply_schema_filter(ctx, component_ids, schema_name).await?;
    }

    if let Some(upgradable) = payload.upgradable {
        component_ids = apply_upgradable_filter(ctx, component_ids, upgradable).await?;
    }

    if let Some(category) = payload.schema_category.clone() {
        component_ids = apply_category_filter(ctx, component_ids, category).await?;
    }

    tracker.track(
        ctx,
        "api_search_components",
        json!({
            "query_string": payload.query_string.as_ref(),
            "schema_name": payload.schema_name.as_ref(),
            "upgradable": payload.upgradable.as_ref(),
            "schema_category": payload.schema_category.as_ref(),
        }),
    );

    Ok(Json(SearchComponentsV1Response {
        components: component_ids,
    }))
}

async fn apply_upgradable_filter(
    ctx: &DalContext,
    component_ids: Vec<ComponentId>,
    upgrade_filter: bool,
) -> ComponentsResult<Vec<ComponentId>> {
    let mut upgradable_filtered_components = Vec::new();

    for component_id in component_ids {
        let is_upgradable = Component::can_be_upgraded_by_id(ctx, component_id).await?;
        if is_upgradable == upgrade_filter {
            upgradable_filtered_components.push(component_id);
        }
    }

    upgradable_filtered_components.sort();
    upgradable_filtered_components.dedup();

    Ok(upgradable_filtered_components)
}

async fn apply_category_filter(
    ctx: &DalContext,
    component_ids: Vec<ComponentId>,
    category: String,
) -> ComponentsResult<Vec<ComponentId>> {
    let mut category_filtered_components = Vec::new();

    for component_id in component_ids {
        let component = Component::get_by_id(ctx, component_id).await?;
        let variant = component.schema_variant(ctx).await?;

        if variant.category() == category {
            category_filtered_components.push(component_id);
        }
    }

    category_filtered_components.sort();
    category_filtered_components.dedup();

    Ok(category_filtered_components)
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
    #[schema(required = false)]
    pub upgradable: Option<bool>,
    #[schema(example = "AWS::EC2", required = false)]
    pub schema_category: Option<String>,
    #[schema(required = false)]
    pub query_string: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SearchComponentsV1Response {
    #[schema(value_type = Vec<Vec<String>>, example = json!(["01H9ZQD35JPMBGHH69BT0Q79AA", "01H9ZQD35JPMBGHH69BT0Q79BB", "01H9ZQD35JPMBGHH69BT0Q79CC"]))]
    pub components: Vec<ComponentId>,
}
