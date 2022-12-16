use std::collections::VecDeque;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use si_data_pg::PgError;
use strum_macros::{AsRefStr, Display, EnumIter, EnumString};
use telemetry::prelude::*;
use thiserror::Error;

use crate::standard_model::{object_option_from_row_option, objects_from_rows};
use crate::{
    attribute::{prototype::AttributePrototype, value::AttributeValue},
    edit_field::widget::WidgetKind,
    func::{
        binding::{FuncBinding, FuncBindingError},
        binding_return_value::FuncBindingReturnValueError,
    },
    impl_standard_model,
    label_list::ToLabelList,
    pk, standard_model, standard_model_accessor, standard_model_belongs_to,
    standard_model_has_many, standard_model_many_to_many, AttributeContext,
    AttributeContextBuilderError, AttributeReadContext, DalContext, Func, FuncId,
    HistoryEventError, InternalProvider, ReadTenancyError, SchemaVariant, SchemaVariantId,
    StandardModel, StandardModelError, Timestamp, Visibility, WriteTenancy,
};
use crate::{AttributeValueError, AttributeValueId};

const ALL_ANCESTOR_PROPS: &str = include_str!("./queries/prop_all_ancestor_props.sql");
const FIND_ROOT_FOR_SCHEMA_VARIANT: &str =
    include_str!("./queries/prop_find_root_for_schema_variant.sql");

