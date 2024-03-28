use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use thiserror::Error;

use crate::attribute::prototype::{
    argument::{
        static_value::StaticArgumentValue,
        value_source::{ValueSource, ValueSourceError},
        AttributePrototypeArgument, AttributePrototypeArgumentError,
    },
    AttributePrototypeError,
};
use crate::attribute::value::{AttributeValueError, ValueIsFor};

use crate::prop::{PropError, PropPath};

use crate::func::execution::{FuncExecution, FuncExecutionError};
use crate::socket::input::InputSocketError;
use crate::socket::output::OutputSocketError;
use crate::workspace_snapshot::node_weight::NodeWeightError;
use crate::workspace_snapshot::WorkspaceSnapshotError;
use crate::{
    AttributePrototype, AttributePrototypeId, AttributeValue, AttributeValueId, Component,
    ComponentError, DalContext, Func, FuncError, FuncId, InputSocket, Prop,
};
use serde_json::Value;
use telemetry::prelude::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttributePrototypeDebugView {
    pub func_id: FuncId,
    pub func_execution: Option<FuncExecution>,
    pub prototype_id: AttributePrototypeId,
    pub func_name: String,
    pub func_args: HashMap<String, Vec<serde_json::Value>>,
    pub arg_sources: HashMap<String, Option<String>>,
    pub all_attribute_values: Vec<AttributeValueId>,
}

type AttributePrototypeDebugViewResult<T> = Result<T, AttributePrototypeDebugViewError>;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum AttributePrototypeDebugViewError {
    #[error(transparent)]
    AttributePrototypeArgumentError(#[from] AttributePrototypeArgumentError),
    #[error(transparent)]
    AttributePrototypeError(#[from] AttributePrototypeError),
    #[error(transparent)]
    AttributeValue(#[from] AttributeValueError),
    #[error(transparent)]
    ComponentError(#[from] ComponentError),
    #[error(transparent)]
    Func(#[from] FuncError),
    #[error(transparent)]
    FuncExecution(#[from] FuncExecutionError),
    #[error(transparent)]
    InputSocketError(#[from] InputSocketError),
    #[error(transparent)]
    NodeWeightError(#[from] NodeWeightError),
    #[error(transparent)]
    OutputSocketError(#[from] OutputSocketError),
    #[error(transparent)]
    PropError(#[from] PropError),
    #[error(transparent)]
    ValueSourceError(#[from] ValueSourceError),
    #[error(transparent)]
    WorkspaceSnapshotError(#[from] WorkspaceSnapshotError),
}
impl AttributePrototypeDebugView {
    #[instrument(level = "debug", skip_all)]
    pub async fn new_from_attribute_value(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> AttributePrototypeDebugViewResult<AttributePrototypeDebugView> {
        let prototype_id = AttributeValue::prototype_id(ctx, attribute_value_id).await?;
        let destination_component_id =
            AttributeValue::component_id(ctx, attribute_value_id).await?;
        let apa_ids = AttributePrototypeArgument::list_ids_for_prototype(ctx, prototype_id).await?;
        let func_id = AttributePrototype::func_id(ctx, prototype_id).await?;
        let all_attribute_values =
            AttributePrototype::attribute_value_ids(ctx, prototype_id).await?;
        let mut func_binding_args: HashMap<String, Vec<serde_json::Value>> = HashMap::new();
        let mut arg_sources: HashMap<String, Option<String>> = HashMap::new();

        let func_name = Func::get_by_id(ctx, func_id).await?.name;

        let func_execution =
            Some(FuncExecution::get_latest_execution_by_func_id(ctx, &func_id).await?);
        for apa_id in apa_ids {
            let apa = AttributePrototypeArgument::get_by_id(ctx, apa_id).await?;
            let expected_source_component_id = apa
                .targets()
                .map(|targets| targets.source_component_id)
                .unwrap_or(destination_component_id);

            if apa.targets().map_or(true, |targets| {
                targets.destination_component_id == destination_component_id
            }) {
                // If the "source" Component is marked for deletion, and we (the destination) are
                // *NOT*, then we should ignore the argument as data should not flow from things
                // that are marked for deletion to ones that are not.
                let destination_component = Component::get_by_id(ctx, destination_component_id)
                    .await
                    .map_err(|e| AttributeValueError::Component(e.to_string()))?;

                let source_component = Component::get_by_id(ctx, expected_source_component_id)
                    .await
                    .map_err(|e| AttributeValueError::Component(e.to_string()))?;

                if source_component.to_delete() && !destination_component.to_delete() {
                    continue;
                }

                let func_arg_id =
                    AttributePrototypeArgument::func_argument_id_by_id(ctx, apa_id).await?;
                let func_arg_name = ctx
                    .workspace_snapshot()?
                    .get_node_weight_by_id(func_arg_id)
                    .await?
                    .get_func_argument_node_weight()?
                    .name()
                    .to_owned();

                let values_for_arg =
                    match AttributePrototypeArgument::value_source_by_id(ctx, apa_id)
                        .await?
                        .ok_or(
                            AttributeValueError::AttributePrototypeArgumentMissingValueSource(
                                apa_id,
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

                            for av_id in other_source
                                .attribute_values_for_component_id(
                                    ctx,
                                    expected_source_component_id,
                                )
                                .await?
                            {
                                let attribute_value = AttributeValue::get_by_id(ctx, av_id).await?;

                                // no prop here, means it's from an input socket!
                                let av_name = AttributeValue::is_for(ctx, av_id).await?;
                                let prop_path: PropPath = match av_name {
                                    ValueIsFor::InputSocket(id) => {
                                        let inputsock = InputSocket::get_by_id(ctx, id).await?;
                                        PropPath::new(["Input Socket", inputsock.name()])
                                    }
                                    ValueIsFor::Prop(_) => {
                                        let prop_id = AttributeValue::prop_id_for_id(
                                            ctx,
                                            attribute_value.id(),
                                        )
                                        .await?;
                                        Prop::path_by_id(ctx, prop_id).await?
                                    }
                                    _ => continue,
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
        Ok(AttributePrototypeDebugView {
            func_args: func_binding_args,
            func_id,
            func_name,
            func_execution,
            arg_sources,
            prototype_id,
            all_attribute_values,
        })
    }
}
