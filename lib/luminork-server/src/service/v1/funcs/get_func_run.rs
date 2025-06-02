use axum::{
    Json,
    extract::Path,
};
use serde::Serialize;
use serde_json::json;
use utoipa::ToSchema;

use super::{
    FuncRunV1RequestPath,
    FuncsError,
    FuncsResult,
};
use crate::{
    api_types::func_run::v1::FuncRunViewV1,
    extract::{
        PosthogEventTracker,
        change_set::ChangeSetDalContext,
    },
};

#[utoipa::path(
    get,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/funcs/runs/{func_run_id}",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
        ("func_run_id" = String, Path, description = "Func run identifier"),
    ),
    tag = "funcs",
    summary = "Get func execution run logs",
    responses(
        (status = 200, description = "Func Run retrieved successfully", body = GetFuncRunV1Response,
          example = json!({
            "funcRun": {
              "id": "01JQCJ0AAXGX5M9QY10AVF4GK1",
              "state": "Success",
              "actor": "System",
              "componentId": "01JP8KHZP3DZKGNXRP83Q6WTQ5",
              "attributeValueId": null,
              "componentName": "NAT Gateway IP 1",
              "schemaName": "AWS::EC2::EIP",
              "actionId": "01JQCHZZY99G3R0C1FA3W4AFR6",
              "actionPrototypeId": "01JPNHEE9Z3DFW48XVZ1FX04KA",
              "actionKind": "Destroy",
              "actionDisplayName": "Destroy",
              "actionOriginatingChangeSetId": "01JQCHZZVTAHHZ7DG0ZSCB9RXB",
              "actionOriginatingChangeSetName": "2025-03-27-19:41",
              "actionResultState": "Success",
              "backendKind": "JsAction",
              "backendResponseType": "Action",
              "functionName": "Delete Asset",
              "functionDisplayName": null,
              "functionKind": "Action",
              "functionDescription": null,
              "functionLink": null,
              "functionArgs": {
                "properties": {
                  "domain": {
                    "Domain": "vpc",
                    "Tags": []
                  },
                  "resource": {
                    "payload": {
                      "AllocationId": "eipalloc-033720f9556a3b0c1",
                      "PublicIp": "3.213.242.163"
                    }
                  },
                  "si": {
                    "name": "NAT Gateway IP 1",
                    "resourceId": "3.213.242.163|eipalloc-033720f9556a3b0c1",
                    "type": "component"
                  }
                }
              },
              "resultValue": {
                "error": null,
                "executionId": "01JQCJ0AAXGX5M9QY10AVF4GK1",
                "message": null,
                "payload": null,
                "resourceId": null,
                "status": "ok"
              },
              "logs": {
                "id": "01JQCJ0ABJSCE01GNQDWVY1ZP5",
                "createdAt": "2025-03-27T19:41:58.514416748Z",
                "updatedAt": "2025-03-27T19:41:58.514416748Z",
                "funcRunId": "01JQCJ0AAXGX5M9QY10AVF4GK1",
                "logs": [
                  {
                    "stream": "stdout",
                    "executionId": "",
                    "level": "info",
                    "group": "log",
                    "message": "Running CLI command",
                    "timestamp": 1743104518
                  },
                  {
                    "stream": "output",
                    "executionId": "01JQCJ0AAXGX5M9QY10AVF4GK1",
                    "level": "info",
                    "group": "log",
                    "message": "Output: {\"status\":\"success\"}",
                    "timestamp": 1743104521
                  }
                ],
                "finalized": true
              },
              "createdAt": "2025-03-27T19:41:58.493298051Z",
              "updatedAt": "2025-03-27T19:42:02.192033089Z"
            }
          })
        ),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 404, description = "Func run not found"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn get_func_run(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    Path(FuncRunV1RequestPath { func_run_id }): Path<FuncRunV1RequestPath>,
) -> FuncsResult<Json<GetFuncRunV1Response>> {
    let maybe_func_run = ctx.layer_db().func_run().read(func_run_id).await?;
    match maybe_func_run {
        Some(func_run) => {
            let func_run_view = FuncRunViewV1::assemble(ctx, &func_run).await?;

            tracker.track(
                ctx,
                "api_get_func_run",
                json!({
                    "func_run_id": func_run_id
                }),
            );

            Ok(Json(GetFuncRunV1Response {
                func_run: func_run_view,
            }))
        }
        None => Err(FuncsError::FuncRunNotFound(func_run_id)),
    }
}

#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GetFuncRunV1Response {
    pub func_run: FuncRunViewV1,
}
