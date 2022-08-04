use super::ChangeSetResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

use axum::extract::Query;
use axum::Json;
use dal::component::stats::ComponentStats;
use dal::Visibility;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetStatsRequest {
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetStatsResponse {
    pub component_stats: ComponentStats,
}

/// Gather statistics for the _current_ change set.
pub async fn get_stats(
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<GetStatsRequest>,
) -> ChangeSetResult<Json<GetStatsResponse>> {
    let txns = txns.start().await?;
    let ctx = builder.build(request_ctx.build(request.visibility), &txns);

    let component_stats = ComponentStats::new(&ctx).await?;

    // TODO(nick,fletcher): determine whether or not we should commit on accessor queries.
    // We will commit for now in case something eventually does get mutated in the DB.
    txns.commit().await?;

    Ok(Json(GetStatsResponse { component_stats }))
}
