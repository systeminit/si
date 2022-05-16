//#[derive(Error, Debug)]
//pub enum PropertyEditorError {
//}

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use si_data::PgError;
use thiserror::Error;

use crate::{
    edit_field::widget::WidgetKind, pk, schema::variant::SchemaVariantError, DalContext, Prop,
    PropId, PropKind, SchemaVariant, SchemaVariantId, StandardModel, StandardModelError,
};

const PROPERTY_EDITOR_SCHEMA_FOR_SCHEMA_VARIANT: &str =
    include_str!("./queries/property_editor_schema_for_schema_variant.sql");

#[derive(Error, Debug)]
pub enum PropertyEditorError {
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
    #[error("schema variant not found: {0}")]
    SchemaVariantNotFound(SchemaVariantId),
    #[error("root prop not found for schema variant")]
    RootPropNotFound,
    #[error("schema variant: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
}

pub type PropertyEditorResult<T> = Result<T, PropertyEditorError>;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PropertyEditorPropKind {
    Array,
    Boolean,
    Integer,
    Object,
    String,
    Map,
}

impl From<&PropKind> for PropertyEditorPropKind {
    fn from(prop_kind: &PropKind) -> Self {
        match prop_kind {
            PropKind::Array => Self::Array,
            PropKind::Boolean => Self::Boolean,
            PropKind::Integer => Self::Integer,
            PropKind::Object => Self::Object,
            PropKind::String => Self::String,
            PropKind::Map => Self::Map,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PropertyEditorPropWidgetKind {
    Array,
    Checkbox,
    Header,
    Map,
    SecretSelect,
    Text,
}

impl From<&WidgetKind> for PropertyEditorPropWidgetKind {
    fn from(widget_kind: &WidgetKind) -> Self {
        match widget_kind {
            WidgetKind::Array => Self::Array,
            WidgetKind::Checkbox => Self::Checkbox,
            WidgetKind::Header => Self::Header,
            WidgetKind::Map => Self::Map,
            WidgetKind::SecretSelect => Self::SecretSelect,
            WidgetKind::Text => Self::Text,
        }
    }
}

pk!(PropertyEditorPropId);

impl From<&PropId> for PropertyEditorPropId {
    fn from(prop_id: &PropId) -> Self {
        let number: i64 = (*prop_id).into();
        PropertyEditorPropId(number)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PropertyEditorProp {
    pub id: PropertyEditorPropId,
    pub name: String,
    pub kind: PropertyEditorPropKind,
    pub widget_kind: PropertyEditorPropWidgetKind,
    pub doc_link: Option<String>,
}

impl From<Prop> for PropertyEditorProp {
    fn from(prop: Prop) -> PropertyEditorProp {
        PropertyEditorProp {
            id: prop.id().into(),
            name: prop.name().into(),
            kind: prop.kind().into(),
            widget_kind: prop.widget_kind().into(),
            doc_link: prop.doc_link().map(Into::into),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PropertyEditorSchema {
    pub root_prop_id: PropertyEditorPropId,
    pub props: HashMap<PropertyEditorPropId, PropertyEditorProp>,
    pub child_props: HashMap<PropertyEditorPropId, Vec<PropertyEditorPropId>>,
}

impl PropertyEditorSchema {
    pub async fn for_schema_variant(
        ctx: &DalContext<'_, '_>,
        schema_variant_id: SchemaVariantId,
    ) -> PropertyEditorResult<Self> {
        let schema_variant = SchemaVariant::get_by_id(ctx, &schema_variant_id)
            .await?
            .ok_or(PropertyEditorError::SchemaVariantNotFound(
                schema_variant_id,
            ))?;
        let root_prop = schema_variant
            .props(ctx)
            .await?
            .into_iter()
            .next()
            .ok_or(PropertyEditorError::RootPropNotFound)?;
        let mut props: HashMap<PropertyEditorPropId, PropertyEditorProp> = HashMap::new();
        let mut child_props: HashMap<PropertyEditorPropId, Vec<PropertyEditorPropId>> =
            HashMap::new();

        let rows = ctx
            .pg_txn()
            .query(
                PROPERTY_EDITOR_SCHEMA_FOR_SCHEMA_VARIANT,
                &[ctx.read_tenancy(), ctx.visibility(), &schema_variant.id()],
            )
            .await?;

        for row in rows {
            let json: serde_json::Value = row.try_get("object")?;
            let prop: Prop = serde_json::from_value(json)?;
            let property_editor_prop: PropertyEditorProp = prop.into();
            let maybe_child_prop_ids: Option<Vec<PropertyEditorPropId>> =
                row.try_get("child_prop_ids")?;
            if let Some(child_prop_ids) = maybe_child_prop_ids {
                child_props.insert(property_editor_prop.id, child_prop_ids);
            }
            props.insert(property_editor_prop.id, property_editor_prop);
        }

        Ok(PropertyEditorSchema {
            root_prop_id: root_prop.id().into(),
            props,
            child_props,
        })
    }
}
