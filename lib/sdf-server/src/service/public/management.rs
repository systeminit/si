use axum::{
    Json,
    Router,
    extract::Path,
    http::StatusCode,
    response::{
        IntoResponse,
        Response,
    },
    routing::post,
};
use dal::{
    ComponentId,
    Func,
    FuncError,
    WsEvent,
    diagram::view::ViewId,
    management::{
        ManagementFuncReturn,
        ManagementOperator,
        prototype::{
            ManagementPrototype,
            ManagementPrototypeId,
        },
    },
    schema::variant::authoring::VariantAuthoringError,
};
use sdf_core::api_error::ApiError;
use serde::{
    Deserialize,
    Serialize,
};
use si_events::audit_log::AuditLogKind;
use thiserror::Error;
use veritech_client::ManagementFuncStatus;

use crate::{
    AppState,
    extract::{
        PosthogEventTracker,
        change_set::ChangeSetDalContext,
    },
};

// /api/public/workspaces/:workspace_id/change-sets/:change_set_id/components
pub fn routes() -> Router<AppState> {
    Router::new().nest(
        "/prototype/:management_prototype_id",
        Router::new().route("/:component_id/:view_id", post(run_prototype)),
    )
}

async fn run_prototype(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    Path(RunPrototypePath {
        management_prototype_id: prototype_id,
        component_id,
        view_id,
    }): Path<RunPrototypePath>,
    Json(request): Json<RunPrototypeRequest>,
) -> Result<Json<RunPrototypeResponse>> {
    let request_ulid = request.request_ulid.unwrap_or_default();

    // TODO check that this is a valid prototypeId
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
                    request_ulid,
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

        return Ok(Json(RunPrototypeResponse { status, message }));
    }

    Err(ManagementApiError::ManagementPrototypeExecutionFailure(
        prototype_id,
    ))
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunPrototypeRequest {
    request_ulid: Option<ulid::Ulid>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunPrototypeResponse {
    status: ManagementFuncStatus,
    message: Option<String>,
}

#[derive(Deserialize)]
struct RunPrototypePath {
    management_prototype_id: ManagementPrototypeId,
    component_id: ComponentId,
    view_id: ViewId,
}

#[remain::sorted]
#[derive(Debug, Error)]
enum ManagementApiError {
    #[error("change set error: {0}")]
    ChangeSet(#[from] dal::ChangeSetError),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("func api error: {0}")]
    FuncAPI(#[from] crate::service::v2::func::FuncAPIError),
    #[error("func authoring error: {0}")]
    FuncAuthoring(#[from] dal::func::authoring::FuncAuthoringError),
    #[error("hyper error: {0}")]
    Http(#[from] axum::http::Error),
    #[error("layer db error: {0}")]
    LayerDb(#[from] si_layer_cache::LayerDbError),
    #[error("management error: {0}")]
    Management(#[from] dal::management::ManagementError),
    #[error("management prototype error: {0}")]
    ManagementPrototype(#[from] dal::management::prototype::ManagementPrototypeError),
    #[error("management prototype execution failure: {0}")]
    ManagementPrototypeExecutionFailure(ManagementPrototypeId),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] dal::SchemaVariantError),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("transactions error: {0}")]
    Transactions(#[from] dal::TransactionsError),
    #[error("variant authoring error: {0}")]
    VariantAuthoring(#[from] VariantAuthoringError),
    #[error("ws event error: {0}")]
    WsEvent(#[from] dal::WsEventError),
}

type Result<T> = std::result::Result<T, ManagementApiError>;

impl IntoResponse for ManagementApiError {
    fn into_response(self) -> Response {
        let (status_code, error_message) = match self {
            ManagementApiError::ManagementPrototype(
                dal::management::prototype::ManagementPrototypeError::FuncExecutionFailure(message),
            ) => (StatusCode::BAD_REQUEST, message),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };
        ApiError::new(status_code, error_message).into_response()
    }
}
