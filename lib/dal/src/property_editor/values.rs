//! This module contains the ability to construct values reflecting the latest state of a
//! [`Component`](crate::Component)'s properties.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::property_editor::{PropertyEditorError, PropertyEditorResult};
use crate::property_editor::{PropertyEditorPropId, PropertyEditorValueId};
use crate::{AttributeValueId, ComponentId, DalContext, FuncId, Prop, PropId, StandardModel};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropertyEditorValues {
    pub root_value_id: PropertyEditorValueId,
    pub values: HashMap<PropertyEditorValueId, PropertyEditorValue>,
    pub child_values: HashMap<PropertyEditorValueId, Vec<PropertyEditorValueId>>,
}

impl PropertyEditorValues {
    pub async fn for_component(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> PropertyEditorResult<serde_json::Value> {
        if let Some(summary) =
            super::values_summary::PropertyEditorValuesSummary::get_by_id(ctx, &component_id)
                .await
                .map_err(|e| PropertyEditorError::PropertyEditorValuesSummary(e.to_string()))?
                .map(|v| v.property_editor_values().clone())
        {
            return Ok(summary);
        }

        // If there's no values summary, calculate it live and return it
        super::values_summary::PropertyEditorValuesSummary::create_or_update_component_entry(
            ctx,
            component_id,
        )
        .await
        .map_err(|e| PropertyEditorError::PropertyEditorValuesSummary(e.to_string()))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropertyEditorValue {
    pub id: PropertyEditorValueId,
    pub prop_id: PropertyEditorPropId,
    pub key: Option<String>,
    pub value: Value,
    pub is_from_external_source: bool,
    pub can_be_set_by_socket: bool,
    pub is_controlled_by_intrinsic_func: bool,
    pub controlling_func_id: FuncId,
    pub controlling_attribute_value_id: AttributeValueId,
    pub overridden: bool,
}

impl PropertyEditorValue {
    pub fn attribute_value_id(&self) -> AttributeValueId {
        self.id.into()
    }

    pub fn value(&self) -> Value {
        self.value.clone()
    }

    pub fn prop_id(&self) -> PropId {
        self.prop_id.into()
    }

    /// Returns the [`Prop`](crate::Prop) corresponding to the "prop_id" field.
    pub async fn prop(&self, ctx: &DalContext) -> PropertyEditorResult<Prop> {
        let prop = Prop::get_by_id(ctx, &self.prop_id.into())
            .await?
            .ok_or_else(|| PropertyEditorError::PropNotFound(self.prop_id.into()))?;
        Ok(prop)
    }
}

impl postgres_types::ToSql for PropertyEditorValues {
    fn to_sql(
        &self,
        ty: &postgres_types::Type,
        out: &mut postgres_types::private::BytesMut,
    ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>>
    where
        Self: Sized,
    {
        let json = serde_json::to_value(self)?;
        postgres_types::ToSql::to_sql(&json, ty, out)
    }

    fn accepts(ty: &postgres_types::Type) -> bool
    where
        Self: Sized,
    {
        ty == &postgres_types::Type::JSONB
    }

    fn to_sql_checked(
        &self,
        ty: &postgres_types::Type,
        out: &mut postgres_types::private::BytesMut,
    ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
        let json = serde_json::to_value(self)?;
        postgres_types::ToSql::to_sql(&json, ty, out)
    }
}
