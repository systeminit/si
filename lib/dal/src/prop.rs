use std::{
    collections::{
        HashMap,
        VecDeque,
    },
    sync::Arc,
};

use petgraph::prelude::*;
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::Value;
use si_events::{
    CasValue,
    ContentHash,
    Timestamp,
};
use si_id::ulid::Ulid;
use si_pkg::PropSpecKind;
use strum::{
    AsRefStr,
    Display,
    EnumIter,
    EnumString,
};
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    AttributePrototype,
    AttributePrototypeId,
    AttributeValueId,
    DalContext,
    Func,
    FuncBackendResponseType,
    FuncId,
    HelperError,
    InputSocketId,
    SchemaError,
    SchemaVariant,
    SchemaVariantError,
    SchemaVariantId,
    TransactionsError,
    attribute::{
        prototype::{
            AttributePrototypeError,
            argument::{
                AttributePrototypeArgument,
                AttributePrototypeArgumentError,
            },
        },
        value::default_subscription::PropSuggestion,
    },
    change_set::ChangeSetError,
    func::{
        FuncError,
        argument::{
            FuncArgument,
            FuncArgumentError,
        },
        intrinsics::IntrinsicFunc,
    },
    implement_add_edge_to,
    label_list::ToLabelList,
    layer_db_types::{
        PropContent,
        PropContentDiscriminants,
        PropContentV2,
    },
    property_editor::schema::WidgetKind,
    slow_rt,
    workspace_snapshot::{
        WorkspaceSnapshotError,
        content_address::{
            ContentAddress,
            ContentAddressDiscriminants,
        },
        edge_weight::{
            EdgeWeightKind,
            EdgeWeightKindDiscriminants,
        },
        node_weight::{
            NodeWeight,
            NodeWeightError,
            PropNodeWeight,
            traits::SiNodeWeight,
        },
        traits::prop::PropExt as _,
    },
};

