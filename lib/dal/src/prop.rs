use petgraph::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use si_events::ContentHash;
use si_pkg::PropSpecKind;
use std::collections::VecDeque;
use std::sync::Arc;
use strum::{AsRefStr, Display, EnumIter, EnumString};
use telemetry::prelude::*;
use thiserror::Error;
use ulid::Ulid;

use crate::attribute::prototype::argument::{
    AttributePrototypeArgument, AttributePrototypeArgumentError,
};
use crate::attribute::prototype::AttributePrototypeError;
use crate::change_set_pointer::ChangeSetPointerError;
use crate::func::argument::{FuncArgument, FuncArgumentError};
use crate::func::intrinsics::IntrinsicFunc;
use crate::func::FuncError;
use crate::layer_db_types::{PropContent, PropContentDiscriminants, PropContentV1};
use crate::workspace_snapshot::content_address::ContentAddressDiscriminants;
use crate::workspace_snapshot::edge_weight::{EdgeWeight, EdgeWeightKind};
use crate::workspace_snapshot::edge_weight::{EdgeWeightError, EdgeWeightKindDiscriminants};
use crate::workspace_snapshot::node_weight::{NodeWeight, NodeWeightError};
use crate::workspace_snapshot::WorkspaceSnapshotError;
use crate::AttributeValueId;
use crate::{
    label_list::ToLabelList, pk, property_editor::schema::WidgetKind, AttributePrototype,
    AttributePrototypeId, DalContext, Func, FuncBackendResponseType, FuncId, SchemaVariantId,
    Timestamp, TransactionsError,
};

