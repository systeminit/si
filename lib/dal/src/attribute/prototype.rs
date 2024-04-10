//! An [`AttributePrototype`] represents, for a specific attribute:
//!
//!   * Which context the following applies to ([`AttributeContext`](crate::AttributeContext))
//!   * The function that should be run to find its value.
//!   * In the case that the [`Prop`](crate::Prop) is the child of an
//!     [`Array`](crate::prop::PropKind::Array): Which index in the `Array` the value
//!     is for.
//!   * In the case that the [`Prop`](crate::Prop) is the child of a
//!     [`Map`](crate::prop::PropKind::Map): Which key of the `Map` the value is
//!     for.

use std::sync::Arc;

use content_node_weight::ContentNodeWeight;
use petgraph::Direction;
use serde::{Deserialize, Serialize};
use si_events::ulid::Ulid;
use si_layer_cache::LayerDbError;
use strum::{AsRefStr, Display, EnumDiscriminants, EnumIter, EnumString};
use telemetry::prelude::*;
use thiserror::Error;

use crate::attribute::prototype::argument::value_source::ValueSource;
use crate::attribute::prototype::argument::AttributePrototypeArgument;
use crate::attribute::value::AttributeValueError;
use crate::change_set::ChangeSetError;
use crate::layer_db_types::{AttributePrototypeContent, AttributePrototypeContentV1};
use crate::workspace_snapshot::content_address::{ContentAddress, ContentAddressDiscriminants};
use crate::workspace_snapshot::edge_weight::{
    EdgeWeightError, EdgeWeightKind, EdgeWeightKindDiscriminants,
};
use crate::workspace_snapshot::node_weight::{
    content_node_weight, NodeWeight, NodeWeightDiscriminants, NodeWeightError,
};
use crate::workspace_snapshot::WorkspaceSnapshotError;
use crate::{
    attribute::prototype::argument::AttributePrototypeArgumentId, implement_add_edge_to, pk,
    AttributeValue, AttributeValueId, ComponentId, DalContext, FuncId, HelperError, InputSocketId,
    OutputSocketId, PropId, SchemaVariant, SchemaVariantError, SchemaVariantId, Timestamp,
    TransactionsError,
};

