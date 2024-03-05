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

use content_store::{ContentHash, Store};
use petgraph::prelude::EdgeRef;
use petgraph::Direction;
use serde::{Deserialize, Serialize};
use strum::EnumDiscriminants;
use telemetry::prelude::*;
use thiserror::Error;

use crate::change_set_pointer::ChangeSetPointerError;
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
    #[error("attribute prototype {0} is missing a function edge")]
    MissingFunction(AttributePrototypeId),
    #[error("node weight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("store error: {0}")]
    Store(#[from] content_store::StoreError),
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

#[derive(EnumDiscriminants, Serialize, Deserialize, PartialEq)]
pub enum AttributePrototypeContent {
    V1(AttributePrototypeContentV1),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct AttributePrototypeContentV1 {
    pub timestamp: Timestamp,
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

    // NOTE(nick): all incoming edges to an attribute prototype must come from one of two places:
    //   - an attribute value whose lineage comes from a component
    //   - a prop or provider whose lineage comes from a schema variant
    // Outgoing edges from an attribute prototype are used for intra and inter component relationships.
    pub async fn new(ctx: &DalContext, func_id: FuncId) -> AttributePrototypeResult<Self> {
        let timestamp = Timestamp::now();

        let content = AttributePrototypeContentV1 { timestamp };
        let hash = ctx
            .content_store()
            .lock()
            .await
            .add(&AttributePrototypeContent::V1(content.clone()))?;

        let change_set = ctx.change_set_pointer()?;
        let id = change_set.generate_ulid()?;
        let node_weight =
            NodeWeight::new_content(change_set, id, ContentAddress::AttributePrototype(hash))?;
        let mut workspace_snapshot = ctx.workspace_snapshot()?.write().await;
        let _node_index = workspace_snapshot.add_node(node_weight)?;

        workspace_snapshot.add_edge(
            id,
            EdgeWeight::new(change_set, EdgeWeightKind::Use)?,
            func_id,
        )?;

        Ok(AttributePrototype::assemble(
            AttributePrototypeId::from(id),
            &content,
        ))
    }

    pub async fn func_id(
        ctx: &DalContext,
        prototype_id: AttributePrototypeId,
    ) -> AttributePrototypeResult<FuncId> {
        let workspace_snapshot = ctx.workspace_snapshot()?.read().await;
        for node_index in workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(prototype_id, EdgeWeightKindDiscriminants::Use)?
        {
            let node_weight = workspace_snapshot.get_node_weight(node_index)?;
            if NodeWeightDiscriminants::Func == node_weight.into() {
                return Ok(node_weight.id().into());
            }
        }

        Err(AttributePrototypeError::MissingFunction(prototype_id))
    }

    pub async fn find_for_prop(
        ctx: &DalContext,
        prop_id: PropId,
        key: &Option<String>,
    ) -> AttributePrototypeResult<Option<AttributePrototypeId>> {
        let workspace_snapshot = ctx.workspace_snapshot()?.read().await;

        if let Some(prototype_idx) = workspace_snapshot
            .edges_directed(prop_id, Direction::Outgoing)?
            .find(|edge_ref| {
                if let EdgeWeightKind::Prototype(maybe_key) = edge_ref.weight().kind() {
                    maybe_key == key
                } else {
                    false
                }
            })
            .map(|edge_ref| edge_ref.target())
        {
            let node_weight = workspace_snapshot.get_node_weight(prototype_idx)?;

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
        let workspace_snapshot = ctx.workspace_snapshot()?.read().await;

        if let Some(prototype_idx) = workspace_snapshot
            .edges_directed(output_socket_id, Direction::Outgoing)?
            .find(|edge_ref| {
                EdgeWeightKindDiscriminants::Prototype == edge_ref.weight().kind().into()
            })
            .map(|edge_ref| edge_ref.target())
        {
            let node_weight = workspace_snapshot.get_node_weight(prototype_idx)?;

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
        let mut workspace_snapshot = ctx.workspace_snapshot()?.write().await;

        let attribute_prototype_idx =
            workspace_snapshot.get_node_index_by_id(attribute_prototype_id)?;

        let current_func_node_idx = workspace_snapshot
            .edges_directed(attribute_prototype_id, Direction::Outgoing)?
            .find(|edge_ref| edge_ref.weight().kind() == &EdgeWeightKind::Use)
            .map(|edge_ref| edge_ref.target())
            .ok_or(AttributePrototypeError::MissingFunction(
                attribute_prototype_id,
            ))?;

        let change_set = ctx.change_set_pointer()?;
        workspace_snapshot.remove_edge(
            change_set,
            attribute_prototype_idx,
            current_func_node_idx,
            EdgeWeightKindDiscriminants::Use,
        )?;

        workspace_snapshot.add_edge(
            attribute_prototype_id,
            EdgeWeight::new(change_set, EdgeWeightKind::Use)?,
            func_id,
        )?;

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
        // prototype <-- Prototype -- (Prop | Provider) <-- Prop|Provider -- Attribute Values
        // (multiple avs possible)

        let mut attribute_value_ids = vec![];

        let workspace_snapshot = ctx.workspace_snapshot()?.read().await;
        for prototype_edge_source in workspace_snapshot.incoming_sources_for_edge_weight_kind(
            attribute_prototype_id,
            EdgeWeightKindDiscriminants::Prototype,
        )? {
            let (target_id, edge_weight_discrim) =
                match workspace_snapshot.get_node_weight(prototype_edge_source)? {
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
                .incoming_sources_for_edge_weight_kind(target_id, edge_weight_discrim)?
            {
                // There are also provider edges from the schema variant to the provider.
                // These should be different edge kinds, I think
                if let NodeWeight::AttributeValue(av_node_weight) =
                    workspace_snapshot.get_node_weight(attribute_value_target)?
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
        let workspace_snapshot = ctx.workspace_snapshot()?.read().await;

        let maybe_value_idxs = workspace_snapshot.incoming_sources_for_edge_weight_kind(
            attribute_prototype_id,
            EdgeWeightKindDiscriminants::Prototype,
        )?;

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
                    workspace_snapshot.get_node_weight(value_idx)?
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
        let mut workspace_snapshot = ctx.workspace_snapshot()?.write().await;

        workspace_snapshot.remove_node_by_id(prototype_id)?;

        Ok(())
    }
}
