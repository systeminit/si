use async_recursion::async_recursion;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use si_data_pg::PgError;
use si_pkg::PropSpecKind;
use std::collections::VecDeque;
use strum::{AsRefStr, Display, EnumIter, EnumString};
use telemetry::prelude::*;
use thiserror::Error;

use crate::standard_model::{
    finish_create_from_row, object_option_from_row_option, objects_from_rows,
};
use crate::{
    attribute::{prototype::AttributePrototype, value::AttributeValue},
    func::{
        binding::{FuncBinding, FuncBindingError},
        binding_return_value::FuncBindingReturnValueError,
    },
    impl_standard_model,
    label_list::ToLabelList,
    pk,
    property_editor::schema::WidgetKind,
    standard_model, standard_model_accessor, standard_model_belongs_to, standard_model_has_many,
    AttributeContext, AttributeContextBuilder, AttributeContextBuilderError,
    AttributePrototypeError, AttributeReadContext, DalContext, Func, FuncError, FuncId,
    HistoryEventError, SchemaVariantId, StandardModel, StandardModelError, Tenancy, Timestamp,
    Visibility,
};
use crate::{AttributeValueError, AttributeValueId, FuncBackendResponseType, TransactionsError};

/// This is the separator used for the "path" column. It is a vertical tab character, which should
/// not (we'll see) be able to be provided by our users in [`Prop`] names.
pub const PROP_PATH_SEPARATOR: &str = "\x0B";

/// This type should be used to manage prop paths instead of a raw string
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PropPath(String);

