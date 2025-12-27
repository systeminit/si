use axum::Json;
use dal::slow_rt;
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::{
    Value,
    json,
};
use si_frontend_mv_types::{
    action::action_diff_list::ActionDiffView,
    luminork_change_set_review::LuminorkChangeSetReview,
};
use si_id::ComponentId;
use utoipa::ToSchema;

use super::ChangeSetResult;
use crate::extract::{
    PosthogEventTracker,
    change_set::ChangeSetDalContext,
};

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GetReviewV1Response {
    pub components: Vec<ComponentReviewV1>,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ComponentReviewV1 {
    #[schema(value_type = String, example = "01FXNV4P306V3KGZ73YSVN8A61")]
    pub id: ComponentId,
    #[schema(example = "My EC2 Instance")]
    pub name: String,
    #[schema(value_type = Option<String>, example = "#FF5733")]
    pub color: Option<String>,
    #[schema(example = "AWS EC2 Instance")]
    pub schema_name: String,
    #[schema(example = "AWS::EC2")]
    pub schema_category: String,
    #[schema(example = true)]
    pub has_resource: bool,
    #[schema(value_type = String, example = "Modified")]
    pub diff_status: String,
    #[schema(value_type = Vec<Object>)]
    pub attribute_diff_trees: Vec<AttributeDiffTreeV1>,
    #[schema(value_type = Vec<Object>)]
    pub action_diffs: Vec<ActionDiffView>,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AttributeDiffTreeV1 {
    /// The full path to this attribute
    #[schema(example = "/domain/Region")]
    pub path: String,

    /// The diff for this attribute
    #[schema(value_type = Object)]
    pub diff: Value,
}

impl From<LuminorkChangeSetReview> for GetReviewV1Response {
    fn from(review: LuminorkChangeSetReview) -> Self {
        GetReviewV1Response {
            components: review.components.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<si_frontend_mv_types::luminork_change_set_review::ComponentReview> for ComponentReviewV1 {
    fn from(component: si_frontend_mv_types::luminork_change_set_review::ComponentReview) -> Self {
        ComponentReviewV1 {
            id: component.component.id,
            name: component.component.name,
            color: component.component.color,
            schema_name: component.component.schema_name,
            schema_category: component.component.schema_category,
            has_resource: component.component.has_resource,
            diff_status: component.corrected_diff_status.to_string(),
            attribute_diff_trees: component
                .attribute_diff_trees
                .into_iter()
                .map(Into::into)
                .collect(),
            action_diffs: component.action_diffs,
        }
    }
}

impl From<si_frontend_mv_types::luminork_change_set_review::AttributeDiffTree>
    for AttributeDiffTreeV1
{
    fn from(tree: si_frontend_mv_types::luminork_change_set_review::AttributeDiffTree) -> Self {
        AttributeDiffTreeV1 {
            path: tree.path,
            diff: serde_json::to_value(tree.diff).unwrap_or(Value::Null),
        }
    }
}

#[utoipa::path(
    get,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/review",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier")
    ),
    tag = "change_sets",
    summary = "Get a comprehensive review of all changes in a Change Set",
    responses(
        (status = 200, description = "Change set review retrieved successfully", body = GetReviewV1Response),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 404, description = "Change set not found"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn get_review(
    ChangeSetDalContext(ref mut ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
) -> ChangeSetResult<Json<GetReviewV1Response>> {
    let workspace_pk = ctx.workspace_pk()?;
    let change_set_id = ctx.change_set_id();

    tracker.track(
        ctx,
        "api_get_change_set_review",
        json!({
            "workspace_pk": workspace_pk.to_string(),
            "change_set_id": change_set_id.to_string(),
        }),
    );

    let review =
        slow_rt::spawn(dal_materialized_views::luminork::change_set_review::assemble(ctx.clone()))?
            .await??;

    Ok(Json(review.into()))
}
