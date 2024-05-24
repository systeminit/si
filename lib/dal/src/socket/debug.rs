use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use telemetry::prelude::*;
use thiserror::Error;
use ulid::Ulid;

use super::{input::InputSocketError, output::OutputSocketError};
use crate::{
    attribute::{
        prototype::{
            debug::{
                AttributePrototypeDebugView, AttributePrototypeDebugViewError, FuncArgDebugView,
            },
            AttributePrototypeError,
        },
        value::AttributeValueError,
    },
    component::{InputSocketMatch, OutputSocketMatch},
    AttributePrototype, AttributePrototypeId, AttributeValue, AttributeValueId, Component,
    ComponentError, DalContext, FuncId, InputSocket, OutputSocket,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SocketDebugView {
    pub path: String,
    pub socket_id: Ulid,
    pub attribute_value_id: AttributeValueId,
    pub func_id: FuncId,
    pub prototype_id: Option<AttributePrototypeId>,
    pub prototype_is_component_specific: bool,
    pub connection_annotations: Vec<String>,
    pub func_name: String,
    pub func_args: HashMap<String, Vec<FuncArgDebugView>>,
    pub value: Option<serde_json::Value>,
    pub view: Option<serde_json::Value>,
    pub name: String,
    pub inferred_connections: Vec<Ulid>,
}
type SocketDebugViewResult<T> = Result<T, SocketDebugViewError>;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum SocketDebugViewError {
    #[error("attribute prototype debug view error: {0}")]
    AttributePrototypeDebugViewError(#[from] AttributePrototypeDebugViewError),
    #[error("attribute prototype error: {0}")]
    AttributePrototypeError(#[from] AttributePrototypeError),
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    #[error("component error: {0}")]
    ComponentError(#[from] ComponentError),
    #[error("input socket error: {0}")]
    InputSocketError(#[from] InputSocketError),
    #[error("output socket error: {0}")]
    OutputSocketError(#[from] OutputSocketError),
}

impl SocketDebugView {
    #[instrument(level = "info", skip_all)]
    pub async fn new_for_output_socket(
        ctx: &DalContext,
        output_socket_match: OutputSocketMatch,
    ) -> SocketDebugViewResult<SocketDebugView> {
        let prototype_id =
            AttributePrototype::find_for_output_socket(ctx, output_socket_match.output_socket_id)
                .await?;

        let attribute_value_id = output_socket_match.attribute_value_id;

        let prototype_debug_view =
            AttributePrototypeDebugView::new(ctx, attribute_value_id).await?;
        let attribute_value = AttributeValue::get_by_id(ctx, attribute_value_id).await?;
        let output_socket =
            OutputSocket::get_by_id(ctx, output_socket_match.output_socket_id).await?;
        let connection_annotations = output_socket
            .connection_annotations()
            .into_iter()
            .map(|f| f.to_string())
            .collect();
        let path = match AttributeValue::get_path_for_id(ctx, attribute_value_id).await? {
            Some(path) => path,
            None => String::new(),
        };

        let view = attribute_value.view(ctx).await?;
        let inferred_connections: Vec<Ulid> =
            AttributeValue::list_input_socket_sources_for_id(ctx, attribute_value_id)
                .await?
                .into_iter()
                .map(Ulid::from)
                .collect();
        Ok(SocketDebugView {
            prototype_id,
            prototype_is_component_specific: prototype_debug_view.is_component_specific,
            func_name: prototype_debug_view.func_name,
            func_args: prototype_debug_view.func_args,
            attribute_value_id,
            socket_id: output_socket_match.output_socket_id.into(),
            func_id: prototype_debug_view.func_id,
            connection_annotations,
            value: attribute_value.unprocessed_value(ctx).await?,
            path,
            view,
            name: output_socket.name().to_string(),
            inferred_connections,
        })
    }

    #[instrument(level = "info", skip_all)]
    pub async fn new_for_input_socket(
        ctx: &DalContext,
        input_socket_match: InputSocketMatch,
    ) -> SocketDebugViewResult<SocketDebugView> {
        let prototype_id =
            AttributePrototype::find_for_input_socket(ctx, input_socket_match.input_socket_id)
                .await?;
        let attribute_value_id = input_socket_match.attribute_value_id;
        let prototype_debug_view =
            AttributePrototypeDebugView::new(ctx, input_socket_match.attribute_value_id).await?;
        info!("prototype_debug_view: {:?}", prototype_debug_view);
        let attribute_value = AttributeValue::get_by_id(ctx, attribute_value_id).await?;
        let input_socket = InputSocket::get_by_id(ctx, input_socket_match.input_socket_id).await?;
        let connection_annotations = input_socket
            .connection_annotations()
            .into_iter()
            .map(|f| f.to_string())
            .collect();
        let path = match AttributeValue::get_path_for_id(ctx, attribute_value_id).await? {
            Some(path) => path,
            None => String::new(),
        };
        let value_view = attribute_value.view(ctx).await?;
        let inferred_connections =
            Component::find_available_inferred_connections_to_input_socket(ctx, input_socket_match)
                .await?
                .into_iter()
                .map(|output_socket| Ulid::from(output_socket.attribute_value_id))
                .collect();
        let view = SocketDebugView {
            prototype_id,
            prototype_is_component_specific: prototype_debug_view.is_component_specific,
            func_name: prototype_debug_view.func_name,
            func_args: prototype_debug_view.func_args,
            attribute_value_id,
            socket_id: input_socket_match.input_socket_id.into(),
            func_id: prototype_debug_view.func_id,
            connection_annotations,
            value: attribute_value.unprocessed_value(ctx).await?,
            path,
            view: value_view,
            name: input_socket.name().to_string(),
            inferred_connections,
        };
        info!("Socket Debug View: {:?}", view);
        Ok(view)
    }
}
