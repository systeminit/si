use axum::extract::OriginalUri;
use axum::{extract::Query, Json};
use dal::{AttributeValue, OutputSocket, OutputSocketId, Prop, PropId, Visibility};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::AttributeResult;
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use crate::service::attribute::AttributeError;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetPrototypeArgumentsRequest {
    pub prop_id: Option<PropId>,
    pub output_socket_id: Option<OutputSocketId>,
    #[serde(flatten)]
    pub visibility: Visibility,
}
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetPrototypeArgumentsResponse {
    pub prepared_arguments: Value,
}

pub async fn get_prototype_arguments(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Query(request): Query<GetPrototypeArgumentsRequest>,
) -> AttributeResult<Json<GetPrototypeArgumentsResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    // Find the attribute values for the provided output location corresponding to the attribute
    // prototype. There should only be one.
    let attribute_value_id = match (request.prop_id, request.output_socket_id) {
        (Some(prop_id), None) => {
            let attribute_value_ids = Prop::attribute_values_for_prop_id(&ctx, prop_id).await?;
            if attribute_value_ids.len() > 1 {
                return Err(AttributeError::MultipleAttributeValuesForProp(
                    attribute_value_ids.to_owned(),
                    prop_id,
                ));
            }
            *attribute_value_ids
                .first()
                .ok_or(AttributeError::NoAttributeValuesFoundForProp(prop_id))?
        }
        (None, Some(output_socket_id)) => {
            let attribute_value_ids =
                OutputSocket::attribute_values_for_output_socket_id(&ctx, output_socket_id).await?;
            if attribute_value_ids.len() > 1 {
                return Err(AttributeError::MultipleAttributeValuesForOutputSocket(
                    attribute_value_ids.to_owned(),
                    output_socket_id,
                ));
            }
            *attribute_value_ids.first().ok_or(
                AttributeError::NoAttributeValuesFoundForOutputSocket(output_socket_id),
            )?
        }
        (None, None) => return Err(AttributeError::NoOutputLocationsProvided),
        (Some(prop_id), Some(ouput_socket_id)) => {
            return Err(AttributeError::MultipleOutputLocationsProvided(
                prop_id,
                ouput_socket_id,
            ))
        }
    };

    let (_, prepared_arguments) =
        AttributeValue::prepare_arguments_for_prototype_function_execution(
            &ctx,
            attribute_value_id,
        )
        .await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "get_prototype_arguments",
        serde_json::json!({
            "how": "/attribute/get_prototype_arguments",
            "prop_id": request.prop_id,
            "output_socket_id": request.output_socket_id,
        }),
    );

    Ok(Json(GetPrototypeArgumentsResponse { prepared_arguments }))
}
