use axum::{
    Json,
    extract::Path,
};
use dal::{
    ChangeSetId,
    ComponentId,
    WorkspacePk,
    management::prototype::{
        ManagementPrototype,
        ManagementPrototypeId,
    },
};

use super::ManagementApiResult;
use crate::{
    extract::HandlerContext,
    service::v2::{
        AccessBuilder,
        func::get_func_run::{
            FuncRunView,
            get_func_run_view,
        },
    },
};

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
