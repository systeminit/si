use std::collections::HashMap;

use serde::{
    Deserialize,
    Serialize,
};
use telemetry::prelude::*;
use thiserror::Error;

use super::ValueIsFor;
use crate::{
    AttributePrototypeId,
    AttributeValue,
    AttributeValueId,
    ComponentError,
    DalContext,
    FuncError,
    FuncId,
    Prop,
    PropKind,
    attribute::{
        prototype::{
            AttributePrototypeError,
            argument::{
                AttributePrototypeArgumentError,
                value_source::ValueSourceError,
            },
            debug::{
                AttributePrototypeDebugView,
                AttributePrototypeDebugViewError,
                FuncArgDebugView,
            },
        },
        value::AttributeValueError,
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

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttributeDebugView {
    pub path: String,
    pub parent_id: Option<AttributeValueId>,
    pub attribute_value_id: AttributeValueId,
    pub func_id: FuncId,
    pub value_is_for: ValueIsFor,
    pub prop: Option<Prop>,
    pub prototype_id: Option<AttributePrototypeId>,
    pub prototype_is_component_specific: bool,
    pub key: Option<String>,
    pub func_name: String,
    pub func_args: HashMap<String, Vec<FuncArgDebugView>>,
    pub value: Option<serde_json::Value>,
    pub prop_kind: Option<PropKind>,
    pub view: Option<serde_json::Value>,
}

type AttributeDebugViewResult<T> = Result<T, AttributeDebugViewError>;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum AttributeDebugViewError {
    #[error("attribute prototype argument error: {0}")]
    AttributePrototypeArgumentError(#[from] Box<AttributePrototypeArgumentError>),
    #[error("attribute debug view error: {0}")]
    AttributePrototypeDebugViewError(#[from] Box<AttributePrototypeDebugViewError>),
    #[error("attribute prototype error: {0}")]
    AttributePrototypeError(#[from] Box<AttributePrototypeError>),
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] Box<AttributeValueError>),
    #[error("component error: {0}")]
    ComponentError(#[from] Box<ComponentError>),
    #[error("func error: {0}")]
    Func(#[from] Box<FuncError>),
    #[error("input socket error: {0}")]
    InputSocketError(#[from] Box<InputSocketError>),
    #[error("node weight error: {0}")]
    NodeWeightError(#[from] Box<NodeWeightError>),
    #[error("output socket error: {0}")]
    OutputSocketError(#[from] Box<OutputSocketError>),
    #[error("prop error: {0}")]
    PropError(#[from] Box<PropError>),
    #[error("value source error: {0}")]
    ValueSourceError(#[from] Box<ValueSourceError>),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshotError(#[from] Box<WorkspaceSnapshotError>),
}
impl AttributeDebugView {
    #[instrument(level = "trace", skip_all)]
    pub async fn new(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
        key: Option<String>,
        parent_id: Option<AttributeValueId>,
    ) -> AttributeDebugViewResult<AttributeDebugView> {
        let attribute_value = AttributeValue::get_by_id(ctx, attribute_value_id).await?;
        let prototype_id = AttributeValue::prototype_id(ctx, attribute_value_id).await?;
        let value_is_for = AttributeValue::is_for(ctx, attribute_value_id).await?;

        let prop_id = AttributeValue::prop_id(ctx, attribute_value_id).await?;

        let prop = Prop::get_by_id(ctx, prop_id).await?;
        let path = AttributeValue::get_path_for_id(ctx, attribute_value_id)
            .await?
            .unwrap_or_else(String::new);
        let prop_opt: Option<Prop> = Some(prop);
        let attribute_prototype_debug_view =
            AttributePrototypeDebugView::new(ctx, attribute_value_id).await?;
        let value_view = AttributeValue::view(ctx, attribute_value_id).await?;
        let prop_kind = prop_opt.clone().map(|prop| prop.kind);
        let value = match attribute_value.unprocessed_value(ctx).await? {
            Some(value) => Some(value),
            None => attribute_value.value(ctx).await?,
        };
        let view = AttributeDebugView {
            path,
            parent_id,
            attribute_value_id,
            func_id: attribute_prototype_debug_view.func_id,
            key,
            prop: prop_opt,
            prototype_id: Some(prototype_id),
            prototype_is_component_specific: attribute_prototype_debug_view.is_component_specific,
            value_is_for,
            func_name: attribute_prototype_debug_view.func_name,
            func_args: attribute_prototype_debug_view.func_args,
            value,
            prop_kind,
            view: value_view,
        };
        Ok(view)
    }
}

impl From<AttributePrototypeArgumentError> for AttributeDebugViewError {
    fn from(value: AttributePrototypeArgumentError) -> Self {
        Box::new(value).into()
    }
}

impl From<AttributePrototypeDebugViewError> for AttributeDebugViewError {
    fn from(value: AttributePrototypeDebugViewError) -> Self {
        Box::new(value).into()
    }
}

impl From<AttributePrototypeError> for AttributeDebugViewError {
    fn from(value: AttributePrototypeError) -> Self {
        Box::new(value).into()
    }
}

impl From<AttributeValueError> for AttributeDebugViewError {
    fn from(value: AttributeValueError) -> Self {
        Box::new(value).into()
    }
}

impl From<ComponentError> for AttributeDebugViewError {
    fn from(value: ComponentError) -> Self {
        Box::new(value).into()
    }
}

impl From<FuncError> for AttributeDebugViewError {
    fn from(value: FuncError) -> Self {
        Box::new(value).into()
    }
}

impl From<InputSocketError> for AttributeDebugViewError {
    fn from(value: InputSocketError) -> Self {
        Box::new(value).into()
    }
}

impl From<NodeWeightError> for AttributeDebugViewError {
    fn from(value: NodeWeightError) -> Self {
        Box::new(value).into()
    }
}

impl From<OutputSocketError> for AttributeDebugViewError {
    fn from(value: OutputSocketError) -> Self {
        Box::new(value).into()
    }
}

impl From<PropError> for AttributeDebugViewError {
    fn from(value: PropError) -> Self {
        Box::new(value).into()
    }
}

impl From<ValueSourceError> for AttributeDebugViewError {
    fn from(value: ValueSourceError) -> Self {
        Box::new(value).into()
    }
}

impl From<WorkspaceSnapshotError> for AttributeDebugViewError {
    fn from(value: WorkspaceSnapshotError) -> Self {
        Box::new(value).into()
    }
}
