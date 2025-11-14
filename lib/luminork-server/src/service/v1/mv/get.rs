use axum::{
    Json,
    extract::Query,
};
use sdf_extract::FriggStore;
use serde::{
    Deserialize,
    Serialize,
};
use si_frontend_mv_types::object::FrontendObject;
use utoipa::{
    IntoParams,
    ToSchema,
};

use super::{
    MvError,
    MvResult,
};
use crate::extract::change_set::ChangeSetDalContext;

#[derive(Deserialize, Serialize, ToSchema, IntoParams, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetParams {
    #[schema(example = "01H9ZQD35JPMBGHH69BT0Q79VY", nullable = false, value_type = String)]
    pub entity_id: String,
    #[schema(example = "ComponentList", nullable = false, value_type = String)]
    pub kind: String,
}

#[derive(Deserialize, Serialize, Debug, ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MvResponse {
    #[schema(example = "ComponentList")]
    pub kind: String,
    #[schema(example = "")]
    pub id: String,
    #[schema(example = "")]
    pub checksum: String,
    #[schema(example = "{}")]
    pub data: serde_json::Value,
}

#[utoipa::path(
    get,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/mv",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
        GetParams,
    ),
    tag = "mv",
    summary = "Identifiers for a materialized view",
    responses(
        (status = 200, description = "Mv retrieved successfully", body = MvResponse),
        (status = 404, description = "Mv not found"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn get(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    Query(params): Query<GetParams>,
    FriggStore(frigg): FriggStore,
) -> MvResult<Json<MvResponse>> {
    let obj = frigg
        .get_current_workspace_object(
            ctx.workspace_pk()?,
            ctx.change_set_id(),
            &params.kind,
            &params.entity_id,
        )
        .await?;
    match obj {
        Some(FrontendObject {
            kind,
            id,
            checksum,
            data,
        }) => Ok(Json(MvResponse {
            kind,
            id,
            checksum,
            data,
        })),
        None => Err(MvError::NotFound(params.kind, params.entity_id)),
    }
}
