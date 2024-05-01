use petgraph::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use si_events::ContentHash;
use si_pkg::PropSpecKind;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use strum::{AsRefStr, Display, EnumIter, EnumString};
use telemetry::prelude::*;
use thiserror::Error;

use crate::attribute::prototype::argument::{
    AttributePrototypeArgument, AttributePrototypeArgumentError,
};
use crate::attribute::prototype::AttributePrototypeError;
use crate::change_set::ChangeSetError;
use crate::func::argument::{FuncArgument, FuncArgumentError};
use crate::func::intrinsics::IntrinsicFunc;
use crate::func::FuncError;
use crate::layer_db_types::{PropContent, PropContentDiscriminants, PropContentV1};
use crate::workspace_snapshot::content_address::ContentAddressDiscriminants;
use crate::workspace_snapshot::edge_weight::EdgeWeightKind;
use crate::workspace_snapshot::edge_weight::{EdgeWeightError, EdgeWeightKindDiscriminants};
use crate::workspace_snapshot::node_weight::{NodeWeight, NodeWeightError, PropNodeWeight};
use crate::workspace_snapshot::WorkspaceSnapshotError;
use crate::{
    implement_add_edge_to, label_list::ToLabelList, pk, property_editor::schema::WidgetKind,
    AttributePrototype, AttributePrototypeId, DalContext, Func, FuncBackendResponseType, FuncId,
    HelperError, SchemaVariant, SchemaVariantError, SchemaVariantId, Timestamp, TransactionsError,
};
use crate::{AttributeValueId, InputSocketId};

