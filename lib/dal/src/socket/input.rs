use std::sync::Arc;

use serde::{
    Deserialize,
    Serialize,
};
use si_events::Timestamp;
use si_frontend_types as frontend_types;
use si_layer_cache::LayerDbError;
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    AttributePrototype,
    AttributePrototypeId,
    AttributeValueId,
    ComponentError,
    ComponentId,
    DalContext,
    EdgeWeight,
    EdgeWeightKind,
    FuncId,
    HelperError,
    SchemaVariantError,
    SchemaVariantId,
    TransactionsError,
    attribute::{
        prototype::{
            AttributePrototypeError,
            argument::AttributePrototypeArgumentError,
        },
        value::AttributeValueError,
    },
    change_set::ChangeSetError,
    func::FuncError,
    implement_add_edge_to,
    layer_db_types::{
        InputSocketContent,
        InputSocketContentV2,
    },
    socket::{
        SocketArity,
        SocketKind,
        connection_annotation::{
            ConnectionAnnotation,
            ConnectionAnnotationError,
        },
        output::OutputSocketError,
    },
    workspace_snapshot::{
        InputSocketExt,
        WorkspaceSnapshotError,
        edge_weight::EdgeWeightKindDiscriminants,
        graph::LineageId,
        node_weight::{
            NodeWeight,
            NodeWeightError,
        },
        traits::socket::input::input_socket_from_node_weight_and_content,
    },
};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum InputSocketError {
    #[error("attribute prototype error: {0}")]
    AttributePrototype(#[from] Box<AttributePrototypeError>),
    #[error("attribute prototype argument error: {0}")]
    AttributePrototypeArgumentError(#[source] Box<AttributePrototypeArgumentError>),
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
    Func(#[from] Box<FuncError>),
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

impl From<AttributePrototypeArgumentError> for InputSocketError {
    fn from(value: AttributePrototypeArgumentError) -> Self {
        Self::AttributePrototypeArgumentError(Box::new(value))
    }
}

impl From<AttributePrototypeError> for InputSocketError {
    fn from(value: AttributePrototypeError) -> Self {
        Box::new(value).into()
    }
}

impl From<FuncError> for InputSocketError {
    fn from(value: FuncError) -> Self {
        Box::new(value).into()
    }
}

pub type InputSocketResult<T> = Result<T, InputSocketError>;

pub use si_id::InputSocketId;

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
        ctx.workspace_snapshot()?
            .get_input_socket(ctx, id)
            .await
            .map_err(Into::into)
    }

    pub fn assemble(id: InputSocketId, arity: SocketArity, inner: InputSocketContentV2) -> Self {
        Self {
            id,
            timestamp: inner.timestamp,
            name: inner.name,
            inbound_type_definition: inner.inbound_type_definition,
            outbound_type_definition: inner.outbound_type_definition,
            arity,
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
        ctx.workspace_snapshot()?
            .get_input_socket_by_name_opt(ctx, name, schema_variant_id)
            .await
            .map_err(Into::into)
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

        let connection_annotations = if let Some(ca) = connection_annotations {
            ca
        } else {
            vec![ConnectionAnnotation::try_from(name.clone())?]
        };

        let content = InputSocketContentV2 {
            timestamp: Timestamp::now(),
            name: name.clone(),
            inbound_type_definition: None,
            outbound_type_definition: None,
            kind,
            required: false,
            ui_hidden: false,
            connection_annotations,
        };
        let (hash, _) = ctx.layer_db().cas().write(
            Arc::new(InputSocketContent::V2(content.clone()).into()),
            None,
            ctx.events_tenancy(),
            ctx.events_actor(),
        )?;

        let snapshot = ctx.workspace_snapshot()?;
        let input_socket_id: InputSocketId = snapshot.generate_ulid().await?.into();
        let lineage_id: LineageId = snapshot.generate_ulid().await?;

        let node_weight =
            NodeWeight::new_input_socket(input_socket_id.into(), lineage_id, arity, hash);
        snapshot.add_or_replace_node(node_weight.clone()).await?;
        let input_socket_node_weight = node_weight.get_input_socket_node_weight()?;

        let edge_weight = EdgeWeight::new(EdgeWeightKind::Socket);
        snapshot
            .add_edge(schema_variant_id, edge_weight, input_socket_id)
            .await?;

        let attribute_prototype = AttributePrototype::new(ctx, func_id).await?;

        snapshot
            .add_edge(
                input_socket_id,
                EdgeWeight::new(EdgeWeightKind::Prototype(None)),
                attribute_prototype.id(),
            )
            .await?;

        let input_socket = input_socket_from_node_weight_and_content(
            &input_socket_node_weight,
            InputSocketContent::V2(content),
        )?;

        Ok(input_socket)
    }

    pub async fn list_ids_for_schema_variant(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> InputSocketResult<Vec<InputSocketId>> {
        ctx.workspace_snapshot()?
            .list_input_socket_ids_for_schema_variant(schema_variant_id)
            .await
            .map_err(Into::into)
    }

    pub async fn list(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> InputSocketResult<Vec<Self>> {
        ctx.workspace_snapshot()?
            .list_input_sockets(ctx, schema_variant_id)
            .await
            .map_err(Into::into)
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
        ctx.workspace_snapshot()?
            .all_attribute_value_ids_everywhere_for_input_socket_id(input_socket_id)
            .await
            .map_err(Into::into)
    }

    pub async fn component_attribute_value_id(
        ctx: &DalContext,
        input_socket_id: InputSocketId,
        component_id: ComponentId,
    ) -> InputSocketResult<AttributeValueId> {
        ctx.workspace_snapshot()?
            .component_attribute_value_id_for_input_socket_id(input_socket_id, component_id)
            .await
            .map_err(Into::into)
    }

    pub async fn find_for_attribute_value_id(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> InputSocketResult<Option<InputSocketId>> {
        ctx.workspace_snapshot()?
            .input_socket_id_find_for_attribute_value_id(attribute_value_id)
            .await
            .map_err(Into::into)
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

    /// Get a short, human-readable title suitable for debugging/display.
    pub async fn fmt_title(ctx: &DalContext, id: InputSocketId) -> String {
        Self::fmt_title_fallible(ctx, id)
            .await
            .unwrap_or_else(|e| e.to_string())
    }

    async fn fmt_title_fallible(ctx: &DalContext, id: InputSocketId) -> InputSocketResult<String> {
        let socket = Self::get_by_id(ctx, id).await?;
        Ok(socket.name)
    }
}

impl From<InputSocket> for frontend_types::InputSocket {
    fn from(value: InputSocket) -> Self {
        Self {
            id: value.id,
            name: value.name,
            eligible_to_send_data: false,
        }
    }
}