pub const PROP_VERSION: PropContentDiscriminants = PropContentDiscriminants::V1;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum PropError {
    #[error("attribute prototype error: {0}")]
    AttributePrototype(#[from] AttributePrototypeError),
    #[error("attribute prototype argument error: {0}")]
    AttributePrototypeArgument(#[from] AttributePrototypeArgumentError),
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetPointerError),
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
    /// A serialized validation format JSON object for the prop.  TODO: useTODO: use
    pub validation_format: Option<String>,
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

// const ALL_ANCESTOR_PROPS: &str = include_str!("queries/prop/all_ancestor_props.sql");
// const FIND_ROOT_PROP_FOR_PROP: &str = include_str!("queries/prop/root_prop_for_prop.sql");
// const FIND_PROP_IN_TREE: &str = include_str!("queries/prop/find_prop_in_tree.sql");

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

impl PropKind {
    pub fn ordered(&self) -> bool {
        matches!(self, PropKind::Array | PropKind::Map | PropKind::Object)
    }

    pub fn empty_value(&self) -> Option<serde_json::Value> {
        match self {
            Self::Array => Some(serde_json::json!([])),
            Self::Map | Self::Object => Some(serde_json::json!({})),
            _ => None,
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

pub enum PropParent {
    OrderedProp(PropId),
    Prop(PropId),
    SchemaVariant(SchemaVariantId),
}

impl Prop {
    pub fn assemble(id: PropId, inner: PropContentV1) -> Self {
        Self {
            id,
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
            validation_format: None,
        }
    }

    pub fn id(&self) -> PropId {
        self.id
    }

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
            None => Ok(None),
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

    pub async fn new_without_ui_optionals(
        ctx: &DalContext,
        name: impl AsRef<str>,
        kind: PropKind,
        prop_parent: PropParent,
    ) -> PropResult<Self> {
        Self::new(ctx, name.as_ref(), kind, false, None, None, prop_parent).await
    }

    /// Create a new [`Prop`]. A corresponding [`AttributePrototype`] and [`AttributeValue`] will be
    /// created when the provided [`SchemaVariant`](crate::SchemaVariant) is
    /// [`finalized`](crate::SchemaVariant::finalize).
    pub async fn new(
        ctx: &DalContext,
        name: impl Into<String>,
        kind: PropKind,
        hidden: bool,
        doc_link: Option<String>,
        widget_kind_and_options: Option<(WidgetKind, Option<Value>)>,
        prop_parent: PropParent,
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

        let change_set = ctx.change_set_pointer()?;
        let id = change_set.generate_ulid()?;
        let node_weight = NodeWeight::new_prop(change_set, id, kind, name, hash)?;
        let workspace_snapshot = ctx.workspace_snapshot()?;
        if ordered {
            workspace_snapshot
                .add_ordered_node(change_set, node_weight)
                .await?;
        } else {
            workspace_snapshot.add_node(node_weight).await?;
        }

        match prop_parent {
            PropParent::OrderedProp(ordered_prop_id) => {
                workspace_snapshot
                    .add_ordered_edge(
                        change_set,
                        ordered_prop_id,
                        EdgeWeight::new(change_set, EdgeWeightKind::new_use())?,
                        id,
                    )
                    .await?;
            }
            PropParent::Prop(prop_id) => {
                workspace_snapshot
                    .add_edge(
                        prop_id,
                        EdgeWeight::new(change_set, EdgeWeightKind::new_use())?,
                        id,
                    )
                    .await?;
            }
            PropParent::SchemaVariant(schema_variant_id) => {
                workspace_snapshot
                    .add_edge(
                        schema_variant_id,
                        EdgeWeight::new(change_set, EdgeWeightKind::new_use())?,
                        id,
                    )
                    .await?;
            }
        };

        Ok(Self::assemble(id.into(), content))
    }

    pub async fn get_by_id(ctx: &DalContext, id: PropId) -> PropResult<Self> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let ulid: ulid::Ulid = id.into();
        let node_index = workspace_snapshot.get_node_index_by_id(ulid).await?;
        let node_weight = workspace_snapshot.get_node_weight(node_index).await?;
        let hash = node_weight.content_hash();

        let content: PropContent = ctx
            .layer_db()
            .cas()
            .try_read_as(&hash)
            .await?
            .ok_or(WorkspaceSnapshotError::MissingContentFromStore(ulid))?;

        // NOTE(nick,jacob,zack): if we had a v2, then there would be migration logic here.
        let PropContent::V1(inner) = content;

        Ok(Prop::assemble(id, inner))
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
        Self::get_by_id(ctx, prop_id).await
    }

    pub async fn set_prototype_id(
        ctx: &DalContext,
        prop_id: PropId,
        attribute_prototype_id: AttributePrototypeId,
    ) -> PropResult<()> {
        ctx.workspace_snapshot()?
            .add_edge(
                prop_id,
                EdgeWeight::new(ctx.change_set_pointer()?, EdgeWeightKind::Prototype(None))?,
                attribute_prototype_id,
            )
            .await?;

        Ok(())
    }

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

    pub async fn set_default_value<T: Serialize>(
        ctx: &DalContext,
        prop_id: PropId,
        value: T,
    ) -> PropResult<()> {
        let value = serde_json::to_value(value)?;

        let prop = Prop::get_by_id(ctx, prop_id).await?;
        if !matches!(
            prop.kind,
            PropKind::String | PropKind::Boolean | PropKind::Integer
        ) {
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

    #[allow(dead_code)]
    async fn get_content(
        ctx: &DalContext,
        prop_id: PropId,
    ) -> PropResult<(ContentHash, PropContentV1)> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let id: Ulid = prop_id.into();
        let node_index = workspace_snapshot.get_node_index_by_id(id).await?;
        let node_weight = workspace_snapshot.get_node_weight(node_index).await?;
        let hash = node_weight.content_hash();

        let content: PropContent = ctx
            .layer_db()
            .cas()
            .try_read_as(&hash)
            .await?
            .ok_or(WorkspaceSnapshotError::MissingContentFromStore(id))?;

        // NOTE(nick,jacob,zack): if we had a v2, then there would be migration logic here.
        let PropContent::V1(inner) = content;

        Ok((hash, inner))
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
                .update_content(ctx.change_set_pointer()?, prop.id.into(), hash)
                .await?;
        }
        Ok(prop)
    }
}