pub const PROP_VERSION: PropContentDiscriminants = PropContentDiscriminants::V1;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum PropError {
    #[error("attribute prototype error: {0}")]
    AttributePrototype(#[from] AttributePrototypeError),
    #[error("attribute prototype argument error: {0}")]
    AttributePrototypeArgument(#[from] AttributePrototypeArgumentError),
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("child prop of {0:?} not found by name: {1}")]
    ChildPropNotFoundByName(NodeIndex, String),
    #[error("edge weight error: {0}")]
    EdgeWeight(#[from] EdgeWeightError),
    #[error("prop {0} of kind {1} does not have an element prop")]
    ElementPropNotOnKind(PropId, PropKind),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("func argument error: {0}")]
    FuncArgument(#[from] FuncArgumentError),
    #[error("helper error: {0}")]
    Helper(#[from] HelperError),
    #[error("layer db error: {0}")]
    LayerDb(#[from] si_layer_cache::LayerDbError),
    #[error("map or array {0} missing element prop")]
    MapOrArrayMissingElementProp(PropId),
    #[error("missing prototype for prop {0}")]
    MissingPrototypeForProp(PropId),
    #[error("node weight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("prop {0} is orphaned")]
    PropIsOrphan(PropId),
    #[error("prop {0} has a non prop or schema variant parent")]
    PropParentInvalid(PropId),
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
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("could not acquire lock: {0}")]
    TryLock(#[from] tokio::sync::TryLockError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

pub type PropResult<T> = Result<T, PropError>;

pk!(PropId);

// TODO: currently we only have string values in all widget_options but we should extend this to
// support other types. However, we cannot use serde_json::Value since postcard will not
// deserialize into a serde_json::Value.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WidgetOption {
    label: String,
    pub value: String,
}
pub type WidgetOptions = Vec<WidgetOption>;

/// An individual "field" within the tree of a [`SchemaVariant`](crate::SchemaVariant).
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Prop {
    pub id: PropId,
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
}

impl From<Prop> for PropContentV1 {
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
    Json,
    Map,
    Object,
    String,
}

impl PropKind {
    pub fn ordered(&self) -> bool {
        matches!(self, PropKind::Array | PropKind::Map | PropKind::Object)
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
            PropKind::String | PropKind::Boolean | PropKind::Integer
        )
    }
}

impl From<PropKind> for PropSpecKind {
    fn from(prop: PropKind) -> Self {
        match prop {
            PropKind::Array => Self::Array,
            PropKind::Boolean => Self::Boolean,
            PropKind::String => Self::String,
            PropKind::Integer => Self::Number,
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
            PropKind::Json | PropKind::String | PropKind::Integer => Self::Text,
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
            PropKind::Json => Self::Json,
            PropKind::Map => Self::Map,
            PropKind::String => Self::String,
        }
    }
}

impl Prop {
    pub fn assemble(prop_node_weight: PropNodeWeight, inner: PropContentV1) -> Self {
        Self {
            id: prop_node_weight.id().into(),
            timestamp: inner.timestamp,
            name: inner.name,
            kind: inner.kind,
            widget_kind: inner.widget_kind,
            widget_options: inner.widget_options,
            doc_link: inner.doc_link,
            documentation: inner.documentation,
            hidden: inner.hidden,
            refers_to_prop_id: inner.refers_to_prop_id,
            diff_func_id: inner.diff_func_id,
            validation_format: inner.validation_format,
            can_be_used_as_prototype_arg: prop_node_weight.can_be_used_as_prototype_arg(),
        }
    }

    /// A wrapper around [`Self::new`] that does not populate UI-relevant information. This is most
    /// useful for [`Props`](Prop) that will be invisible to the user in the property editor.
    pub async fn new_without_ui_optionals(
        ctx: &DalContext,
        name: impl AsRef<str>,
        kind: PropKind,
        parent_prop_id: PropId,
    ) -> PropResult<Self> {
        Self::new(
            ctx,
            name.as_ref(),
            kind,
            false,
            None,
            None,
            None,
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
        widget_kind_and_options: Option<(WidgetKind, Option<Value>)>,
        validation_format: Option<String>,
        parent_prop_id: PropId,
    ) -> PropResult<Self> {
        let prop = Self::new_inner(
            ctx,
            name,
            kind,
            hidden,
            doc_link,
            widget_kind_and_options,
            validation_format,
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
        widget_kind_and_options: Option<(WidgetKind, Option<Value>)>,
        validation_format: Option<String>,
        schema_variant_id: SchemaVariantId,
    ) -> PropResult<Self> {
        let root_prop = Self::new_inner(
            ctx,
            name,
            kind,
            hidden,
            doc_link,
            widget_kind_and_options,
            validation_format,
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
    async fn new_inner(
        ctx: &DalContext,
        name: impl Into<String>,
        kind: PropKind,
        hidden: bool,
        doc_link: Option<String>,
        widget_kind_and_options: Option<(WidgetKind, Option<Value>)>,
        validation_format: Option<String>,
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

        let content = PropContentV1 {
            timestamp,
            name: name.clone(),
            kind,
            widget_kind,
            widget_options,
            doc_link,
            documentation: None,
            hidden,
            refers_to_prop_id: None,
            diff_func_id: None,
            validation_format,
        };

        let (hash, _) = ctx
            .layer_db()
            .cas()
            .write(
                Arc::new(PropContent::V1(content.clone()).into()),
                None,
                ctx.events_tenancy(),
                ctx.events_actor(),
            )
            .await?;

        let change_set = ctx.change_set()?;
        let id = change_set.generate_ulid()?;
        let node_weight = NodeWeight::new_prop(change_set, id, kind, name, hash)?;
        let prop_node_weight = node_weight.get_prop_node_weight()?;

        let workspace_snapshot = ctx.workspace_snapshot()?;
        if ordered {
            workspace_snapshot
                .add_ordered_node(change_set, node_weight)
                .await?;
        } else {
            workspace_snapshot.add_node(node_weight).await?;
        }

        Ok(Self::assemble(prop_node_weight, content))
    }

    pub fn id(&self) -> PropId {
        self.id
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
                    _ => return Err(PropError::PropParentInvalid(prop_id)),
                },
            ),
            None => Err(PropError::PropIsOrphan(prop_id)),
        }
    }

    pub async fn direct_child_prop_ids(
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
            Self::direct_child_prop_ids(ctx, prop_id).await?;

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
            .get_node_weight_by_id(prop_id)
            .await?
            .get_prop_node_weight()?
            .name()
            .to_owned();

        let mut parts = VecDeque::from([name]);
        let mut work_queue = VecDeque::from([prop_id]);

        while let Some(prop_id) = work_queue.pop_front() {
            if let Some(prop_id) = Prop::parent_prop_id_by_id(ctx, prop_id).await? {
                let workspace_snapshot = ctx.workspace_snapshot()?;
                let node_idx = workspace_snapshot.get_node_index_by_id(prop_id).await?;

                if let NodeWeight::Prop(inner) =
                    workspace_snapshot.get_node_weight(node_idx).await?
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

    pub async fn attribute_values_for_prop_id(
        ctx: &DalContext,
        prop_id: PropId,
    ) -> PropResult<Vec<AttributeValueId>> {
        let mut result = vec![];
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let av_sources = workspace_snapshot
            .incoming_sources_for_edge_weight_kind(prop_id, EdgeWeightKindDiscriminants::Prop)
            .await?;

        for av_source_idx in av_sources {
            let av_id: AttributeValueId = workspace_snapshot
                .get_node_weight(av_source_idx)
                .await?
                .get_attribute_value_node_weight()?
                .id()
                .into();

            result.push(av_id)
        }

        Ok(result)
    }

    pub async fn get_by_id_or_error(ctx: &DalContext, id: PropId) -> PropResult<Self> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let ulid: ::si_events::ulid::Ulid = id.into();
        let node_index = workspace_snapshot.get_node_index_by_id(ulid).await?;
        let node_weight = workspace_snapshot
            .get_node_weight(node_index)
            .await?
            .get_prop_node_weight()?;
        let hash = node_weight.content_hash();

        let content: PropContent = ctx
            .layer_db()
            .cas()
            .try_read_as(&hash)
            .await?
            .ok_or(WorkspaceSnapshotError::MissingContentFromStore(ulid))?;

        // NOTE(nick,jacob,zack): if we had a v2, then there would be migration logic here.
        let PropContent::V1(inner) = content;

        Ok(Prop::assemble(node_weight, inner))
    }

    pub async fn element_prop_id(&self, ctx: &DalContext) -> PropResult<PropId> {
        if !matches!(self.kind, PropKind::Array | PropKind::Map) {
            return Err(PropError::ElementPropNotOnKind(self.id, self.kind));
        }

        let workspace_snapshot = ctx.workspace_snapshot()?;
        for maybe_elem_node_idx in workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(self.id, EdgeWeightKind::new_use().into())
            .await?
        {
            if let NodeWeight::Prop(prop_inner) = workspace_snapshot
                .get_node_weight(maybe_elem_node_idx)
                .await?
            {
                return Ok(prop_inner.id().into());
            }
        }

        Err(PropError::MapOrArrayMissingElementProp(self.id))
    }

    pub async fn find_child_prop_index_by_name(
        ctx: &DalContext,
        node_index: NodeIndex,
        child_name: impl AsRef<str>,
    ) -> PropResult<NodeIndex> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        for prop_node_index in workspace_snapshot
            .outgoing_targets_for_edge_weight_kind_by_index(
                node_index,
                EdgeWeightKindDiscriminants::Use,
            )
            .await?
        {
            if let NodeWeight::Prop(prop_inner) =
                workspace_snapshot.get_node_weight(prop_node_index).await?
            {
                if prop_inner.name() == child_name.as_ref() {
                    return Ok(prop_node_index);
                }
            }
        }

        Err(PropError::ChildPropNotFoundByName(
            node_index,
            child_name.as_ref().to_string(),
        ))
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

        let schema_variant_node_index = workspace_snapshot
            .get_node_index_by_id(schema_variant_id)
            .await?;

        let path_parts = path.as_parts();

        let mut current_node_index = schema_variant_node_index;
        for part in path_parts {
            current_node_index =
                Self::find_child_prop_index_by_name(ctx, current_node_index, part).await?;
        }

        Ok(workspace_snapshot
            .get_node_weight(current_node_index)
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
        Self::get_by_id_or_error(ctx, prop_id).await
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

        Ok(Func::get_by_id(ctx, prototype_func_id)
            .await?
            .map(|f| f.is_dynamic())
            .unwrap_or(false))
    }

    pub async fn set_default_value<T: Serialize>(
        ctx: &DalContext,
        prop_id: PropId,
        value: T,
    ) -> PropResult<()> {
        let value = serde_json::to_value(value)?;

        let prop = Prop::get_by_id_or_error(ctx, prop_id).await?;
        if !prop.kind.is_scalar() {
            return Err(PropError::SetDefaultForNonScalar(prop_id, prop.kind));
        }

        let prototype_id = Prop::prototype_id(ctx, prop_id).await?;
        let intrinsic: IntrinsicFunc = prop.kind.into();
        let intrinsic_id = Func::find_intrinsic(ctx, intrinsic).await?;
        let func_arg_id = *FuncArgument::list_ids_for_func(ctx, intrinsic_id)
            .await?
            .first()
            .ok_or(FuncArgumentError::IntrinsicMissingFuncArgumentEdge(
                intrinsic.name().into(),
                intrinsic_id,
            ))?;

        AttributePrototype::update_func_by_id(ctx, prototype_id, intrinsic_id).await?;
        AttributePrototypeArgument::new(ctx, prototype_id, func_arg_id)
            .await?
            .set_value_from_static_value(ctx, value)
            .await?;

        Ok(())
    }

    /// List [`Props`](Prop) for a given list of [`PropIds`](Prop).
    pub async fn list_content(ctx: &DalContext, prop_ids: Vec<PropId>) -> PropResult<Vec<Self>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let mut node_weights = vec![];
        let mut content_hashes = vec![];
        for prop_id in prop_ids {
            let prop_node_index = workspace_snapshot.get_node_index_by_id(prop_id).await?;
            let node_weight = workspace_snapshot
                .get_node_weight(prop_node_index)
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

        let mut props = Vec::new();
        for node_weight in node_weights {
            match content_map.get(&node_weight.content_hash()) {
                Some(content) => {
                    // NOTE(nick,jacob,zack): if we had a v2, then there would be migration logic here.
                    let PropContent::V1(inner) = content;

                    props.push(Self::assemble(node_weight, inner.to_owned()));
                }
                None => Err(WorkspaceSnapshotError::MissingContentFromStore(
                    node_weight.id(),
                ))?,
            }
        }
        Ok(props)
    }

    pub async fn modify<L>(self, ctx: &DalContext, lambda: L) -> PropResult<Self>
    where
        L: FnOnce(&mut Self) -> PropResult<()>,
    {
        let mut prop = self;

        let before = PropContentV1::from(prop.clone());
        lambda(&mut prop)?;
        let updated = PropContentV1::from(prop.clone());

        if updated != before {
            let (hash, _) = ctx
                .layer_db()
                .cas()
                .write(
                    Arc::new(PropContent::V1(updated.clone()).into()),
                    None,
                    ctx.events_tenancy(),
                    ctx.events_actor(),
                )
                .await?;

            ctx.workspace_snapshot()?
                .update_content(ctx.change_set()?, prop.id.into(), hash)
                .await?;
        }
        Ok(prop)
    }

    pub async fn direct_child_props_ordered(
        ctx: &DalContext,
        prop_id: PropId,
    ) -> PropResult<Vec<Prop>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let kind = workspace_snapshot
            .get_node_weight_by_id(prop_id)
            .await?
            .get_prop_node_weight()?
            .kind();

        let ordered_child_props = match kind {
            PropKind::Json | PropKind::Boolean | PropKind::Integer | PropKind::String => Vec::new(),
            PropKind::Array | PropKind::Map | PropKind::Object => {
                let ordered_child_prop_ids = workspace_snapshot
                    .ordered_children_for_node(prop_id)
                    .await?
                    .ok_or(WorkspaceSnapshotError::OrderingNotFound(prop_id.into()))?;

                let mut ordered_child_props = Vec::new();
                for ordered_child_prop_id in ordered_child_prop_ids {
                    if let NodeWeight::Prop(child_prop_weight) = workspace_snapshot
                        .get_node_weight_by_id(ordered_child_prop_id)
                        .await?
                    {
                        let child_prop =
                            Self::get_by_id_or_error(ctx, child_prop_weight.id().into()).await?;
                        ordered_child_props.push(child_prop);
                    }
                }
                ordered_child_props
            }
        };

        Ok(ordered_child_props)
    }

    // TODO(nick): this is straight up broken and inverted. You need to encapsulate the child type
    // in the parent type. The problem is that this was ported from a recursive function in the old
    // engine and is now a work queue. The collection pattern is ironically inverted: now, we start
    // at the highest level and descend (instead of immediately descending and popping the result
    // back up).
    pub async fn ts_type(&self, ctx: &DalContext) -> PropResult<String> {
        let mut work_queue = VecDeque::new();
        work_queue.push_back((self.id, self.kind));

        let mut ts_type = "".to_string();
        while let Some((prop_id, prop_kind)) = work_queue.pop_front() {
            ts_type = match prop_kind {
                PropKind::Array => {
                    let children = Self::direct_child_props_ordered(ctx, prop_id).await?;
                    let array_element_type = children
                        .first()
                        .ok_or(PropError::MapOrArrayMissingElementProp(prop_id))?;

                    work_queue.push_back((array_element_type.id, array_element_type.kind));
                    format!("{ts_type}[] | null | undefined")
                }
                PropKind::Json => {
                    "object | array | string | number | boolean | null | undefined".into()
                }
                PropKind::Boolean => "boolean | null | undefined".into(),
                PropKind::Integer => "number | null | undefined".into(),
                PropKind::Object => {
                    let mut object_interface = "{\n".to_string();
                    for child in &Self::direct_child_props_ordered(ctx, prop_id).await? {
                        // We serialize the object key as a JSON string because its
                        // the easiest way to ensure we create a valid TS interface
                        // even with keys that are not valid javascript identifiers.
                        // (e.g., we escape quotes in the prop name this way)
                        let name_value = serde_json::to_value(&child.name)?;
                        let name_serialized = serde_json::to_string(&name_value)?;
                        object_interface
                            .push_str(format!("{name_serialized}: {ts_type};\n").as_str());
                        work_queue.push_back((child.id, child.kind));
                    }
                    object_interface.push('}');

                    object_interface
                }
                PropKind::Map => {
                    let children = Self::direct_child_props_ordered(ctx, prop_id).await?;

                    let map_element_type = children
                        .first()
                        .ok_or(PropError::MapOrArrayMissingElementProp(prop_id))?;

                    work_queue.push_back((map_element_type.id, map_element_type.kind));
                    format!("Record<string, {ts_type}> | null | undefined")
                }
                PropKind::String => "string | null | undefined".into(),
            }
        }

        Ok(ts_type)
    }
}
