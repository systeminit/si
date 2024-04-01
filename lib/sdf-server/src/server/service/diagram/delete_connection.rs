use axum::extract::OriginalUri;
use axum::{response::IntoResponse, Json};
use dal::attribute::prototype::argument::value_source::ValueSource;
use dal::attribute::prototype::argument::{
    AttributePrototypeArgument, AttributePrototypeArgumentError, AttributePrototypeArgumentId,
};
use dal::attribute::prototype::AttributePrototypeError;
use dal::attribute::value::ValueIsFor;
use dal::workspace_snapshot::node_weight::NodeWeightDiscriminants;
use dal::{
    AttributePrototype, AttributeValue, ChangeSet, Component, InputSocket, OutputSocket, Visibility,
};
use serde::{Deserialize, Serialize};

use super::DiagramResult;
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;

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

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let attribute_prototype_argument =
        AttributePrototypeArgument::get_by_id(&ctx, request.edge_id).await?;
    let targets = attribute_prototype_argument.targets().ok_or(
        AttributePrototypeArgumentError::NoTargets(attribute_prototype_argument.id()),
    )?;
    let output_socket_id = match attribute_prototype_argument
        .value_source(&ctx)
        .await?
        .ok_or(AttributePrototypeArgumentError::MissingSource(
            attribute_prototype_argument.id(),
        ))? {
        ValueSource::OutputSocket(source_id) => source_id,

        // Any other source should be considered an error, as connections on the diagram
        // are only from output sockets to input sockets.
        ValueSource::Prop(_) => {
            return Err(AttributePrototypeArgumentError::UnexpectedValueSourceNode(
                attribute_prototype_argument.id(),
                NodeWeightDiscriminants::Prop,
            )
            .into());
        }
        ValueSource::InputSocket(_) | ValueSource::StaticArgumentValue(_) => {
            return Err(AttributePrototypeArgumentError::UnexpectedValueSourceNode(
                attribute_prototype_argument.id(),
                NodeWeightDiscriminants::Content,
            )
            .into());
        }
    };
    let prototype_id = attribute_prototype_argument.prototype_id(&ctx).await?;
    let value_id = AttributePrototype::attribute_value_ids(&ctx, prototype_id)
        .await?
        .first()
        .copied()
        .ok_or(AttributePrototypeError::NoAttributeValues(prototype_id))?;
    let input_socket_id = match AttributeValue::is_for(&ctx, value_id).await? {
        ValueIsFor::InputSocket(socket_id) => socket_id,

        // Any other destination should be considered an error, as connections on the diagram
        // are only from output sockets to input sockets.
        ValueIsFor::Prop(_) => {
            return Err(AttributePrototypeArgumentError::UnexpectedValueSourceNode(
                attribute_prototype_argument.id(),
                NodeWeightDiscriminants::Prop,
            )
            .into());
        }
        ValueIsFor::OutputSocket(_) => {
            return Err(AttributePrototypeArgumentError::UnexpectedValueSourceNode(
                attribute_prototype_argument.id(),
                NodeWeightDiscriminants::Content,
            )
            .into());
        }
    };

    let from_component_schema = Component::get_by_id(&ctx, targets.source_component_id)
        .await?
        .schema(&ctx)
        .await?;
    let to_component_schema = Component::get_by_id(&ctx, targets.destination_component_id)
        .await?
        .schema(&ctx)
        .await?;
    let output_socket = OutputSocket::get_by_id(&ctx, output_socket_id).await?;
    let input_socket = InputSocket::get_by_id(&ctx, input_socket_id).await?;

    AttributePrototypeArgument::remove(&ctx, attribute_prototype_argument.id()).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "delete_connection",
        serde_json::json!({
            "from_component_id": targets.source_component_id,
            "from_component_schema_name": from_component_schema.name(),
            "from_socket_id": output_socket_id,
            "from_socket_name": &output_socket.name(),
            "to_component_id": targets.destination_component_id,
            "to_component_schema_name": to_component_schema.name(),
            "to_socket_id": input_socket_id,
            "to_socket_name":  &input_socket.name(),
        }),
    );

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    if let Some(force_change_set_id) = force_change_set_id {
        response = response.header("force_change_set_id", force_change_set_id.to_string());
    }
    Ok(response.body(axum::body::Empty::new())?)
}
