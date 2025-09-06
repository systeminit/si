use std::collections::HashMap;

use serde::{
    Deserialize,
    Serialize,
};
use serde_json::Value;
use telemetry::prelude::*;
use thiserror::Error;
use ulid::Ulid;

use crate::{
    AttributePrototype,
    AttributePrototypeId,
    AttributeValue,
    AttributeValueId,
    Component,
    ComponentError,
    DalContext,
    Func,
    FuncError,
    FuncId,
    SecretError,
    attribute::{
        prototype::{
            AttributePrototypeError,
            argument::{
                AttributePrototypeArgument,
                AttributePrototypeArgumentError,
                static_value::StaticArgumentValue,
                value_source::{
                    ValueSource,
                    ValueSourceError,
                },
            },
        },
        value::{
            AttributeValueError,
            ValueIsFor,
        },
    },
    component::socket::ComponentInputSocket,
    func::argument::{
        FuncArgument,
        FuncArgumentError,
    },
    prop::PropError,
    socket::{
        input::InputSocketError,
        output::OutputSocketError,
    },
    workspace_snapshot::{
        WorkspaceSnapshotError,
        node_weight::NodeWeightError,
    },
};

type AttributePrototypeDebugViewResult<T> = Result<T, AttributePrototypeDebugViewError>;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum AttributePrototypeDebugViewError {
    #[error("attribute prototype argument Error: {0}")]
    AttributePrototypeArgumentError(#[from] Box<AttributePrototypeArgumentError>),
    #[error("attribute prototype error: {0}")]
    AttributePrototypeError(#[from] Box<AttributePrototypeError>),
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] Box<AttributeValueError>),
    #[error("component error: {0}")]
    ComponentError(#[from] Box<ComponentError>),
    #[error("func error: {0}")]
    Func(#[from] Box<FuncError>),
    #[error("func argument error: {0}")]
    FuncArgumentError(#[from] Box<FuncArgumentError>),
    #[error("input socket error: {0}")]
    InputSocketError(#[from] Box<InputSocketError>),
    #[error("node weight error: {0}")]
    NodeWeightError(#[from] Box<NodeWeightError>),
    #[error("output socket error: {0}")]
    OutputSocketError(#[from] Box<OutputSocketError>),
    #[error("prop error: {0}")]
    PropError(#[from] Box<PropError>),
    #[error("secret error: {0}")]
    SecretError(#[from] Box<SecretError>),
    #[error("serde json error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("value source error: {0}")]
    ValueSourceError(#[from] Box<ValueSourceError>),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshotError(#[from] Box<WorkspaceSnapshotError>),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttributePrototypeDebugView {
    pub func_id: FuncId,
    pub id: AttributePrototypeId,
    pub func_name: String,
    pub func_args: HashMap<String, Vec<FuncArgDebugView>>,
    pub attribute_values: Vec<AttributeValueId>,
    pub is_component_specific: bool,
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

impl AttributePrototypeDebugView {
    #[instrument(level = "trace", skip_all)]
    pub async fn new(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> AttributePrototypeDebugViewResult<AttributePrototypeDebugView> {
        let prototype_id = AttributeValue::prototype_id(ctx, attribute_value_id).await?;
        let has_component_prototype =
            AttributeValue::component_prototype_id(ctx, attribute_value_id)
                .await?
                .is_some();
        let component_id = AttributeValue::component_id(ctx, attribute_value_id).await?;
        let mut func_binding_args: HashMap<String, Vec<FuncArgDebugView>> = HashMap::new();
        let attribute_prototype_arg_ids =
            AttributePrototypeArgument::list_ids_for_prototype(ctx, prototype_id).await?;
        for attribute_prototype_arg_id in attribute_prototype_arg_ids {
            let func_arg_id =
                AttributePrototypeArgument::func_argument_id(ctx, attribute_prototype_arg_id)
                    .await?;

            let func_arg_name = FuncArgument::get_name_by_id(ctx, func_arg_id).await?;
            let value_source =
                AttributePrototypeArgument::value_source(ctx, attribute_prototype_arg_id).await?;
            let values_for_arg = match value_source {
                ValueSource::ValueSubscription(ref subscription) => {
                    let value_source_id = subscription.attribute_value_id.into();
                    let view = match subscription.resolve(ctx).await? {
                        Some(av_id) => AttributeValue::view(ctx, av_id).await?,
                        None => None,
                    };
                    // TODO add subscription path to FuncArgDebugView (path right now is prop path)
                    // and also make value_source_id optional since there is not necessarily a source
                    vec![FuncArgDebugView {
                        value: view.unwrap_or(Value::Null),
                        name: func_arg_name.clone(),
                        value_source: format!("{value_source:?}"),
                        value_source_id,
                        path: None,
                        socket_source_kind: None,
                        is_used: true,
                    }]
                }
                ValueSource::Secret(secret_id) => {
                    vec![FuncArgDebugView {
                        value: serde_json::to_value(format!(
                            "[REDACTED KEY FOR SECRET (ID: {secret_id})]"
                        ))?,
                        name: func_arg_name.clone(),
                        value_source: format!("{value_source:?}"),
                        value_source_id: secret_id.into(),
                        path: None,
                        socket_source_kind: None,
                        is_used: true,
                    }]
                }
                ValueSource::StaticArgumentValue(static_argument_value_id) => {
                    let val = StaticArgumentValue::get_by_id(ctx, static_argument_value_id)
                        .await?
                        .value;
                    vec![FuncArgDebugView {
                        value: val,
                        name: func_arg_name.clone(),
                        value_source: format!("{value_source:?}"),
                        value_source_id: static_argument_value_id.into(),
                        path: None,
                        socket_source_kind: None,
                        is_used: true,
                    }]
                }
                ValueSource::Prop(prop_id) => {
                    let mut values = vec![];

                    for attribute_value_id in value_source
                        .attribute_values_for_component_id(ctx, component_id)
                        .await?
                    {
                        let prop_path =
                            AttributeValue::get_path_for_id(ctx, attribute_value_id).await?;
                        let view = AttributeValue::view(ctx, attribute_value_id)
                            .await?
                            .unwrap_or(Value::Null);
                        let func_arg_debug = FuncArgDebugView {
                            value: view,
                            name: func_arg_name.clone(),
                            value_source: format!("{value_source:?}"),
                            value_source_id: prop_id.into(),
                            socket_source_kind: None,
                            path: prop_path,
                            is_used: true,
                        };
                        values.push(func_arg_debug);
                    }

                    values
                }
                ValueSource::InputSocket(input_socket_id) => {
                    let mut values = vec![];

                    for attribute_value_id in value_source
                        .attribute_values_for_component_id(ctx, component_id)
                        .await?
                    {
                        let attribute_value_path =
                            AttributeValue::get_path_for_id(ctx, attribute_value_id).await?;

                        let value_view = AttributeValue::view(ctx, attribute_value_id)
                            .await?
                            .unwrap_or(Value::Null);
                        let func_arg_debug = FuncArgDebugView {
                            value: value_view,
                            name: func_arg_name.clone(),
                            value_source: format!("{value_source:?}"),
                            value_source_id: input_socket_id.into(),
                            socket_source_kind: Some(SocketSourceKind::Manual),
                            path: attribute_value_path,
                            is_used: true,
                        };
                        values.push(func_arg_debug);
                    }

                    values
                }
                ValueSource::OutputSocket(output_socket_id) => {
                    let mut values = vec![];

                    for attribute_value_id in value_source
                        .attribute_values_for_component_id(ctx, component_id)
                        .await?
                    {
                        let attribute_value_path =
                            AttributeValue::get_path_for_id(ctx, attribute_value_id).await?;

                        let value_view = AttributeValue::view(ctx, attribute_value_id)
                            .await?
                            .unwrap_or(Value::Null);
                        let func_arg_debug = FuncArgDebugView {
                            value: value_view,
                            name: func_arg_name.clone(),
                            value_source: format!("{value_source:?}"),
                            value_source_id: output_socket_id.into(),
                            socket_source_kind: Some(SocketSourceKind::Manual),
                            path: attribute_value_path,
                            is_used: true,
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
        // if this attribute value is for an input socket, need to also get any inferred inputs if they exist!
        if let ValueIsFor::InputSocket(input_socket_id) =
            AttributeValue::is_for(ctx, attribute_value_id).await?
        {
            if let Some(component_input_socket) =
                ComponentInputSocket::get_by_ids(ctx, component_id, input_socket_id).await?
            {
                // now get inferred func binding args and values!
                for output_match in component_input_socket
                    .find_inferred_connections(ctx)
                    .await?
                {
                    let arg_used = Component::should_data_flow_between_components(
                        ctx,
                        component_input_socket.component_id,
                        output_match.component_id,
                    )
                    .await?;
                    let attribute_value_path =
                        AttributeValue::get_path_for_id(ctx, attribute_value_id).await?;
                    let value_view = AttributeValue::view(ctx, output_match.attribute_value_id)
                        .await?
                        .unwrap_or(Value::Null);
                    let input_func = AttributePrototype::func_id(ctx, prototype_id).await?;
                    if let Some(func_argument) =
                        FuncArgument::list_for_func(ctx, input_func).await?.pop()
                    {
                        let func_arg_name = func_argument.name.clone();
                        let func_arg_debug = FuncArgDebugView {
                            value: value_view,
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

        let func_name = Func::get_by_id(ctx, func_id).await?.name;
        let view = AttributePrototypeDebugView {
            func_args: func_binding_args,
            func_id,
            func_name,
            id: prototype_id,
            attribute_values,
            is_component_specific: has_component_prototype,
        };
        Ok(view)
    }
}

impl From<AttributePrototypeArgumentError> for AttributePrototypeDebugViewError {
    fn from(value: AttributePrototypeArgumentError) -> Self {
        Box::new(value).into()
    }
}

impl From<AttributePrototypeError> for AttributePrototypeDebugViewError {
    fn from(value: AttributePrototypeError) -> Self {
        Box::new(value).into()
    }
}

impl From<AttributeValueError> for AttributePrototypeDebugViewError {
    fn from(value: AttributeValueError) -> Self {
        Box::new(value).into()
    }
}

impl From<ComponentError> for AttributePrototypeDebugViewError {
    fn from(value: ComponentError) -> Self {
        Box::new(value).into()
    }
}

impl From<FuncError> for AttributePrototypeDebugViewError {
    fn from(value: FuncError) -> Self {
        Box::new(value).into()
    }
}

impl From<FuncArgumentError> for AttributePrototypeDebugViewError {
    fn from(value: FuncArgumentError) -> Self {
        Box::new(value).into()
    }
}

impl From<InputSocketError> for AttributePrototypeDebugViewError {
    fn from(value: InputSocketError) -> Self {
        Box::new(value).into()
    }
}

impl From<NodeWeightError> for AttributePrototypeDebugViewError {
    fn from(value: NodeWeightError) -> Self {
        Box::new(value).into()
    }
}

impl From<OutputSocketError> for AttributePrototypeDebugViewError {
    fn from(value: OutputSocketError) -> Self {
        Box::new(value).into()
    }
}

impl From<PropError> for AttributePrototypeDebugViewError {
    fn from(value: PropError) -> Self {
        Box::new(value).into()
    }
}

impl From<SecretError> for AttributePrototypeDebugViewError {
    fn from(value: SecretError) -> Self {
        Box::new(value).into()
    }
}

impl From<ValueSourceError> for AttributePrototypeDebugViewError {
    fn from(value: ValueSourceError) -> Self {
        Box::new(value).into()
    }
}

impl From<WorkspaceSnapshotError> for AttributePrototypeDebugViewError {
    fn from(value: WorkspaceSnapshotError) -> Self {
        Box::new(value).into()
    }
}
