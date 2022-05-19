use serde::{Deserialize, Serialize};
use si_data::PgError;
use strum_macros::{AsRefStr, Display, EnumIter, EnumString};
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    attribute::{prototype::AttributePrototype, value::AttributeValue},
    edit_field::widget::WidgetKind,
    func::{
        binding::{FuncBinding, FuncBindingError},
        binding_return_value::{FuncBindingReturnValue, FuncBindingReturnValueError},
    },
    impl_standard_model,
    label_list::ToLabelList,
    pk, standard_model, standard_model_accessor, standard_model_belongs_to,
    standard_model_has_many, standard_model_many_to_many, AttributeContext,
    AttributeContextBuilderError, AttributeReadContext, DalContext, Func, HistoryEventError,
    ReadTenancyError, SchemaVariant, SchemaVariantId, StandardModel, StandardModelError, Timestamp,
    Visibility, WriteTenancy,
};

#[derive(Error, Debug)]
pub enum PropError {
    #[error("AttributeContext error: {0}")]
    AttributeContext(#[from] AttributeContextBuilderError),
    // Can't #[from] here, or we'll end up with circular error definitions.
    #[error("AttributePrototype error: {0}")]
    AttributePrototype(String),
    #[error("AttributeValue error: {0}")]
    AttributeValue(String),
    #[error("FuncBinding error: {0}")]
    FuncBinding(#[from] FuncBindingError),
    #[error("FuncBindingReturnValue error: {0}")]
    FuncBindingReturnValue(#[from] FuncBindingReturnValueError),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("missing a func: {0}")]
    MissingFunc(String),
    #[error("prop not found: {0} ({1:?})")]
    NotFound(PropId, Visibility),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
    #[error("read tenancy error: {0}")]
    ReadTenancy(#[from] ReadTenancyError),
    #[error("cannot set parent for a non object, array, or map prop: id {0} is a {1}. Bug!")]
    ParentNotAllowed(PropId, PropKind),
}

pub type PropResult<T> = Result<T, PropError>;

pk!(PropPk);
pk!(PropId);

#[derive(
    AsRefStr,
    Clone,
    Copy,
    Debug,
    Deserialize,
    Display,
    EnumIter,
    EnumString,
    Eq,
    PartialEq,
    Serialize,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum PropKind {
    Array,
    Boolean,
    Integer,
    Object,
    String,
    Map,
}

impl ToLabelList for PropKind {}

impl From<PropKind> for WidgetKind {
    fn from(prop: PropKind) -> Self {
        match prop {
            PropKind::Array => Self::Array,
            PropKind::Boolean => Self::Checkbox,
            PropKind::String | PropKind::Integer => Self::Text,
            PropKind::Object => Self::Header,
            PropKind::Map => Self::Map,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Prop {
    pk: PropPk,
    id: PropId,
    name: String,
    kind: PropKind,
    widget_kind: WidgetKind,
    doc_link: Option<String>,
    #[serde(flatten)]
    tenancy: WriteTenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
}

impl_standard_model! {
    model: Prop,
    pk: PropPk,
    id: PropId,
    table_name: "props",
    history_event_label_base: "prop",
    history_event_message_name: "Prop"
}

impl Prop {
    #[allow(clippy::too_many_arguments)]
    #[instrument(skip_all)]
    pub async fn new(
        ctx: &DalContext<'_, '_>,
        name: impl AsRef<str>,
        kind: PropKind,
    ) -> PropResult<Self> {
        let name = name.as_ref();
        let widget_kind = WidgetKind::from(kind);
        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM prop_create_v1($1, $2, $3, $4, $5)",
                &[
                    ctx.write_tenancy(),
                    ctx.visibility(),
                    &name,
                    &kind.as_ref(),
                    &widget_kind.as_ref(),
                ],
            )
            .await?;
        let object: Prop = standard_model::finish_create_from_row(ctx, row).await?;

        let func_name = "si:unset".to_string();
        let mut funcs = Func::find_by_attr(ctx, "name", &func_name).await?;
        let func = funcs.pop().ok_or(PropError::MissingFunc(func_name))?;
        let (func_binding, created) = FuncBinding::find_or_create(
            ctx,
            serde_json::json![null],
            *func.id(),
            *func.backend_kind(),
        )
        .await?;

        let attribute_context = AttributeContext::builder()
            .set_prop_id(*object.id())
            .to_context()?;

        // No matter what, we need a FuncBindingReturnValueId to create a new attribute prototype.
        // If the func binding was created, we execute on it to generate our value id. Otherwise,
        // we try to find a value by id and then fallback to executing anyway if one was not found.
        let func_binding_return_value = if created {
            func_binding.execute(ctx).await?
        } else {
            FuncBindingReturnValue::get_by_func_binding_id_or_execute(ctx, *func_binding.id())
                .await?
        };

        AttributePrototype::new(
            ctx,
            *func.id(),
            *func_binding.id(),
            *func_binding_return_value.id(),
            attribute_context,
            None,
            None,
            None,
        )
        .await
        .map_err(|e| PropError::AttributePrototype(format!("{e}")))?;

        Ok(object)
    }

    standard_model_accessor!(name, String, PropResult);
    standard_model_accessor!(kind, Enum(PropKind), PropResult);
    standard_model_accessor!(widget_kind, Enum(WidgetKind), PropResult);
    standard_model_accessor!(doc_link, Option<String>, PropResult);

    standard_model_many_to_many!(
        lookup_fn: schema_variants,
        associate_fn: add_schema_variant,
        disassociate_fn: remove_schema_variant,
        table_name: "prop_many_to_many_schema_variants",
        left_table: "props",
        left_id: PropId,
        right_table: "schema_variants",
        right_id: SchemaVariantId,
        which_table_is_this: "left",
        returns: SchemaVariant,
        result: PropResult,
    );

    standard_model_belongs_to!(
        lookup_fn: parent_prop,
        set_fn: set_parent_prop_unchecked,
        unset_fn: unset_parent_prop,
        table: "prop_belongs_to_prop",
        model_table: "props",
        belongs_to_id: PropId,
        returns: Prop,
        result: PropResult,
    );

    standard_model_has_many!(
        lookup_fn: child_props,
        table: "prop_belongs_to_prop",
        model_table: "props",
        returns: Prop,
        result: PropResult,
    );

    /// Sets a parent for a given [`Prop`]. For the provided [`AttributeReadContext`], the [`PropId`]
    /// is ignored.
    pub async fn set_parent_prop(
        &self,
        ctx: &DalContext<'_, '_>,
        parent_prop_id: PropId,
    ) -> PropResult<()> {
        let parent_prop = Prop::get_by_id(ctx, &parent_prop_id)
            .await?
            .ok_or_else(|| PropError::NotFound(parent_prop_id, *ctx.visibility()))?;
        match parent_prop.kind() {
            PropKind::Object | PropKind::Map | PropKind::Array => (),
            kind => {
                return Err(PropError::ParentNotAllowed(parent_prop_id, *kind));
            }
        }

        let attribute_read_context = AttributeReadContext {
            prop_id: Some(*self.id()),
            ..AttributeReadContext::default()
        };
        let our_attribute_value = AttributeValue::find_for_context(ctx, attribute_read_context)
            .await
            .map_err(|e| PropError::AttributeValue(format!("{e}")))?
            .pop()
            .ok_or_else(|| {
                PropError::AttributeValue(format!(
                    "missing attribute value for context: {:?}",
                    attribute_read_context
                ))
            })?;

        let parent_attribute_read_context = AttributeReadContext {
            prop_id: Some(parent_prop_id),
            ..AttributeReadContext::default()
        };
        let parent_attribute_value =
            AttributeValue::find_for_context(ctx, parent_attribute_read_context)
                .await
                .map_err(|e| PropError::AttributeValue(format!("{e}")))?
                .pop()
                .ok_or_else(|| {
                    PropError::AttributeValue(format!(
                        "missing attribute value for context: {:?}",
                        parent_attribute_read_context
                    ))
                })?;

        our_attribute_value
            .unset_parent_attribute_value(ctx)
            .await
            .map_err(|e| PropError::AttributeValue(format!("{e}")))?;
        our_attribute_value
            .set_parent_attribute_value(ctx, parent_attribute_value.id())
            .await
            .map_err(|e| PropError::AttributeValue(format!("{e}")))?;

        self.set_parent_prop_unchecked(ctx, &parent_prop_id).await
    }
}
