use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use si_data_pg::PgError;
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    impl_standard_model, pk, standard_model, standard_model_accessor, AttributeReadContext,
    AttributeValue, AttributeValueError, AttributeValueId, Component, ComponentError, ComponentId,
    DalContext, HistoryEventError, StandardModel, StandardModelError, Tenancy, Timestamp,
    TransactionsError, Visibility,
};

use super::{
    values::{PropertyEditorValue, PropertyEditorValues},
    PropertyEditorError, PropertyEditorValueId,
};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum PropertyEditorValuesSummaryError {
    #[error(transparent)]
    AttributeValue(#[from] AttributeValueError),
    #[error(transparent)]
    Component(#[from] ComponentError),
    #[error(transparent)]
    HistoryEvent(#[from] HistoryEventError),
    #[error(transparent)]
    Pg(#[from] PgError),
    #[error(transparent)]
    PropertyEditor(#[from] PropertyEditorError),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error(transparent)]
    Transactions(#[from] TransactionsError),
}

pub type PropertyEditorValuesSummaryResult<T> = Result<T, PropertyEditorValuesSummaryError>;

pk!(PropertyEditorValuesSummaryPk);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct PropertyEditorValuesSummary {
    pk: PropertyEditorValuesSummaryPk,
    id: ComponentId,
    #[serde(flatten)]
    tenancy: Tenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
    property_editor_values: serde_json::Value,
}

impl_standard_model! {
    model: PropertyEditorValuesSummary,
    pk: PropertyEditorValuesSummaryPk,
    id: ComponentId,
    table_name: "property_editor_values_summaries",
    history_event_label_base: "property_editor_values",
    history_event_message_name: "Property Editor Values Summary"
}

impl PropertyEditorValuesSummary {
    standard_model_accessor!(
        property_editor_values,
        Json<JsonValue>,
        PropertyEditorValuesSummaryResult
    );

    pub async fn create_or_update_component_entry(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> PropertyEditorValuesSummaryResult<serde_json::Value> {
        let mut root_value_id = None;
        let mut values = HashMap::new();
        let mut child_values: HashMap<PropertyEditorValueId, Vec<PropertyEditorValueId>> =
            HashMap::new();
        let mut work_queue = AttributeValue::list_payload_for_read_context(
            ctx,
            AttributeReadContext {
                prop_id: None,
                component_id: Some(component_id),
                ..AttributeReadContext::default()
            },
        )
        .await?;

        // We sort the work queue according to the order of every nested IndexMap. This ensures that
        // when we reconstruct the final properties data, we don't have to worry about the order things
        // appear in - they are certain to be the right order.
        let attribute_value_order: Vec<AttributeValueId> = work_queue
            .iter()
            .filter_map(|avp| avp.attribute_value.index_map())
            .flat_map(|index_map| index_map.order())
            .copied()
            .collect();
        work_queue.sort_by_cached_key(|avp| {
            attribute_value_order
                .iter()
                .position(|attribute_value_id| attribute_value_id == avp.attribute_value.id())
                .unwrap_or(0)
        });
        let attribute_value_controlling_func_info =
            AttributeValue::get_controlling_func_id(ctx, component_id).await?;

        //let overrides = AttributeValue::list_attributes_with_overridden(ctx, component_id).await?;
        let overrides: HashMap<AttributeValueId, bool> = HashMap::new();

        for work in work_queue {
            let work_attribute_value_id = *work.attribute_value.id();
            let (
                work_controlling_func_id,
                work_controlling_attribute_value_id,
                work_controlling_func_name,
            ) = attribute_value_controlling_func_info
                .get(&work_attribute_value_id)
                .ok_or(AttributeValueError::MissingForId(work_attribute_value_id))?;

            let sockets = Component::list_input_sockets_for_attribute_value(
                ctx,
                work_attribute_value_id,
                component_id,
            )
            .await?;

            let can_be_set_by_socket: bool = !sockets.is_empty();

            let is_from_external_source = sockets.iter().any(|(_socket, has_edge)| *has_edge);

            let is_controlled_by_intrinsic_func = work_controlling_func_name == "si:setObject"
                || work_controlling_func_name == "si:setMap"
                || work_controlling_func_name == "si:setArray"
                || work_controlling_func_name == "si:setString"
                || work_controlling_func_name == "si:setInteger"
                || work_controlling_func_name == "si:setBoolean"
                || work_controlling_func_name == "si:unset";

            let overridden = overrides
                .get(&work_attribute_value_id)
                .copied()
                .unwrap_or(false);

            values.insert(
                work_attribute_value_id.into(),
                PropertyEditorValue {
                    id: work_attribute_value_id.into(),
                    prop_id: (*work.prop.id()).into(),
                    key: work.attribute_value.key().map(Into::into),
                    value: work
                        .func_binding_return_value
                        .and_then(|f| f.value().cloned())
                        .unwrap_or(serde_json::Value::Null),
                    is_from_external_source,
                    can_be_set_by_socket,
                    is_controlled_by_intrinsic_func,
                    controlling_func_id: *work_controlling_func_id,
                    controlling_attribute_value_id: *work_controlling_attribute_value_id,
                    overridden,
                },
            );
            if let Some(parent_id) = work.parent_attribute_value_id {
                child_values
                    .entry(parent_id.into())
                    .or_default()
                    .push(work_attribute_value_id.into());
            } else {
                root_value_id = Some(work_attribute_value_id.into());
            }
        }

        if let Some(root_value_id) = root_value_id {
            let property_editor_values = PropertyEditorValues {
                root_value_id,
                child_values,
                values,
            };
            let row = ctx
                .txns()
                .await?
                .pg()
                .query_one(
                    "SELECT object FROM property_editor_values_summary_create_or_update_v1($1, $2, $3, $4)",
                    &[ctx.tenancy(), ctx.visibility(), &component_id, &property_editor_values],
                )
                .await?;

            standard_model::object_from_row::<Self>(row)
                .map_err(Into::into)
                .map(|summary| summary.property_editor_values().clone())
        } else {
            Err(Into::into(PropertyEditorError::RootPropNotFound))
        }
    }
}