pub const PROP_VERSION: PropContentDiscriminants = PropContentDiscriminants::V1;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum PropError {
    #[error("array missing child element: {0}")]
    ArrayMissingChildElement(PropId),
    #[error("attribute prototype error: {0}")]
    AttributePrototype(#[from] Box<AttributePrototypeError>),
    #[error("attribute prototype argument error: {0}")]
    AttributePrototypeArgument(#[from] Box<AttributePrototypeArgumentError>),
    #[error("path cannot include - (next element) because it will never yield a result")]
    CannotSubscribeToNextElement(PropId, String),
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("child prop of {0} not found by name: {1}")]
    ChildPropNotFoundByName(Ulid, String),
    #[error("prop {0} of kind {1} does not have an element prop")]
    ElementPropNotOnKind(PropId, PropKind),
    #[error("func error: {0}")]
    Func(#[from] Box<FuncError>),
    #[error("func argument error: {0}")]
    FuncArgument(#[from] Box<FuncArgumentError>),
    #[error("helper error: {0}")]
    Helper(#[from] HelperError),
    #[error("tokio join error: {0}")]
    JoinError(#[from] tokio::task::JoinError),
    #[error("layer db error: {0}")]
    LayerDb(#[from] si_layer_cache::LayerDbError),
    #[error("map or array {0} missing element prop")]
    MapOrArrayMissingElementProp(PropId),
    #[error("missing prototype for prop {0}")]
    MissingPrototypeForProp(PropId),
    #[error("Multiple prototypes for Prop {0}")]
    MultiplePrototypesForProp(PropId),
    #[error("node weight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("prop {0} is orphaned")]
    PropIsOrphan(PropId),
    #[error("prop not found: {0}")]
    PropNotFound(PropId),
    #[error("prop {0} has a non prop or schema variant parent")]
    PropParentInvalid(PropId),
    #[error("schema error: {0}")]
    Schema(#[from] Box<SchemaError>),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] Box<SchemaVariantError>),
    #[error("serde error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("can only set default values for scalars (string, integer, boolean), prop {0} is {1}")]
    SetDefaultForNonScalar(PropId, PropKind),
    #[error("for parent prop {0}, there is a child prop {1} that has unexpected siblings: {2:?}")]
    SingleChildPropHasUnexpectedSiblings(PropId, PropId, Vec<PropId>),
    #[error("no single child prop found for parent: {0}")]
    SingleChildPropNotFound(PropId),
    #[error("slow runtime: {0}")]
    SlowRuntimeError(#[from] slow_rt::SlowRuntimeError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("could not acquire lock: {0}")]
    TryLock(#[from] tokio::sync::TryLockError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
    #[error("workspace snapshot graph error: {0}")]
    WorkspaceSnapshotGraph(#[from] crate::workspace_snapshot::graph::WorkspaceSnapshotGraphError),
}

pub type PropResult<T> = Result<T, PropError>;

pub const SECRET_KIND_WIDGET_OPTION_LABEL: &str = "secretKind";

pub use si_frontend_mv_types::prop_schema::PropSchemaV1;
pub use si_id::PropId;

// TODO: currently we only have string values in all widget_options but we should extend this to
// support other types. However, we cannot use serde_json::Value since postcard will not
// deserialize into a serde_json::Value.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WidgetOption {
    pub label: String,
    pub value: String,
}

impl WidgetOption {
    pub fn label(&self) -> &str {
        &self.label
    }
}
pub type WidgetOptions = Vec<WidgetOption>;

/// An individual "field" within the tree of a [`SchemaVariant`](crate::SchemaVariant).
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Prop {
    /// Unique ID in the workspace for this [`Prop`].
    pub id: PropId,
    /// Create/update timestamps
    #[serde(flatten)]
    pub timestamp: Timestamp,
    /// The name of the [`Prop`].
    pub name: String,
    /// The kind of the [`Prop`].
    pub kind: PropKind,
    /// The kind of "widget" that should be used for this [`Prop`].
    pub widget_kind: WidgetKind,
    /// The configuration of the "widget".
    pub widget_options: Option<WidgetOptions>,
    /// A link to external documentation for working with this specific [`Prop`].
    pub doc_link: Option<String>,
    /// Embedded documentation for working with this specific [`Prop`].
    pub documentation: Option<String>,
    /// A toggle for whether or not the [`Prop`] should be visually hidden.
    pub hidden: bool,
    /// Props can be connected to eachother to signify that they should contain the same value
    /// This is useful for diffing the resource with the domain, to suggest actions if the real world changes
    pub refers_to_prop_id: Option<PropId>,
    /// Connected props may need a custom diff function
    pub diff_func_id: Option<FuncId>,
    /// A serialized validation format JSON object for the prop.
    pub validation_format: Option<String>,
    /// Indicates whether this prop is a valid input for a function
    pub can_be_used_as_prototype_arg: bool,
    /// Extra data for this prop that we don't need in Rust, but still need to carry through to
    /// the frontend, such as suggestions and eventually documentation, docLinks, hidden, widgetKind,
    /// widgetOptions, etc.
    /// This allows us to carry more properties through without having to thread them through
    /// all the Rust code.
    pub ui_optionals: HashMap<String, CasValue>,
}

impl From<Prop> for PropContentV2 {
    fn from(value: Prop) -> Self {
        Self {
            timestamp: value.timestamp,
            name: value.name,
            kind: value.kind,
            widget_kind: value.widget_kind,
            widget_options: value.widget_options,
            doc_link: value.doc_link,
            documentation: value.documentation,
            hidden: value.hidden,
            refers_to_prop_id: value.refers_to_prop_id,
            diff_func_id: value.diff_func_id,
            validation_format: value.validation_format,
            ui_optionals: if value.ui_optionals.is_empty() {
                None
            } else {
                Some(value.ui_optionals)
            },
        }
    }
}

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

    pub fn as_prop_suggestion_path(&self) -> String {
        format!(
            "/{}",
            self.as_parts()
                .into_iter()
                .skip(1)
                .collect::<Vec<&str>>()
                .join("/")
        )
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

    pub fn with_replaced_sep_and_prefix(&self, sep: &str) -> String {
        let mut path = self.with_replaced_sep(sep);
        path.insert_str(0, sep);
        path
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
    Hash,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum PropKind {
    Array,
    Boolean,
    Integer,
    Json,
    Map,
    Object,
    String,
    Float,
}

impl From<PropKind> for si_frontend_types::PropKind {
    fn from(value: PropKind) -> Self {
        match value {
            PropKind::Array => si_frontend_types::PropKind::Array,
            PropKind::Boolean => si_frontend_types::PropKind::Boolean,
            PropKind::Float => si_frontend_types::PropKind::Float,
            PropKind::Integer => si_frontend_types::PropKind::Integer,
            PropKind::Json => si_frontend_types::PropKind::Json,
            PropKind::Map => si_frontend_types::PropKind::Map,
            PropKind::Object => si_frontend_types::PropKind::Object,
            PropKind::String => si_frontend_types::PropKind::String,
        }
    }
}

impl PropKind {
    pub fn is_container(&self) -> bool {
        matches!(self, PropKind::Array | PropKind::Map | PropKind::Object)
    }

    pub fn ordered(&self) -> bool {
        self.is_container()
    }

    pub fn empty_value(&self) -> Option<serde_json::Value> {
        match self {
            Self::Array => Some(serde_json::json!([])),
            Self::Map | Self::Object | Self::Json => Some(serde_json::json!({})),
            _ => None,
        }
    }

    pub fn is_scalar(&self) -> bool {
        matches!(
            self,
            PropKind::String | PropKind::Boolean | PropKind::Integer | PropKind::Float
        )
    }

    /// The intrinsic function used to set a static value for this prop kind.
    pub fn intrinsic_set_func(&self) -> IntrinsicFunc {
        match self {
            PropKind::Array => IntrinsicFunc::SetArray,
            PropKind::Boolean => IntrinsicFunc::SetBoolean,
            PropKind::Integer => IntrinsicFunc::SetInteger,
            PropKind::Float => IntrinsicFunc::SetFloat,
            PropKind::Json => IntrinsicFunc::SetJson,
            PropKind::Map => IntrinsicFunc::SetMap,
            PropKind::Object => IntrinsicFunc::SetObject,
            PropKind::String => IntrinsicFunc::SetString,
        }
    }

    /// Check if the two PropKinds are both the same JavaScript type ({}, [], string, number, boolean)
    pub fn js_compatible_with(&self, other: PropKind) -> bool {
        match self {
            PropKind::Array => matches!(other, PropKind::Array),
            PropKind::Boolean => matches!(other, PropKind::Boolean),
            PropKind::Integer | PropKind::Float => {
                matches!(other, PropKind::Integer | PropKind::Float)
            }
            PropKind::Json | PropKind::String => matches!(other, PropKind::Json | PropKind::String),
            PropKind::Map | PropKind::Object => matches!(other, PropKind::Map | PropKind::Object),
        }
    }
}

impl From<PropKind> for PropSpecKind {
    fn from(prop: PropKind) -> Self {
        match prop {
            PropKind::Array => Self::Array,
            PropKind::Boolean => Self::Boolean,
            PropKind::String => Self::String,
            PropKind::Integer => Self::Number,
            PropKind::Float => Self::Float,
            PropKind::Json => PropSpecKind::Json,
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
            PropKind::Json | PropKind::String | PropKind::Integer | PropKind::Float => Self::Text,
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
            PropKind::Float => Self::Float,
            PropKind::Object => Self::Object,
            PropKind::Json => Self::Json,
            PropKind::Map => Self::Map,
            PropKind::String => Self::String,
        }
    }
}

impl Prop {
    pub async fn into_frontend_type(self, ctx: &DalContext) -> PropResult<si_frontend_types::Prop> {
        let path = self.path(ctx).await?.with_replaced_sep_and_prefix("/");
        Ok(si_frontend_types::Prop {
            id: self.id(),
            kind: self.kind.into(),
            name: self.name.to_owned(),
            path: path.to_owned(),
            hidden: self.hidden,
            eligible_to_receive_data: {
                // props can receive data if they're on a certain part of the prop tree
                // or if they're not a child of an array/map (for now?)
                let eligible_by_path = path == "/root/resource_value"
                    || path == "/root/si/color"
                    || path.starts_with("/root/domain/")
                    || path.starts_with("/root/resource_value/");
                eligible_by_path && self.can_be_used_as_prototype_arg
            },
            eligible_to_send_data: self.can_be_used_as_prototype_arg,
        })
    }

    /// Mark whether a prop can be used as an input to a function. Props below
    /// Maps and Arrays are not valid inputs. Only be used when
    /// "finalizing" a schema variant.
    pub async fn set_can_be_used_as_prototype_arg(
        ctx: &DalContext,
        prop_id: PropId,
    ) -> PropResult<()> {
        let snapshot = ctx.workspace_snapshot()?;
        let mut prop_node_weight = snapshot.get_node_weight(prop_id).await?;
        if let NodeWeight::Prop(prop_inner) = &mut prop_node_weight {
            prop_inner.set_can_be_used_as_prototype_arg(true);
        }
        snapshot.add_or_replace_node(prop_node_weight).await?;
        Ok(())
    }

    pub fn assemble(prop_node_weight: PropNodeWeight, content: PropContent) -> Self {
        // Destructure here to convince ourselves we are using all the fields
        let PropContentV2 {
            timestamp,
            name,
            kind,
            widget_kind,
            widget_options,
            doc_link,
            documentation,
            hidden,
            refers_to_prop_id,
            diff_func_id,
            validation_format,
            ui_optionals,
        } = PropContentV2::from(content);
        Self {
            id: prop_node_weight.id().into(),
            timestamp,
            name,
            kind,
            widget_kind,
            widget_options,
            doc_link,
            documentation,
            hidden,
            refers_to_prop_id,
            diff_func_id,
            validation_format,
            can_be_used_as_prototype_arg: prop_node_weight.can_be_used_as_prototype_arg(),
            ui_optionals: ui_optionals.unwrap_or_default(),
        }
    }

    /// A wrapper around [`Self::new`] that does not populate UI-relevant information. This is most
    /// useful for [`Props`](Prop) that will be invisible to the user in the property editor.
    pub async fn new_without_ui_optionals(
        ctx: &DalContext,
        name: impl Into<String>,
        kind: PropKind,
        parent_prop_id: PropId,
    ) -> PropResult<Self> {
        Self::new(
            ctx,
            name,
            kind,
            false,
            None,
            None,
            None,
            None,
            Default::default(),
            parent_prop_id,
        )
        .await
    }

    /// Creates a [`Prop`] that is a child of a provided parent [`Prop`].
    ///
    /// If you want to create the first, "root" [`Prop`] for a [`SchemaVariant`], use
    /// [`Self::new_root`].
    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        ctx: &DalContext,
        name: impl Into<String>,
        kind: PropKind,
        hidden: bool,
        doc_link: Option<String>,
        documentation: Option<String>,
        widget_kind_and_options: Option<(WidgetKind, Option<Value>)>,
        validation_format: Option<String>,
        ui_optionals: HashMap<String, CasValue>,
        parent_prop_id: PropId,
    ) -> PropResult<Self> {
        let prop = Self::new_inner(
            ctx,
            name,
            kind,
            hidden,
            doc_link,
            documentation,
            widget_kind_and_options,
            validation_format,
            ui_optionals,
        )
        .await?;

        Self::add_edge_to_prop_ordered(ctx, parent_prop_id, prop.id, EdgeWeightKind::new_use())
            .await?;

        Ok(prop)
    }

    /// Creates a root [`Prop`] for a given [`SchemaVariantId`](SchemaVariant).
    #[allow(clippy::too_many_arguments)]
    pub async fn new_root(
        ctx: &DalContext,
        name: impl Into<String>,
        kind: PropKind,
        hidden: bool,
        doc_link: Option<String>,
        documentation: Option<String>,
        widget_kind_and_options: Option<(WidgetKind, Option<Value>)>,
        validation_format: Option<String>,
        ui_optionals: HashMap<String, CasValue>,
        schema_variant_id: SchemaVariantId,
    ) -> PropResult<Self> {
        let root_prop = Self::new_inner(
            ctx,
            name,
            kind,
            hidden,
            doc_link,
            documentation,
            widget_kind_and_options,
            validation_format,
            ui_optionals,
        )
        .await?;

        SchemaVariant::add_edge_to_prop(
            ctx,
            schema_variant_id,
            root_prop.id,
            EdgeWeightKind::new_use(),
        )
        .await
        .map_err(Box::new)?;

        Ok(root_prop)
    }

    /// This _private_ method creates a new [`Prop`]. It does not handle the parentage of the prop
    /// and _public_ methods should be used to do so.
    ///
    /// A corresponding [`AttributePrototype`] and [`AttributeValue`] will be created when the
    /// provided [`SchemaVariant`] is [`finalized`](SchemaVariant::finalize).
    #[allow(clippy::too_many_arguments)]
    async fn new_inner(
        ctx: &DalContext,
        name: impl Into<String>,
        kind: PropKind,
        hidden: bool,
        doc_link: Option<String>,
        documentation: Option<String>,
        widget_kind_and_options: Option<(WidgetKind, Option<Value>)>,
        validation_format: Option<String>,
        ui_optionals: HashMap<String, CasValue>,
    ) -> PropResult<Self> {
        let ordered = kind.ordered();
        let name = name.into();

        let timestamp = Timestamp::now();
        let (widget_kind, widget_options): (WidgetKind, Option<WidgetOptions>) =
            match widget_kind_and_options {
                Some((kind, options)) => (
                    kind,
                    match options {
                        Some(options) => Some(serde_json::from_value(options)?),
                        None => None,
                    },
                ),
                None => (WidgetKind::from(kind), None),
            };

        let content = PropContent::V2(PropContentV2 {
            timestamp,
            name: name.clone(),
            kind,
            widget_kind,
            widget_options,
            doc_link,
            documentation,
            hidden,
            refers_to_prop_id: None,
            diff_func_id: None,
            validation_format,
            ui_optionals: if ui_optionals.is_empty() {
                None
            } else {
                Some(ui_optionals)
            },
        });

        let (hash, _) = ctx.layer_db().cas().write(
            Arc::new(content.clone().into()),
            None,
            ctx.events_tenancy(),
            ctx.events_actor(),
        )?;

        let workspace_snapshot = ctx.workspace_snapshot()?;
        let id = workspace_snapshot.generate_ulid().await?;
        let lineage_id = workspace_snapshot.generate_ulid().await?;
        let node_weight = NodeWeight::new_prop(id, lineage_id, kind, name, hash);
        let prop_node_weight = node_weight.get_prop_node_weight()?;

        if ordered {
            workspace_snapshot.add_ordered_node(node_weight).await?;
        } else {
            workspace_snapshot.add_or_replace_node(node_weight).await?;
        }

        Ok(Self::assemble(prop_node_weight, content))
    }

    pub fn id(&self) -> PropId {
        self.id
    }

    pub fn secret_kind_widget_option(&self) -> Option<WidgetOption> {
        self.widget_options
            .as_ref()
            .and_then(|options| {
                options
                    .iter()
                    .find(|opt| opt.label == SECRET_KIND_WIDGET_OPTION_LABEL)
            })
            .cloned()
    }

    /// Returns `Some` with the parent [`PropId`](Prop) or returns `None` if the parent is a
    /// [`SchemaVariant`].
    pub async fn parent_prop_id_by_id(
        ctx: &DalContext,
        prop_id: PropId,
    ) -> PropResult<Option<PropId>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        match workspace_snapshot
            .incoming_sources_for_edge_weight_kind(prop_id, EdgeWeightKindDiscriminants::Use)
            .await?
            .first()
        {
            Some(parent_node_idx) => Ok(
                match workspace_snapshot.get_node_weight(*parent_node_idx).await? {
                    NodeWeight::Prop(prop_inner) => Some(prop_inner.id().into()),
                    NodeWeight::Content(content_inner) => {
                        let content_addr_discrim: ContentAddressDiscriminants =
                            content_inner.content_address().into();
                        match content_addr_discrim {
                            ContentAddressDiscriminants::SchemaVariant => None,
                            _ => return Err(PropError::PropParentInvalid(prop_id)),
                        }
                    }
                    NodeWeight::SchemaVariant(_) => None,
                    _ => return Err(PropError::PropParentInvalid(prop_id)),
                },
            ),
            None => Err(PropError::PropIsOrphan(prop_id)),
        }
    }

    pub async fn direct_child_prop_ids_unordered(
        ctx: &DalContext,
        prop_id: PropId,
    ) -> PropResult<Vec<PropId>> {
        let mut result = vec![];
        let workspace_snapshot = ctx.workspace_snapshot()?;
        for (_, _, target_idx) in workspace_snapshot
            .edges_directed_for_edge_weight_kind(
                prop_id,
                Outgoing,
                EdgeWeightKindDiscriminants::Use,
            )
            .await?
        {
            let prop_node = workspace_snapshot
                .get_node_weight(target_idx)
                .await?
                .get_prop_node_weight()?;

            result.push(prop_node.id().into());
        }

        Ok(result)
    }

    /// Finds and expects a single child [`Prop`]. If zero or more than one [`Prop`] is found, an error is returned.
    ///
    /// This is most useful for maps and arrays, but can also be useful for objects with single fields
    /// (e.g. "/root/secrets" under certain scenarios).
    pub async fn direct_single_child_prop_id(
        ctx: &DalContext,
        prop_id: PropId,
    ) -> PropResult<PropId> {
        let mut direct_child_prop_ids_should_only_be_one =
            Self::direct_child_prop_ids_unordered(ctx, prop_id).await?;

        let single_child_prop_id = direct_child_prop_ids_should_only_be_one
            .pop()
            .ok_or(PropError::SingleChildPropNotFound(prop_id))?;

        if !direct_child_prop_ids_should_only_be_one.is_empty() {
            return Err(PropError::SingleChildPropHasUnexpectedSiblings(
                prop_id,
                single_child_prop_id,
                direct_child_prop_ids_should_only_be_one,
            ));
        }

        Ok(single_child_prop_id)
    }

    pub async fn path_by_id(ctx: &DalContext, prop_id: PropId) -> PropResult<PropPath> {
        let name = ctx
            .workspace_snapshot()?
            .get_node_weight(prop_id)
            .await?
            .get_prop_node_weight()?
            .name()
            .to_owned();

        let mut parts = VecDeque::from([name]);
        let mut work_queue = VecDeque::from([prop_id]);

        while let Some(prop_id) = work_queue.pop_front() {
            if let Some(prop_id) = Self::parent_prop_id_by_id(ctx, prop_id).await? {
                let workspace_snapshot = ctx.workspace_snapshot()?;

                if let NodeWeight::Prop(inner) = workspace_snapshot.get_node_weight(prop_id).await?
                {
                    parts.push_front(inner.name().to_owned());
                    work_queue.push_back(inner.id().into());
                }
            }
        }

        Ok(PropPath::new(parts))
    }

    pub async fn path(&self, ctx: &DalContext) -> PropResult<PropPath> {
        Self::path_by_id(ctx, self.id).await
    }

    ///
    /// Get all attribute values from all components associated with this prop id.
    ///
    /// NOTE: If you want a component's prop value, use
    /// `Component::attribute_values_for_prop_id()` instead.
    ///
    pub async fn all_attribute_values_everywhere_for_prop_id(
        ctx: &DalContext,
        prop_id: PropId,
    ) -> PropResult<Vec<AttributeValueId>> {
        let mut result = vec![];
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let av_sources = workspace_snapshot
            .incoming_sources_for_edge_weight_kind(prop_id, EdgeWeightKindDiscriminants::Prop)
            .await?;

        for av_source_id in av_sources {
            let av_id: AttributeValueId = workspace_snapshot
                .get_node_weight(av_source_id)
                .await?
                .get_attribute_value_node_weight()?
                .id()
                .into();

            result.push(av_id)
        }

        Ok(result)
    }

    pub async fn get_by_id(ctx: &DalContext, id: PropId) -> PropResult<Self> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let node_weight = workspace_snapshot
            .get_node_weight(id)
            .await?
            .get_prop_node_weight()?;
        let hash = node_weight.content_hash();

        let content: PropContent = ctx
            .layer_db()
            .cas()
            .try_read_as(&hash)
            .await?
            .ok_or(WorkspaceSnapshotError::MissingContentFromStore(id.into()))?;

        Ok(Self::assemble(node_weight, content))
    }

    pub async fn node_weight(ctx: &DalContext, id: PropId) -> PropResult<PropNodeWeight> {
        Ok(ctx
            .workspace_snapshot()?
            .get_node_weight(id)
            .await?
            .get_prop_node_weight()?)
    }

    pub async fn kind(ctx: &DalContext, id: PropId) -> PropResult<PropKind> {
        Ok(Self::node_weight(ctx, id).await?.kind)
    }

    pub async fn name(ctx: &DalContext, id: PropId) -> PropResult<String> {
        Ok(Self::node_weight(ctx, id).await?.name)
    }

    pub async fn element_prop_id(ctx: &DalContext, prop_id: PropId) -> PropResult<PropId> {
        Self::direct_child_prop_ids_unordered(ctx, prop_id)
            .await?
            .first()
            .copied()
            .ok_or(PropError::MapOrArrayMissingElementProp(prop_id))
    }

    pub async fn child_prop_id(
        ctx: &DalContext,
        parent_node_id: Ulid,
        child_name: impl AsRef<str>,
    ) -> PropResult<PropId> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        for prop_node_id in workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(parent_node_id, EdgeWeightKindDiscriminants::Use)
            .await?
        {
            if let NodeWeight::Prop(prop_inner) =
                workspace_snapshot.get_node_weight(prop_node_id).await?
            {
                if prop_inner.name() == child_name.as_ref() {
                    return Ok(prop_node_id.into());
                }
            }
        }

        Err(PropError::ChildPropNotFoundByName(
            parent_node_id,
            child_name.as_ref().to_string(),
        ))
    }

    /// Find the `SchemaVariantId`` for a given prop. If the prop tree is
    /// orphaned, we just return `None`
    pub async fn schema_variant_id(
        ctx: &DalContext,
        prop_id: PropId,
    ) -> PropResult<Option<SchemaVariantId>> {
        let root_prop_id = Self::root_prop_for_prop_id(ctx, prop_id).await?;
        let workspace_snapshot = ctx.workspace_snapshot()?;

        match workspace_snapshot
            .incoming_sources_for_edge_weight_kind(root_prop_id, EdgeWeightKindDiscriminants::Use)
            .await?
            .first()
        {
            Some(parent_node_idx) => {
                match workspace_snapshot.get_node_weight(*parent_node_idx).await? {
                    NodeWeight::Content(content_inner)
                        if matches!(
                            content_inner.content_address(),
                            ContentAddress::SchemaVariant(_)
                        ) =>
                    {
                        Ok(Some(content_inner.id().into()))
                    }
                    NodeWeight::SchemaVariant(schema_variant) => {
                        Ok(Some(schema_variant.id().into()))
                    }
                    _ => Err(PropError::PropParentInvalid(root_prop_id)),
                }
            }
            None => Ok(None),
        }
    }

    // Return all the parent prop ids from a given prop id
    pub async fn all_parent_prop_ids_from_prop_id(
        ctx: &DalContext,
        prop_id: PropId,
    ) -> PropResult<Vec<PropId>> {
        let mut cursor = prop_id;
        let mut result = vec![];

        while let Some(parent) = Self::parent_prop_id_by_id(ctx, cursor).await? {
            result.push(parent);
            cursor = parent;
        }
        Ok(result)
    }

    /// Walk the prop tree up, finding the root prop for the passed in `prop_id`
    pub async fn root_prop_for_prop_id(ctx: &DalContext, prop_id: PropId) -> PropResult<PropId> {
        let mut cursor = prop_id;

        while let Some(new_cursor) = Self::parent_prop_id_by_id(ctx, cursor).await? {
            cursor = new_cursor;
        }

        Ok(cursor)
    }

    pub async fn find_prop_id_by_path_opt(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
        path: &PropPath,
    ) -> PropResult<Option<PropId>> {
        match Self::find_prop_id_by_path(ctx, schema_variant_id, path).await {
            Ok(prop_id) => Ok(Some(prop_id)),
            Err(err) => match err {
                PropError::ChildPropNotFoundByName(_, _) => Ok(None),
                err => Err(err),
            },
        }
    }

    pub async fn find_prop_id_by_path(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
        path: &PropPath,
    ) -> PropResult<PropId> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let path_parts = path.as_parts();

        let mut current_id: Ulid = schema_variant_id.into();
        for part in path_parts {
            current_id = Self::child_prop_id(ctx, current_id, part).await?.into();
        }

        Ok(workspace_snapshot
            .get_node_weight(current_id)
            .await?
            .id()
            .into())
    }

    pub async fn find_prop_by_path(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
        path: &PropPath,
    ) -> PropResult<Self> {
        let prop_id = Self::find_prop_id_by_path(ctx, schema_variant_id, path).await?;
        Self::get_by_id(ctx, prop_id).await
    }

    implement_add_edge_to!(
        source_id: PropId,
        destination_id: AttributePrototypeId,
        add_fn: add_edge_to_attribute_prototype,
        discriminant: EdgeWeightKindDiscriminants::Prototype,
        result: PropResult,
    );

    implement_add_edge_to!(
        source_id: PropId,
        destination_id: PropId,
        add_fn: add_edge_to_prop,
        discriminant: EdgeWeightKindDiscriminants::Use,
        result: PropResult,
    );

    pub async fn prototypes_by_key(
        ctx: &DalContext,
        prop_id: PropId,
    ) -> PropResult<Vec<(Option<String>, AttributePrototypeId)>> {
        let mut result = vec![];
        let workspace_snapshot = ctx.workspace_snapshot()?;

        for (edge_weight, _, target_idx) in workspace_snapshot
            .edges_directed_for_edge_weight_kind(
                prop_id,
                Outgoing,
                EdgeWeightKindDiscriminants::Prototype,
            )
            .await?
        {
            if let (EdgeWeightKind::Prototype(key), Some(node_weight)) = (
                edge_weight.kind(),
                workspace_snapshot.get_node_weight(target_idx).await.ok(),
            ) {
                result.push((key.to_owned(), node_weight.id().into()))
            }
        }

        Ok(result)
    }

    pub async fn prototype_id(
        ctx: &DalContext,
        prop_id: PropId,
    ) -> PropResult<AttributePrototypeId> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let prototype_node_index = *workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(prop_id, EdgeWeightKindDiscriminants::Prototype)
            .await?
            .first()
            .ok_or(PropError::MissingPrototypeForProp(prop_id))?;

        Ok(workspace_snapshot
            .get_node_weight(prototype_node_index)
            .await?
            .id()
            .into())
    }

    pub async fn input_socket_sources(&self, ctx: &DalContext) -> PropResult<Vec<InputSocketId>> {
        let prototype_id = Self::prototype_id(ctx, self.id).await?;
        Ok(AttributePrototype::list_input_socket_sources_for_id(ctx, prototype_id).await?)
    }

    /// Is this prop set by a function that takes another prop (or socket) as an input?
    pub async fn is_set_by_dependent_function(
        ctx: &DalContext,
        prop_id: PropId,
    ) -> PropResult<bool> {
        let prototype_id = Self::prototype_id(ctx, prop_id).await?;
        let prototype_func_id = AttributePrototype::func_id(ctx, prototype_id).await?;
        Ok(Func::is_dynamic(ctx, prototype_func_id).await?)
    }

    pub async fn default_value(
        ctx: &DalContext,
        prop_id: PropId,
    ) -> PropResult<Option<serde_json::Value>> {
        let prototype_id = Self::prototype_id(ctx, prop_id).await?;
        let prototype_func_id = AttributePrototype::func_id(ctx, prototype_id).await?;
        if Func::is_dynamic(ctx, prototype_func_id).await? {
            return Ok(None);
        }

        Ok(
            if let Some(apa_id) =
                AttributePrototypeArgument::list_ids_for_prototype(ctx, prototype_id)
                    .await?
                    .first()
            {
                match AttributePrototypeArgument::static_value_by_id(ctx, *apa_id).await? {
                    Some(value) => Some(value.value),
                    _ => None,
                }
            } else {
                None
            },
        )
    }

    pub async fn set_default_value<T: Serialize>(
        ctx: &DalContext,
        prop_id: PropId,
        value: T,
    ) -> PropResult<()> {
        let value = serde_json::to_value(value)?;

        let prop = Self::get_by_id(ctx, prop_id).await?;
        if !prop.kind.is_scalar() {
            return Err(PropError::SetDefaultForNonScalar(prop_id, prop.kind));
        }

        let prototype_id = Self::prototype_id(ctx, prop_id).await?;
        let intrinsic: IntrinsicFunc = prop.kind.intrinsic_set_func();
        let intrinsic_id = Func::find_intrinsic(ctx, intrinsic).await?;
        let func_arg_id = FuncArgument::single_arg_for_func(ctx, intrinsic_id).await?;

        AttributePrototype::update_func_by_id(ctx, prototype_id, intrinsic_id).await?;

        if let Some(apa_id) =
            AttributePrototypeArgument::find_by_func_argument_id_and_attribute_prototype_id(
                ctx,
                func_arg_id,
                prototype_id,
            )
            .await?
        {
            AttributePrototypeArgument::set_static_value_source(ctx, apa_id, value).await?;
        } else {
            AttributePrototypeArgument::new_static_value(ctx, prototype_id, func_arg_id, value)
                .await?;
        };
        Ok(())
    }

    /// List [`Props`](Prop) for a given list of [`PropIds`](Prop).
    pub async fn list_content(ctx: &DalContext, prop_ids: Vec<PropId>) -> PropResult<Vec<Self>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let mut node_weights = vec![];
        let mut content_hashes = vec![];
        for prop_id in prop_ids {
            let node_weight = workspace_snapshot
                .get_node_weight(prop_id)
                .await?
                .get_prop_node_weight()?;
            content_hashes.push(node_weight.content_hash());
            node_weights.push(node_weight);
        }

        let content_map: HashMap<ContentHash, PropContent> = ctx
            .layer_db()
            .cas()
            .try_read_many_as(content_hashes.as_slice())
            .await?;

        let mut props = Vec::with_capacity(node_weights.len());
        for node_weight in node_weights {
            match content_map.get(&node_weight.content_hash()) {
                Some(content) => {
                    props.push(Self::assemble(node_weight, content.clone()));
                }
                None => Err(WorkspaceSnapshotError::MissingContentFromStore(
                    node_weight.id(),
                ))?,
            }
        }
        Ok(props)
    }

    // Gets child props, in order
    pub async fn direct_child_prop_ids_ordered(
        ctx: &DalContext,
        prop_id: PropId,
    ) -> PropResult<Vec<PropId>> {
        match ctx
            .workspace_snapshot()?
            .ordered_children_for_node(prop_id)
            .await?
        {
            Some(child_ulids) => Ok(child_ulids.into_iter().map(Into::into).collect()),
            // All props are either ordered, or have no children.
            None => Ok(vec![]),
        }
    }

    pub async fn direct_child_props_ordered(
        ctx: &DalContext,
        prop_id: PropId,
    ) -> PropResult<Vec<Prop>> {
        let child_prop_ids = Self::direct_child_prop_ids_ordered(ctx, prop_id).await?;

        let mut ordered_child_props = Vec::with_capacity(child_prop_ids.len());
        for child_prop_id in child_prop_ids {
            ordered_child_props.push(Self::get_by_id(ctx, child_prop_id).await?)
        }

        Ok(ordered_child_props)
    }

    pub async fn find_equivalent_in_schema_variant(
        ctx: &DalContext,
        prop_id: PropId,
        schema_variant_id: SchemaVariantId,
    ) -> PropResult<PropId> {
        let prop_path = Self::path_by_id(ctx, prop_id).await?;

        Self::find_prop_id_by_path(ctx, schema_variant_id, &prop_path).await
    }

    pub async fn ts_type(ctx: &DalContext, prop_id: PropId) -> PropResult<String> {
        ctx.workspace_snapshot()?.ts_type(prop_id).await
    }

    /// Get the value, formatted for debugging/display.
    pub async fn fmt_title(ctx: &DalContext, prop_id: PropId) -> String {
        Self::fmt_title_fallible(ctx, prop_id)
            .await
            .unwrap_or_else(|e| e.to_string())
    }

    async fn fmt_title_fallible(ctx: &DalContext, prop_id: PropId) -> PropResult<String> {
        Ok(Self::get_by_id(ctx, prop_id).await?.name)
    }

    pub fn suggested_sources_for(&self) -> PropResult<Vec<PropSuggestion>> {
        let Some(suggest_sources) = self.ui_optionals.get("suggestSources").cloned() else {
            return Ok(vec![]);
        };

        let suggestion_serde: serde_json::Value = suggest_sources.into();
        let suggestions: Option<Vec<PropSuggestion>> =
            serde_json::from_value(suggestion_serde).ok();
        let Some(suggestions) = suggestions else {
            return Ok(vec![]);
        };

        Ok(suggestions)
    }

    pub fn suggested_as_source_for(&self) -> PropResult<Vec<PropSuggestion>> {
        let Some(suggest_as_source_for) = self.ui_optionals.get("suggestAsSourceFor").cloned()
        else {
            return Ok(vec![]);
        };

        let suggestion_serde: serde_json::Value = suggest_as_source_for.into();
        let suggestions: Option<Vec<PropSuggestion>> =
            serde_json::from_value(suggestion_serde).ok();
        let Some(suggestions) = suggestions else {
            return Ok(vec![]);
        };

        Ok(suggestions)
    }

    /// Walk the prop trees underneath `self` and `other` and compare their types.
    pub async fn is_same_type_as(&self, ctx: &DalContext, other: &Prop) -> PropResult<bool> {
        struct PropTypeInfo {
            id: PropId,
            kind: PropKind,
            name: Option<String>,
        }
        impl From<&Prop> for PropTypeInfo {
            fn from(prop: &Prop) -> Self {
                Self {
                    id: prop.id(),
                    kind: prop.kind,
                    name: Some(prop.name.to_owned()),
                }
            }
        }

        let mut self_queue: VecDeque<PropTypeInfo> = VecDeque::from([PropTypeInfo {
            id: self.id,
            kind: self.kind,
            name: None,
        }]);

        let mut other_queue: VecDeque<PropTypeInfo> = VecDeque::from([PropTypeInfo {
            id: other.id,
            kind: other.kind,
            name: None,
        }]);

        loop {
            match (self_queue.pop_front(), other_queue.pop_front()) {
                (Some(self_prop), Some(other_prop)) => {
                    if self_prop.kind != other_prop.kind || self_prop.name != other_prop.name {
                        return Ok(false);
                    }

                    let mut self_children =
                        Prop::direct_child_props_ordered(ctx, self_prop.id).await?;
                    self_queue.reserve(self_children.len());
                    let mut other_children =
                        Prop::direct_child_props_ordered(ctx, other_prop.id).await?;
                    other_queue.reserve(other_children.len());

                    // The name of the child is only relevant for objects.
                    // Sorting by name to ensure we compare the matching child props
                    if self_prop.kind == PropKind::Object {
                        other_children.sort_by_cached_key(|prop| prop.name.to_owned());
                        self_children.sort_by_cached_key(|prop| prop.name.to_owned());
                        self_queue.extend(self_children.iter().map(Into::into));
                        other_queue.extend(other_children.iter().map(Into::into));
                    } else {
                        self_queue.extend(self_children.iter().map(|prop| PropTypeInfo {
                            id: prop.id,
                            kind: prop.kind,
                            name: None,
                        }));
                        other_queue.extend(other_children.iter().map(|prop| PropTypeInfo {
                            id: prop.id,
                            kind: prop.kind,
                            name: None,
                        }));
                    }
                }
                (None, Some(_)) | (Some(_), None) => return Ok(false),
                (None, None) => return Ok(true),
            }
        }
    }
}

impl From<AttributePrototypeError> for PropError {
    fn from(value: AttributePrototypeError) -> Self {
        Box::new(value).into()
    }
}

impl From<AttributePrototypeArgumentError> for PropError {
    fn from(value: AttributePrototypeArgumentError) -> Self {
        Box::new(value).into()
    }
}

impl From<FuncError> for PropError {
    fn from(value: FuncError) -> Self {
        Box::new(value).into()
    }
}

impl From<FuncArgumentError> for PropError {
    fn from(value: FuncArgumentError) -> Self {
        Box::new(value).into()
    }
}
