use serde::{Deserialize, Serialize};

use crate::{
    attribute_resolver::UNSET_ID_VALUE, ComponentId, PropId, SchemaId, SchemaVariantId, SystemId,
};

/// An `AttributeReadContext` allows for saying "do not use this filed
/// to filter results" by providing [`None`] for the field's value.
///
/// For example:
///
/// ```rust
/// # use dal::deculture::attribute::context::read::AttributeReadContext;
/// # const UNSET_ID_VALUE: i64 = -1;
/// let read_context = AttributeReadContext {
///     prop_id: None,
///     schema_id: Some(1.into()),
///     schema_variant_id: Some(1.into()),
///     component_id: Some(1.into()),
///     system_id: Some(UNSET_ID_VALUE.into()),
/// };
/// ```
///
/// The above `AttributeReadContext` would be used for finding all
/// attributes, across all [`Props`](Prop) that have been set for a
/// given [`SchemaId`], [`SchemaVariantId`], [`ComponentId`]
/// specificity.
#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttributeReadContext {
    #[serde(rename = "attribute_context_prop_id")]
    pub prop_id: Option<PropId>,
    #[serde(rename = "attribute_context_schema_id")]
    pub schema_id: Option<SchemaId>,
    #[serde(rename = "attribute_context_schema_variant_id")]
    pub schema_variant_id: Option<SchemaVariantId>,
    #[serde(rename = "attribute_context_component_id")]
    pub component_id: Option<ComponentId>,
    #[serde(rename = "attribute_context_system_id")]
    pub system_id: Option<SystemId>,
}

impl Default for AttributeReadContext {
    fn default() -> Self {
        Self {
            prop_id: Some(UNSET_ID_VALUE.into()),
            schema_id: Some(UNSET_ID_VALUE.into()),
            schema_variant_id: Some(UNSET_ID_VALUE.into()),
            component_id: Some(UNSET_ID_VALUE.into()),
            system_id: Some(UNSET_ID_VALUE.into()),
        }
    }
}

impl AttributeReadContext {
    pub fn prop_id(&self) -> Option<PropId> {
        self.prop_id
    }

    pub fn has_prop_id(&self) -> bool {
        self.prop_id.is_some()
    }

    pub fn schema_id(&self) -> Option<SchemaId> {
        self.schema_id
    }

    pub fn has_schema_id(&self) -> bool {
        self.schema_id.is_some()
    }

    pub fn schema_variant_id(&self) -> Option<SchemaVariantId> {
        self.schema_variant_id
    }

    pub fn has_schema_variant_id(&self) -> bool {
        self.schema_variant_id.is_some()
    }

    pub fn component_id(&self) -> Option<ComponentId> {
        self.component_id
    }

    pub fn has_component_id(&self) -> bool {
        self.component_id.is_some()
    }

    pub fn system_id(&self) -> Option<SystemId> {
        self.system_id
    }

    pub fn has_system_id(&self) -> bool {
        self.system_id.is_some()
    }
}

impl postgres_types::ToSql for AttributeReadContext {
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