pub mod argument;
pub mod debug;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum AttributePrototypeError {
    #[error("attribute prototype argument error: {0}")]
    AttributePrototypeArgument(String),
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] Box<AttributeValueError>),
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("edge weight error: {0}")]
    EdgeWeight(#[from] EdgeWeightError),
    #[error("helper error: {0}")]
    Helper(#[from] HelperError),
    #[error("layer db error: {0}")]
    LayerDb(#[from] LayerDbError),
    #[error("attribute prototype {0} is missing a function edge")]
    MissingFunction(AttributePrototypeId),
    #[error("No attribute values for: {0}")]
    NoAttributeValues(AttributePrototypeId),
    #[error("node weight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("Attribute Prototype not found: {0}")]
    NotFound(AttributePrototypeId),
    #[error("attribute prototype has been orphaned: {0}")]
    Orphaned(AttributePrototypeId),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] Box<SchemaVariantError>),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("could not acquire lock: {0}")]
    TryLock(#[from] tokio::sync::TryLockError),
    #[error("unexpected node ({0}) using attribute prototype ({1})")]
    UnexpectedNodeUsingAttributePrototype(Ulid, AttributePrototypeId),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

pub type AttributePrototypeResult<T> = Result<T, AttributePrototypeError>;

/// Indicates the _one and only one_ eventual parent of a corresponding [`AttributePrototype`].
///
/// - If an [`AttributePrototype`] is used by an [`AttributeValue`], its eventual parent is a
///   [`SchemaVariant`].
/// - If an [`AttributePrototype`] is used by a [`Prop`](crate::Prop), an
///   [`InputSocket`](crate::InputSocket), or an [`OutputSocket`](crate::OutputSocket), its eventual
///   parent is a [`Component`](crate::Component).
#[remain::sorted]
#[derive(Debug, Clone, Copy, EnumDiscriminants)]
pub enum AttributePrototypeEventualParent {
    Component(ComponentId),
    SchemaVariantFromInputSocket(SchemaVariantId, InputSocketId),
    SchemaVariantFromOutputSocket(SchemaVariantId, OutputSocketId),
    SchemaVariantFromProp(SchemaVariantId, PropId),
}

// TODO(nick): switch to the "id!" macro once the frontend doesn't use the old nil id to indicate
// that the argument is a new one.
pk!(AttributePrototypeId);

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct AttributePrototype {
    pub id: AttributePrototypeId,
    pub timestamp: Timestamp,
}

impl AttributePrototype {
    pub fn assemble(id: AttributePrototypeId, inner: &AttributePrototypeContentV1) -> Self {
        let inner: AttributePrototypeContentV1 = inner.to_owned();
        Self {
            id,
            timestamp: inner.timestamp,
        }
    }

    implement_add_edge_to!(
        source_id: AttributePrototypeId,
        destination_id: FuncId,
        add_fn: add_edge_to_func,
        discriminant: EdgeWeightKindDiscriminants::Use,
        result: AttributePrototypeResult,
    );
    implement_add_edge_to!(
        source_id: AttributePrototypeId,
        destination_id: AttributePrototypeArgumentId,
        add_fn: add_edge_to_argument,
        discriminant: EdgeWeightKindDiscriminants::PrototypeArgument,
        result: AttributePrototypeResult,
    );

    pub fn id(&self) -> AttributePrototypeId {
        self.id
    }

    pub async fn new(ctx: &DalContext, func_id: FuncId) -> AttributePrototypeResult<Self> {
        let timestamp = Timestamp::now();

        let content = AttributePrototypeContentV1 { timestamp };
        let (hash, _) = ctx
            .layer_db()
            .cas()
            .write(
                Arc::new(AttributePrototypeContent::V1(content.clone()).into()),
                None,
                ctx.events_tenancy(),
                ctx.events_actor(),
            )
            .await?;

        let change_set = ctx.change_set()?;
        let id = change_set.generate_ulid()?;
        let node_weight =
            NodeWeight::new_content(change_set, id, ContentAddress::AttributePrototype(hash))?;
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let _node_index = workspace_snapshot.add_node(node_weight).await?;

        let prototype = AttributePrototype::assemble(id.into(), &content);

        Self::add_edge_to_func(ctx, prototype.id, func_id, EdgeWeightKind::new_use()).await?;

        Ok(prototype)
    }

    pub async fn func_id(
        ctx: &DalContext,
        prototype_id: AttributePrototypeId,
    ) -> AttributePrototypeResult<FuncId> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        for node_index in workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(prototype_id, EdgeWeightKindDiscriminants::Use)
            .await?
        {
            let node_weight = workspace_snapshot.get_node_weight(node_index).await?;
            let node_weight_id = node_weight.id();
            if NodeWeightDiscriminants::Func == node_weight.into() {
                return Ok(node_weight_id.into());
            }
        }

        Err(AttributePrototypeError::MissingFunction(prototype_id))
    }

    pub async fn find_for_prop(
        ctx: &DalContext,
        prop_id: PropId,
        key: &Option<String>,
    ) -> AttributePrototypeResult<Option<AttributePrototypeId>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        if let Some(prototype_idx) = workspace_snapshot
            .edges_directed(prop_id, Direction::Outgoing)
            .await?
            .iter()
            .find(|(edge_weight, _, _)| {
                if let EdgeWeightKind::Prototype(maybe_key) = edge_weight.kind() {
                    maybe_key == key
                } else {
                    false
                }
            })
            .map(|(_, _, target_idx)| target_idx)
        {
            let node_weight = workspace_snapshot.get_node_weight(*prototype_idx).await?;

            if matches!(
                node_weight.content_address_discriminants(),
                Some(ContentAddressDiscriminants::AttributePrototype)
            ) {
                return Ok(Some(node_weight.id().into()));
            }
        }

        Ok(None)
    }

    pub async fn find_for_output_socket(
        ctx: &DalContext,
        output_socket_id: OutputSocketId,
    ) -> AttributePrototypeResult<Option<AttributePrototypeId>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        if let Some(prototype_idx) = workspace_snapshot
            .edges_directed(output_socket_id, Direction::Outgoing)
            .await?
            .iter()
            .find(|(edge_weight, _, _)| {
                EdgeWeightKindDiscriminants::Prototype == edge_weight.kind().into()
            })
            .map(|(_, _, target_idx)| target_idx)
        {
            let node_weight = workspace_snapshot.get_node_weight(*prototype_idx).await?;

            if matches!(
                node_weight.content_address_discriminants(),
                Some(ContentAddressDiscriminants::AttributePrototype)
            ) {
                return Ok(Some(node_weight.id().into()));
            }
        }

        Ok(None)
    }
    pub async fn find_for_input_socket(
        ctx: &DalContext,
        input_socket_id: InputSocketId,
    ) -> AttributePrototypeResult<Option<AttributePrototypeId>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        if let Some(prototype_idx) = workspace_snapshot
            .edges_directed(input_socket_id, Direction::Outgoing)
            .await?
            .iter()
            .find(|(edge_weight, _, _)| {
                EdgeWeightKindDiscriminants::Prototype == edge_weight.kind().into()
            })
            .map(|(_, _, target_idx)| target_idx)
        {
            let node_weight = workspace_snapshot.get_node_weight(*prototype_idx).await?;

            if matches!(
                node_weight.content_address_discriminants(),
                Some(ContentAddressDiscriminants::AttributePrototype)
            ) {
                return Ok(Some(node_weight.id().into()));
            }
        }

        Ok(None)
    }

    pub async fn get_by_id(
        ctx: &DalContext,
        prototype_id: AttributePrototypeId,
    ) -> AttributePrototypeResult<Self> {
        let (_node_weight, content) = Self::get_node_weight_and_content(ctx, prototype_id).await?;
        Ok(Self::assemble(prototype_id, &content))
    }

    async fn get_node_weight_and_content(
        ctx: &DalContext,
        prototype_id: AttributePrototypeId,
    ) -> AttributePrototypeResult<(ContentNodeWeight, AttributePrototypeContentV1)> {
        let content_weight = ctx
            .workspace_snapshot()?
            .get_node_weight_by_id(prototype_id)
            .await?;
        let prototype_node_weight = content_weight
            .get_content_node_weight_of_kind(ContentAddressDiscriminants::AttributePrototype)?;

        let content: AttributePrototypeContent = ctx
            .layer_db()
            .cas()
            .try_read_as(&prototype_node_weight.content_hash())
            .await?
            .ok_or(WorkspaceSnapshotError::MissingContentFromStore(
                prototype_id.into(),
            ))?;

        // Do "upgrading" of the storage format from old versions to the latest here.
        let AttributePrototypeContent::V1(inner) = content;

        Ok((prototype_node_weight, inner))
    }

    pub async fn update_func_by_id(
        ctx: &DalContext,
        attribute_prototype_id: AttributePrototypeId,
        func_id: FuncId,
    ) -> AttributePrototypeResult<()> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let attribute_prototype_idx = workspace_snapshot
            .get_node_index_by_id(attribute_prototype_id)
            .await?;

        let current_func_node_idx = workspace_snapshot
            .edges_directed(attribute_prototype_id, Direction::Outgoing)
            .await?
            .iter()
            .find(|(edge_weight, _, _)| edge_weight.kind() == &EdgeWeightKind::new_use())
            .map(|(_, _, target_idx)| *target_idx)
            .ok_or(AttributePrototypeError::MissingFunction(
                attribute_prototype_id,
            ))?;

        let change_set = ctx.change_set()?;
        workspace_snapshot
            .remove_edge(
                change_set,
                attribute_prototype_idx,
                current_func_node_idx,
                EdgeWeightKindDiscriminants::Use,
            )
            .await?;

        Self::add_edge_to_func(
            ctx,
            attribute_prototype_id,
            func_id,
            EdgeWeightKind::new_use(),
        )
        .await?;

        Ok(())
    }

    pub async fn attribute_value_ids(
        ctx: &DalContext,
        attribute_prototype_id: AttributePrototypeId,
    ) -> AttributePrototypeResult<Vec<AttributeValueId>> {
        if let Some(attribute_value_id) =
            Self::attribute_value_id(ctx, attribute_prototype_id).await?
        {
            return Ok(vec![attribute_value_id]);
        }

        // Remaining edges
        // prototype <-- Prototype -- (Prop | Socket) <-- Prop|Socket -- Attribute Values
        // (multiple avs possible)

        let mut attribute_value_ids = vec![];

        let workspace_snapshot = ctx.workspace_snapshot()?;
        for prototype_edge_source in workspace_snapshot
            .incoming_sources_for_edge_weight_kind(
                attribute_prototype_id,
                EdgeWeightKindDiscriminants::Prototype,
            )
            .await?
        {
            let (target_id, edge_weight_discrim) = match workspace_snapshot
                .get_node_weight(prototype_edge_source)
                .await?
            {
                NodeWeight::Prop(prop_inner) => {
                    (prop_inner.id(), EdgeWeightKindDiscriminants::Prop)
                }
                NodeWeight::Content(content_inner) => match content_inner.content_address() {
                    ContentAddress::OutputSocket(_) | ContentAddress::InputSocket(_) => {
                        (content_inner.id(), EdgeWeightKindDiscriminants::Socket)
                    }
                    _ => {
                        return Err(WorkspaceSnapshotError::UnexpectedEdgeSource(
                            content_inner.id(),
                            attribute_prototype_id.into(),
                            EdgeWeightKindDiscriminants::Prototype,
                        )
                        .into());
                    }
                },
                other => {
                    return Err(WorkspaceSnapshotError::UnexpectedEdgeSource(
                        other.id(),
                        attribute_prototype_id.into(),
                        EdgeWeightKindDiscriminants::Prototype,
                    )
                    .into());
                }
            };

            for attribute_value_target in workspace_snapshot
                .incoming_sources_for_edge_weight_kind(target_id, edge_weight_discrim)
                .await?
            {
                if let NodeWeight::AttributeValue(av_node_weight) = workspace_snapshot
                    .get_node_weight(attribute_value_target)
                    .await?
                {
                    attribute_value_ids.push(av_node_weight.id().into())
                }
            }
        }

        Ok(attribute_value_ids)
    }

    /// If this prototype is defined at the component level, it will have an incoming edge from the
    /// AttributeValue for which it is the prototype. Otherwise this will return None, indicating a
    /// prototype defined at the schema variant level (which has no attribute value)
    pub async fn attribute_value_id(
        ctx: &DalContext,
        attribute_prototype_id: AttributePrototypeId,
    ) -> AttributePrototypeResult<Option<AttributeValueId>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let maybe_value_idxs = workspace_snapshot
            .incoming_sources_for_edge_weight_kind(
                attribute_prototype_id,
                EdgeWeightKindDiscriminants::Prototype,
            )
            .await?;

        if maybe_value_idxs.len() > 1 {
            return Err(WorkspaceSnapshotError::UnexpectedNumberOfIncomingEdges(
                EdgeWeightKindDiscriminants::Prototype,
                NodeWeightDiscriminants::Content,
                attribute_prototype_id.into(),
            )
            .into());
        }

        Ok(match maybe_value_idxs.first().copied() {
            Some(value_idx) => {
                if let NodeWeight::AttributeValue(av_node_weight) =
                    workspace_snapshot.get_node_weight(value_idx).await?
                {
                    Some(av_node_weight.id().into())
                } else {
                    None
                }
            }
            None => None,
        })
    }

    pub async fn remove(
        ctx: &DalContext,
        prototype_id: AttributePrototypeId,
    ) -> AttributePrototypeResult<()> {
        let change_set = ctx.change_set()?;

        ctx.workspace_snapshot()?
            .remove_node_by_id(change_set, prototype_id)
            .await?;

        Ok(())
    }

    pub async fn list_input_socket_sources_for_id(
        ctx: &DalContext,
        ap_id: AttributePrototypeId,
    ) -> AttributePrototypeResult<Vec<InputSocketId>> {
        let apa_ids = AttributePrototypeArgument::list_ids_for_prototype(ctx, ap_id)
            .await
            .map_err(|e| AttributePrototypeError::AttributePrototypeArgument(e.to_string()))?;

        let mut input_socket_ids = Vec::<InputSocketId>::new();
        for apa_id in apa_ids {
            let maybe_value_source = AttributePrototypeArgument::value_source_by_id(ctx, apa_id)
                .await
                .map_err(|e| AttributePrototypeError::AttributePrototypeArgument(e.to_string()))?;
            if let Some(ValueSource::InputSocket(socket_id)) = maybe_value_source {
                input_socket_ids.push(socket_id);
            }
        }

        Ok(input_socket_ids)
    }

    pub async fn list_ids_for_func_id(
        ctx: &DalContext,
        func_id: FuncId,
    ) -> AttributePrototypeResult<Vec<AttributePrototypeId>> {
        let mut attribute_prototype_argument_ids = Vec::new();

        let workspace_snapshot = ctx.workspace_snapshot()?;

        for node_index in workspace_snapshot
            .incoming_sources_for_edge_weight_kind(func_id, EdgeWeightKindDiscriminants::Use)
            .await?
        {
            let node_weight = workspace_snapshot.get_node_weight(node_index).await?;
            let node_weight_id = node_weight.id();
            if let Some(ContentAddressDiscriminants::AttributePrototype) =
                node_weight.content_address_discriminants()
            {
                attribute_prototype_argument_ids.push(node_weight_id.into());
            }
        }

        Ok(attribute_prototype_argument_ids)
    }

    /// Returns the [eventual parent](AttributePrototypeEventualParent) of the
    /// [`AttributePrototype`], which will either be a [`Component`](crate::Component) or a
    /// [`SchemaVariant`].
    pub async fn eventual_parent(
        ctx: &DalContext,
        id: AttributePrototypeId,
    ) -> AttributePrototypeResult<AttributePrototypeEventualParent> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let source = *workspace_snapshot
            .incoming_sources_for_edge_weight_kind(id, EdgeWeightKindDiscriminants::Prototype)
            .await?
            .first()
            .ok_or(AttributePrototypeError::Orphaned(id))?;

        let node_weight = workspace_snapshot.get_node_weight(source).await?;
        let node_weight_id = node_weight.id();

        let eventual_parent = match node_weight {
            NodeWeight::AttributeValue(_) => AttributePrototypeEventualParent::Component(
                AttributeValue::component_id(ctx, node_weight_id.into())
                    .await
                    .map_err(Box::new)?,
            ),
            NodeWeight::Prop(_) => AttributePrototypeEventualParent::SchemaVariantFromProp(
                SchemaVariant::find_for_prop_id(ctx, node_weight_id.into())
                    .await
                    .map_err(Box::new)?,
                node_weight_id.into(),
            ),
            NodeWeight::Content(inner) => match inner.content_address().into() {
                ContentAddressDiscriminants::InputSocket => {
                    AttributePrototypeEventualParent::SchemaVariantFromInputSocket(
                        SchemaVariant::find_for_input_socket_id(ctx, node_weight_id.into())
                            .await
                            .map_err(Box::new)?,
                        node_weight_id.into(),
                    )
                }
                ContentAddressDiscriminants::OutputSocket => {
                    AttributePrototypeEventualParent::SchemaVariantFromOutputSocket(
                        SchemaVariant::find_for_output_socket_id(ctx, node_weight_id.into())
                            .await
                            .map_err(Box::new)?,
                        node_weight_id.into(),
                    )
                }
                _ => {
                    return Err(
                        AttributePrototypeError::UnexpectedNodeUsingAttributePrototype(
                            node_weight_id,
                            id,
                        ),
                    )
                }
            },
            _ => {
                return Err(
                    AttributePrototypeError::UnexpectedNodeUsingAttributePrototype(
                        node_weight_id,
                        id,
                    ),
                )
            }
        };

        Ok(eventual_parent)
    }

    pub async fn input_sources(
        ctx: &DalContext,
        id: AttributePrototypeId,
    ) -> AttributePrototypeResult<Vec<AttributePrototypeSource>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let mut sources = Vec::new();
        for (edge_weight, prototype_edge_source, _) in workspace_snapshot
            .edges_directed(id, Direction::Incoming)
            .await?
        {
            let key = match edge_weight.kind() {
                EdgeWeightKind::Prototype(key) => key.clone(),
                _ => continue,
            };

            match workspace_snapshot
                .get_node_weight(prototype_edge_source)
                .await?
            {
                NodeWeight::AttributeValue(av) => {
                    sources.push(AttributePrototypeSource::AttributeValue(
                        av.id().into(),
                        key,
                    ));
                }
                NodeWeight::Prop(prop) => {
                    sources.push(AttributePrototypeSource::Prop(prop.id().into(), key));
                }
                NodeWeight::Content(content_inner) => match content_inner.content_address() {
                    ContentAddress::InputSocket(_) => {
                        sources.push(AttributePrototypeSource::InputSocket(
                            content_inner.id().into(),
                            key,
                        ));
                    }
                    ContentAddress::OutputSocket(_) => {
                        sources.push(AttributePrototypeSource::OutputSocket(
                            content_inner.id().into(),
                            key,
                        ));
                    }
                    _ => {
                        return Err(WorkspaceSnapshotError::UnexpectedEdgeSource(
                            content_inner.id(),
                            id.into(),
                            EdgeWeightKindDiscriminants::Prototype,
                        )
                        .into());
                    }
                },
                other => {
                    return Err(WorkspaceSnapshotError::UnexpectedEdgeSource(
                        other.id(),
                        id.into(),
                        EdgeWeightKindDiscriminants::Prototype,
                    )
                    .into());
                }
            }
        }
        Ok(sources)
    }
}

#[remain::sorted]
#[derive(
    AsRefStr, Clone, Debug, Deserialize, Display, EnumIter, EnumString, Eq, PartialEq, Serialize,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum AttributePrototypeSource {
    AttributeValue(AttributeValueId, Option<String>),
    InputSocket(InputSocketId, Option<String>),
    OutputSocket(OutputSocketId, Option<String>),
    Prop(PropId, Option<String>),
}
