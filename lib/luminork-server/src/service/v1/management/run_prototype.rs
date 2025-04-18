use axum::{extract::Path, response::Json};
use dal::{
    diagram::view::ViewId,
    management::{
        prototype::{ManagementPrototype, ManagementPrototypeId},
        ManagementFuncReturn, ManagementOperator,
    },
    ComponentId, Func, WsEvent,
};
use serde::{Deserialize, Serialize};
use si_events::audit_log::AuditLogKind;
use utoipa::{self, ToSchema};
use veritech_client::ManagementFuncStatus;

use crate::extract::{change_set::ChangeSetDalContext, PosthogEventTracker};
use crate::service::v1::ManagementApiError;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RunPrototypeV1Request {
    #[schema(value_type = String, nullable = true, example = "01FXNV4P306V3KGZ73YSVN8A60")]
    pub request_ulid: Option<ulid::Ulid>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RunPrototypeV1Response {
    #[schema(value_type = String, example = "Ok")]
    pub status: ManagementFuncStatus,
    #[schema(value_type = String, nullable = true, example = "Successfully executed management function")]
    pub message: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct RunPrototypePath {
    #[schema(value_type = String, example = "01FXNV4P306V3KGZ73YSVN8A60")]
    pub management_prototype_id: ManagementPrototypeId,
    #[schema(value_type = String, example = "01FXNV4P306V3KGZ73YSVN8A60")]
    pub component_id: ComponentId,
    #[schema(value_type = String, example = "01FXNV4P306V3KGZ73YSVN8A60")]
    pub view_id: ViewId,
}

#[utoipa::path(
    post,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/management/prototype/{management_prototype_id}/{component_id}/{view_id}",
    params(
        ("workspace_id", description = "Workspace identifier"),
        ("change_set_id", description = "Change set identifier"),
        ("management_prototype_id", description = "Management prototype identifier"),
        ("component_id", description = "Component identifier"),
        ("view_id", description = "View identifier")
    ),
    tag = "management",
    request_body = RunPrototypeV1Request,
    responses(
        (status = 200, description = "Management prototype executed successfully", body = RunPrototypeV1Response),
        (status = 400, description = "Bad request - Execution failed"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn run_prototype(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    Path(RunPrototypePath {
        management_prototype_id: prototype_id,
        component_id,
        view_id,
    }): Path<RunPrototypePath>,
    Json(request): Json<RunPrototypeV1Request>,
) -> Result<Json<RunPrototypeV1Response>, ManagementApiError> {
    let mut execution_result =
        ManagementPrototype::execute_by_id(ctx, prototype_id, component_id, view_id.into()).await?;

    tracker.track(
        ctx,
        "run_prototype",
        serde_json::json!({
            "how": "/public/management/run_prototype",
            "view_id": view_id,
            "prototype_id": prototype_id,
            "component_id": component_id,
        }),
    );

    if let Some(result) = execution_result.result.take() {
        let ManagementFuncReturn {
            status,
            message,
            operations,
            ..
        } = result.try_into()?;
        let mut created_component_ids = None;
        if status == ManagementFuncStatus::Ok {
            if let Some(operations) = operations {
                created_component_ids = ManagementOperator::new(
                    ctx,
                    component_id,
                    operations,
                    execution_result,
                    Some(view_id),
                )
                .await?
                .operate()
                .await?;
            }
        }

        let func_id = ManagementPrototype::func_id(ctx, prototype_id).await?;
        let func = Func::get_by_id(ctx, func_id).await?;

        WsEvent::management_operations_complete(
            ctx,
            request.request_ulid,
            func.name.clone(),
            message.clone(),
            status,
            created_component_ids,
        )
        .await?
        .publish_on_commit(ctx)
        .await?;

        ctx.write_audit_log(
            AuditLogKind::ManagementOperationsComplete {
                component_id,
                prototype_id,
                func_id,
                func_name: func.name.clone(),
                status: match status {
                    ManagementFuncStatus::Ok => "ok",
                    ManagementFuncStatus::Error => "error",
                }
                .to_string(),
                message: message.clone(),
            },
            func.name,
        )
        .await?;

        ctx.commit().await?;

        return Ok(Json(RunPrototypeV1Response { status, message }));
    }

    Err(ManagementApiError::ManagementPrototypeExecutionFailure(
        prototype_id,
    ))
}
