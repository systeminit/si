use axum::{response::IntoResponse, Json};
use dal::func::argument::FuncArgument;
use dal::{
    AttributePrototype, ChangeSet, Func, FuncBackendKind, FuncId, StandardModel, Visibility,
    WsEvent,
};
use serde::{Deserialize, Serialize};

use super::{FuncError, FuncResult};
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RevertFuncRequest {
    pub id: FuncId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RevertFuncResponse {
    pub success: bool,
}

pub async fn revert_func(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<RevertFuncRequest>,
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

    let func = Func::get_by_id(&ctx, &request.id)
        .await?
        .ok_or(FuncError::FuncNotFound)?;

    let is_revertible = super::is_func_revertible(&ctx, &func).await?;

    if !is_revertible {
        Err(FuncError::FuncNotRevertible)?
    } else {
        if func.backend_kind() == &FuncBackendKind::JsAttribute {
            for proto in AttributePrototype::find_for_func(&ctx, func.id()).await? {
                if proto.visibility().in_change_set() {
                    AttributePrototype::hard_delete_if_in_changeset(&ctx, proto.id()).await?;
                }
            }
        }

        for arg in FuncArgument::list_for_func(&ctx, *func.id()).await? {
            if arg.visibility().in_change_set() {
                arg.hard_delete(&ctx).await?;
            }
        }

        func.hard_delete(&ctx).await?;

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
        Ok(response.body(serde_json::to_string(&RevertFuncResponse {
            success: true,
        })?)?)
    }
}