#[derive(Error, Debug)]
pub enum PropError {
    #[error("AttributeContext error: {0}")]
    AttributeContext(#[from] AttributeContextBuilderError),
    // Can't #[from] here, or we'll end up with circular error definitions.
    #[error("AttributePrototype error: {0}")]
    AttributePrototype(String),
    #[error("AttributeValue error: {0}")]
    AttributeValue(String),
    #[error("expected child prop not found with name {0}")]
    ExpectedChildNotFound(String),
    #[error("FuncBinding error: {0}")]
    FuncBinding(#[from] FuncBindingError),
    #[error("FuncBindingReturnValue error: {0}")]
    FuncBindingReturnValue(#[from] FuncBindingReturnValueError),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("missing a func: {0}")]
    MissingFunc(String),
    #[error("missing a func by id: {0}")]
    MissingFuncById(FuncId),
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
    #[error("cannot find internal provider for prop id {0}: {1}")]
    InternalProviderNotFound(PropId, String),
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
    widget_options: Option<Value>,
    doc_link: Option<String>,
    index: i64,
    hidden: bool,
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
    /// Create a new [`Prop`](Self), which will additionally result in the creation of an
    /// [`AttributeValue`](crate::AttributeValue) in a [`Prop`](Self)-specific context.
    #[allow(clippy::too_many_arguments)]
    #[instrument(skip_all)]
    pub async fn new(
        ctx: &DalContext,
        name: impl AsRef<str>,
        kind: PropKind,
        widget_kind_and_options: Option<(WidgetKind, Value)>,
    ) -> PropResult<Self> {
        let name = name.as_ref();
        let (widget_kind, widget_options) = match widget_kind_and_options {
            Some((kind, options)) => (kind, Some(options)),
            None => (WidgetKind::from(kind), None),
        };

        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM prop_create_v1($1, $2, $3, $4, $5, $6)",
                &[
                    ctx.write_tenancy(),
                    ctx.visibility(),
                    &name,
                    &kind.as_ref(),
                    &widget_kind.as_ref(),
                    &widget_options.as_ref(),
                ],
            )
            .await?;
        let object: Prop = standard_model::finish_create_from_row(ctx, row).await?;

        Ok(object)
    }

    standard_model_accessor!(name, String, PropResult);
    standard_model_accessor!(kind, Enum(PropKind), PropResult);
    standard_model_accessor!(widget_kind, Enum(WidgetKind), PropResult);
    standard_model_accessor!(widget_options, Option<Value>, PropResult);
    standard_model_accessor!(doc_link, Option<String>, PropResult);
    standard_model_accessor!(index, i64, PropResult);
    standard_model_accessor!(hidden, bool, PropResult);

    // FIXME(nick): change the relationship to a "belongs to" relationship and the name to
    // "prop_belongs_to_schema_variant".
    standard_model_many_to_many!(
        lookup_fn: schema_variants,
        associate_fn: add_schema_variant,
        disassociate_fn: remove_schema_variant,
        disassociate_all_fn: remove_all_schema_variants,
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
        ctx: &DalContext,
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

        if let Some(attribute_value) = AttributeValue::find_for_context(ctx, attribute_read_context)
            .await
            .map_err(|e| PropError::AttributeValue(format!("{e}")))?
        {
            let parent_attribute_read_context = AttributeReadContext {
                prop_id: Some(parent_prop_id),
                ..AttributeReadContext::default()
            };
            let parent_attribute_value =
                AttributeValue::find_for_context(ctx, parent_attribute_read_context)
                    .await
                    .map_err(|e| PropError::AttributeValue(format!("{e}")))?
                    .ok_or_else(|| {
                        PropError::AttributeValue(format!(
                            "missing attribute value for context: {:?}",
                            parent_attribute_read_context
                        ))
                    })?;

            attribute_value
                .unset_parent_attribute_value(ctx)
                .await
                .map_err(|e| PropError::AttributeValue(format!("{e}")))?;
            attribute_value
                .set_parent_attribute_value(ctx, parent_attribute_value.id())
                .await
                .map_err(|e| PropError::AttributeValue(format!("{e}")))?;
        };

        self.set_parent_prop_unchecked(ctx, &parent_prop_id).await
    }

    /// Returns the root [`Prop`] for a given [`SchemaVariantId`](crate::SchemaVariant). Returns
    /// [`None`] if no [`Props`](Self) have been created for the [`SchemaVariant`](crate::SchemaVariant).
    pub async fn find_root_for_schema_variant(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> PropResult<Option<Prop>> {
        let row = ctx
            .pg_txn()
            .query_opt(
                FIND_ROOT_FOR_SCHEMA_VARIANT,
                &[ctx.read_tenancy(), ctx.visibility(), &schema_variant_id],
            )
            .await?;
        Ok(object_option_from_row_option(row)?)
    }

    /// Returns the given [`Prop`] and all ancestor [`Props`](crate::Prop) back to the root.
    /// Ancestor props are ordered by depth, starting from the root prop.
    pub async fn all_ancestor_props(ctx: &DalContext, prop_id: PropId) -> PropResult<Vec<Self>> {
        let rows = ctx
            .pg_txn()
            .query(
                ALL_ANCESTOR_PROPS,
                &[ctx.read_tenancy(), ctx.visibility(), &prop_id],
            )
            .await?;
        Ok(objects_from_rows(rows)?)
    }

    // Should JsonPointers be a type of their own?
    pub async fn json_pointer(&self, ctx: &DalContext) -> PropResult<String> {
        Ok([
            "/".to_string(),
            Prop::all_ancestor_props(ctx, *self.id())
                .await?
                .iter()
                .map(|prop| prop.name().to_string())
                .collect::<Vec<String>>()
                .join("/"),
        ]
        .join(""))
    }

    pub async fn internal_provider(
        &self,
        ctx: &DalContext,
    ) -> PropResult<Option<InternalProvider>> {
        // some custom handling here to avoid a drop-check cycle
        match InternalProvider::find_for_prop(ctx, *self.id()).await {
            Ok(maybe_ip) => Ok(maybe_ip),
            Err(err) => Err(PropError::InternalProviderNotFound(
                *self.id(),
                format!("{err}"),
            )),
        }
    }

    pub async fn create_default_prototypes_and_values(
        ctx: &DalContext,
        prop_id: PropId,
    ) -> PropResult<()> {
        #[derive(Debug)]
        struct WorkItem {
            maybe_parent: Option<AttributeValueId>,
            prop: Prop,
        }

        let mut root_prop = Prop::get_by_id(ctx, &prop_id)
            .await?
            .ok_or_else(|| PropError::NotFound(prop_id, *ctx.visibility()))?;

        // We should make sure that we're creating AttributePrototypes & AttributeValues
        // contiguously from the root.
        while let Some(parent) = root_prop.parent_prop(ctx).await? {
            root_prop = parent;
        }

        let mut work_queue: VecDeque<WorkItem> = VecDeque::from(vec![WorkItem {
            maybe_parent: None,
            prop: root_prop,
        }]);

        let func_name = "si:unset".to_string();
        let mut funcs = Func::find_by_attr(ctx, "name", &func_name).await?;
        let func = funcs.pop().ok_or(PropError::MissingFunc(func_name))?;

        // No matter what, we need a FuncBindingReturnValueId to create a new attribute prototype.
        // If the func binding was created, we execute on it to generate our value id. Otherwise,
        // we try to find a value by id and then fallback to executing anyway if one was not found.
        let (func_binding, func_binding_return_value) =
            FuncBinding::create_and_execute(ctx, serde_json::json![null], *func.id()).await?;

        while let Some(WorkItem { maybe_parent, prop }) = work_queue.pop_front() {
            let attribute_context = AttributeContext::builder()
                .set_prop_id(*prop.id())
                .to_context()?;

            let attribute_value = if let Some(attribute_value) =
                AttributeValue::find_for_context(ctx, attribute_context.into())
                    .await
                    .map_err(|e| PropError::AttributeValue(e.to_string()))?
            {
                attribute_value
            } else {
                AttributePrototype::new(
                    ctx,
                    *func.id(),
                    *func_binding.id(),
                    *func_binding_return_value.id(),
                    attribute_context,
                    None,
                    maybe_parent,
                )
                .await
                .map_err(|e| PropError::AttributePrototype(e.to_string()))?;

                AttributeValue::find_for_context(ctx, attribute_context.into())
                    .await
                    .map_err(|e| PropError::AttributeValue(e.to_string()))?
                    .ok_or(AttributeValueError::Missing)
                    .map_err(|e| PropError::AttributeValue(e.to_string()))?
            };

            if *prop.kind() == PropKind::Object {
                let child_props = prop.child_props(ctx).await?;
                if !child_props.is_empty() {
                    work_queue.extend(child_props.iter().map(|p| WorkItem {
                        maybe_parent: Some(*attribute_value.id()),
                        prop: p.clone(),
                    }));
                }
            }
        }

        Ok(())
    }
}
