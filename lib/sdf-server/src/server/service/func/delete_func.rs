use super::FuncResult;
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use crate::service::func::{get_func_view, FuncAssociations, FuncError};
use axum::extract::OriginalUri;
use axum::{response::IntoResponse, Json};
use dal::{ChangeSet, Func, FuncId, StandardModel, Visibility, WsEvent};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeleteFuncRequest {
    pub id: FuncId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeleteFuncResponse {
    pub success: bool,
}

pub async fn delete_func(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<DeleteFuncRequest>,
) -> FuncResult<impl IntoResponse> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let mut force_changeset_pk = None;
    if ctx.visibility().is_head() {
        let change_set = ChangeSet::new(&ctx, ChangeSet::generate_name(), None).await?;

        let new_visibility = Visibility::new(change_set.pk, request.visibility.deleted_at);

        ctx.update_visibility(new_visibility);

        force_changeset_pk = Some(change_set.pk);

        WsEvent::change_set_created(&ctx, change_set.pk)
            .await?
            .publish_on_commit(&ctx)
            .await?;
    };

    let mut func = Func::get_by_id(&ctx, &request.id)
        .await?
        .ok_or(FuncError::FuncNotFound)?;

    let func_details = get_func_view(&ctx, &func).await?;
    if let Some(associations) = func_details.associations {
        let has_associations = match associations {
            FuncAssociations::Action {
                schema_variant_ids,
                kind: _,
            } => !schema_variant_ids.is_empty(),
            FuncAssociations::Attribute {
                prototypes,
                arguments,
            } => !prototypes.is_empty() || !arguments.is_empty(),
            FuncAssociations::CodeGeneration {
                schema_variant_ids,
                component_ids,
                inputs: _,
            } => !schema_variant_ids.is_empty() || !component_ids.is_empty(),
            FuncAssociations::Qualification {
                schema_variant_ids,
                component_ids,
                inputs: _,
            } => !schema_variant_ids.is_empty() || !component_ids.is_empty(),
            FuncAssociations::SchemaVariantDefinitions { schema_variant_ids } => {
                schema_variant_ids.is_empty()
            }
            FuncAssociations::Validation { prototypes } => !prototypes.is_empty(),
        };

        if has_associations {
            return Err(FuncError::FuncHasAssociations(*func.id()));
        }
    };

    func.delete_by_id(&ctx).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "deleted_func",
        serde_json::json!({
                    "func_id": func.id().to_owned(),
                    "func_name": func.name().to_owned(),
        }),
    );

    WsEvent::change_set_written(&ctx)
        .await?
        .publish_on_commit(&ctx)
        .await?;
    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    response = response.header("Content-Type", "application/json");
    if let Some(force_changeset_pk) = force_changeset_pk {
        response = response.header("force_changeset_pk", force_changeset_pk.to_string());
    }
    Ok(response.body(serde_json::to_string(&DeleteFuncResponse {
        success: true,
    })?)?)
}
