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

use petgraph::Direction;
use serde::{Deserialize, Serialize};
use si_events::ContentHash;
use si_layer_cache::LayerDbError;
use telemetry::prelude::*;
use thiserror::Error;

use crate::change_set_pointer::ChangeSetPointerError;
use crate::layer_db_types::{AttributePrototypeContent, AttributePrototypeContentV1};
use crate::workspace_snapshot::content_address::{ContentAddress, ContentAddressDiscriminants};
use crate::workspace_snapshot::edge_weight::{
    EdgeWeight, EdgeWeightError, EdgeWeightKind, EdgeWeightKindDiscriminants,
};
use crate::workspace_snapshot::node_weight::{
    NodeWeight, NodeWeightDiscriminants, NodeWeightError,
};
use crate::workspace_snapshot::WorkspaceSnapshotError;
use crate::{
    pk, AttributeValueId, DalContext, FuncId, OutputSocketId, PropId, Timestamp, TransactionsError,
};

pub mod argument;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum AttributePrototypeError {
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetPointerError),
    #[error("edge weight error: {0}")]
    EdgeWeight(#[from] EdgeWeightError),
    #[error("layer db error: {0}")]
    LayerDb(#[from] LayerDbError),
    #[error("attribute prototype {0} is missing a function edge")]
    MissingFunction(AttributePrototypeId),
    #[error("node weight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("could not acquire lock: {0}")]
    TryLock(#[from] tokio::sync::TryLockError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

pub type AttributePrototypeResult<T> = Result<T, AttributePrototypeError>;

pk!(AttributePrototypeId);

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct AttributePrototype {
    id: AttributePrototypeId,
    timestamp: Timestamp,
}

#[derive(Debug, PartialEq)]
pub struct AttributePrototypeGraphNode {
    id: AttributePrototypeId,
    content_address: ContentAddress,
    content: AttributePrototypeContentV1,
}

impl AttributePrototypeGraphNode {
    pub fn assemble(
        id: impl Into<AttributePrototypeId>,
        content_hash: ContentHash,
        content: AttributePrototypeContentV1,
    ) -> Self {
        Self {
            id: id.into(),
            content_address: ContentAddress::AttributePrototype(content_hash),
            content,
        }
    }
}

impl AttributePrototype {
    pub fn assemble(id: AttributePrototypeId, inner: &AttributePrototypeContentV1) -> Self {
        let inner: AttributePrototypeContentV1 = inner.to_owned();
        Self {
            id,
            timestamp: inner.timestamp,
        }
    }

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

        let change_set = ctx.change_set_pointer()?;
        let id = change_set.generate_ulid()?;
        let node_weight =
            NodeWeight::new_content(change_set, id, ContentAddress::AttributePrototype(hash))?;
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let _node_index = workspace_snapshot.add_node(node_weight).await?;

        workspace_snapshot
            .add_edge(
                id,
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())?,
                func_id,
            )
            .await?;

        Ok(AttributePrototype::assemble(
            AttributePrototypeId::from(id),
            &content,
        ))
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

        let change_set = ctx.change_set_pointer()?;
        workspace_snapshot
            .remove_edge(
                change_set,
                attribute_prototype_idx,
                current_func_node_idx,
                EdgeWeightKindDiscriminants::Use,
            )
            .await?;

        workspace_snapshot
            .add_edge(
                attribute_prototype_id,
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())?,
                func_id,
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
        let change_set = ctx.change_set_pointer()?;

        ctx.workspace_snapshot()?
            .remove_node_by_id(change_set, prototype_id)
            .await?;

        Ok(())
    }
}
