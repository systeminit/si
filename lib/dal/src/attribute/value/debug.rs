use super::ValueIsFor;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use telemetry::prelude::*;
use thiserror::Error;

use crate::attribute::prototype::debug::{
    AttributePrototypeDebugView, AttributePrototypeDebugViewError,
};
use crate::attribute::prototype::{
    argument::{value_source::ValueSourceError, AttributePrototypeArgumentError},
    AttributePrototypeError,
};
use crate::attribute::value::AttributeValueError;
use crate::func::execution::{FuncExecution, FuncExecutionError};
use crate::prop::PropError;
use crate::socket::input::InputSocketError;
use crate::socket::output::OutputSocketError;
use crate::workspace_snapshot::node_weight::NodeWeightError;
use crate::workspace_snapshot::WorkspaceSnapshotError;
use crate::ComponentError;
use crate::{
    AttributePrototypeId, AttributeValue, AttributeValueId, DalContext, FuncError, FuncId, Prop,
    PropKind,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttributeDebugView {
    pub path: String,
    pub parent_id: Option<AttributeValueId>,
    pub attribute_value_id: AttributeValueId,
    pub func_id: FuncId,
    pub func_execution: Option<FuncExecution>,
    pub value_is_for: ValueIsFor,
    pub prop: Option<Prop>,
    pub prototype_id: Option<AttributePrototypeId>,
    pub key: Option<String>,
    pub func_name: String,
    pub func_args: HashMap<String, Vec<serde_json::Value>>,
    pub arg_sources: HashMap<String, Option<String>>,
    pub value: Option<serde_json::Value>,
    pub prop_kind: Option<PropKind>,
    pub materialized_view: Option<serde_json::Value>,
}

type AttributeDebugViewResult<T> = Result<T, AttributeDebugViewError>;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum AttributeDebugViewError {
    #[error("attribute prototype argument error: {0}")]
    AttributePrototypeArgumentError(#[from] AttributePrototypeArgumentError),
    #[error("attribute debug view error: {0}")]
    AttributePrototypeDebugViewError(#[from] AttributePrototypeDebugViewError),
    #[error("attribute prototype error: {0}")]
    AttributePrototypeError(#[from] AttributePrototypeError),
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    #[error("component error: {0}")]
    ComponentError(#[from] ComponentError),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
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
impl AttributeDebugView {
    #[instrument(level = "info", skip_all)]
    pub async fn new(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
        key: Option<String>,
        parent_id: Option<AttributeValueId>,
    ) -> AttributeDebugViewResult<AttributeDebugView> {
        let attribute_value = AttributeValue::get_by_id(ctx, attribute_value_id).await?;
        let prototype_id = AttributeValue::prototype_id(ctx, attribute_value_id).await?;
        let value_is_for = AttributeValue::is_for(ctx, attribute_value_id).await?;

        let prop_id = AttributeValue::prop_id_for_id_or_error(ctx, attribute_value_id).await?;

        let prop = Prop::get_by_id_or_error(ctx, prop_id).await?;
        let path = AttributeValue::get_path_for_id(ctx, attribute_value_id)
            .await?
            .unwrap_or_else(String::new);
        let prop_opt: Option<Prop> = Some(prop);
        let attribute_prototype_debug_view =
            AttributePrototypeDebugView::assemble(ctx, attribute_value_id).await?;
        let materialized_view = attribute_value.materialized_view(ctx).await?;
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
            func_execution: attribute_prototype_debug_view.func_execution,
            prop: prop_opt,
            prototype_id: Some(prototype_id),
            value_is_for,
            func_name: attribute_prototype_debug_view.func_name,
            func_args: attribute_prototype_debug_view.func_args,
            arg_sources: attribute_prototype_debug_view.arg_sources,
            value,
            prop_kind,
            materialized_view,
        };
        Ok(view)
    }
}
