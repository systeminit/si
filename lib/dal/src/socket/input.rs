use serde::{Deserialize, Serialize};
use si_events::ContentHash;
use si_frontend_types as frontend_types;
use si_layer_cache::LayerDbError;
use std::collections::HashMap;
use std::sync::Arc;
use telemetry::prelude::*;
use thiserror::Error;

use crate::attribute::prototype::argument::AttributePrototypeArgumentError;
use crate::attribute::prototype::AttributePrototypeError;
use crate::attribute::value::AttributeValueError;
use crate::change_set::ChangeSetError;
use crate::func::FuncError;
use crate::layer_db_types::{InputSocketContent, InputSocketContentV1};

use crate::socket::{SocketArity, SocketKind};
use crate::workspace_snapshot::content_address::{ContentAddress, ContentAddressDiscriminants};
use crate::workspace_snapshot::edge_weight::{EdgeWeightKind, EdgeWeightKindDiscriminants};
use crate::workspace_snapshot::node_weight::{NodeWeight, NodeWeightError};
use crate::workspace_snapshot::WorkspaceSnapshotError;
use crate::{
    implement_add_edge_to, pk, AttributePrototype, AttributePrototypeId, AttributeValue,
    AttributeValueId, ComponentError, ComponentId, DalContext, FuncId, HelperError, SchemaVariant,
    SchemaVariantError, SchemaVariantId, Timestamp, TransactionsError,
};

