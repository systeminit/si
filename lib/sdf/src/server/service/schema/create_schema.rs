use super::SchemaResult;
use crate::server::extract::{Authorization, HandlerContext, HistoryActor, Tenancy};
use axum::Json;
use dal::{component::ComponentKind, Schema, SchemaKind, Visibility, WriteTenancy, WsEvent};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateSchemaRequest {
    pub name: String,
    pub kind: SchemaKind,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateSchemaResponse {
    pub schema: Schema,
}

pub async fn create_schema(
    HandlerContext(builder): HandlerContext,
    Authorization(claim): Authorization,
    Tenancy(_write_tenancy, read_tenancy): Tenancy,
    HistoryActor(history_actor): HistoryActor,
    Json(request): Json<CreateSchemaRequest>,
) -> SchemaResult<Json<CreateSchemaResponse>> {
    let ctx = builder
        .build(
            dal::context::AccessBuilder::new(
                read_tenancy,
                WriteTenancy::new_billing_account(claim.billing_account_id),
                history_actor,
            )
            .build(request.visibility),
        )
        .await?;

    let schema = Schema::new(&ctx, &request.name, &request.kind, &ComponentKind::Standard).await?;
    let response = CreateSchemaResponse { schema };

    WsEvent::change_set_written(&ctx).publish(&ctx).await?;

    ctx.commit().await?;

    Ok(Json(response))
}
