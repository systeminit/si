use axum::{
    Json,
    extract::{
        Host,
        OriginalUri,
    },
};
use dal::{
    Component,
    ComponentId,
    Visibility,
    action::{
        Action,
        prototype::{
            ActionKind,
            ActionPrototype,
        },
    },
};
use sdf_core::tracking::track;
use sdf_extract::{
    HandlerContext,
    PosthogClient,
    v1::AccessBuilder,
};
use serde::{
    Deserialize,
    Serialize,
};

use super::ComponentResult;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RefreshRequest {
    pub component_id: ComponentId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RefreshResponse {
    pub success: bool,
}

pub async fn refresh(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Json(request): Json<RefreshRequest>,
) -> ComponentResult<Json<RefreshResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let component_ids = vec![request.component_id];

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "refresh_resource",
        serde_json::json!({
            "component_ids": &component_ids,
        }),
    );

    // Parallelizes resource refreshing
    for component_id in component_ids {
        let variant = Component::schema_variant_for_component_id(&ctx, component_id).await?;

        let all_prototypes_for_variant: Vec<ActionPrototype> =
            ActionPrototype::for_variant(&ctx, variant.id()).await?;
        for prototype in all_prototypes_for_variant {
            if prototype.kind == ActionKind::Refresh {
                Action::new(&ctx, prototype.id(), Some(component_id)).await?;
            }
        }
    }

    ctx.commit().await?;

    Ok(Json(RefreshResponse { success: true }))
}
