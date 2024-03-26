use axum::extract::OriginalUri;
use axum::{response::IntoResponse, Json};
use dal::attribute::prototype::argument::{AttributePrototypeArgumentId, AttributePrototypeArgument };
use dal::{AttributePrototype, ChangeSetPointer, Visibility, AttributeValue};
use dal::attribute::prototype::AttributePrototypeError;
use serde::{Deserialize, Serialize};

use super::DiagramResult;
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use crate::service::diagram::DiagramError;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]

pub struct DeleteConnectionRequest {
    pub edge_id: AttributePrototypeArgumentId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

/// Delete a [`Connection`](dal::Connection) via its EdgeId. Creating change-set if on head.
pub async fn delete_connection(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<DeleteConnectionRequest>,
) -> DiagramResult<impl IntoResponse> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let force_changeset_pk = ChangeSetPointer::force_new(&mut ctx).await?;

    let attribute_prototype_argument = AttributePrototypeArgument::get_by_id(&ctx, request.edge_id).await?;
    let targets = attribute_prototype_argument.targets();
    let source = attribute_prototype_argument.value_source(&ctx).await?;
    let prototype_id = attribute_prototype_argument.prototype_id(&ctx).await?;
    let value_id = AttributePrototype::attribute_value_ids(&ctx, prototype_id).await?.first().copied().ok_or(AttributePrototypeError::NoAttributeValues(prototype_id))?;
    let destination = AttributeValue::is_for(&ctx, value_id).await?;

    //attribute_prototype_argument.delete(&ctx).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "delete_connection",
        serde_json::json!({
            "from_component_id": targets.source_component_id,
            "from_component_schema_name": from_component_schema.name(),
            "from_socket_id": conn.source.socket_id,
            "from_socket_name": &from_socket.name(),
            "to_component_id": targets.destination_component_id,
            "to_component_schema_name": to_component_schema.name(),
            "to_socket_id": conn.destination.socket_id,
            "to_socket_name":  &to_socket.name(),
        }),
    );

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    if let Some(force_changeset_pk) = force_changeset_pk {
        response = response.header("force_changeset_pk", force_changeset_pk.to_string());
    }
    Ok(response.body(axum::body::Empty::new())?)
}