impl PropPath {
    pub fn new<S>(parts: impl IntoIterator<Item = S>) -> Self
    where
        S: AsRef<str>,
    {
        Self(
            parts
                .into_iter()
                .map(|part| part.as_ref().to_owned())
                .collect::<Vec<String>>()
                .join(PROP_PATH_SEPARATOR),
        )
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn as_parts(&self) -> Vec<&str> {
        self.0.split(PROP_PATH_SEPARATOR).collect()
    }

    pub fn as_owned_parts(&self) -> Vec<String> {
        self.0.split(PROP_PATH_SEPARATOR).map(Into::into).collect()
    }

    pub fn join(&self, path: &PropPath) -> Self {
        Self::new([self.as_str(), path.as_str()])
    }

    pub fn with_replaced_sep(&self, sep: &str) -> String {
        self.0.to_owned().replace(PROP_PATH_SEPARATOR, sep)
    }

    /// Returns true if this PropPath is a descendant (at any depth) of `maybe_parent`
    pub fn is_descendant_of(&self, maybe_parent: &PropPath) -> bool {
        let this_parts = self.as_parts();
        let maybe_parent_parts = maybe_parent.as_parts();

        for (idx, parent_part) in maybe_parent_parts.iter().enumerate() {
            if Some(parent_part) != this_parts.get(idx) {
                return false;
            }
        }

        true
    }
}

impl AsRef<str> for PropPath {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::fmt::Display for PropPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<PropPath> for String {
    fn from(value: PropPath) -> Self {
        value.0
    }
}

impl From<&String> for PropPath {
    fn from(value: &String) -> Self {
        Self(value.to_owned())
    }
}

impl From<String> for PropPath {
    fn from(value: String) -> Self {
        Self(value)
    }
}

const ALL_ANCESTOR_PROPS: &str = include_str!("queries/prop/all_ancestor_props.sql");
const FIND_ROOT_PROP_FOR_PROP: &str = include_str!("queries/prop/root_prop_for_prop.sql");
const FIND_PROP_IN_TREE: &str = include_str!("queries/prop/find_prop_in_tree.sql");

#[remain::sorted]
#[derive(Error, Debug)]
pub enum PropError {
    #[error("Array prop {0} is missing element child")]
    ArrayMissingElementChild(PropId),
    #[error("AttributeContext error: {0}")]
    AttributeContext(#[from] AttributeContextBuilderError),
    #[error("AttributePrototype error: {0}")]
    AttributePrototype(#[from] AttributePrototypeError),
    #[error("AttributeValue error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    #[error("default diff function not found")]
    DefaultDiffFunctionNotFound,
    #[error("expected child prop not found with name {0}")]
    ExpectedChildNotFound(String),
    #[error("Func error: {0}")]
    Func(#[from] FuncError),
    #[error("FuncBinding error: {0}")]
    FuncBinding(#[from] FuncBindingError),
    #[error("FuncBindingReturnValue error: {0}")]
    FuncBindingReturnValue(#[from] FuncBindingReturnValueError),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("Map prop {0} is missing element child")]
    MapMissingElementChild(PropId),
    #[error("missing a func: {0}")]
    MissingFunc(String),
    #[error("missing a func by id: {0}")]
    MissingFuncById(FuncId),
    #[error("prop not found: {0} ({1:?})")]
    NotFound(PropId, Visibility),
    #[error("prop not found at path: {0} {1:?}")]
    NotFoundAtPath(String, Visibility),
    #[error("parent prop kind is not \"Object\", which is required for setting default values on props (found {0})")]
    ParentPropIsNotObjectForPropWithDefaultValue(PropKind),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error("unable to set default value for non scalar prop type")]
    SetDefaultForNonScalar(PropKind),
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
}

pub type PropResult<T> = Result<T, PropError>;

pk!(PropPk);
pk!(PropId);

#[remain::sorted]
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
    Map,
    Object,
    String,
}

impl From<PropKind> for PropSpecKind {
    fn from(prop: PropKind) -> Self {
        match prop {
            PropKind::Array => Self::Array,
            PropKind::Boolean => Self::Boolean,
            PropKind::String => Self::String,
            PropKind::Integer => Self::Number,
            PropKind::Object => Self::Object,
            PropKind::Map => Self::Map,
        }
    }
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

impl From<PropKind> for FuncBackendResponseType {
    fn from(prop: PropKind) -> Self {
        match prop {
            PropKind::Array => Self::Array,
            PropKind::Boolean => Self::Boolean,
            PropKind::Integer => Self::Integer,
            PropKind::Object => Self::Object,
            PropKind::Map => Self::Map,
            PropKind::String => Self::String,
        }
    }
}

/// An individual "field" within the tree of a [`SchemaVariant`](crate::SchemaVariant).
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Prop {
    pk: PropPk,
    id: PropId,
    #[serde(flatten)]
    tenancy: Tenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,

    /// The name of the [`Prop`].
    name: String,
    /// The kind of the [`Prop`].
    kind: PropKind,
    /// The kind of "widget" that should be used for this [`Prop`].
    widget_kind: WidgetKind,
    /// The configuration of the "widget".
    widget_options: Option<Value>,
    /// A link to external documentation for working with this specific [`Prop`].
    doc_link: Option<String>,
    /// Embedded documentation for working with this specific [`Prop`].
    documentation: Option<String>,
    /// A toggle for whether or not the [`Prop`] should be visually hidden.
    hidden: bool,
    /// The "path" for a given [`Prop`]. It is a concatenation of [`Prop`] names based on lineage
    /// with [`PROP_PATH_SEPARATOR`] as the separator between each parent and child.
    ///
    /// This is useful for finding and querying for specific [`Props`](Prop) in a
    /// [`SchemaVariant`](crate::SchemaVariant)'s tree.
    path: String,
    /// The [`SchemaVariant`](crate::SchemaVariant) whose tree we (the [`Prop`]) reside in.
    schema_variant_id: SchemaVariantId,
    /// Props can be connected to eachother to signify that they should contain the same value
    /// This is useful for diffing the resource with the domain, to suggest actions if the real world changes
    refers_to_prop_id: Option<PropId>,
    /// Connected props may need a custom diff function
    diff_func_id: Option<FuncId>,
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
    /// Create a new [`Prop`]. A corresponding [`AttributePrototype`] and [`AttributeValue`] will be
    /// created when the provided [`SchemaVariant`](crate::SchemaVariant) is
    /// [`finalized`](crate::SchemaVariant::finalize).
    #[instrument(skip_all)]
    pub async fn new(
        ctx: &DalContext,
        name: impl AsRef<str>,
        kind: PropKind,
        widget_kind_and_options: Option<(WidgetKind, Option<Value>)>,
        schema_variant_id: SchemaVariantId,
        parent_prop_id: Option<PropId>,
        documentation: Option<String>,
    ) -> PropResult<Self> {
        let name = name.as_ref();
        let (widget_kind, widget_options) = match widget_kind_and_options {
            Some((kind, options)) => (kind, options),
            None => (WidgetKind::from(kind), None),
        };

        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "SELECT object FROM prop_create_v2($1, $2, $3, $4, $5, $6, $7, $8, $9)",
                &[
                    ctx.tenancy(),
                    ctx.visibility(),
                    &name,
                    &kind.as_ref(),
                    &widget_kind.as_ref(),
                    &widget_options.as_ref(),
                    &schema_variant_id,
                    &parent_prop_id,
                    &documentation,
                ],
            )
            .await?;
        Ok(finish_create_from_row(ctx, row).await?)
    }

    standard_model_accessor!(name, String, PropResult);
    standard_model_accessor!(kind, Enum(PropKind), PropResult);
    standard_model_accessor!(widget_kind, Enum(WidgetKind), PropResult);
    standard_model_accessor!(widget_options, Option<Value>, PropResult);
    standard_model_accessor!(doc_link, Option<String>, PropResult);
    standard_model_accessor!(documentation, Option<String>, PropResult);
    standard_model_accessor!(hidden, bool, PropResult);
    standard_model_accessor!(refers_to_prop_id, Option<Pk(PropId)>, PropResult);
    standard_model_accessor!(diff_func_id, Option<Pk(FuncId)>, PropResult);
    standard_model_accessor!(schema_variant_id, Pk(SchemaVariantId), PropResult);

    pub fn path(&self) -> PropPath {
        self.path.to_owned().into()
    }

    // TODO(nick): replace this table with a foreign key relationship.
    standard_model_belongs_to!(
        lookup_fn: parent_prop,
        set_fn: set_parent_prop_do_not_use,
        unset_fn: unset_parent_prop_do_not_use,
        table: "prop_belongs_to_prop",
        model_table: "props",
        belongs_to_id: PropId,
        returns: Prop,
        result: PropResult,
    );

    // TODO(nick): replace this table with a foreign key relationship.
    standard_model_has_many!(
        lookup_fn: child_props,
        table: "prop_belongs_to_prop",
        model_table: "props",
        returns: Prop,
        result: PropResult,
    );

    pub async fn find_root_prop_for_prop(
        ctx: &DalContext,
        prop_id: PropId,
    ) -> PropResult<Option<Self>> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_opt(
                FIND_ROOT_PROP_FOR_PROP,
                &[ctx.tenancy(), ctx.visibility(), &prop_id],
            )
            .await?;

        Ok(standard_model::object_option_from_row_option::<Self>(row)?)
    }

    /// Returns the given [`Prop`] and all ancestor [`Props`](crate::Prop) back to the root.
    /// Ancestor props are ordered by depth, starting from the root prop.
    pub async fn all_ancestor_props(ctx: &DalContext, prop_id: PropId) -> PropResult<Vec<Self>> {
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(
                ALL_ANCESTOR_PROPS,
                &[ctx.tenancy(), ctx.visibility(), &prop_id],
            )
            .await?;
        Ok(objects_from_rows(rows)?)
    }

    #[instrument(skip_all)]
    #[async_recursion]
    pub async fn ts_type(&self, ctx: &DalContext) -> PropResult<String> {
        // XXX: Hack! The payload prop kind is a string but we're actually storing arbitrary json
        // there and expect it to be a JSON object in most of our code. However, the resource_value
        // work is likely to remove the need for this entirely
        if self.path() == PropPath::new(["root", "resource", "payload"]) {
            return Ok("any".to_string());
        }

        // Note: we should fix this by having propper enums as prop types
        if self.path() == PropPath::new(["root", "resource", "status"]) {
            return Ok("'ok' | 'warning' | 'error' | undefined | null".to_owned());
        }

        Ok(match self.kind() {
            PropKind::Array => format!(
                "{}[]",
                self.child_props(ctx)
                    .await?
                    .get(0)
                    .ok_or(PropError::ArrayMissingElementChild(self.id))?
                    .ts_type(ctx)
                    .await?
            ),
            PropKind::Boolean => "boolean".to_string(),
            PropKind::Integer => "number".to_string(),
            PropKind::Map => format!(
                "Record<string, {}>",
                self.child_props(ctx)
                    .await?
                    .get(0)
                    .ok_or(PropError::MapMissingElementChild(self.id))?
                    .ts_type(ctx)
                    .await?
            ),
            PropKind::Object => {
                let mut object_type = "{\n".to_string();
                for child in self.child_props(ctx).await? {
                    let name_value = serde_json::to_value(&child.name)?;
                    let name_serialized = serde_json::to_string(&name_value)?;
                    object_type.push_str(
                        format!(
                            "{}: {} | null | undefined;\n",
                            &name_serialized,
                            child.ts_type(ctx).await?
                        )
                        .as_str(),
                    );
                }
                object_type.push('}');

                object_type
            }
            PropKind::String => "string".to_string(),
        })
    }

    /// Assembles the "json_pointer" representing the full "path" to a [`Prop`] based on its
    /// lineage.
    ///
    /// For examples, if a [`Prop`] named "poop" had a parent named "domain" and a grandparent named
    /// "root", then the "json_pointer" would be "/root/domain/poop".
    pub async fn json_pointer(&self, ctx: &DalContext) -> PropResult<String> {
        // NOTE(nick,zack): if this ends up getting used frequently to manage paths corresponding
        // to attribute (and/or property editor) values, then we should consider strongly typing
        // "json_pointer".
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

    /// Finds a prop by a path made up of prop names separated by
    /// [`PROP_PATH_SEPARATOR`](crate::prop::PROP_PATH_SEPARATOR) for each depth level
    pub async fn find_prop_by_path(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
        path: &PropPath,
    ) -> PropResult<Self> {
        Self::find_prop_by_path_opt(ctx, schema_variant_id, path)
            .await?
            .ok_or(PropError::NotFoundAtPath(
                path.to_string(),
                *ctx.visibility(),
            ))
    }

    /// Finds a prop by a path made up of prop names separated by
    /// [`PROP_PATH_SEPARATOR`](crate::prop::PROP_PATH_SEPARATOR) for each depth level
    pub async fn find_prop_by_path_opt(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
        path: &PropPath,
    ) -> PropResult<Option<Self>> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_opt(
                FIND_PROP_IN_TREE,
                &[
                    ctx.tenancy(),
                    ctx.visibility(),
                    &schema_variant_id,
                    &path.as_str(),
                ],
            )
            .await?;

        Ok(object_option_from_row_option(row)?)
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
            FuncBinding::create_and_execute(ctx, serde_json::json![null], *func.id(), vec![])
                .await?;

        while let Some(WorkItem { maybe_parent, prop }) = work_queue.pop_front() {
            let attribute_context = AttributeContext::builder()
                .set_prop_id(*prop.id())
                .to_context()?;

            let attribute_value = if let Some(attribute_value) =
                AttributeValue::find_for_context(ctx, attribute_context.into()).await?
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
                .await?;

                AttributeValue::find_for_context(ctx, attribute_context.into())
                    .await?
                    .ok_or(AttributeValueError::NotFoundForReadContext(
                        attribute_context.into(),
                    ))?
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

    pub async fn set_default_value<T: Serialize>(
        &self,
        ctx: &DalContext,
        value: T,
    ) -> PropResult<()> {
        let value = serde_json::to_value(value)?;
        match self.kind() {
            PropKind::String | PropKind::Boolean | PropKind::Integer => {
                let attribute_read_context = AttributeReadContext::default_with_prop(self.id);
                let attribute_value = AttributeValue::find_for_context(ctx, attribute_read_context)
                    .await?
                    .ok_or(AttributeValueError::NotFoundForReadContext(
                        attribute_read_context,
                    ))?;
                let parent_attribute_value = attribute_value
                    .parent_attribute_value(ctx)
                    .await?
                    .ok_or_else(|| AttributeValueError::ParentNotFound(*attribute_value.id()))?;

                // Ensure the parent project is an object. Technically, we should ensure that every
                // prop in entire lineage is of kind object, but this should (hopefully) suffice
                // for now. Ideally, this would be handled in a query.
                let parent_prop = Prop::get_by_id(ctx, &parent_attribute_value.context.prop_id())
                    .await?
                    .ok_or_else(|| {
                        PropError::NotFound(
                            parent_attribute_value.context.prop_id(),
                            *ctx.visibility(),
                        )
                    })?;
                if parent_prop.kind() != &PropKind::Object {
                    return Err(PropError::ParentPropIsNotObjectForPropWithDefaultValue(
                        *parent_prop.kind(),
                    ));
                }

                let context = AttributeContextBuilder::from(attribute_read_context).to_context()?;
                AttributeValue::update_for_context(
                    ctx,
                    *attribute_value.id(),
                    Some(*parent_attribute_value.id()),
                    context,
                    Some(value),
                    None,
                )
                .await?;
                Ok(())
            }
            _ => Err(PropError::SetDefaultForNonScalar(*self.kind())),
        }
    }

    pub async fn set_default_diff(&mut self, ctx: &DalContext) -> PropResult<()> {
        let func = Func::find_by_attr(ctx, "name", &"si:diff")
            .await?
            .pop()
            .ok_or(PropError::DefaultDiffFunctionNotFound)?;
        self.set_diff_func_id(ctx, Some(*func.id())).await
    }
}