use super::connection_annotation::{ConnectionAnnotation, ConnectionAnnotationError};
use super::output::OutputSocketError;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum InputSocketError {
    #[error("attribute prototype error: {0}")]
    AttributePrototype(#[from] AttributePrototypeError),
    #[error("attribute prototype argument error: {0}")]
    AttributePrototypeArgumentError(#[from] AttributePrototypeArgumentError),
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] Box<AttributeValueError>),
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error(transparent)]
    ComponentError(#[from] Box<ComponentError>),
    #[error(transparent)]
    ConnectionAnnotation(#[from] ConnectionAnnotationError),
    #[error("found too many matches for input and socket: {0}, {1}")]
    FoundTooManyForInputSocketId(InputSocketId, ComponentId),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("helper error: {0}")]
    Helper(#[from] HelperError),
    #[error("layer db error: {0}")]
    LayerDb(#[from] LayerDbError),
    #[error("Input Socket ({0}) Missing Attribute Value for Component: {1}")]
    MissingAttributeValueForComponent(InputSocketId, ComponentId),
    #[error("Input Socket Missing Prototype: {0}")]
    MissingPrototype(InputSocketId),
    #[error("found multiple input sockets for attribute value: {0}")]
    MultipleSocketsForAttributeValue(AttributeValueId),
    #[error("node weight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("input socket not found by name {0} in schema variant {1}")]
    NotFoundByName(String, SchemaVariantId),
    #[error("schema variant error: {0}")]
    OutputSocketError(#[from] Box<OutputSocketError>),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] Box<SchemaVariantError>),
    #[error("store error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("could not acquire lock: {0}")]
    TryLock(#[from] tokio::sync::TryLockError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

pub type InputSocketResult<T> = Result<T, InputSocketError>;

pk!(InputSocketId);

impl From<si_events::InputSocketId> for InputSocketId {
    fn from(value: si_events::InputSocketId) -> Self {
        Self(value.into_raw_id())
    }
}

impl From<InputSocketId> for si_events::InputSocketId {
    fn from(value: InputSocketId) -> Self {
        Self::from_raw_id(value.0)
    }
}

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

impl InputSocket {
    pub async fn get_by_id(ctx: &DalContext, id: InputSocketId) -> InputSocketResult<Self> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let node_weight = workspace_snapshot.get_node_weight_by_id(id).await?;

        Self::get_from_node_weight(ctx, &node_weight).await
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
            .layer_db()
            .cas()
            .try_read_as(&node_weight.content_hash())
            .await?
            .ok_or(WorkspaceSnapshotError::MissingContentFromStore(
                node_weight.id(),
            ))?;

        let InputSocketContent::V1(inner) = content;

        Ok(Self::assemble(node_weight.id().into(), inner))
    }

    implement_add_edge_to!(
        source_id: InputSocketId,
        destination_id: AttributePrototypeId,
        add_fn: add_edge_to_attribute_prototype,
        discriminant: EdgeWeightKindDiscriminants::Prototype,
        result: InputSocketResult,
    );

    pub async fn find_with_name(
        ctx: &DalContext,
        name: impl AsRef<str>,
        schema_variant_id: SchemaVariantId,
    ) -> InputSocketResult<Option<Self>> {
        let name = name.as_ref();

        let workspace_snapshot = ctx.workspace_snapshot()?;

        for socket_node_index in workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(
                schema_variant_id,
                EdgeWeightKindDiscriminants::Socket,
            )
            .await?
        {
            let node_weight = workspace_snapshot
                .get_node_weight(socket_node_index)
                .await?;
            if let NodeWeight::Content(content_inner) = &node_weight {
                if ContentAddressDiscriminants::InputSocket
                    == content_inner.content_address().into()
                {
                    let input_socket = Self::get_from_node_weight(ctx, &node_weight).await?;
                    if input_socket.name() == name {
                        return Ok(Some(input_socket));
                    }
                }
            }
        }

        Ok(None)
    }

    pub async fn find_with_name_or_error(
        ctx: &DalContext,
        name: impl AsRef<str>,
        schema_variant_id: SchemaVariantId,
    ) -> InputSocketResult<Self> {
        let name = name.as_ref();
        Self::find_with_name(ctx, name, schema_variant_id)
            .await?
            .ok_or_else(|| InputSocketError::NotFoundByName(name.to_string(), schema_variant_id))
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
        let name = name.into();
        debug!(%schema_variant_id, %name, "creating input socket");

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
        let (hash, _) = ctx
            .layer_db()
            .cas()
            .write(
                Arc::new(InputSocketContent::V1(content.clone()).into()),
                None,
                ctx.events_tenancy(),
                ctx.events_actor(),
            )
            .await?;

        let workspace_snapshot = ctx.workspace_snapshot()?;
        let id = workspace_snapshot.generate_ulid().await?;
        let lineage_id = workspace_snapshot.generate_ulid().await?;

        let node_weight =
            NodeWeight::new_content(id, lineage_id, ContentAddress::InputSocket(hash));
        workspace_snapshot.add_node(node_weight).await?;
        SchemaVariant::add_edge_to_input_socket(
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

    pub async fn list_ids_for_schema_variant(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> InputSocketResult<Vec<InputSocketId>> {
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
                .get_option_content_node_weight_of_kind(ContentAddressDiscriminants::InputSocket)
            {
                content_hashes.push(content_node_weight.content_hash());
                node_weights.push(content_node_weight);
            }
        }

        let content_map: HashMap<ContentHash, InputSocketContent> = ctx
            .layer_db()
            .cas()
            .try_read_many_as(content_hashes.as_slice())
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

    ///
    /// Get all attribute values from all components that are connected to this input socket.
    ///
    /// NOTE: call component_attribute_value_for_input_socket_id() if you want the attribute
    /// value for a specific component.
    ///
    pub async fn all_attribute_values_everywhere_for_input_socket_id(
        ctx: &DalContext,
        input_socket_id: InputSocketId,
    ) -> InputSocketResult<Vec<AttributeValueId>> {
        let mut result = vec![];

        let workspace_snapshot = ctx.workspace_snapshot()?;
        let av_sources = workspace_snapshot
            .incoming_sources_for_edge_weight_kind(
                input_socket_id,
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

    pub async fn component_attribute_value_for_input_socket_id(
        ctx: &DalContext,
        input_socket_id: InputSocketId,
        component_id: ComponentId,
    ) -> InputSocketResult<AttributeValueId> {
        let mut result = None;
        for attribute_value_id in
            Self::all_attribute_values_everywhere_for_input_socket_id(ctx, input_socket_id)
                .await?
        {
            if AttributeValue::component_id(ctx, attribute_value_id)
                .await
                .map_err(Box::new)?
                == component_id
            {
                if result.is_some() {
                    return Err(InputSocketError::FoundTooManyForInputSocketId(
                        input_socket_id,
                        component_id,
                    ));
                }
                result = Some(attribute_value_id);
            }
        }
        match result {
            Some(attribute_value_id) => Ok(attribute_value_id),
            None => Err(InputSocketError::MissingAttributeValueForComponent(
                input_socket_id,
                component_id,
            ))
        }    }

    pub async fn find_for_attribute_value_id(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> InputSocketResult<Option<InputSocketId>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let mut raw_sources = workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(
                attribute_value_id,
                EdgeWeightKindDiscriminants::Socket,
            )
            .await?;
        let maybe_input_socket = if let Some(raw_parent) = raw_sources.pop() {
            if !raw_sources.is_empty() {
                return Err(InputSocketError::MultipleSocketsForAttributeValue(
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

    pub async fn find_equivalent_in_schema_variant(
        ctx: &DalContext,
        input_socket_id: InputSocketId,
        schema_variant_id: SchemaVariantId,
    ) -> InputSocketResult<InputSocketId> {
        let socket_name = Self::get_by_id(ctx, input_socket_id).await?.name;
        Ok(
            Self::find_with_name_or_error(ctx, socket_name, schema_variant_id)
                .await?
                .id,
        )
    }
}

impl From<InputSocket> for frontend_types::InputSocket {
    fn from(value: InputSocket) -> Self {
        Self {
            id: value.id.into(),
            name: value.name,
            eligible_to_send_data: false,
        }
    }
}
