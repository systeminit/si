use anyhow::Result;
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
    component::socket::{ComponentInputSocket, ComponentOutputSocket},
    AttributePrototype, AttributePrototypeId, AttributeValue, AttributeValueId, ComponentError,
    DalContext, FuncId, InputSocket, OutputSocket,
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
type SocketDebugViewResult<T> = Result<T>;

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
    #[instrument(level = "trace", skip_all)]
    pub async fn new_for_output_socket(
        ctx: &DalContext,
        component_output_socket: ComponentOutputSocket,
    ) -> SocketDebugViewResult<SocketDebugView> {
        let prototype_id = AttributePrototype::find_for_output_socket(
            ctx,
            component_output_socket.output_socket_id,
        )
        .await?;

        let attribute_value_id = component_output_socket.attribute_value_id;

        let prototype_debug_view =
            AttributePrototypeDebugView::new(ctx, attribute_value_id).await?;
        let attribute_value = AttributeValue::get_by_id(ctx, attribute_value_id).await?;
        let output_socket =
            OutputSocket::get_by_id(ctx, component_output_socket.output_socket_id).await?;
        let connection_annotations = output_socket
            .connection_annotations()
            .into_iter()
            .map(|f| f.to_string())
            .collect();
        let path =
            (AttributeValue::get_path_for_id(ctx, attribute_value_id).await?).unwrap_or_default();

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
            socket_id: component_output_socket.output_socket_id.into(),
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
        component_input_socket: ComponentInputSocket,
    ) -> SocketDebugViewResult<SocketDebugView> {
        let prototype_id =
            AttributePrototype::find_for_input_socket(ctx, component_input_socket.input_socket_id)
                .await?;
        let attribute_value_id = component_input_socket.attribute_value_id;
        let prototype_debug_view =
            AttributePrototypeDebugView::new(ctx, component_input_socket.attribute_value_id)
                .await?;
        let attribute_value = AttributeValue::get_by_id(ctx, attribute_value_id).await?;
        let input_socket =
            InputSocket::get_by_id(ctx, component_input_socket.input_socket_id).await?;
        let connection_annotations = input_socket
            .connection_annotations()
            .into_iter()
            .map(|f| f.to_string())
            .collect();
        let path =
            (AttributeValue::get_path_for_id(ctx, attribute_value_id).await?).unwrap_or_default();
        let value_view = attribute_value.view(ctx).await?;
        let inferred_connections =
            ComponentInputSocket::find_inferred_connections(ctx, component_input_socket)
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
            socket_id: component_input_socket.input_socket_id.into(),
            func_id: prototype_debug_view.func_id,
            connection_annotations,
            value: attribute_value.unprocessed_value(ctx).await?,
            path,
            view: value_view,
            name: input_socket.name().to_string(),
            inferred_connections,
        };
        Ok(view)
    }
}
