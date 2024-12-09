use axum::{extract::Path, Json};
use dal::{
    management::prototype::{ManagementPrototype, ManagementPrototypeId},
    ChangeSetId, ComponentId, WorkspacePk,
};

use crate::{
    extract::{AccessBuilder, HandlerContext},
    service::v2::func::get_func_run::{get_func_run_view, FuncRunView},
};

use super::ManagementApiResult;

pub async fn latest(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Path((workspace_pk, change_set_id, prototype_id, component_id)): Path<(
        WorkspacePk,
        ChangeSetId,
        ManagementPrototypeId,
        ComponentId,
    )>,
) -> ManagementApiResult<Json<Option<FuncRunView>>> {
    let ctx = builder
        .build(request_ctx.build(change_set_id.into()))
        .await?;

    let func_id = ManagementPrototype::func_id(&ctx, prototype_id).await?;

    Ok(
        match ctx
            .layer_db()
            .func_run()
            .get_last_management_run_for_func_and_component_id(
                workspace_pk,
                change_set_id,
                component_id,
                func_id.into_inner().into(),
            )
            .await?
        {
            Some(func_run) => Json(Some(get_func_run_view(&ctx, &func_run).await?)),
            None => Json(None),
        },
    )
}
