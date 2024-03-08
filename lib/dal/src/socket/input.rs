use content_store::{ContentHash, Store};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum::EnumDiscriminants;
use telemetry::prelude::*;
use thiserror::Error;

use crate::attribute::prototype::AttributePrototypeError;
use crate::change_set_pointer::ChangeSetPointerError;
use crate::func::FuncError;
use crate::socket::{SocketArity, SocketKind};
use crate::workspace_snapshot::content_address::{ContentAddress, ContentAddressDiscriminants};
use crate::workspace_snapshot::edge_weight::{
    EdgeWeight, EdgeWeightError, EdgeWeightKind, EdgeWeightKindDiscriminants,
};
use crate::workspace_snapshot::node_weight::{NodeWeight, NodeWeightError};
use crate::workspace_snapshot::WorkspaceSnapshotError;
use crate::{
    pk, AttributePrototype, AttributePrototypeId, AttributeValueId, DalContext, FuncId,
    SchemaVariantId, Timestamp, TransactionsError,
};

use super::connection_annotation::{ConnectionAnnotation, ConnectionAnnotationError};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum InputSocketError {
    #[error("attribute prototype error: {0}")]
    AttributePrototype(#[from] AttributePrototypeError),
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetPointerError),
    #[error(transparent)]
    ConnectionAnnotation(#[from] ConnectionAnnotationError),
    #[error("edge weight error: {0}")]
    EdgeWeight(#[from] EdgeWeightError),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
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

pub type InputSocketResult<T> = Result<T, InputSocketError>;

pk!(InputSocketId);

/// This socket can only provide data within its own [`SchemaVariants`](crate::SchemaVariant). It
/// can only consume data from external [`SchemaVariants`](crate::SchemaVariant).
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct InputSocket {
    id: InputSocketId,
    #[serde(flatten)]
    timestamp: Timestamp,
    /// Name for [`Self`] that can be used for identification.
    name: String,
    /// Definition of the inbound type (e.g. "JSONSchema" or "Number").
    inbound_type_definition: Option<String>,
    /// Definition of the outbound type (e.g. "JSONSchema" or "Number").
    outbound_type_definition: Option<String>,
    arity: SocketArity,
    kind: SocketKind,
    required: bool,
    ui_hidden: bool,
    connection_annotations: Vec<ConnectionAnnotation>,
}

#[derive(EnumDiscriminants, Serialize, Deserialize, PartialEq)]
pub enum InputSocketContent {
    V1(InputSocketContentV1),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct InputSocketContentV1 {
    pub timestamp: Timestamp,
    /// Name for [`Self`] that can be used for identification.
    pub name: String,
    /// Definition of the inbound type (e.g. "JSONSchema" or "Number").
    pub inbound_type_definition: Option<String>,
    /// Definition of the outbound type (e.g. "JSONSchema" or "Number").
    pub outbound_type_definition: Option<String>,
    pub arity: SocketArity,
    pub kind: SocketKind,
    pub required: bool,
    pub ui_hidden: bool,
    pub connection_annotations: Vec<ConnectionAnnotation>,
}

impl InputSocket {
    pub async fn get_by_id(ctx: &DalContext, id: InputSocketId) -> InputSocketResult<Self> {
        let workspace_snapshot = ctx.workspace_snapshot()?.read().await;
        let node_weight = workspace_snapshot.get_node_weight_by_id(id)?;

        Self::get_from_node_weight(ctx, node_weight).await
    }

    pub fn assemble(id: InputSocketId, inner: InputSocketContentV1) -> Self {
        Self {
            id,
            timestamp: inner.timestamp,
            name: inner.name,
            inbound_type_definition: inner.inbound_type_definition,
            outbound_type_definition: inner.outbound_type_definition,
            arity: inner.arity,
            kind: inner.kind,
            required: inner.required,
            ui_hidden: inner.ui_hidden,
            connection_annotations: inner.connection_annotations,
        }
    }

    pub fn id(&self) -> InputSocketId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn arity(&self) -> SocketArity {
        self.arity
    }

    pub fn ui_hidden(&self) -> bool {
        self.ui_hidden
    }

    pub fn required(&self) -> bool {
        self.required
    }

    pub fn connection_annotations(&self) -> Vec<ConnectionAnnotation> {
        self.connection_annotations.clone()
    }

    async fn get_from_node_weight(
        ctx: &DalContext,
        node_weight: &NodeWeight,
    ) -> InputSocketResult<Self> {
        let content: InputSocketContent = ctx
            .content_store()
            .try_lock()?
            .get(&node_weight.content_hash())
            .await?
            .ok_or(WorkspaceSnapshotError::MissingContentFromStore(
                node_weight.id(),
            ))?;

        let InputSocketContent::V1(inner) = content;

        Ok(Self::assemble(node_weight.id().into(), inner))
    }

    pub async fn add_prototype_edge(
        ctx: &DalContext,
        input_socket_id: InputSocketId,
        attribute_prototype_id: AttributePrototypeId,
        key: &Option<String>,
    ) -> InputSocketResult<()> {
        let mut workspace_snapshot = ctx.workspace_snapshot()?.write().await;
        workspace_snapshot.add_edge(
            input_socket_id,
            EdgeWeight::new(
                ctx.change_set_pointer()?,
                EdgeWeightKind::Prototype(key.to_owned()),
            )?,
            attribute_prototype_id,
        )?;

        Ok(())
    }

    pub async fn find_with_name(
        ctx: &DalContext,
        name: impl AsRef<str>,
        schema_variant_id: SchemaVariantId,
    ) -> InputSocketResult<Option<Self>> {
        let name = name.as_ref();

        let workspace_snapshot = ctx.workspace_snapshot()?.read().await;

        for socket_node_index in workspace_snapshot.outgoing_targets_for_edge_weight_kind(
            schema_variant_id,
            EdgeWeightKindDiscriminants::Socket,
        )? {
            let node_weight = workspace_snapshot.get_node_weight(socket_node_index)?;
            if let NodeWeight::Content(content_inner) = node_weight {
                if ContentAddressDiscriminants::InputSocket
                    == content_inner.content_address().into()
                {
                    let input_socket = Self::get_from_node_weight(ctx, node_weight).await?;
                    if input_socket.name() == name {
                        return Ok(Some(input_socket));
                    }
                }
            }
        }

        Ok(None)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
        name: impl Into<String>,
        func_id: FuncId,
        arity: SocketArity,
        kind: SocketKind,
        connection_annotations: Option<Vec<ConnectionAnnotation>>,
    ) -> InputSocketResult<Self> {
        info!("creating input socket");
        let name = name.into();

        let connection_annotations = if let Some(ca) = connection_annotations {
            ca
        } else {
            vec![ConnectionAnnotation::try_from(name.clone())?]
        };

        let content = InputSocketContentV1 {
            timestamp: Timestamp::now(),
            name: name.clone(),
            inbound_type_definition: None,
            outbound_type_definition: None,
            arity,
            kind,
            required: false,
            ui_hidden: false,
            connection_annotations,
        };
        let hash = ctx
            .content_store()
            .try_lock()?
            .add(&InputSocketContent::V1(content.clone()))?;

        let change_set = ctx.change_set_pointer()?;
        let id = change_set.generate_ulid()?;

        {
            let mut workspace_snapshot = ctx.workspace_snapshot()?.write().await;
            let node_weight =
                NodeWeight::new_content(change_set, id, ContentAddress::InputSocket(hash))?;
            let _node_index = workspace_snapshot.add_node(node_weight)?;
            workspace_snapshot.add_edge(
                schema_variant_id,
                EdgeWeight::new(change_set, EdgeWeightKind::Socket)?,
                id,
            )?;
        }

        let attribute_prototype = AttributePrototype::new(ctx, func_id).await?;

        {
            let mut workspace_snapshot = ctx.workspace_snapshot()?.write().await;
            workspace_snapshot.add_edge(
                id,
                EdgeWeight::new(change_set, EdgeWeightKind::Prototype(None))?,
                attribute_prototype.id(),
            )?;
        }

        Ok(Self::assemble(id.into(), content))
    }

    pub async fn list_ids_for_schema_variant(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> InputSocketResult<Vec<InputSocketId>> {
        let workspace_snapshot = ctx.workspace_snapshot()?.read().await;

        let node_indices = workspace_snapshot.outgoing_targets_for_edge_weight_kind(
            schema_variant_id,
            EdgeWeightKindDiscriminants::Socket,
        )?;

        let mut result = vec![];
        for node_index in node_indices {
            let node_weight = workspace_snapshot.get_node_weight(node_index)?;
            if node_weight
                .get_option_content_node_weight_of_kind(ContentAddressDiscriminants::InputSocket)
                .is_some()
            {
                result.push(node_weight.id().into());
            }
        }

        Ok(result)
    }

    pub async fn list(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> InputSocketResult<Vec<Self>> {
        let workspace_snapshot = ctx.workspace_snapshot()?.read().await;

        let node_indices = workspace_snapshot.outgoing_targets_for_edge_weight_kind(
            schema_variant_id,
            EdgeWeightKindDiscriminants::Socket,
        )?;

        let mut content_hashes = Vec::new();
        let mut node_weights = Vec::new();
        for node_index in node_indices {
            let node_weight = workspace_snapshot.get_node_weight(node_index)?;
            if let Some(content_node_weight) = node_weight
                .get_option_content_node_weight_of_kind(ContentAddressDiscriminants::InputSocket)
            {
                content_hashes.push(content_node_weight.content_hash());
                node_weights.push(content_node_weight);
            }
        }

        let content_map: HashMap<ContentHash, InputSocketContent> = ctx
            .content_store()
            .try_lock()?
            .get_bulk(content_hashes.as_slice())
            .await?;

        let mut input_sockets = Vec::new();
        for node_weight in node_weights {
            match content_map.get(&node_weight.content_hash()) {
                Some(content) => {
                    // NOTE(nick,jacob,zack): if we had a v2, then there would be migration logic here.
                    let InputSocketContent::V1(inner) = content;

                    input_sockets.push(Self::assemble(node_weight.id().into(), inner.to_owned()));
                }
                None => Err(WorkspaceSnapshotError::MissingContentFromStore(
                    node_weight.id(),
                ))?,
            }
        }

        Ok(input_sockets)
    }

    pub async fn attribute_values_for_input_socket_id(
        ctx: &DalContext,
        input_socket_id: InputSocketId,
    ) -> InputSocketResult<Vec<AttributeValueId>> {
        let mut result = vec![];

        let workspace_snapshot = ctx.workspace_snapshot()?.read().await;
        let av_sources = workspace_snapshot.incoming_sources_for_edge_weight_kind(
            input_socket_id,
            EdgeWeightKindDiscriminants::Socket,
        )?;
        for av_source_idx in av_sources {
            if let NodeWeight::AttributeValue(av_node_weight) =
                workspace_snapshot.get_node_weight(av_source_idx)?
            {
                result.push(av_node_weight.id().into());
            }
        }

        Ok(result)
    }
}
