//#[derive(Error, Debug)]
//pub enum PropertyEditorError {
//}

use serde::{Deserialize, Serialize};
use serde_json::Value;
use si_data::PgError;
use std::{cmp::Ordering, collections::HashMap};
use thiserror::Error;

use crate::attribute::value::FuncWithPrototypeContext;
use crate::{
    edit_field::widget::WidgetKind, pk, schema::variant::SchemaVariantError, AttributeReadContext,
    AttributeValue, AttributeValueError, AttributeValueId, ComponentError, ComponentId, DalContext,
    LabelEntry, LabelList, Prop, PropId, PropKind, SchemaVariant, SchemaVariantId, Secret,
    StandardModel, StandardModelError, SystemId, ValidationResolver, ValidationResolverError,
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
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    #[error("invalid AttributeReadContext: {0}")]
    BadAttributeReadContext(String),
    #[error("component not found")]
    ComponentNotFound,
    #[error("component error: {0}")]
    Component(#[from] ComponentError),
    #[error("validation resolver error: {0}")]
    ValidationResolver(#[from] ValidationResolverError),
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

// TODO(nick,theo,wendy): consider passing "widget options" to _all_ widgets.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum PropertyEditorPropWidgetKind {
    Array,
    Checkbox,
    Header,
    Map,
    SecretSelect { options: LabelList<i64> },
    Select { options: Option<Value> },
    Text,
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
            WidgetKind::SecretSelect => Self::SecretSelect {
                options: LabelList::new(
                    Secret::list(ctx)
                        .await?
                        .into_iter()
                        .map(|s| LabelEntry::new(s.name(), i64::from(*s.id())))
                        .collect(),
                ),
            },
            WidgetKind::Text => Self::Text,
        })
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
            id: prop.id().into(),
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
            let property_editor_prop = PropertyEditorProp::new(ctx, prop).await?;
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

pk!(PropertyEditorValueId);

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropertyEditorValue {
    id: PropertyEditorValueId,
    pub prop_id: PropertyEditorPropId,
    key: Option<String>,
    value: Value,
    func: FuncWithPrototypeContext,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropertyEditorValues {
    root_value_id: PropertyEditorValueId,
    pub values: HashMap<PropertyEditorValueId, PropertyEditorValue>,
    child_values: HashMap<PropertyEditorValueId, Vec<PropertyEditorValueId>>,
}

impl PropertyEditorValues {
    pub async fn for_context(
        ctx: &DalContext,
        context: AttributeReadContext,
    ) -> PropertyEditorResult<Self> {
        let mut root_value_id = None;
        let mut values = HashMap::new();
        let mut child_values = HashMap::new();

        let mut work_queue = AttributeValue::list_payload_for_read_context(ctx, context).await?;

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

        for work in work_queue {
            values.insert(
                i64::from(*work.attribute_value.id()).into(),
                PropertyEditorValue {
                    id: i64::from(*work.attribute_value.id()).into(),
                    prop_id: i64::from(*work.prop.id()).into(),
                    key: work.attribute_value.key().map(Into::into),
                    value: work
                        .func_binding_return_value
                        .and_then(|f| f.value().cloned())
                        .unwrap_or(Value::Null),
                    func: work.func_with_prototype_context,
                },
            );
            if let Some(parent_id) = work.parent_attribute_value_id {
                child_values
                    .entry(i64::from(parent_id).into())
                    .or_insert(vec![])
                    .push(i64::from(*work.attribute_value.id()).into());
            } else {
                root_value_id = Some(i64::from(*work.attribute_value.id()).into());
            }
        }

        // Note: hackish ordering to ensure consistency in the frontend
        for value in child_values.values_mut() {
            value.sort_by(|a, b| {
                let a = &values[a];
                let b = &values[b];
                match a.prop_id.cmp(&b.prop_id) {
                    Ordering::Equal => a.id.cmp(&b.id),
                    ordering => ordering,
                }
            });
        }

        if let Some(root_value_id) = root_value_id {
            Ok(PropertyEditorValues {
                root_value_id,
                child_values,
                values,
            })
        } else {
            Err(PropertyEditorError::RootPropNotFound)
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropertyEditorValidationError {
    message: String,
    level: Option<String>,
    kind: Option<String>,
    link: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropertyEditorValidation {
    value_id: PropertyEditorValueId,
    valid: bool,
    errors: Vec<PropertyEditorValidationError>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropertyEditorValidations {
    validations: Vec<PropertyEditorValidation>,
}

impl PropertyEditorValidations {
    pub async fn for_component(
        ctx: &DalContext,
        component_id: ComponentId,
        system_id: SystemId,
    ) -> PropertyEditorResult<Self> {
        let status = ValidationResolver::find_status(ctx, component_id, system_id).await?;

        let mut validations = Vec::new();
        for stat in status {
            validations.push(PropertyEditorValidation {
                value_id: i64::from(stat.attribute_value_id).into(),
                valid: stat.errors.is_empty(),
                errors: stat
                    .errors
                    .into_iter()
                    .map(|err| PropertyEditorValidationError {
                        message: err.message,
                        level: err.level,
                        kind: Some(err.kind.as_str().to_string()),
                        link: err.link,
                    })
                    .collect(),
            });
        }
        Ok(Self { validations })
    }
}

// TODO(nick): once shape is finalized and we stop serializing this within builtins, please
// convert to a more formal type.
#[derive(Deserialize, Serialize, Debug)]
pub struct SelectWidgetOption {
    pub(crate) label: String,
    pub(crate) value: String,
}
