use serde::{Deserialize, Serialize};
use si_events::ContentHash;
use si_layer_cache::LayerDbError;
use std::collections::HashMap;
use std::sync::Arc;
use telemetry::prelude::*;
use thiserror::Error;

use crate::attribute::prototype::argument::AttributePrototypeArgumentId;
use crate::attribute::prototype::AttributePrototypeError;
use crate::change_set::ChangeSetError;
use crate::layer_db_types::{OutputSocketContent, OutputSocketContentV1};
use crate::socket::{SocketArity, SocketKind};
use crate::workspace_snapshot::content_address::{ContentAddress, ContentAddressDiscriminants};
use crate::workspace_snapshot::edge_weight::{
    EdgeWeightError, EdgeWeightKind, EdgeWeightKindDiscriminants,
};
use crate::workspace_snapshot::node_weight::{ContentNodeWeight, NodeWeight, NodeWeightError};
use crate::workspace_snapshot::WorkspaceSnapshotError;
use crate::{
    implement_add_edge_to, AttributePrototypeId, AttributeValueId, ComponentId, InputSocketId,
    SchemaVariantId,
};
use crate::{
    pk, AttributePrototype, DalContext, FuncId, HelperError, InputSocket, SchemaVariant,
    SchemaVariantError, Timestamp, TransactionsError,
};

