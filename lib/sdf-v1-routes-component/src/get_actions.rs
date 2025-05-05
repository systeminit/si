use axum::{
    Json,
    extract::{
        Host,
        OriginalUri,
        Query,
    },
};
use dal::{
    ActionPrototypeId,
    Component,
    ComponentId,
    DalContext,
    Func,
    action::prototype::{
        ActionKind,
        ActionPrototype,
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
use si_db::Visibility;

use super::ComponentResult;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionPrototypeView {
    id: ActionPrototypeId,
    name: String,
    display_name: Option<String>,
}

impl ActionPrototypeView {
    pub async fn new(
        ctx: &DalContext,
        prototype: ActionPrototype,
    ) -> ComponentResult<ActionPrototypeView> {
        let func = Func::get_by_id(ctx, ActionPrototype::func_id(ctx, prototype.id).await?).await?;
        let display_name = func.display_name.map(|dname| dname.to_string());
        Ok(Self {
            id: prototype.id,
            name: prototype.kind.to_string(),
            display_name,
        })
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetActionsResponse {
    pub actions: Vec<ActionPrototypeView>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetActionsRequest {
    pub component_id: ComponentId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub async fn get_actions(
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    PosthogClient(posthog_client): PosthogClient,
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<GetActionsRequest>,
) -> ComponentResult<Json<GetActionsResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let schema_variant = Component::get_by_id(&ctx, request.component_id)
        .await?
        .schema_variant(&ctx)
        .await?;

    let mut action_views: Vec<ActionPrototypeView> = Vec::new();
    let action_prototypes = ActionPrototype::for_variant(&ctx, schema_variant.id()).await?;
    for action_prototype in action_prototypes {
        if action_prototype.kind == ActionKind::Refresh {
            continue;
        }

        let view = ActionPrototypeView::new(&ctx, action_prototype).await?;
        action_views.push(view);
    }

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "get_actions",
        serde_json::json!({
            "how": "/component/get_actions",
            "component_id": request.component_id.clone(),
            "change_set_id": ctx.change_set_id(),
        }),
    );

    Ok(Json(GetActionsResponse {
        actions: action_views,
    }))
}
