//! This module contains the ability to construct a [`schema`](PropertyEditorSchema) for a
//! [`Component`](crate::Component)'s properties.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use strum::{AsRefStr, Display, EnumString};

use si_pkg::PropSpecWidgetKind;

use crate::property_editor::{PropertyEditorError, PropertyEditorPropId, PropertyEditorResult};
use crate::{
    DalContext, LabelEntry, LabelList, Prop, PropKind, SchemaVariant, SchemaVariantId, Secret,
    SecretId, StandardModel,
};

const PROPERTY_EDITOR_SCHEMA_FOR_SCHEMA_VARIANT: &str =
    include_str!("../queries/property_editor_schema_for_schema_variant.sql");

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropertyEditorSchema {
    pub root_prop_id: PropertyEditorPropId,
    pub props: HashMap<PropertyEditorPropId, PropertyEditorProp>,
    pub child_props: HashMap<PropertyEditorPropId, Vec<PropertyEditorPropId>>,
}

impl PropertyEditorSchema {
    pub async fn for_schema_variant(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> PropertyEditorResult<Self> {
        let schema_variant = SchemaVariant::get_by_id(ctx, &schema_variant_id)
            .await?
            .ok_or(PropertyEditorError::SchemaVariantNotFound(
                schema_variant_id,
            ))?;
        let mut props: HashMap<PropertyEditorPropId, PropertyEditorProp> = HashMap::new();
        let mut child_props: HashMap<PropertyEditorPropId, Vec<PropertyEditorPropId>> =
            HashMap::new();

        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(
                PROPERTY_EDITOR_SCHEMA_FOR_SCHEMA_VARIANT,
                &[ctx.tenancy(), ctx.visibility(), &schema_variant.id()],
            )
            .await?;

        for row in rows {
            let json: Value = row.try_get("object")?;
            let prop: Prop = serde_json::from_value(json)?;
            // Omit any secret definition props in the result
            if prop
                .json_pointer(ctx)
                .await?
                .starts_with("/root/secret_definition")
            {
                continue;
            }
            let property_editor_prop = PropertyEditorProp::new(ctx, prop).await?;
            let maybe_child_prop_ids: Option<Vec<PropertyEditorPropId>> =
                row.try_get("child_prop_ids")?;
            if let Some(child_prop_ids) = maybe_child_prop_ids {
                child_props.insert(property_editor_prop.id, child_prop_ids);
            }

            props.insert(property_editor_prop.id, property_editor_prop);
        }

        let root_prop_id = schema_variant
            .root_prop_id()
            .ok_or(PropertyEditorError::RootPropNotFound)?;
        Ok(PropertyEditorSchema {
            root_prop_id: (*root_prop_id).into(),
            props,
            child_props,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropertyEditorProp {
    pub id: PropertyEditorPropId,
    pub name: String,
    pub kind: PropertyEditorPropKind,
    pub widget_kind: PropertyEditorPropWidgetKind,
    pub doc_link: Option<String>,
}

impl PropertyEditorProp {
    pub async fn new(ctx: &DalContext, prop: Prop) -> PropertyEditorResult<PropertyEditorProp> {
        Ok(PropertyEditorProp {
            id: (*prop.id()).into(),
            name: prop.name().into(),
            kind: prop.kind().into(),
            widget_kind: PropertyEditorPropWidgetKind::new(
                ctx,
                *prop.widget_kind(),
                prop.widget_options().map(|v| v.to_owned()),
            )
            .await?,
            doc_link: prop.doc_link().map(Into::into),
        })
    }
}

#[remain::sorted]
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PropertyEditorPropKind {
    Array,
    Boolean,
    Integer,
    Map,
    Object,
    String,
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

// TODO(nick,theo,wendy): consider passing "widget options" to _all_ widgets.
#[remain::sorted]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum PropertyEditorPropWidgetKind {
    Array,
    Checkbox,
    Color,
    ComboBox { options: Option<Value> },
    Header,
    Map,
    Secret { options: LabelList<SecretId> },
    Select { options: Option<Value> },
    Text,
    TextArea,
}

// NOTE(nick): this was from the old edit fields code, but it's interesting that we have multiple
// widget kind enums? Not important to look at right now, though.
#[remain::sorted]
#[derive(
    AsRefStr, Clone, Deserialize, Serialize, Debug, PartialEq, Eq, Display, EnumString, Copy,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum WidgetKind {
    Array,
    Checkbox,
    Color,
    /// Provides a text input with auto-completion for corresponding "primitive" (e.g. string, number, boolean)
    /// [`PropKinds`](crate::PropKind).
    ComboBox,
    Header,
    Map,
    Secret,
    /// Provides a select box for corresponding "primitive" (e.g. string, number, boolean)
    /// [`PropKinds`](crate::PropKind).
    Select,
    Text,
    TextArea,
}

impl From<WidgetKind> for PropSpecWidgetKind {
    fn from(value: WidgetKind) -> Self {
        match value {
            WidgetKind::Array => Self::Array,
            WidgetKind::Checkbox => Self::Checkbox,
            WidgetKind::Header => Self::Header,
            WidgetKind::Map => Self::Map,
            WidgetKind::Select => Self::Select,
            WidgetKind::Color => Self::Color,
            WidgetKind::Secret => Self::Secret,
            WidgetKind::Text => Self::Text,
            WidgetKind::TextArea => Self::TextArea,
            WidgetKind::ComboBox => Self::ComboBox,
        }
    }
}

impl From<&PropSpecWidgetKind> for WidgetKind {
    fn from(value: &PropSpecWidgetKind) -> Self {
        match value {
            PropSpecWidgetKind::Array => Self::Array,
            PropSpecWidgetKind::Checkbox => Self::Checkbox,
            PropSpecWidgetKind::Header => Self::Header,
            PropSpecWidgetKind::Map => Self::Map,
            PropSpecWidgetKind::Select => Self::Select,
            PropSpecWidgetKind::Color => Self::Color,
            PropSpecWidgetKind::Secret => Self::Secret,
            PropSpecWidgetKind::Text => Self::Text,
            PropSpecWidgetKind::TextArea => Self::TextArea,
            PropSpecWidgetKind::ComboBox => Self::ComboBox,
        }
    }
}

impl PropertyEditorPropWidgetKind {
    pub async fn new(
        ctx: &DalContext,
        widget_kind: WidgetKind,
        widget_options: Option<Value>,
    ) -> PropertyEditorResult<Self> {
        Ok(match widget_kind {
            WidgetKind::Array => Self::Array,
            WidgetKind::Checkbox => Self::Checkbox,
            WidgetKind::Header => Self::Header,
            WidgetKind::Map => Self::Map,
            WidgetKind::Select => Self::Select {
                options: widget_options,
            },
            WidgetKind::Color => Self::Color,
            WidgetKind::Secret => Self::Secret {
                options: LabelList::new(
                    Secret::list(ctx)
                        .await?
                        .into_iter()
                        .map(|s| LabelEntry::new(s.name(), *s.id()))
                        .collect(),
                ),
            },
            WidgetKind::Text => Self::Text,
            WidgetKind::TextArea => Self::TextArea,
            WidgetKind::ComboBox => Self::ComboBox {
                options: widget_options,
            },
        })
    }
}
