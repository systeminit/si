use axum::extract::Query;
use axum::Json;
use dal::{Component, ComponentId, StandardModel, SystemId, Visibility, WorkspaceId};
use serde::{Deserialize, Serialize};

use crate::server::extract::{AccessBuilder, HandlerContext};
use crate::service::resource::{ResourceError, ResourceResult};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListResourcesByComponentRequest {
    pub component_id: ComponentId,
    pub system_id: Option<SystemId>,
    pub workspace_id: WorkspaceId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListResourcesByComponentResponse {
    value: serde_json::Value,
}

pub async fn list_resources_by_component(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<ListResourcesByComponentRequest>,
) -> ResourceResult<Json<ListResourcesByComponentResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let component = Component::get_by_id(&ctx, &request.component_id)
        .await?
        .ok_or(ResourceError::ComponentNotFound(request.component_id))?;

    let default_case = serde_json::json![{
      "foo": {
        "bar": "baz"
      }
    }];

    let value = match component.schema_variant(&ctx).await? {
        Some(sv) => match sv.name() {
            "docker_image" => serde_json::json![{
              "foo": {
                "bar": "baz"
              }
            }],
            _ => default_case,
        },
        None => default_case,
    };

    Ok(Json(ListResourcesByComponentResponse { value }))
}