use super::connection_annotation::{ConnectionAnnotation, ConnectionAnnotationError};
use super::input::InputSocketError;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum OutputSocketError {
    #[error("attribute prototype error: {0}")]
    AttributePrototype(#[from] AttributePrototypeError),
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error(transparent)]
    ConnectionAnnotation(#[from] ConnectionAnnotationError),
    #[error("edge weight error: {0}")]
    EdgeWeight(#[from] EdgeWeightError),
    #[error("found too many matches for input and socket: {0}, {1}")]
    FoundTooManyForInputSocketId(InputSocketId, ComponentId),
    #[error("helper error: {0}")]
    Helper(#[from] HelperError),
    #[error("layer db error: {0}")]
    InputSocketError(#[from] InputSocketError),
    #[error("layer db error: {0}")]
    LayerDb(#[from] LayerDbError),
    #[error("found multiple input sockets for attribute value: {0}")]
    MultipleSocketsForAttributeValue(AttributeValueId),
    #[error(
        "found two output sockets ({0} and {1}) of the same name for the same schema variant: {2}"
    )]
    NameCollision(OutputSocketId, OutputSocketId, SchemaVariantId),
    #[error("node weight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] Box<SchemaVariantError>),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("could not acquire lock: {0}")]
    TryLock(#[from] tokio::sync::TryLockError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

pub type OutputSocketResult<T> = Result<T, OutputSocketError>;

pk!(OutputSocketId);

/// This socket can only provide data to external [`SchemaVariants`](crate::SchemaVariant). It can
/// only consume data within its own [`SchemaVariant`](crate::SchemaVariant).
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct OutputSocket {
    id: OutputSocketId,
    #[serde(flatten)]
    pub timestamp: Timestamp,
    /// Name for [`Self`] that can be used for identification.
    name: String,
    /// Definition of the data type (e.g. "JSONSchema" or "Number").
    type_definition: Option<String>,
    arity: SocketArity,
    kind: SocketKind,
    required: bool,
    ui_hidden: bool,
    connection_annotations: Vec<ConnectionAnnotation>,
}

impl OutputSocket {
    pub fn assemble(id: OutputSocketId, inner: OutputSocketContentV1) -> Self {
        Self {
            id,
            timestamp: inner.timestamp,
            name: inner.name,
            type_definition: inner.type_definition,
            arity: inner.arity,
            kind: inner.kind,
            ui_hidden: inner.ui_hidden,
            required: inner.required,
            connection_annotations: inner.connection_annotations,
        }
    }

    pub fn id(&self) -> OutputSocketId {
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

    implement_add_edge_to!(
        source_id: OutputSocketId,
        destination_id: AttributePrototypeId,
        add_fn: add_edge_to_attribute_prototype,
        discriminant: EdgeWeightKindDiscriminants::Prototype,
        result: OutputSocketResult,
    );

    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
        name: impl Into<String>,
        type_definition: Option<String>,
        func_id: FuncId,
        arity: SocketArity,
        kind: SocketKind,
        connection_annotations: Option<Vec<ConnectionAnnotation>>,
    ) -> OutputSocketResult<Self> {
        let name = name.into();
        debug!(%schema_variant_id, %name, "creating output socket");

        let connection_annotations = if let Some(ca) = connection_annotations {
            ca
        } else {
            vec![ConnectionAnnotation::try_from(name.clone())?]
        };

        let content = OutputSocketContentV1 {
            timestamp: Timestamp::now(),
            name: name.clone(),
            type_definition,
            arity,
            kind,
            required: false,
            ui_hidden: false,
            connection_annotations,
        };
        let (hash, _) = ctx
            .layer_db()
            .cas()
            .write(
                Arc::new(OutputSocketContent::V1(content.clone()).into()),
                None,
                ctx.events_tenancy(),
                ctx.events_actor(),
            )
            .await?;

        let change_set = ctx.change_set()?;
        let id = change_set.generate_ulid()?;
        let node_weight =
            NodeWeight::new_content(change_set, id, ContentAddress::OutputSocket(hash))?;

        let workspace_snapshot = ctx.workspace_snapshot()?;
        workspace_snapshot.add_node(node_weight).await?;

        SchemaVariant::add_edge_to_output_socket(
            ctx,
            schema_variant_id,
            id.into(),
            EdgeWeightKind::Socket,
        )
        .await
        .map_err(Box::new)?;

        let attribute_prototype = AttributePrototype::new(ctx, func_id).await?;

        let socket = Self::assemble(id.into(), content);
        Self::add_edge_to_attribute_prototype(
            ctx,
            socket.id,
            attribute_prototype.id(),
            EdgeWeightKind::Prototype(None),
        )
        .await?;
        Ok(socket)
    }

    pub async fn get_by_id(
        ctx: &DalContext,
        output_socket_id: OutputSocketId,
    ) -> OutputSocketResult<Self> {
        let (_, content) = Self::get_node_weight_and_content(ctx, output_socket_id).await?;

        Ok(Self::assemble(output_socket_id, content))
    }

    async fn get_node_weight_and_content(
        ctx: &DalContext,
        output_socket_id: OutputSocketId,
    ) -> OutputSocketResult<(ContentNodeWeight, OutputSocketContentV1)> {
        let weight = ctx
            .workspace_snapshot()?
            .get_node_weight_by_id(output_socket_id)
            .await?
            .get_content_node_weight_of_kind(ContentAddressDiscriminants::OutputSocket)?;
        let content: OutputSocketContent = ctx
            .layer_db()
            .cas()
            .try_read_as(&weight.content_hash())
            .await?
            .ok_or(WorkspaceSnapshotError::MissingContentFromStore(
                output_socket_id.into(),
            ))?;

        // Do inner content "upgrading" here when there becomes a need for a V2 storage format.
        let OutputSocketContent::V1(inner) = content;

        Ok((weight, inner))
    }

    pub async fn attribute_values_for_output_socket_id(
        ctx: &DalContext,
        output_socket_id: OutputSocketId,
    ) -> OutputSocketResult<Vec<AttributeValueId>> {
        let mut result = vec![];

        let workspace_snapshot = ctx.workspace_snapshot()?;
        let av_sources = workspace_snapshot
            .incoming_sources_for_edge_weight_kind(
                output_socket_id,
                EdgeWeightKindDiscriminants::Socket,
            )
            .await?;
        for av_source_idx in av_sources {
            if let NodeWeight::AttributeValue(av_node_weight) =
                workspace_snapshot.get_node_weight(av_source_idx).await?
            {
                result.push(av_node_weight.id().into());
            }
        }

        Ok(result)
    }

    pub async fn list_ids_for_schema_variant(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> OutputSocketResult<Vec<OutputSocketId>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let node_indices = workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(
                schema_variant_id,
                EdgeWeightKindDiscriminants::Socket,
            )
            .await?;

        let mut result = vec![];
        for node_index in node_indices {
            let node_weight = workspace_snapshot.get_node_weight(node_index).await?;
            if node_weight
                .get_option_content_node_weight_of_kind(ContentAddressDiscriminants::OutputSocket)
                .is_some()
            {
                result.push(node_weight.id().into())
            }
        }

        Ok(result)
    }

    pub async fn list(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> OutputSocketResult<Vec<Self>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let node_indices = workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(
                schema_variant_id,
                EdgeWeightKindDiscriminants::Socket,
            )
            .await?;

        let mut content_hashes = Vec::new();
        let mut node_weights = Vec::new();
        for node_index in node_indices {
            let node_weight = workspace_snapshot.get_node_weight(node_index).await?;
            if let Some(content_node_weight) = node_weight
                .get_option_content_node_weight_of_kind(ContentAddressDiscriminants::OutputSocket)
            {
                content_hashes.push(content_node_weight.content_hash());
                node_weights.push(content_node_weight);
            }
        }

        let content_map: HashMap<ContentHash, OutputSocketContent> = ctx
            .layer_db()
            .cas()
            .try_read_many_as(content_hashes.as_slice())
            .await?;

        let mut output_sockets = Vec::new();
        for node_weight in node_weights {
            match content_map.get(&node_weight.content_hash()) {
                Some(content) => {
                    // NOTE(nick,jacob,zack): if we had a v2, then there would be migration logic here.
                    let OutputSocketContent::V1(inner) = content;

                    output_sockets.push(Self::assemble(node_weight.id().into(), inner.to_owned()));
                }
                None => Err(WorkspaceSnapshotError::MissingContentFromStore(
                    node_weight.id(),
                ))?,
            }
        }

        Ok(output_sockets)
    }

    // TODO(nick): this function uses the underlying list call since it needs to perform bulk content store retrieval
    // for the name.
    pub async fn find_with_name(
        ctx: &DalContext,
        name: impl AsRef<str>,
        schema_variant_id: SchemaVariantId,
    ) -> OutputSocketResult<Option<Self>> {
        let name = name.as_ref();

        // NOTE(nick): at the time of writing, we do not have connection annotations and we do not enforce socket
        // names being unique for the same schema variant. Should we do that? That's a different question, but this
        // function will ensure that we find one and only one based on the name.
        let mut maybe_output_socket: Option<Self> = None;
        for output_socket in Self::list(ctx, schema_variant_id).await? {
            if name == output_socket.name() {
                match maybe_output_socket {
                    Some(already_found) => {
                        return Err(OutputSocketError::NameCollision(
                            already_found.id(),
                            output_socket.id(),
                            schema_variant_id,
                        ));
                    }
                    None => maybe_output_socket = Some(output_socket),
                }
            }
        }
        Ok(maybe_output_socket)
    }
    #[instrument(level="debug" skip(ctx))]
    pub async fn fits_input_by_id(
        ctx: &DalContext,
        input_socket_id: InputSocketId,
        output_socket_id: OutputSocketId,
    ) -> OutputSocketResult<bool> {
        let output_socket = OutputSocket::get_by_id(ctx, output_socket_id).await?;
        let input_socket = InputSocket::get_by_id(ctx, input_socket_id).await?;
        Ok(output_socket.fits_input(&input_socket))
    }

    pub fn fits_input(&self, input: &InputSocket) -> bool {
        let out_annotations = self.connection_annotations();
        let in_annotations = input.connection_annotations();
        for annotation_src in &out_annotations {
            for annotation_dest in &in_annotations {
                if ConnectionAnnotation::target_fits_reference(annotation_src, annotation_dest) {
                    return true;
                }
            }
        }

        false
    }

    pub async fn find_for_attribute_value_id(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> OutputSocketResult<Option<OutputSocketId>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let mut raw_sources = workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(
                attribute_value_id,
                EdgeWeightKindDiscriminants::Socket,
            )
            .await?;
        let maybe_input_socket = if let Some(raw_parent) = raw_sources.pop() {
            if !raw_sources.is_empty() {
                return Err(OutputSocketError::MultipleSocketsForAttributeValue(
                    attribute_value_id,
                ));
            }
            Some(
                workspace_snapshot
                    .get_node_weight(raw_parent)
                    .await?
                    .id()
                    .into(),
            )
        } else {
            None
        };

        Ok(maybe_input_socket)
    }

    // `AttributePrototypeArguemnts` that use this `OutputSocket` as their source of data.
    pub async fn prototype_arguments_using(
        &self,
        ctx: &DalContext,
    ) -> OutputSocketResult<Vec<AttributePrototypeArgumentId>> {
        let mut results = Vec::new();
        for (_edge_weight, tail_idx, _head_idx) in ctx
            .workspace_snapshot()?
            .edges_directed_for_edge_weight_kind(
                self.id(),
                petgraph::Direction::Incoming,
                EdgeWeightKindDiscriminants::PrototypeArgumentValue,
            )
            .await?
        {
            if let NodeWeight::AttributePrototypeArgument(attribute_prototype_argument_weight) =
                ctx.workspace_snapshot()?.get_node_weight(tail_idx).await?
            {
                results.push(attribute_prototype_argument_weight.id().into());
            }
        }

        Ok(results)
    }
}
