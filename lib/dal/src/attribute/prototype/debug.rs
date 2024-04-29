use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use telemetry::prelude::*;
use thiserror::Error;
use ulid::Ulid;

use crate::attribute::prototype::argument::{
    static_value::StaticArgumentValue,
    value_source::{ValueSource, ValueSourceError},
    AttributePrototypeArgument, AttributePrototypeArgumentError,
};
use crate::attribute::prototype::AttributePrototypeError;
use crate::attribute::value::{AttributeValueError, ValueIsFor};
use crate::func::argument::FuncArgument;
use crate::func::argument::FuncArgumentError;
use crate::func::execution::{FuncExecution, FuncExecutionError};
use crate::prop::PropError;
use crate::socket::input::InputSocketError;
use crate::socket::output::OutputSocketError;
use crate::workspace_snapshot::node_weight::NodeWeightError;
use crate::workspace_snapshot::WorkspaceSnapshotError;
use crate::{
    AttributePrototype, AttributePrototypeId, AttributeValue, AttributeValueId, Component,
    ComponentError, DalContext, Func, FuncError, FuncId,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttributePrototypeDebugView {
    pub func_id: FuncId,
    pub func_execution: Option<FuncExecution>,
    pub id: AttributePrototypeId,
    pub func_name: String,
    pub func_args: HashMap<String, Vec<FuncArgDebugView>>,
    pub attribute_values: Vec<AttributeValueId>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FuncArgDebugView {
    pub value: serde_json::Value,
    pub name: String,
    pub value_source: String,
    pub value_source_id: Ulid,
    pub socket_source_kind: Option<SocketSourceKind>,
    pub path: Option<String>,
    pub is_used: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SocketSourceKind {
    Inferred,
    Manual,
}

type AttributePrototypeDebugViewResult<T> = Result<T, AttributePrototypeDebugViewError>;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum AttributePrototypeDebugViewError {
    #[error("attribute prototype argument Error: {0}")]
    AttributePrototypeArgumentError(#[from] AttributePrototypeArgumentError),
    #[error("attribute prototype error: {0}")]
    AttributePrototypeError(#[from] AttributePrototypeError),
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    #[error("component error: {0}")]
    ComponentError(#[from] ComponentError),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("func argument error: {0}")]
    FuncArgumentError(#[from] FuncArgumentError),
    #[error("func execution error: {0}")]
    FuncExecution(#[from] FuncExecutionError),
    #[error("input socket error: {0}")]
    InputSocketError(#[from] InputSocketError),
    #[error("node weight error: {0}")]
    NodeWeightError(#[from] NodeWeightError),
    #[error("output socket error: {0}")]
    OutputSocketError(#[from] OutputSocketError),
    #[error("prop error: {0}")]
    PropError(#[from] PropError),
    #[error("value source error: {0}")]
    ValueSourceError(#[from] ValueSourceError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshotError(#[from] WorkspaceSnapshotError),
}

impl AttributePrototypeDebugView {
    #[instrument(level = "info", skip_all)]
    pub async fn new(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> AttributePrototypeDebugViewResult<AttributePrototypeDebugView> {
        let prototype_id = AttributeValue::prototype_id(ctx, attribute_value_id).await?;
        let destination_component_id =
            AttributeValue::component_id(ctx, attribute_value_id).await?;
        let mut func_binding_args: HashMap<String, Vec<FuncArgDebugView>> = HashMap::new();
        let attribute_prototype_arg_ids =
            AttributePrototypeArgument::list_ids_for_prototype(ctx, prototype_id).await?;
        info!("attribute prototype ids: {:?}", attribute_prototype_arg_ids);
        for attribute_prototype_arg_id in attribute_prototype_arg_ids {
            let attribute_prototype_argument =
                AttributePrototypeArgument::get_by_id(ctx, attribute_prototype_arg_id).await?;
            info!(
                "Attribute Prototype Argument: {:?}",
                attribute_prototype_argument
            );
            let targets = attribute_prototype_argument.targets();
            info!("targets: {:?}", targets);
            let expected_source_component_id = attribute_prototype_argument
                .targets()
                .map(|targets| targets.source_component_id)
                .unwrap_or(destination_component_id);
            info!("expected source id: {}", expected_source_component_id);
            if attribute_prototype_argument
                .targets()
                .map_or(true, |targets| {
                    targets.destination_component_id == destination_component_id
                })
            {
                let arg_used = Component::should_data_flow_between_components(
                    ctx,
                    destination_component_id,
                    expected_source_component_id,
                )
                .await?;

                let func_arg_id = AttributePrototypeArgument::func_argument_id_by_id(
                    ctx,
                    attribute_prototype_arg_id,
                )
                .await?;
                info!("func arg id: {:?}", func_arg_id);

                let func_arg_name = FuncArgument::get_name_by_id(ctx, func_arg_id).await?;
                let value_source =
                    AttributePrototypeArgument::value_source_by_id(ctx, attribute_prototype_arg_id)
                        .await?
                        .ok_or(
                            AttributeValueError::AttributePrototypeArgumentMissingValueSource(
                                attribute_prototype_arg_id,
                            ),
                        )?;
                let values_for_arg = match value_source {
                    ValueSource::StaticArgumentValue(static_argument_value_id) => {
                        let val = StaticArgumentValue::get_by_id(ctx, static_argument_value_id)
                            .await?
                            .value;
                        vec![FuncArgDebugView {
                            value: val,
                            name: func_arg_name.clone(),
                            value_source: value_source.to_string(),
                            value_source_id: static_argument_value_id.into(),
                            path: None,
                            socket_source_kind: None,
                            is_used: arg_used,
                        }]
                    }

                    ValueSource::Prop(prop_id) => {
                        let mut values = vec![];

                        for attribute_value_id in value_source
                            .attribute_values_for_component_id(ctx, expected_source_component_id)
                            .await?
                        {
                            let attribute_value =
                                AttributeValue::get_by_id(ctx, attribute_value_id).await?;
                            let prop_path =
                                AttributeValue::get_path_for_id(ctx, attribute_value_id).await?;
                            let mat_view = attribute_value
                                .materialized_view(ctx)
                                .await?
                                .unwrap_or(Value::Null);
                            info!("Materialized View: {:?}", mat_view);
                            let func_arg_debug = FuncArgDebugView {
                                value: mat_view,
                                name: func_arg_name.clone(),
                                value_source: value_source.to_string(),
                                value_source_id: prop_id.into(),
                                socket_source_kind: None,
                                path: prop_path,
                                is_used: arg_used,
                            };
                            values.push(func_arg_debug);
                        }

                        values
                    }
                    ValueSource::InputSocket(input_socket_id) => {
                        let mut values = vec![];

                        for attribute_value_id in value_source
                            .attribute_values_for_component_id(ctx, expected_source_component_id)
                            .await?
                        {
                            let attribute_value =
                                AttributeValue::get_by_id(ctx, attribute_value_id).await?;
                            let attribute_value_path =
                                AttributeValue::get_path_for_id(ctx, attribute_value_id).await?;

                            let mat_view = attribute_value
                                .materialized_view(ctx)
                                .await?
                                .unwrap_or(Value::Null);
                            info!("Materialized View: {:?}", mat_view);
                            let func_arg_debug = FuncArgDebugView {
                                value: mat_view,
                                name: func_arg_name.clone(),
                                value_source: value_source.to_string(),
                                value_source_id: input_socket_id.into(),
                                socket_source_kind: Some(SocketSourceKind::Manual),
                                path: attribute_value_path,
                                is_used: arg_used,
                            };
                            values.push(func_arg_debug);
                        }

                        values
                    }
                    ValueSource::OutputSocket(output_socket_id) => {
                        let mut values = vec![];

                        for attribute_value_id in value_source
                            .attribute_values_for_component_id(ctx, expected_source_component_id)
                            .await?
                        {
                            let attribute_value =
                                AttributeValue::get_by_id(ctx, attribute_value_id).await?;
                            let attribute_value_path =
                                AttributeValue::get_path_for_id(ctx, attribute_value_id).await?;

                            let mat_view = attribute_value
                                .materialized_view(ctx)
                                .await?
                                .unwrap_or(Value::Null);
                            info!("Materialized View: {:?}", mat_view);
                            let func_arg_debug = FuncArgDebugView {
                                value: mat_view,
                                name: func_arg_name.clone(),
                                value_source: value_source.to_string(),
                                value_source_id: output_socket_id.into(),
                                socket_source_kind: Some(SocketSourceKind::Manual),
                                path: attribute_value_path,
                                is_used: arg_used,
                            };
                            values.push(func_arg_debug);
                        }

                        values
                    }
                };

                func_binding_args
                    .entry(func_arg_name)
                    .and_modify(|values| values.extend(values_for_arg.clone()))
                    .or_insert(values_for_arg);
            }
        }
        // if this attribute value is for an input socket, need to also get any inferred inputs if they exist!
        if let ValueIsFor::InputSocket(input_socket_id) =
            AttributeValue::is_for(ctx, attribute_value_id).await?
        {
            info!("value is for input socket!");
            if let Some(input_socket_match) =
                Component::input_socket_match(ctx, destination_component_id, input_socket_id)
                    .await?
            {
                info!("Input socket match: {:?}", input_socket_match);
                // now get inferred func binding args and values!
                if let Some(output_match) =
                    Component::find_potential_inferred_connection_to_input_socket(
                        ctx,
                        input_socket_match,
                    )
                    .await?
                {
                    info!("output socket match: {:?}", output_match);
                    let arg_used = Component::should_data_flow_between_components(
                        ctx,
                        input_socket_match.component_id,
                        output_match.component_id,
                    )
                    .await?;
                    let attribute_value_path =
                        AttributeValue::get_path_for_id(ctx, attribute_value_id).await?;
                    let output_av =
                        AttributeValue::get_by_id(ctx, output_match.attribute_value_id).await?;
                    let mat_view = output_av
                        .materialized_view(ctx)
                        .await?
                        .unwrap_or(Value::Null);
                    let input_func = AttributePrototype::func_id(ctx, prototype_id).await?;
                    if let Some(func_argument) =
                        FuncArgument::list_for_func(ctx, input_func).await?.pop()
                    {
                        let func_arg_name = func_argument.name.clone();
                        let func_arg_debug = FuncArgDebugView {
                            value: mat_view,
                            name: func_argument.name,
                            value_source: "Output Socket".to_string(),
                            value_source_id: output_match.output_socket_id.into(),
                            socket_source_kind: Some(SocketSourceKind::Inferred),
                            path: attribute_value_path,
                            is_used: arg_used,
                        };
                        func_binding_args
                            .entry(func_arg_name)
                            .and_modify(|values| values.push(func_arg_debug.clone()))
                            .or_insert(vec![func_arg_debug]);
                    }
                }
            }
        }

        let attribute_values = AttributePrototype::attribute_value_ids(ctx, prototype_id).await?;
        let func_id = AttributePrototype::func_id(ctx, prototype_id).await?;

        let func_execution =
            Some(FuncExecution::get_latest_execution_by_func_id(ctx, &func_id).await?);
        let func_name = Func::get_by_id_or_error(ctx, func_id).await?.name;
        let view = AttributePrototypeDebugView {
            func_args: func_binding_args,
            func_id,
            func_name,
            func_execution,
            id: prototype_id,
            attribute_values,
        };
        info!("AttributePrototype Debug View: {:?}", view);
        Ok(view)
    }
}
