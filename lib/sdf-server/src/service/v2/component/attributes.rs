use axum::{
    BoxError,
    Json,
    Router,
    error_handling::HandleErrorLayer,
    extract::Path,
    http::StatusCode,
    response::{
        IntoResponse,
        Response,
    },
    routing::{
        post,
        put,
    },
};
use dal::{
    AttributeValue,
    ChangeSet,
    ChangeSetId,
    ComponentId,
    WorkspacePk,
    attribute::attributes::{
        AttributeSources,
        AttributeValueIdent,
    },
};
use sdf_core::{
    api_error::ApiError,
    force_change_set_response::ForceChangeSetResponse,
};
use sdf_extract::{
    PosthogEventTracker,
    change_set::ChangeSetDalContext,
};
use serde_json::json;
use tower::ServiceBuilder;
use tower_http::decompression::RequestDecompressionLayer;

use super::{
    ComponentIdFromPath,
    Result,
};
use crate::app_state::AppState;

pub fn v2_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/",
            put(update_attributes).layer(
                ServiceBuilder::new()
                    .layer(HandleErrorLayer::new(handle_decompression_error))
                    .layer(
                        RequestDecompressionLayer::new()
                            .gzip(true)
                            .deflate(true)
                            .pass_through_unaccepted(true),
                    ),
            ),
        )
        .route("/enqueue", post(enqueue_prototype_function))
}

async fn handle_decompression_error(err: BoxError) -> Response {
    ApiError::new(StatusCode::UNSUPPORTED_MEDIA_TYPE, err.to_string()).into_response()
}

async fn update_attributes(
    ChangeSetDalContext(ref mut ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    Path(ComponentIdFromPath { component_id }): Path<ComponentIdFromPath>,
    Json(updates): Json<AttributeSources>,
) -> Result<ForceChangeSetResponse<()>> {
    let force_change_set_id = ChangeSet::force_new(ctx).await?;

    let counts = dal::update_attributes(ctx, component_id, updates).await?;

    ctx.commit().await?;

    tracker.track(
        ctx,
        "component_attributes_updated",
        json!({
            "how": "/component/attributes",
            "component_id": component_id,
            "change_set_id": ctx.change_set_id(),
            "set_count": counts.set_count,
            "unset_count": counts.unset_count,
            "subscription_count": counts.subscription_count,
        }),
    );

    Ok(ForceChangeSetResponse::new(force_change_set_id, ()))
}

async fn enqueue_prototype_function(
    ChangeSetDalContext(ref mut ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    Path((_workspace_pk, _change_set_id, component_id)): Path<(
        WorkspacePk,
        ChangeSetId,
        ComponentId,
    )>,
    Json(values): Json<Vec<AttributeValueIdent>>,
) -> Result<()> {
    let mut did_enqueue = false;
    for value in values {
        if let Some(attribute_value_id) = value.resolve(ctx, component_id).await? {
            if AttributeValue::is_set_by_dependent_function(ctx, attribute_value_id).await? {
                ctx.add_dependent_values_and_enqueue(vec![attribute_value_id])
                    .await?;
                did_enqueue = true;
            }
        }
    }
    if did_enqueue {
        ctx.commit().await?;
    }

    tracker.track(
        ctx,
        "enqueue_attribute_prototype",
        json!({
            "how": "/component/attributes",
            "component_id": component_id,
            "change_set_id": ctx.change_set_id(),
        }),
    );

    Ok(())
}
