use serde::{Deserialize, Serialize};
use si_events::ContentHash;
use si_layer_cache::LayerDbError;
use std::collections::HashMap;
use std::sync::Arc;
use telemetry::prelude::*;
use thiserror::Error;

use crate::attribute::prototype::AttributePrototypeError;
use crate::change_set_pointer::ChangeSetPointerError;
use crate::layer_db_types::{OutputSocketContent, OutputSocketContentV1};
use crate::socket::{SocketArity, SocketKind};
use crate::workspace_snapshot::content_address::{ContentAddress, ContentAddressDiscriminants};
use crate::workspace_snapshot::edge_weight::{
    EdgeWeight, EdgeWeightError, EdgeWeightKind, EdgeWeightKindDiscriminants,
};
use crate::workspace_snapshot::node_weight::{NodeWeight, NodeWeightError};
use crate::workspace_snapshot::WorkspaceSnapshotError;
use crate::{
    pk, AttributePrototype, DalContext, FuncId, InputSocket, Timestamp, TransactionsError,
};
use crate::{AttributeValueId, SchemaVariantId};

use super::connection_annotation::{ConnectionAnnotation, ConnectionAnnotationError};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum OutputSocketError {
    #[error("attribute prototype error: {0}")]
    AttributePrototype(#[from] AttributePrototypeError),
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetPointerError),
    #[error(transparent)]
    ConnectionAnnotation(#[from] ConnectionAnnotationError),
    #[error("edge weight error: {0}")]
    EdgeWeight(#[from] EdgeWeightError),
    #[error("layer db error: {0}")]
    LayerDb(#[from] LayerDbError),
    #[error(
        "found two output sockets ({0} and {1}) of the same name for the same schema variant: {2}"
    )]
    NameCollision(OutputSocketId, OutputSocketId, SchemaVariantId),
    #[error("node weight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
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

        let change_set = ctx.change_set_pointer()?;
        let id = change_set.generate_ulid()?;
        let node_weight =
            NodeWeight::new_content(change_set, id, ContentAddress::OutputSocket(hash))?;

        let workspace_snapshot = ctx.workspace_snapshot()?;
        workspace_snapshot.add_node(node_weight).await?;
        workspace_snapshot
            .add_edge(
                schema_variant_id,
                EdgeWeight::new(change_set, EdgeWeightKind::Socket)?,
                id,
            )
            .await?;

        let attribute_prototype = AttributePrototype::new(ctx, func_id).await?;

        workspace_snapshot
            .add_edge(
                id,
                EdgeWeight::new(change_set, EdgeWeightKind::Prototype(None))?,
                attribute_prototype.id(),
            )
            .await?;

        Ok(Self::assemble(id.into(), content))
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
}
