use axum::extract::Query;
use axum::Json;
use dal::{ComponentId, Schema, StandardModel, Visibility, WorkspaceId, WriteTenancy};
use serde::{Deserialize, Serialize};

use crate::server::extract::{AccessBuilder, HandlerContext};

use super::{ApplicationError, ApplicationResult};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListApplicationRequest {
    pub workspace_id: WorkspaceId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationView {
    id: ComponentId,
    name: String,
    visibility: Visibility,
    tenancy: WriteTenancy,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListApplicationItem {
    pub application: ApplicationView,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListApplicationResponse {
    pub list: Vec<ListApplicationItem>,
}

pub async fn list_applications(
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<ListApplicationRequest>,
) -> ApplicationResult<Json<ListApplicationResponse>> {
    let txns = txns.start().await?;
    let ctx = builder.build(request_ctx.build(request.visibility), &txns);

    let schemas = Schema::find_by_attr(&ctx, "name", &"application".to_string()).await?;
    let schema = schemas.first().ok_or(ApplicationError::SchemaNotFound)?;
    let mut list = Vec::new();
    for application in schema.components(&ctx).await? {
        let name = application
            .find_value_by_json_pointer::<String>(&ctx, "/root/si/name")
            .await?
            .ok_or(ApplicationError::NameNotFound)?;
        list.push(ListApplicationItem {
            application: ApplicationView {
                id: *application.id(),
                name,
                visibility: *application.visibility(),
                tenancy: application.tenancy().clone(),
            },
        })
    }
    let response = ListApplicationResponse { list };

    txns.commit().await?;

    Ok(Json(response))
}
