use axum::{
    Json,
    extract::Path,
};
use dal::{
    ChangeSetId,
    Component,
    ComponentId,
    WorkspacePk,
    management::prototype::{
        ManagementPrototype,
        ManagementPrototypeId,
    },
};
use sdf_extract::change_set::ChangeSetDalContext;

use super::ManagementApiResult;
use crate::service::v2::func::get_func_run::{
    FuncRunView,
    get_func_run_view,
};

pub async fn latest(
    ChangeSetDalContext(ref mut ctx): ChangeSetDalContext,
    Path((workspace_pk, change_set_id, prototype_id, component_id)): Path<(
        WorkspacePk,
        ChangeSetId,
        ManagementPrototypeId,
        ComponentId,
    )>,
) -> ManagementApiResult<Json<Option<FuncRunView>>> {
    let func_id = ManagementPrototype::func_id(ctx, prototype_id).await?;

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
            Some(func_run) => Json(Some(get_func_run_view(ctx, &func_run).await?)),
            None => Json(None),
        },
    )
}

pub async fn all_latest_for_component(
    ChangeSetDalContext(ref mut ctx): ChangeSetDalContext,
    Path((workspace_pk, change_set_id, component_id)): Path<(
        WorkspacePk,
        ChangeSetId,
        ComponentId,
    )>,
) -> ManagementApiResult<Json<Vec<FuncRunView>>> {
    let sv_id = Component::schema_variant_id(ctx, component_id).await?;

    let mut runs = vec![];

    for mgmt_prototype in ManagementPrototype::list_for_variant_id(ctx, sv_id).await? {
        let func_id = ManagementPrototype::func_id(ctx, mgmt_prototype.id).await?;

        let Some(run) = ctx
            .layer_db()
            .func_run()
            .get_last_management_run_for_func_and_component_id(
                workspace_pk,
                change_set_id,
                component_id,
                func_id.into_inner().into(),
            )
            .await?
        else {
            continue;
        };

        runs.push(get_func_run_view(ctx, &run).await?);
    }

    Ok(Json(runs))
}
