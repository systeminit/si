use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use telemetry::prelude::*;
use thiserror::Error;

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
use crate::prop::{PropError, PropPath};
use crate::socket::input::InputSocketError;
use crate::socket::output::OutputSocketError;
use crate::workspace_snapshot::node_weight::NodeWeightError;
use crate::workspace_snapshot::WorkspaceSnapshotError;
use crate::{
    AttributePrototype, AttributePrototypeId, AttributeValue, AttributeValueId, Component,
    ComponentError, DalContext, Func, FuncError, FuncId, InputSocket, Prop,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttributePrototypeDebugView {
    pub func_id: FuncId,
    pub func_execution: Option<FuncExecution>,
    pub id: AttributePrototypeId,
    pub func_name: String,
    pub func_args: HashMap<String, Vec<serde_json::Value>>,
    pub arg_sources: HashMap<String, Option<String>>,
    pub attribute_values: Vec<AttributeValueId>,
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
    #[instrument(level = "debug", skip_all)]
    pub async fn assemble(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> AttributePrototypeDebugViewResult<AttributePrototypeDebugView> {
        let prototype_id = AttributeValue::prototype_id(ctx, attribute_value_id).await?;

        let destination_component_id =
            AttributeValue::component_id(ctx, attribute_value_id).await?;

        let mut func_binding_args: HashMap<String, Vec<serde_json::Value>> = HashMap::new();
        let mut arg_sources: HashMap<String, Option<String>> = HashMap::new();

        let attribute_prototype_ids =
            AttributePrototypeArgument::list_ids_for_prototype(ctx, prototype_id).await?;
        for attribute_prototype_id in attribute_prototype_ids {
            let attribute_prototype_argument =
                AttributePrototypeArgument::get_by_id(ctx, attribute_prototype_id).await?;
            let expected_source_component_id = attribute_prototype_argument
                .targets()
                .map(|targets| targets.source_component_id)
                .unwrap_or(destination_component_id);

            if attribute_prototype_argument
                .targets()
                .map_or(true, |targets| {
                    targets.destination_component_id == destination_component_id
                })
            {
                // If the "source" Component is marked for deletion, and we (the destination) are
                // *NOT*, then we should ignore the argument as data should not flow from things
                // that are marked for deletion to ones that are not.
                let destination_component = Component::get_by_id(ctx, destination_component_id)
                    .await
                    .map_err(|e| AttributeValueError::Component(Box::new(e)))?;

                let source_component = Component::get_by_id(ctx, expected_source_component_id)
                    .await
                    .map_err(|e| AttributeValueError::Component(Box::new(e)))?;

                if source_component.to_delete() && !destination_component.to_delete() {
                    continue;
                }

                let func_arg_id =
                    AttributePrototypeArgument::func_argument_id_by_id(ctx, attribute_prototype_id)
                        .await?;
                let func_arg_name = FuncArgument::get_name_by_id(ctx, func_arg_id).await?;

                let values_for_arg = match AttributePrototypeArgument::value_source_by_id(
                    ctx,
                    attribute_prototype_id,
                )
                .await?
                .ok_or(
                    AttributeValueError::AttributePrototypeArgumentMissingValueSource(
                        attribute_prototype_id,
                    ),
                )? {
                    ValueSource::StaticArgumentValue(static_argument_value_id) => {
                        let val = StaticArgumentValue::get_by_id(ctx, static_argument_value_id)
                            .await?
                            .value;
                        arg_sources.insert(
                            func_arg_name.clone(),
                            Some(PropPath::new(["Static Value"]).to_string()),
                        );
                        vec![val]
                    }
                    other_source => {
                        let mut values = vec![];

                        for attribute_value_id in other_source
                            .attribute_values_for_component_id(ctx, expected_source_component_id)
                            .await?
                        {
                            let attribute_value =
                                AttributeValue::get_by_id(ctx, attribute_value_id).await?;

                            let attribute_value_name =
                                AttributeValue::is_for(ctx, attribute_value_id).await?;
                            let prop_path: PropPath = match attribute_value_name {
                                ValueIsFor::InputSocket(id) => {
                                    let inputsock = InputSocket::get_by_id(ctx, id).await?;
                                    PropPath::new(["Input Socket", inputsock.name()])
                                }
                                ValueIsFor::Prop(_) => {
                                    let prop_id =
                                        AttributeValue::prop_id_for_id(ctx, attribute_value.id())
                                            .await?;
                                    Prop::path_by_id(ctx, prop_id).await?
                                }
                                ValueIsFor::OutputSocket(_) => continue,
                            };
                            arg_sources.insert(
                                func_arg_name.clone(),
                                Some(prop_path.with_replaced_sep("/")),
                            );
                            values.push(
                                attribute_value
                                    .materialized_view(ctx)
                                    .await?
                                    .unwrap_or(Value::Null),
                            );
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
        let attribute_values = AttributePrototype::attribute_value_ids(ctx, prototype_id).await?;
        let func_id = AttributePrototype::func_id(ctx, prototype_id).await?;

        let func_execution =
            Some(FuncExecution::get_latest_execution_by_func_id(ctx, &func_id).await?);
        let func_name = Func::get_by_id(ctx, func_id).await?.name;

        Ok(AttributePrototypeDebugView {
            func_args: func_binding_args,
            func_id,
            func_name,
            func_execution,
            arg_sources,
            id: prototype_id,
            attribute_values,
        })
    }
}
