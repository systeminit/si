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
use si_db::FuncRunDb;

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

    let maybe_view = if let Some(func_run) =
        FuncRunDb::get_last_management_run_for_func_and_component_id(
            ctx,
            workspace_pk,
            change_set_id,
            component_id,
            func_id.into_inner().into(),
        )
        .await?
    {
        Some(get_func_run_view(ctx, &func_run).await?)
    } else {
        None
    };

    Ok(Json(maybe_view))
}

pub async fn all_latest_for_component(
    ChangeSetDalContext(ref mut ctx): ChangeSetDalContext,
    Path((workspace_pk, change_set_id, component_id)): Path<(
        WorkspacePk,
        ChangeSetId,
        ComponentId,
    )>,
) -> ManagementApiResult<Json<Vec<FuncRunView>>> {
    let schema_variant_id = Component::schema_variant_id(ctx, component_id).await?;

    let mut runs = vec![];

    for mgmt_prototype in
        ManagementPrototype::list_for_schema_and_variant_id(ctx, schema_variant_id).await?
    {
        let func_id = ManagementPrototype::func_id(ctx, mgmt_prototype.id).await?;

        let Some(run) = FuncRunDb::get_last_management_run_for_func_and_component_id(
            ctx,
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
